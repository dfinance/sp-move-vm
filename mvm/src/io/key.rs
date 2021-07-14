use alloc::vec::Vec;
use diem_types::access_path::AccessPath;
use move_core_types::account_address::AccountAddress;
use move_core_types::language_storage::{ModuleId, StructTag};

pub struct AccessKey(Vec<u8>);

pub enum KeyType {
    Resource,
    Module,
}

impl AccessKey {
    pub fn new(path: AccessPath, k_type: KeyType) -> AccessKey {
        match k_type {
            KeyType::Resource => {
                let mut key = Vec::with_capacity(AccountAddress::LENGTH + path.path.len());
                key.extend_from_slice(path.address.as_ref());
                key.extend_from_slice(path.path.as_ref());
                AccessKey(key)
            }
            KeyType::Module => AccessKey(path.path),
        }
    }
}

impl From<(&AccountAddress, &StructTag)> for AccessKey {
    fn from((addr, tag): (&AccountAddress, &StructTag)) -> Self {
        let tag = tag.access_vector();
        let mut key = Vec::with_capacity(AccountAddress::LENGTH + tag.len());
        key.extend_from_slice(addr.as_ref());
        key.extend_from_slice(&tag);
        AccessKey(key)
    }
}

impl From<&ModuleId> for AccessKey {
    fn from(id: &ModuleId) -> Self {
        AccessKey(id.access_vector())
    }
}

impl AsRef<[u8]> for AccessKey {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}
