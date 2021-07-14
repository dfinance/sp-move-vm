use core::convert::TryFrom;
use move_core_types::identifier::Identifier;
use move_core_types::language_storage::{StructTag, TypeTag, CORE_CODE_ADDRESS};
use move_core_types::value::MoveValue;
use mvm::types::{parse_type_params, ModulePackage, Transaction};
use vm::access::ModuleAccess;
use vm::file_format::CompiledScript;
use vm::CompiledModule;

#[test]
fn test_parse_type_params() {
    test(
        "0x01::Token::BTC",
        vec![TypeTag::Struct(StructTag {
            address: CORE_CODE_ADDRESS,
            module: Identifier::new("Token").unwrap(),
            name: Identifier::new("BTC").unwrap(),
            type_params: vec![],
        })],
    );

    test(
        "0x01::Token::BTC,u8",
        vec![
            TypeTag::Struct(StructTag {
                address: CORE_CODE_ADDRESS,
                module: Identifier::new("Token").unwrap(),
                name: Identifier::new("BTC").unwrap(),
                type_params: vec![],
            }),
            TypeTag::U8,
        ],
    );

    test(
        "0x01::Token::BTC;0x01::Balance::Acc<0x01::Foo::Bar>",
        vec![
            TypeTag::Struct(StructTag {
                address: CORE_CODE_ADDRESS,
                module: Identifier::new("Token").unwrap(),
                name: Identifier::new("BTC").unwrap(),
                type_params: vec![],
            }),
            TypeTag::Struct(StructTag {
                address: CORE_CODE_ADDRESS,
                module: Identifier::new("Balance").unwrap(),
                name: Identifier::new("Acc").unwrap(),
                type_params: vec![TypeTag::Struct(StructTag {
                    address: CORE_CODE_ADDRESS,
                    module: Identifier::new("Foo").unwrap(),
                    name: Identifier::new("Bar").unwrap(),
                    type_params: vec![],
                })],
            }),
        ],
    );

    test("", vec![]);

    fn test(tp: &str, expected: Vec<TypeTag>) {
        assert_eq!(parse_type_params(tp).unwrap(), expected);
    }
}

#[test]
fn test_parse_transaction() {
    let tx =
        Transaction::try_from(&include_bytes!("assets/artifacts/transactions/tx_test.mvt")[..])
            .unwrap();
    assert_eq!(tx.signers_count(), 0);
    let script = tx.into_script(vec![]).unwrap();
    CompiledScript::deserialize(script.code()).unwrap();
    assert_eq!(
        script.args(),
        &[MoveValue::U64(100).simple_serialize().unwrap()][..]
    );
    assert_eq!(
        script.type_parameters(),
        &[TypeTag::Struct(StructTag {
            address: CORE_CODE_ADDRESS,
            module: Identifier::new("Pontem").unwrap(),
            name: Identifier::new("T").unwrap(),
            type_params: vec![],
        })][..]
    )
}

#[test]
fn test_parse_mvt() {
    let tx =
        Transaction::try_from(&include_bytes!("assets/artifacts/transactions/store_u64.mvt")[..])
            .unwrap();
    assert_eq!(tx.signers_count(), 1);
    let script = tx.into_script(vec![CORE_CODE_ADDRESS]).unwrap();
    CompiledScript::deserialize(script.code()).unwrap();
    assert_eq!(
        script.args(),
        &[MoveValue::U64(13).simple_serialize().unwrap()][..]
    );
    assert_eq!(script.type_parameters(), &[][..])
}

#[test]
#[should_panic]
fn test_transaction_invalid_signer() {
    let tx = hex::decode("015fa11ceb0b0100000005010002030205050705070c10081c200000000100010002060c03000553746f72650973746f72655f7536340000000000000000000000000000000000000000000000000000000000000001000001040b000a01110002010164000000000000000107200000000000000000000000000000000000000000000000000000000000000001084466696e616e6365015400").unwrap();
    let tx = Transaction::try_from(tx.as_ref()).unwrap();
    assert_eq!(tx.signers_count(), 1);
    tx.into_script(vec![CORE_CODE_ADDRESS, CORE_CODE_ADDRESS])
        .unwrap();
}

#[test]
fn test_parse_pac() {
    let pac =
        ModulePackage::try_from(&include_bytes!("assets/artifacts/bundles/valid_pack.pac")[..])
            .unwrap();
    let tx = pac.into_tx(CORE_CODE_ADDRESS);
    let (modules, address) = tx.into_inner();

    assert_eq!(address, CORE_CODE_ADDRESS);

    let modules = modules
        .iter()
        .map(|module| CompiledModule::deserialize(&module).unwrap())
        .map(|module| module.name().to_string())
        .collect::<Vec<_>>();
    assert_eq!(
        modules.iter().map(|s| s.as_str()).collect::<Vec<_>>(),
        vec!["Abort", "EventProxy", "Store", "Foo"]
    );
}
