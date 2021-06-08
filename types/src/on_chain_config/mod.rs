// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    account_config::CORE_CODE_ADDRESS,
};
use anyhow::{format_err, Result};
use move_core_types::{
    identifier::Identifier,
    language_storage::{StructTag, TypeTag},
    move_resource::MoveResource,
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use hashbrown::HashMap;
use alloc::sync::Arc;
use alloc::fmt;
mod diem_version;
mod registered_currencies;
mod vm_config;
mod vm_publishing_option;

pub use self::{
    diem_version::{DiemVersion, DIEM_MAX_KNOWN_VERSION, DIEM_VERSION_2},
    registered_currencies::RegisteredCurrencies,
    vm_config::VMConfig,
    vm_publishing_option::VMPublishingOption,
};
use move_core_types::account_address::AccountAddress;
use alloc::vec::Vec;
use alloc::string::ToString;
use crate::access_path::AccessPath;

/// To register an on-chain config in Rust:
/// 1. Implement the `OnChainConfig` trait for the Rust representation of the config
/// 2. Add the config's `ConfigID` to `ON_CHAIN_CONFIG_REGISTRY`

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub struct ConfigID(&'static str, &'static str);

const CONFIG_ADDRESS_STR: &str = "0xA550C18";

pub fn config_address() -> AccountAddress {
    AccountAddress::from_hex_literal(CONFIG_ADDRESS_STR).expect("failed to get address")
}

impl ConfigID {
    pub fn access_path(self) -> AccessPath {
        access_path_for_config(
            AccountAddress::from_hex_literal(self.0).expect("failed to get address"),
            Identifier::new(self.1).expect("failed to get Identifier"),
        )
    }
}

impl fmt::Display for ConfigID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "OnChain config ID [address: {}, identifier: {}]",
            self.0, self.1
        )
    }
}

/// State sync will panic if the value of any config in this registry is uninitialized
pub const ON_CHAIN_CONFIG_REGISTRY: &[ConfigID] = &[
    VMConfig::CONFIG_ID,
    VMPublishingOption::CONFIG_ID,
    DiemVersion::CONFIG_ID,
    RegisteredCurrencies::CONFIG_ID,
];

#[derive(Clone, Debug, PartialEq)]
pub struct OnChainConfigPayload {
    epoch: u64,
    configs: Arc<HashMap<ConfigID, Vec<u8>>>,
}

impl OnChainConfigPayload {
    pub fn new(epoch: u64, configs: Arc<HashMap<ConfigID, Vec<u8>>>) -> Self {
        Self { epoch, configs }
    }

    pub fn epoch(&self) -> u64 {
        self.epoch
    }

    pub fn get<T: OnChainConfig>(&self) -> Result<T> {
        let bytes = self
            .configs
            .get(&T::CONFIG_ID)
            .ok_or_else(|| format_err!("[on-chain cfg] config not in payload"))?;
        T::deserialize_into_config(bytes)
    }

    pub fn configs(&self) -> &HashMap<ConfigID, Vec<u8>> {
        &self.configs
    }
}

impl fmt::Display for OnChainConfigPayload {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut config_ids = "".to_string();
        for id in self.configs.keys() {
            config_ids += &id.to_string();
        }
        write!(
            f,
            "OnChainConfigPayload [epoch: {}, configs: {}]",
            self.epoch, config_ids
        )
    }
}

/// Trait to be implemented by a storage type from which to read on-chain configs
pub trait ConfigStorage {
    fn fetch_config(&self, access_path: AccessPath) -> Option<Vec<u8>>;
}

/// Trait to be implemented by a Rust struct representation of an on-chain config
/// that is stored in storage as a serialized byte array
pub trait OnChainConfig: Send + Sync + DeserializeOwned {
    // diem_root_address
    const ADDRESS: &'static str = CONFIG_ADDRESS_STR;
    const IDENTIFIER: &'static str;
    const CONFIG_ID: ConfigID = ConfigID(Self::ADDRESS, Self::IDENTIFIER);

    // Single-round BCS deserialization from bytes to `Self`
    // This is the expected deserialization pattern for most Rust representations,
    // but sometimes `deserialize_into_config` may need an extra customized round of deserialization
    // (e.g. enums like `VMPublishingOption`)
    // In the override, we can reuse this default logic via this function
    // Note: we cannot directly call the default `deserialize_into_config` implementation
    // in its override - this will just refer to the override implementation itself
    fn deserialize_default_impl(bytes: &[u8]) -> Result<Self> {
        bcs::from_bytes::<Self>(&bytes)
            .map_err(|e| format_err!("[on-chain config] Failed to deserialize into config: {}", e))
    }

    // Function for deserializing bytes to `Self`
    // It will by default try one round of BCS deserialization directly to `Self`
    // The implementation for the concrete type should override this function if this
    // logic needs to be customized
    fn deserialize_into_config(bytes: &[u8]) -> Result<Self> {
        Self::deserialize_default_impl(bytes)
    }

    fn fetch_config<T>(storage: &T) -> Option<Self>
    where
        T: ConfigStorage,
    {
        storage
            .fetch_config(Self::CONFIG_ID.access_path())
            .and_then(|bytes| Self::deserialize_into_config(&bytes).ok())
    }
}


pub fn access_path_for_config(address: AccountAddress, config_name: Identifier) -> AccessPath {
    AccessPath::new(
        address,
        AccessPath::resource_access_vec(StructTag {
            address: CORE_CODE_ADDRESS,
            module: Identifier::new("DiemConfig").unwrap(),
            name: Identifier::new("DiemConfig").unwrap(),
            type_params: vec![TypeTag::Struct(StructTag {
                address: CORE_CODE_ADDRESS,
                module: config_name.clone(),
                name: config_name,
                type_params: vec![],
            })],
        }),
    )
}
