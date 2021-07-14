#[macro_use]
extern crate alloc;

use common::mock::Utils;
use common::{assets::*, contains_core_module, mock::*, vm};
use diem_types::event::EventKey;
use move_core_types::account_address::AccountAddress;
use move_core_types::identifier::Identifier;
use move_core_types::language_storage::{ModuleId, StructTag, TypeTag, CORE_CODE_ADDRESS};
use move_core_types::vm_status::{AbortLocation, StatusCode, VMStatus};
use move_vm_runtime::data_cache::RemoteCache;
use mvm::io::balance::CurrencyInfo;
use mvm::io::context::ExecutionContext;
use mvm::io::state::State;
use mvm::io::traits::BalanceAccess;
use mvm::types::Gas;
use mvm::Vm;

mod common;

#[test]
fn test_public_module() {
    let (vm, store, _, _) = vm();
    let state = State::new(store);

    vm.pub_mod(store_module());

    let store_module_id = ModuleId::new(CORE_CODE_ADDRESS, Identifier::new("Store").unwrap());

    assert_eq!(
        &state.get_module(&store_module_id).unwrap().unwrap(),
        store_module().code()
    );
}

#[test]
fn test_public_module_without_gas() {
    let (vm, _, _, _) = vm();
    let gas = Gas::new(1, 1).unwrap();
    let res = vm.publish_module(gas, store_module(), false);
    assert_eq!(res.status_code, StatusCode::OUT_OF_GAS);
}

#[test]
fn test_execute_script() {
    let test_value = 13;

    let (vm, store, _, _) = vm();
    let state = State::new(store);

    vm.pub_mod(store_module());

    vm.exec(store_u64_script(addr("0x1"), test_value));

    let tag = StructTag {
        address: CORE_CODE_ADDRESS,
        module: Identifier::new("Store").unwrap(),
        name: Identifier::new("U64").unwrap(),
        type_params: vec![],
    };
    let blob = state
        .get_resource(&CORE_CODE_ADDRESS, &tag)
        .unwrap()
        .unwrap();
    let store: StoreU64 = bcs::from_bytes(&blob).unwrap();
    assert_eq!(test_value, store.val);
}

#[test]
fn test_store_event() {
    let test_value = 13;

    let (vm, _, event, _) = vm();
    vm.pub_mod(event_proxy_module());

    vm.exec(emit_event_script(addr("0x1"), test_value));

    let (guid, seq, tag, msg) = event.data.borrow_mut().remove(0);
    assert_eq!(
        guid,
        vec![
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1
        ]
    );
    assert_eq!(seq, 0);
    assert_eq!(test_value, bcs::from_bytes::<StoreU64>(&msg).unwrap().val);
    assert_eq!(
        TypeTag::Struct(StructTag {
            address: CORE_CODE_ADDRESS,
            module: Identifier::new("EventProxy").unwrap(),
            name: Identifier::new("U64").unwrap(),
            type_params: vec![],
        }),
        tag
    );
}

#[test]
fn test_load_system_resources() {
    let (vm, store, _, _) = vm();
    let state = State::new(store);
    vm.pub_mod(store_module());

    let block = 1000;
    let timestamp = 10;

    vm.exec_with_context(
        ExecutionContext::new(timestamp, block),
        store_sys_resources_script(addr("0x1"), addr("0x2")),
    );

    let tag = StructTag {
        address: CORE_CODE_ADDRESS,
        module: Identifier::new("Store").unwrap(),
        name: Identifier::new("U64").unwrap(),
        type_params: vec![],
    };
    let blob = state.get_resource(&addr("0x1"), &tag).unwrap().unwrap();
    let store: StoreU64 = bcs::from_bytes(&blob).unwrap();
    assert_eq!(store.val, block);

    let blob = state.get_resource(&addr("0x2"), &tag).unwrap().unwrap();
    let store: StoreU64 = bcs::from_bytes(&blob).unwrap();
    assert_eq!(store.val, timestamp);
}

#[test]
fn test_error_event() {
    let (vm, _, events, _) = vm();
    vm.pub_mod(abort_module());
    let sender = AccountAddress::random();
    vm.execute_script(
        gas(),
        ExecutionContext::new(0, 0),
        error_script(sender),
        false,
    );

    let (guid, seq, tag, msg) = events.pop().unwrap();
    let key = EventKey::from_bytes(&guid).unwrap();
    assert_eq!(sender, key.get_creator_address());
    assert_eq!(0, key.get_creation_number());
    assert_eq!(0, seq);

    assert_eq!(
        TypeTag::Struct(StructTag {
            address: CORE_CODE_ADDRESS,
            module: Identifier::new("VMStatus").unwrap(),
            name: Identifier::new("VMStatus").unwrap(),
            type_params: vec![],
        }),
        tag
    );
    let status = bcs::from_bytes::<VMStatus>(&msg).unwrap();
    assert_eq!(
        status,
        VMStatus::MoveAbort(
            AbortLocation::Module(ModuleId::new(
                CORE_CODE_ADDRESS,
                Identifier::new("Abort").unwrap(),
            )),
            13,
        )
    );
}

#[test]
fn test_publish_pac() {
    let (vm, state, _, _) = vm();
    let state = State::new(state);

    let pac = valid_package().into_tx(CORE_CODE_ADDRESS);

    let res = vm.publish_module_package(gas(), pac, false);
    if res.status_code != StatusCode::EXECUTED {
        panic!("Transaction failed: {:?}", res);
    }

    contains_core_module(&state, "Abort");
    contains_core_module(&state, "EventProxy");
    contains_core_module(&state, "Foo");
    contains_core_module(&state, "Store");
}

#[test]
fn test_invalid_pac() {
    let (vm, _, _, _) = vm();
    let pac = invalid_package().into_tx(CORE_CODE_ADDRESS);
    let res = vm.publish_module_package(gas(), pac, false);
    assert_eq!(res.status_code, StatusCode::LINKER_ERROR);
}

#[test]
fn pont_info() {
    let (vm, _, _, bank) = vm();
    bank.set_currency_info("PONT".as_bytes(), CurrencyInfo { total_value: 42 });

    vm.exec(pont_info_script(CORE_CODE_ADDRESS, 42));
}

#[test]
fn test_balance() {
    let (vm, _, _, bank) = vm();
    bank.set_currency_info("PONT".as_bytes(), CurrencyInfo { total_value: 1001 });
    let root = AccountAddress::random();
    vm.exec(create_root_account_script(root));

    let alice = AccountAddress::random();
    let alice_balance = 1000;
    vm.exec(create_account_script(root, alice));

    let move_to_bob = 500;

    let bob = AccountAddress::random();
    let bob_balance = 1;
    vm.exec(create_account_script(root, bob));

    bank.set_balance(&alice, "PONT".as_bytes(), alice_balance);
    bank.set_balance(&bob, "PONT".as_bytes(), bob_balance);

    vm.exec(transfer_script(
        alice,
        alice_balance,
        bob,
        bob_balance,
        move_to_bob,
    ));

    assert_eq!(
        bank.get_balance(&alice, "PONT".as_bytes()),
        Some(alice_balance - move_to_bob)
    );
    assert_eq!(
        bank.get_balance(&bob, "PONT".as_bytes()),
        Some(bob_balance + move_to_bob)
    );

    let move_to_alice = 42;
    vm.exec(transfer_script(
        bob,
        bob_balance + move_to_bob,
        alice,
        alice_balance - move_to_bob,
        move_to_alice,
    ));

    assert_eq!(
        bank.get_balance(&alice, "PONT".as_bytes()),
        Some(alice_balance - move_to_bob + move_to_alice)
    );
    assert_eq!(
        bank.get_balance(&bob, "PONT".as_bytes()),
        Some(bob_balance + move_to_bob - move_to_alice)
    );
}
