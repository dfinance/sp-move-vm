use crate::io::key::{AccessKey, KeyType};
use crate::io::traits::Storage;
use alloc::vec::Vec;
use diem_types::access_path::AccessPath;
use diem_types::on_chain_config::ConfigStorage;

pub struct ConfigStore<'a, S: Storage> {
    store: &'a S,
}

impl<'a, S: Storage> From<&'a S> for ConfigStore<'a, S> {
    fn from(store: &'a S) -> Self {
        ConfigStore { store }
    }
}

impl<'a, S: Storage> ConfigStorage for ConfigStore<'a, S> {
    fn fetch_config(&self, access_path: AccessPath) -> Option<Vec<u8>> {
        self.store
            .get(AccessKey::new(access_path, KeyType::Resource).as_ref())
    }
}
