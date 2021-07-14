use mvm::genesis::{init_storage, GenesisConfig};

use crate::common::contains_core_module;
use crate::common::mock::StorageMock;
use mvm::io::state::State;

mod common;

#[test]
fn test_genesis_success() {
    let store = StorageMock::new();
    init_storage(store.clone(), GenesisConfig::default()).unwrap();
    let state = State::new(store);

    contains_core_module(&state, "PONT");
    contains_core_module(&state, "Signer");
    contains_core_module(&state, "U256");
    contains_core_module(&state, "Errors");
    contains_core_module(&state, "CoreAddresses");
    contains_core_module(&state, "DiemTimestamp");
    contains_core_module(&state, "SlidingNonce");
    contains_core_module(&state, "Signature");
    contains_core_module(&state, "Roles");
    contains_core_module(&state, "FixedPoint32");
    contains_core_module(&state, "Vector");
    contains_core_module(&state, "BCS");
    contains_core_module(&state, "Event");
    contains_core_module(&state, "DiemConfig");
    contains_core_module(&state, "RegisteredCurrencies");
    contains_core_module(&state, "Diem");
    contains_core_module(&state, "AccountLimits");
    contains_core_module(&state, "ValidatorOperatorConfig");
    contains_core_module(&state, "Option");
    contains_core_module(&state, "ValidatorConfig");
    contains_core_module(&state, "VASP");
    contains_core_module(&state, "TransactionFee");
    contains_core_module(&state, "Hash");
    contains_core_module(&state, "DualAttestation");
    contains_core_module(&state, "DiemTransactionPublishingOption");
    contains_core_module(&state, "DesignatedDealer");
    contains_core_module(&state, "ChainId");
    contains_core_module(&state, "AccountFreezing");
    contains_core_module(&state, "DiemAccount");
    contains_core_module(&state, "Authenticator");
    contains_core_module(&state, "SharedEd25519PublicKey");
    contains_core_module(&state, "RecoveryAddress");
    contains_core_module(&state, "AccountAdministrationScripts");
    contains_core_module(&state, "AccountCreationScripts");
    contains_core_module(&state, "DiemSystem");
    contains_core_module(&state, "DiemBlock");
    contains_core_module(&state, "DiemConsensusConfig");
    contains_core_module(&state, "DiemVMConfig");
    contains_core_module(&state, "DiemVersion");
    contains_core_module(&state, "Genesis");
    contains_core_module(&state, "PaymentScripts");
    contains_core_module(&state, "SystemAdministrationScripts");
    contains_core_module(&state, "TreasuryComplianceScripts");
    contains_core_module(&state, "ValidatorAdministrationScripts");
}
