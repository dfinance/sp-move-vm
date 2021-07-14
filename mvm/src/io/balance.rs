use alloc::borrow::Cow;
use alloc::borrow::ToOwned;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use anyhow::Error;
use cell::Lazy;
use core::cell::RefCell;
use diem_types::account_config;
use diem_types::account_config::{ACCOUNT_MODULE_IDENTIFIER, CORE_CODE_ADDRESS};
use diem_types::resources::currency_info::CurrencyInfoResource;
use hashbrown::HashMap;
use move_core_types::account_address::AccountAddress;
use move_core_types::effects::ChangeSet;
use move_core_types::identifier::Identifier;
use move_core_types::language_storage::{StructTag, TypeTag};
use move_core_types::vm_status::StatusCode;
use move_vm_runtime::data_cache::RemoteCache;
use move_vm_types::natives::function::PartialVMError;
use serde::{Deserialize, Serialize};
use vm::errors::{Location, PartialVMResult, VMResult};

use crate::io::traits::{Balance, BalanceAccess, CurrencyAccessPath};
use move_core_types::vm_status::known_locations::DIEM_MODULE_IDENTIFIER;

pub const DIEM_BALANCE: &str = "Balance";
pub const CURRENCY_INFO: &str = "CurrencyInfo";

pub static DIEM_COIN_IDENTIFIER: Lazy<Identifier> =
    Lazy::new(|| Identifier::new(DIEM_BALANCE).unwrap());

pub static BALANCE_TEMPLATE: Lazy<StructTag> = Lazy::new(|| StructTag {
    address: CORE_CODE_ADDRESS,
    module: ACCOUNT_MODULE_IDENTIFIER.clone(),
    name: DIEM_COIN_IDENTIFIER.clone(),
    type_params: vec![TypeTag::U8],
});

pub struct MasterOfCoin<B: BalanceAccess> {
    access: B,
    native_mapper: RefCell<HashMap<StructTag, Option<Vec<u8>>>>,
}

impl<B: BalanceAccess> MasterOfCoin<B> {
    pub fn new(access: B) -> MasterOfCoin<B> {
        MasterOfCoin {
            access,
            native_mapper: Default::default(),
        }
    }

    pub fn session<'b, 'r, R: RemoteCache>(
        &'b self,
        remote: &'r R,
    ) -> MasterOfCoinSession<'b, 'r, B, R> {
        MasterOfCoinSession {
            master_of_coin: self,
            balances: RefCell::new(Default::default()),
            remote,
        }
    }

    pub fn clear(&self) {
        self.native_mapper.borrow_mut().clear();
    }

    pub fn update_balance(&self, balance_op: BalanceOp) {
        match balance_op {
            BalanceOp::Add(acc, path, diff) => self.access.add(&acc, path.as_ref(), diff),
            BalanceOp::Sub(acc, path, diff) => self.access.sub(&acc, path.as_ref(), diff),
        }
    }

    fn get_bridge<R: RemoteCache>(&self, remote: &R, coin: &StructTag) -> Option<Vec<u8>> {
        let mut mapper = self.native_mapper.borrow_mut();

        match mapper.get(coin) {
            Some(path) => path.to_owned(),
            None => {
                let addr = account_config::diem_root_address();
                let mut currency = native_currency(coin);
                let bridge = remote
                    .get_resource(&addr, &currency)
                    .ok()?
                    .and_then(|buff| bcs::from_bytes(&buff).ok());
                if let TypeTag::Struct(coin) = currency.type_params.remove(0) {
                    mapper.insert(coin, bridge.to_owned());
                }
                bridge
            }
        }
    }

    fn get_balance(&self, address: &AccountAddress, path: &CurrencyAccessPath) -> Option<Balance> {
        self.access.get_balance(address, path)
    }

    fn get_currency_info(&self, path: &CurrencyAccessPath) -> Option<CurrencyInfo> {
        self.access.get_currency_info(path)
    }

    fn make_coin_tag(&self, sample: &CurrencyAccessPath) -> Option<TypeTag> {
        self.native_mapper.borrow().iter().find_map(|(key, path)| {
            if let Some(path) = path {
                if path == sample {
                    return Some(TypeTag::Struct(key.to_owned()));
                }
            }
            None
        })
    }
}

pub struct MasterOfCoinSession<'b, 'r, B: BalanceAccess, R: RemoteCache> {
    master_of_coin: &'b MasterOfCoin<B>,
    balances: RefCell<HashMap<AccountAddress, HashMap<Cow<'static, CurrencyAccessPath>, Balance>>>,
    remote: &'r R,
}

impl<'b, 'r, B: BalanceAccess, R: RemoteCache> MasterOfCoinSession<'b, 'r, B, R> {
    fn get_bridge(&self, coin: &StructTag) -> Option<Vec<u8>> {
        self.master_of_coin.get_bridge(self.remote, coin)
    }

    fn get_balance(&self, address: &AccountAddress, path: Vec<u8>) -> Option<Vec<u8>> {
        let balance = self.master_of_coin.get_balance(address, &path)?;
        let mut balances = self.balances.borrow_mut();
        let acc = balances.entry(*address).or_insert_with(HashMap::new);
        acc.insert(Cow::Owned(path), balance);
        bcs::to_bytes(&DiemBalance { value: balance }).ok()
    }

    pub fn resolve(
        &self,
        address: &AccountAddress,
        tag: &StructTag,
    ) -> PartialVMResult<Option<Vec<u8>>> {
        if tag.module.as_ref() == ACCOUNT_MODULE_IDENTIFIER.as_ref() {
            if tag.name.as_str() == DIEM_BALANCE {
                let bridge = coin_type(&tag.type_params).and_then(|coin| self.get_bridge(coin));
                if let Some(bridge) = bridge {
                    return Ok(self.get_balance(address, bridge));
                }
            }
        } else if tag.module.as_ref() == DIEM_MODULE_IDENTIFIER.as_ref() {
            if tag.name.as_str() == CURRENCY_INFO {
                let bridge = coin_type(&tag.type_params).and_then(|coin| self.get_bridge(coin));
                if let Some(bridge) = bridge {
                    return Ok(self
                        .remote
                        .get_resource(address, tag)?
                        .and_then(|val| {
                            self.master_of_coin
                                .get_currency_info(&bridge)
                                .map(|info| (val, info))
                        })
                        .map(|(info, path)| path.apply(&info).unwrap()));
                }
            }
        }

        Ok(None)
    }

    pub fn finish(self, changes: &mut ChangeSet) -> VMResult<Vec<BalanceOp>> {
        let mut ops = vec![];
        let balances = self.balances.take();
        if !balances.is_empty() {
            let mut balance_tag = BALANCE_TEMPLATE.clone();
            for (acc, balances) in balances.into_iter() {
                if let Some(changes) = changes.accounts.get_mut(&acc) {
                    self.make_acc_ops(
                        &mut ops,
                        acc,
                        balances,
                        &mut changes.resources,
                        &mut balance_tag,
                    )?;
                }
            }
        }
        Ok(ops)
    }

    fn make_acc_ops(
        &self,
        ops: &mut Vec<BalanceOp>,
        acc: AccountAddress,
        balances: HashMap<Cow<'static, CurrencyAccessPath>, Balance>,
        resources: &mut BTreeMap<StructTag, Option<Vec<u8>>>,
        balance_tag: &mut StructTag,
    ) -> VMResult<()> {
        for (path, initial_balance) in balances {
            let tag = match self.master_of_coin.make_coin_tag(path.as_ref()) {
                None => continue,
                Some(tag) => tag,
            };
            balance_tag.type_params[0] = tag;

            match resources.remove(&balance_tag) {
                None => {
                    //There is no diff. Nothing to do.
                }
                Some(None) => {
                    // Drop resource.
                    ops.push(BalanceOp::Sub(acc, path, initial_balance));
                }
                Some(Some(buff)) => {
                    let current_balance = bcs::from_bytes::<DiemBalance>(&buff).map_err(|_| {
                        PartialVMError::new(StatusCode::INTERNAL_TYPE_ERROR)
                            .finish(Location::Undefined)
                    })?;
                    let current_balance = current_balance.value;
                    if current_balance == initial_balance {
                        continue;
                    }
                    if current_balance > initial_balance {
                        ops.push(BalanceOp::Add(acc, path, current_balance - initial_balance));
                    } else {
                        ops.push(BalanceOp::Sub(acc, path, initial_balance - current_balance));
                    }
                }
            }
        }
        Ok(())
    }
}

fn coin_type(t_params: &[TypeTag]) -> Option<&StructTag> {
    if t_params.len() != 1 {
        None
    } else {
        match &t_params[0] {
            TypeTag::Struct(tag) => Some(tag),
            _ => None,
        }
    }
}

fn native_currency(coin: &StructTag) -> StructTag {
    StructTag {
        address: CORE_CODE_ADDRESS,
        module: Identifier::new("NativeCurrencies").expect("Valid identifier"),
        name: Identifier::new("NativeCurrency").expect("Valid identifier"),
        type_params: vec![TypeTag::Struct(coin.to_owned())],
    }
}

#[derive(Debug, Serialize, Deserialize, Copy, Clone)]
pub struct CurrencyInfo {
    pub total_value: u128,
}

impl CurrencyInfo {
    pub fn apply(&self, buff: &[u8]) -> Result<Vec<u8>, Error> {
        let mut info = CurrencyInfoResource::try_from_bytes(buff)?;
        info.total_value = self.total_value;
        bcs::to_bytes(&info).map_err(Error::msg)
    }
}

pub enum BalanceOp {
    Add(AccountAddress, Cow<'static, CurrencyAccessPath>, Balance),
    Sub(AccountAddress, Cow<'static, CurrencyAccessPath>, Balance),
}

#[derive(Debug, Serialize, Deserialize, Copy, Clone)]
pub struct DiemBalance {
    pub value: Balance,
}
