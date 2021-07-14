use crate::io::balance::{BalanceOp, MasterOfCoinSession};
use crate::io::context::ExecutionContext;
use crate::io::traits::BalanceAccess;
use alloc::vec::Vec;
use diem_types::account_config;
use move_core_types::account_address::AccountAddress;
use move_core_types::effects::{ChangeSet, Event};
use move_core_types::language_storage::{ModuleId, StructTag, CORE_CODE_ADDRESS};
use move_vm_runtime::data_cache::RemoteCache;
use vm::errors::{PartialVMResult, VMResult};

pub struct StateSession<'b, 'r, R: RemoteCache, B: BalanceAccess> {
    remote: &'r R,
    context: Option<ExecutionContext>,
    coin_session: MasterOfCoinSession<'b, 'r, B, R>,
}

impl<'b, 'r, R: RemoteCache, B: BalanceAccess> StateSession<'b, 'r, R, B> {
    pub(crate) fn new(
        remote: &'r R,
        context: Option<ExecutionContext>,
        coin_session: MasterOfCoinSession<'b, 'r, B, R>,
    ) -> StateSession<'b, 'r, R, B> {
        StateSession {
            remote,
            context,
            coin_session,
        }
    }

    pub fn finish(
        self,
        (mut changes, events): (ChangeSet, Vec<Event>),
    ) -> VMResult<(ChangeSet, Vec<Event>, Vec<BalanceOp>)> {
        let balance_op = self.coin_session.finish(&mut changes)?;
        Ok((changes, events, balance_op))
    }
}

impl<'b, 'r, R: RemoteCache, B: BalanceAccess> RemoteCache for StateSession<'b, 'r, R, B> {
    fn get_module(&self, module_id: &ModuleId) -> VMResult<Option<Vec<u8>>> {
        self.remote.get_module(module_id)
    }

    fn get_resource(
        &self,
        address: &AccountAddress,
        tag: &StructTag,
    ) -> PartialVMResult<Option<Vec<u8>>> {
        if tag.address == CORE_CODE_ADDRESS {
            if address == &account_config::diem_root_address() {
                if let Some(ctx) = &self.context {
                    if let Some(blob) = ctx.resolve(tag) {
                        return Ok(Some(blob));
                    }
                }
            }
            if let Some(blob) = self.coin_session.resolve(address, tag)? {
                return Ok(Some(blob));
            }
        }
        self.remote.get_resource(address, tag)
    }
}
