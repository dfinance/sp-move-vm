use crate::io::balance::MasterOfCoin;
use crate::io::context::ExecutionContext;
use crate::io::key::AccessKey;
use crate::io::session::StateSession;
use crate::io::traits::{BalanceAccess, Storage};
use alloc::vec::Vec;
use move_core_types::account_address::AccountAddress;
use move_core_types::language_storage::{ModuleId, StructTag};
use move_vm_runtime::data_cache::RemoteCache;
use vm::errors::{PartialVMResult, VMResult};

pub struct State<S: Storage> {
    store: S,
}

impl<S: Storage> State<S> {
    pub fn new(store: S) -> State<S> {
        State { store }
    }

    pub fn state_session<'c, B: BalanceAccess>(
        &self,
        context: Option<ExecutionContext>,
        master_of_coin: &'c MasterOfCoin<B>,
    ) -> StateSession<'c, '_, State<S>, B> {
        StateSession::new(&self, context, master_of_coin.session(&self))
    }
}

impl<S: Storage> RemoteCache for State<S> {
    fn get_module(&self, module_id: &ModuleId) -> VMResult<Option<Vec<u8>>> {
        Ok(self.store.get(AccessKey::from(module_id).as_ref()))
    }

    fn get_resource(
        &self,
        address: &AccountAddress,
        tag: &StructTag,
    ) -> PartialVMResult<Option<Vec<u8>>> {
        Ok(self.store.get(AccessKey::from((address, tag)).as_ref()))
    }
}

impl<S: Storage> WriteEffects for State<S> {
    fn delete(&self, key: AccessKey) {
        self.store.remove(key.as_ref());
    }

    fn insert(&self, key: AccessKey, blob: Vec<u8>) {
        self.store.insert(key.as_ref(), &blob);
    }
}

pub trait WriteEffects {
    fn delete(&self, path: AccessKey);
    fn insert(&self, path: AccessKey, blob: Vec<u8>);
}
