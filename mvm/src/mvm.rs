use anyhow::Error;

use move_core_types::gas_schedule::CostTable;
use move_core_types::gas_schedule::{AbstractMemorySize, GasAlgebra, GasUnits};
use move_core_types::vm_status::StatusCode;
use move_vm_runtime::data_cache::TransactionEffects;
use move_vm_runtime::logging::NoContextLog;
use move_vm_runtime::move_vm::MoveVM;
use move_vm_types::gas_schedule::CostStrategy;
use move_vm_types::natives::balance::BalanceOperation;
use vm::errors::{Location, PartialVMError, VMError};
use vm::CompiledModule;

use crate::access_path::AccessPath;
use crate::data::{
    BalanceAccess, Bank, EventHandler, ExecutionContext, Oracle, State, StateSession, Storage,
    WriteEffects,
};
use crate::types::{Gas, ModuleTx, ScriptTx, VmResult};
use crate::vm_config::loader::load_vm_config;
use crate::Vm;

/// MoveVM.
pub struct Mvm<S, E, O, B>
where
    S: Storage,
    E: EventHandler,
    O: Oracle,
    B: BalanceAccess,
{
    vm: MoveVM,
    cost_table: CostTable,
    state: State<S, O>,
    event_handler: E,
    bank: Bank<B>,
}

impl<S, E, O, B> Mvm<S, E, O, B>
where
    S: Storage,
    E: EventHandler,
    O: Oracle,
    B: BalanceAccess,
{
    /// Creates a new move vm with given store and event handler.
    pub fn new(
        store: S,
        event_handler: E,
        oracle: O,
        balance: B,
    ) -> Result<Mvm<S, E, O, B>, Error> {
        let config = load_vm_config(&store)?;

        Ok(Mvm {
            vm: MoveVM::new(),
            cost_table: config.gas_schedule,
            state: State::new(store, oracle),
            event_handler,
            bank: Bank::new(balance),
        })
    }

    /// Stores write set into storage and handle events.
    fn handle_tx_effects(&self, tx_effects: TransactionEffects) -> Result<(), VMError> {
        for (addr, vals) in tx_effects.resources {
            for (struct_tag, val_opt) in vals {
                let ap = AccessPath::new(addr, struct_tag.access_vector());
                match val_opt {
                    None => {
                        self.state.delete(ap);
                    }
                    Some((ty_layout, val)) => {
                        let blob = val.simple_serialize(&ty_layout).ok_or_else(|| {
                            PartialVMError::new(StatusCode::UNKNOWN_INVARIANT_VIOLATION_ERROR)
                                .finish(Location::Undefined)
                        })?;
                        self.state.insert(ap, blob);
                    }
                };
            }
        }

        for (module_id, blob) in tx_effects.modules {
            self.state.insert(
                AccessPath::new(*module_id.address(), module_id.access_vector()),
                blob,
            );
        }

        for (address, ty_tag, ty_layout, val, caller) in tx_effects.events {
            let msg = val.simple_serialize(&ty_layout).ok_or_else(|| {
                PartialVMError::new(StatusCode::UNKNOWN_INVARIANT_VIOLATION_ERROR)
                    .finish(Location::Undefined)
            })?;
            self.event_handler.on_event(address, ty_tag, msg, caller);
        }

        for (id, op) in tx_effects.wallet_ops.into_iter() {
            match op {
                BalanceOperation::Deposit(amount) => self.bank.deposit(&id, amount),
                BalanceOperation::Withdraw(amount) => self.bank.withdrawal(&id, amount),
            }
        }

        Ok(())
    }

    /// Handle vm result and return transaction status code.
    fn handle_vm_result(
        &self,
        cost_strategy: CostStrategy,
        gas_meta: Gas,
        result: Result<TransactionEffects, VMError>,
    ) -> VmResult {
        let gas_used = GasUnits::new(gas_meta.max_gas_amount)
            .sub(cost_strategy.remaining_gas())
            .get();

        match result.and_then(|e| self.handle_tx_effects(e)) {
            Ok(_) => VmResult::new(StatusCode::EXECUTED, None, gas_used),
            Err(err) => {
                //todo log error.
                VmResult::new(err.major_status(), err.sub_status(), gas_used)
            }
        }
    }
}

impl<S, E, O, B> Vm for Mvm<S, E, O, B>
where
    S: Storage,
    E: EventHandler,
    O: Oracle,
    B: BalanceAccess,
{
    fn publish_module(&self, gas: Gas, module: ModuleTx) -> VmResult {
        let (module, sender) = module.into_inner();

        let mut cost_strategy =
            CostStrategy::transaction(&self.cost_table, GasUnits::new(gas.max_gas_amount()));

        let result = cost_strategy
            .charge_intrinsic_gas(AbstractMemorySize::new(module.len() as u64))
            .and_then(|_| {
                CompiledModule::deserialize(&module)
                    .map_err(|e| e.finish(Location::Undefined))
                    .and_then(|compiled_module| {
                        let module_id = compiled_module.self_id();
                        if sender != *module_id.address() {
                            return Err(PartialVMError::new(
                                StatusCode::MODULE_ADDRESS_DOES_NOT_MATCH_SENDER,
                            )
                            .finish(Location::Module(module_id)));
                        }

                        cost_strategy
                            .charge_intrinsic_gas(AbstractMemorySize::new(module.len() as u64))?;

                        let mut session = self.vm.new_session(&self.state, &self.bank);
                        session
                            .publish_module(
                                module.to_vec(),
                                sender,
                                &mut cost_strategy,
                                &NoContextLog::new(),
                            )
                            .and_then(|_| session.finish())
                    })
            });
        self.handle_vm_result(cost_strategy, gas, result)
    }

    fn execute_script(&self, gas: Gas, context: ExecutionContext, tx: ScriptTx) -> VmResult {
        let state_session = StateSession::new(&self.state, context);
        let mut session = self.vm.new_session(&state_session, &self.bank);

        let (script, args, type_args, senders) = tx.into_inner();
        let mut cost_strategy =
            CostStrategy::transaction(&self.cost_table, GasUnits::new(gas.max_gas_amount()));

        let result = session
            .execute_script(
                script,
                type_args,
                args,
                senders,
                &mut cost_strategy,
                &NoContextLog::new(),
            )
            .and_then(|_| session.finish());

        self.handle_vm_result(cost_strategy, gas, result)
    }

    fn clear(&self) {
        self.vm.clear();
    }
}
