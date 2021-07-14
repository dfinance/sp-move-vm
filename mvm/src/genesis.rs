use alloc::borrow::Cow;
use alloc::borrow::ToOwned;
use alloc::vec::Vec;
use core::cell::RefCell;

use anyhow::Error;
use anyhow::{anyhow, ensure};
use hashbrown::HashMap;

use diem_crypto::HashValue;
use diem_types::account_config;
use diem_types::chain_id::ChainId;
use diem_types::on_chain_config::VMConfig;
use move_core_types::account_address::AccountAddress;
use move_core_types::gas_schedule::CostTable;
use move_core_types::identifier::{IdentStr, Identifier};
use move_core_types::language_storage::{ModuleId, TypeTag, CORE_CODE_ADDRESS};
use move_core_types::value::{serialize_values, MoveValue};
use move_core_types::vm_status::StatusCode;

use crate::gas_schedule::cost_table;
use crate::io::balance::CurrencyInfo;
use crate::io::traits::{Balance, BalanceAccess, CurrencyAccessPath, EventHandler, Storage};
use crate::mvm::Mvm;
use crate::types::{Gas, ModulePackage, PublishPackageTx};
use crate::Vm;

const GENESIS_MODULE_NAME: &str = "Genesis";
const INIT_FUN_NAME: &str = "initialize";

pub fn init_storage<S>(storage: S, config: GenesisConfig) -> Result<(), Error>
where
    S: Storage,
{
    let fork = StorageFork::new(storage);
    let vm_config = VMConfig {
        gas_schedule: config.cost_table.clone(),
    };
    let vm = Mvm::new_with_config(&fork, NopeEventHandler, NopeBalance, vm_config)?;
    let result = vm.publish_module_package(Gas::infinite(), config.stdlib, false);
    ensure!(
        result.status_code == StatusCode::EXECUTED,
        "Failed to publish stdlib:{:?}",
        result
    );

    let initial_allow_list = MoveValue::Vector(
        config
            .script_allow_list
            .into_iter()
            .map(|hash| MoveValue::vector_u8(hash.to_vec().into_iter().collect()))
            .collect(),
    );

    let instr_gas_costs = bcs::to_bytes(&config.cost_table.instruction_table)
        .map_err(|err| anyhow!("Failure serializing genesis instr gas costs. {:?}", err))?;
    let native_gas_costs = bcs::to_bytes(&config.cost_table.native_table)
        .map_err(|err| anyhow!("Failure serializing genesis native gas costs. {:?}", err))?;

    let res = vm.execute_function(
        CORE_CODE_ADDRESS,
        Gas::infinite(),
        &ModuleId::new(CORE_CODE_ADDRESS, Identifier::new(GENESIS_MODULE_NAME)?),
        IdentStr::new(INIT_FUN_NAME)?,
        vec![],
        serialize_values(&vec![
            MoveValue::Signer(config.diem_root_address),
            MoveValue::Signer(config.treasury_compliance_account_address),
            MoveValue::vector_u8(config.diem_root_address.to_vec()),
            MoveValue::vector_u8(config.treasury_compliance_account_address.to_vec()),
            initial_allow_list,
            MoveValue::Bool(config.is_open_module),
            MoveValue::vector_u8(instr_gas_costs),
            MoveValue::vector_u8(native_gas_costs),
            MoveValue::U8(config.chain_id.id()),
        ]),
        None,
    );

    if res.status_code != StatusCode::EXECUTED {
        return Err(anyhow!("Failed to execution genesis function:{:?}", res));
    }

    fork.merge();
    Ok(())
}

pub struct GenesisConfig {
    pub stdlib: PublishPackageTx,
    pub script_allow_list: Vec<HashValue>,
    pub cost_table: CostTable,
    pub is_open_module: bool,
    pub chain_id: ChainId,
    diem_root_address: AccountAddress,
    pub treasury_compliance_account_address: AccountAddress,
}

impl Default for GenesisConfig {
    #[cfg(feature = "move_stdlib")]
    fn default() -> Self {
        use core::convert::TryFrom;
        let stdlib = ModulePackage::try_from(stdlib::stdlib_package())
            .expect("Expected valid stdlib")
            .into_tx(CORE_CODE_ADDRESS);

        GenesisConfig {
            stdlib,
            script_allow_list: vec![],
            cost_table: cost_table(),
            is_open_module: true,
            chain_id: Default::default(),
            diem_root_address: account_config::diem_root_address(),
            treasury_compliance_account_address:
                account_config::treasury_compliance_account_address(),
        }
    }

    #[cfg(not(feature = "move_stdlib"))]
    fn default() -> Self {
        let stdlib = ModulePackage::default().into_tx(CORE_CODE_ADDRESS);

        GenesisConfig {
            stdlib,
            script_allow_list: vec![],
            cost_table: cost_table(),
            is_open_module: true,
            chain_id: Default::default(),
            diem_root_address: account_config::diem_root_address(),
            treasury_compliance_account_address:
                account_config::treasury_compliance_account_address(),
        }
    }
}

struct NopeEventHandler;

impl EventHandler for NopeEventHandler {
    fn on_event(&self, _: Vec<u8>, _: u64, _: TypeTag, _: Vec<u8>) {
        //no-op
    }
}

struct NopeBalance;

impl BalanceAccess for NopeBalance {
    fn get_currency_info(&self, _: &CurrencyAccessPath) -> Option<CurrencyInfo> {
        None
    }

    fn get_balance(&self, _: &AccountAddress, _: &CurrencyAccessPath) -> Option<Balance> {
        None
    }

    fn add(&self, _: &AccountAddress, _: &CurrencyAccessPath, _: Balance) {
        //no-op
    }

    fn sub(&self, _: &AccountAddress, _: &CurrencyAccessPath, _: Balance) {
        //no-op
    }
}

pub struct StorageFork<S: Storage> {
    inner: S,
    diff: RefCell<HashMap<Cow<'static, [u8]>, Option<Vec<u8>>>>,
}

impl<S: Storage> StorageFork<S> {
    pub fn new(storage: S) -> StorageFork<S> {
        StorageFork {
            inner: storage,
            diff: RefCell::new(Default::default()),
        }
    }

    pub fn merge(self) {
        for (key, val) in self.diff.take() {
            match val {
                None => {
                    self.inner.remove(key.as_ref());
                }
                Some(val) => {
                    self.inner.insert(key.as_ref(), val.as_ref());
                }
            }
        }
    }
}

impl<S: Storage> Storage for &StorageFork<S> {
    fn get(&self, key: &[u8]) -> Option<Vec<u8>> {
        let diff = self.diff.borrow();
        if let Some(val) = diff.get(key) {
            val.to_owned()
        } else {
            self.inner.get(key)
        }
    }

    fn insert(&self, key: &[u8], value: &[u8]) {
        let mut diff = self.diff.borrow_mut();

        diff.insert(Cow::Owned(key.to_vec()), Some(value.to_vec()));
    }

    fn remove(&self, key: &[u8]) {
        let mut diff = self.diff.borrow_mut();
        diff.insert(Cow::Owned(key.to_vec()), None);
    }
}
