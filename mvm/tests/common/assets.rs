use std::convert::TryFrom;

use serde::Deserialize;

use diem_types::account_config::treasury_compliance_account_address;
use move_core_types::account_address::AccountAddress;
use move_core_types::language_storage::CORE_CODE_ADDRESS;
use mvm::io::traits::Balance;
use mvm::types::{Gas, ModulePackage, ModuleTx, ScriptArg, ScriptTx};

pub fn gas() -> Gas {
    Gas::new(10_000, 1).unwrap()
}

pub fn store_module() -> ModuleTx {
    ModuleTx::new(
        include_bytes!("../assets/artifacts/modules/Store.mv").to_vec(),
        CORE_CODE_ADDRESS,
    )
}

pub fn event_proxy_module() -> ModuleTx {
    ModuleTx::new(
        include_bytes!("../assets/artifacts/modules/EventProxy.mv").to_vec(),
        CORE_CODE_ADDRESS,
    )
}

pub fn abort_module() -> ModuleTx {
    ModuleTx::new(
        include_bytes!("../assets/artifacts/modules/Abort.mv").to_vec(),
        CORE_CODE_ADDRESS,
    )
}

pub fn emit_event_script(addr: AccountAddress, args: u64) -> ScriptTx {
    ScriptTx::new(
        include_bytes!("../assets/artifacts/scripts/emit_event.mv").to_vec(),
        vec![ScriptArg::U64(args)],
        vec![],
        vec![addr],
    )
    .unwrap()
}

pub fn store_sys_resources_script(
    addr_for_block: AccountAddress,
    addr_for_timestamp: AccountAddress,
) -> ScriptTx {
    ScriptTx::new(
        include_bytes!("../assets/artifacts/scripts/store_system_resources.mv").to_vec(),
        vec![],
        vec![],
        vec![addr_for_block, addr_for_timestamp],
    )
    .unwrap()
}

pub fn store_u64_script(addr: AccountAddress, args: u64) -> ScriptTx {
    ScriptTx::new(
        include_bytes!("../assets/artifacts/scripts/store_u64.mv").to_vec(),
        vec![ScriptArg::U64(args)],
        vec![],
        vec![addr],
    )
    .unwrap()
}

// pub fn test_transfer_script(alice: AccountAddress, bob: AccountAddress, amount: u128) -> ScriptTx {
//     ScriptTx::new(
//         include_bytes!("../assets/target/scripts/test_balance_transfer.mv").to_vec(),
//         vec![ScriptArg::U128(amount)],
//         vec![],
//         vec![alice, bob],
//     )
//     .unwrap()
// }

pub fn pont_info_script(address: AccountAddress, total: u128) -> ScriptTx {
    ScriptTx::new(
        include_bytes!("../assets/artifacts/scripts/pont_info.mv").to_vec(),
        vec![ScriptArg::U128(total)],
        vec![],
        vec![address],
    )
    .unwrap()
}

pub fn error_script(addr: AccountAddress) -> ScriptTx {
    ScriptTx::new(
        include_bytes!("../assets/artifacts/scripts/error.mv").to_vec(),
        vec![],
        vec![],
        vec![addr],
    )
    .unwrap()
}

pub fn create_root_account_script(addr: AccountAddress) -> ScriptTx {
    ScriptTx::new(
        include_bytes!("../assets/artifacts/scripts/make_root_account.mv").to_vec(),
        vec![ScriptArg::Address(addr)],
        vec![],
        vec![treasury_compliance_account_address()],
    )
    .unwrap()
}

pub fn create_account_script(root: AccountAddress, addr: AccountAddress) -> ScriptTx {
    ScriptTx::new(
        include_bytes!("../assets/artifacts/scripts/make_account.mv").to_vec(),
        vec![ScriptArg::Address(addr)],
        vec![],
        vec![root],
    )
    .unwrap()
}

pub fn transfer_script(
    from: AccountAddress,
    from_balance: Balance,
    to: AccountAddress,
    to_balance: Balance,
    to_move: Balance,
) -> ScriptTx {
    ScriptTx::new(
        include_bytes!("../assets/artifacts/scripts/transfer.mv").to_vec(),
        vec![
            ScriptArg::U64(from_balance),
            ScriptArg::U64(to_balance),
            ScriptArg::U64(to_move),
        ],
        vec![],
        vec![from, to],
    )
    .unwrap()
}

pub fn valid_package() -> ModulePackage {
    ModulePackage::try_from(&include_bytes!("../assets/artifacts/bundles/valid_pack.pac")[..])
        .unwrap()
}

pub fn invalid_package() -> ModulePackage {
    ModulePackage::try_from(&include_bytes!("../assets/artifacts/bundles/invalid_pack.pac")[..])
        .unwrap()
}

#[derive(Deserialize)]
pub struct StoreU64 {
    pub val: u64,
}

#[derive(Deserialize)]
pub struct StoreU128 {
    pub val: u128,
}
