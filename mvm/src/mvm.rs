use alloc::vec::Vec;

use anyhow::{anyhow, Error};

use diem_types::event::EventKey;
use diem_types::on_chain_config::{OnChainConfig, VMConfig};
use move_core_types::account_address::AccountAddress;
use move_core_types::effects::{ChangeSet, Event};
use move_core_types::gas_schedule::CostTable;
use move_core_types::gas_schedule::InternalGasUnits;
use move_core_types::gas_schedule::{AbstractMemorySize, GasAlgebra, GasUnits};
use move_core_types::identifier::{IdentStr, Identifier};
use move_core_types::language_storage::{ModuleId, StructTag, TypeTag, CORE_CODE_ADDRESS};
use move_core_types::vm_status::{StatusCode, VMStatus};
use move_vm_runtime::data_cache::RemoteCache;
use move_vm_runtime::logging::NoContextLog;
use move_vm_runtime::move_vm::MoveVM;
use move_vm_runtime::session::Session;
use move_vm_types::gas_schedule::CostStrategy;
use vm::errors::{Location, VMError, VMResult};

use crate::io::balance::{BalanceOp, MasterOfCoin};
use crate::io::config::ConfigStore;
use crate::io::context::ExecutionContext;
use crate::io::key::AccessKey;
use crate::io::state::{State, WriteEffects};
use crate::io::traits::{BalanceAccess, EventHandler, Storage};
use crate::types::{Gas, ModuleTx, PublishPackageTx, ScriptTx, VmResult};
use crate::Vm;

/// MoveVM.
pub struct Mvm<S, E, B>
where
    S: Storage,
    E: EventHandler,
    B: BalanceAccess,
{
    vm: MoveVM,
    cost_table: CostTable,
    state: State<S>,
    event_handler: E,
    master_of_coin: MasterOfCoin<B>,
}

impl<S, E, B> Mvm<S, E, B>
where
    S: Storage,
    E: EventHandler,
    B: BalanceAccess,
{
    /// Creates a new move vm with given store and event handler.
    pub fn new(store: S, event_handler: E, balance: B) -> Result<Mvm<S, E, B>, Error> {
        let config = VMConfig::fetch_config(&ConfigStore::from(&store))
            .ok_or_else(|| anyhow!("Failed to load VMConfig."))?;
        Self::new_with_config(store, event_handler, balance, config)
    }

    pub(crate) fn new_with_config(
        store: S,
        event_handler: E,
        balance: B,
        config: VMConfig,
    ) -> Result<Mvm<S, E, B>, Error> {
        Ok(Mvm {
            vm: MoveVM::new(),
            cost_table: config.gas_schedule,
            state: State::new(store),
            event_handler,
            master_of_coin: MasterOfCoin::new(balance),
        })
    }

    pub(crate) fn execute_function(
        &self,
        sender: AccountAddress,
        gas: Gas,
        module: &ModuleId,
        function_name: &IdentStr,
        ty_args: Vec<TypeTag>,
        args: Vec<Vec<u8>>,
        context: Option<ExecutionContext>,
    ) -> VmResult {
        let state_session = self.state.state_session(context, &self.master_of_coin);
        let mut session = self.vm.new_session(&state_session);
        let mut cost_strategy =
            CostStrategy::transaction(&self.cost_table, GasUnits::new(gas.max_gas_amount()));

        let result = session.execute_function(
            module,
            function_name,
            ty_args,
            args,
            &mut cost_strategy,
            &NoContextLog::new(),
        );

        self.handle_vm_result(
            sender,
            cost_strategy,
            gas,
            result.and_then(|_| session.finish().map(|(ws, e)| (ws, e, vec![]))),
            false,
        )
    }

    /// Stores write set into storage and handle events.
    fn handle_tx_effects(
        &self,
        tx_effects: (ChangeSet, Vec<Event>, Vec<BalanceOp>),
    ) -> Result<(), VMError> {
        let (change_set, events, balance_op) = tx_effects;

        for (addr, acc) in change_set.accounts {
            for (ident, val) in acc.modules {
                let key = AccessKey::from(&ModuleId::new(addr, ident));
                match val {
                    None => {
                        self.state.delete(key);
                    }
                    Some(blob) => {
                        self.state.insert(key, blob);
                    }
                }
            }
            for (tag, val) in acc.resources {
                let key = AccessKey::from((&addr, &tag));
                match val {
                    None => {
                        self.state.delete(key);
                    }
                    Some(blob) => {
                        self.state.insert(key, blob);
                    }
                }
            }
        }

        for (guid, seq_num, ty_tag, msg) in events {
            self.event_handler.on_event(guid, seq_num, ty_tag, msg);
        }

        for op in balance_op.into_iter() {
            self.master_of_coin.update_balance(op);
        }

        Ok(())
    }

    /// Handle vm result and return transaction status code.
    fn handle_vm_result(
        &self,
        sender: AccountAddress,
        cost_strategy: CostStrategy,
        gas_meta: Gas,
        result: Result<(ChangeSet, Vec<Event>, Vec<BalanceOp>), VMError>,
        dry_run: bool,
    ) -> VmResult {
        let gas_used = GasUnits::new(gas_meta.max_gas_amount)
            .sub(cost_strategy.remaining_gas())
            .get();

        if dry_run {
            return match result {
                Ok(_) => VmResult::new(StatusCode::EXECUTED, None, None, gas_used),
                Err(err) => VmResult::new(
                    err.major_status(),
                    err.sub_status(),
                    Some(err.location().clone()),
                    gas_used,
                ),
            };
        }

        match result.and_then(|e| self.handle_tx_effects(e)) {
            Ok(_) => VmResult::new(StatusCode::EXECUTED, None, None, gas_used),
            Err(err) => {
                let status = err.major_status();
                let sub_status = err.sub_status();
                let loc = err.location().clone();
                if let Err(err) = self.emit_vm_status_event(sender, err.into_vm_status()) {
                    log::warn!("Failed to emit vm status event:{:?}", err);
                }
                VmResult::new(status, sub_status, Some(loc), gas_used)
            }
        }
    }

    fn emit_vm_status_event(&self, sender: AccountAddress, status: VMStatus) -> Result<(), Error> {
        let tag = TypeTag::Struct(StructTag {
            address: CORE_CODE_ADDRESS,
            module: Identifier::new("VMStatus").unwrap(),
            name: Identifier::new("VMStatus").unwrap(),
            type_params: vec![],
        });

        let msg = bcs::to_bytes(&status)
            .map_err(|err| Error::msg(format!("Failed to generate event message: {:?}", err)))?;

        let guid = EventKey::new_from_address(&sender, 0).to_vec();
        self.event_handler.on_event(guid, 0, tag, msg);
        Ok(())
    }

    fn _publish_module<R>(
        &self,
        session: &mut Session<'_, '_, R>,
        module: Vec<u8>,
        sender: AccountAddress,
        cost_strategy: &mut CostStrategy,
    ) -> VMResult<()>
    where
        R: RemoteCache,
    {
        cost_strategy.charge_intrinsic_gas(AbstractMemorySize::new(module.len() as u64))?;

        let result = session.publish_module(module, sender, cost_strategy, &NoContextLog::new());
        Self::charge_global_write_gas_usage(cost_strategy, session, &sender)?;
        result
    }

    fn charge_global_write_gas_usage<R>(
        cost_strategy: &mut CostStrategy,
        session: &mut Session<'_, '_, R>,
        sender: &AccountAddress,
    ) -> VMResult<()>
    where
        R: RemoteCache,
    {
        let total_cost = session.num_mutated_accounts(sender)
            * cost_strategy
                .cost_table()
                .gas_constants
                .global_memory_per_byte_write_cost
                .mul(
                    cost_strategy
                        .cost_table()
                        .gas_constants
                        .default_account_size,
                )
                .get();
        cost_strategy
            .deduct_gas(InternalGasUnits::new(total_cost))
            .map_err(|p_err| p_err.finish(Location::Undefined))
    }
}

impl<S, E, B> Vm for Mvm<S, E, B>
where
    S: Storage,
    E: EventHandler,
    B: BalanceAccess,
{
    fn publish_module(&self, gas: Gas, module: ModuleTx, dry_run: bool) -> VmResult {
        let (module, sender) = module.into_inner();
        let mut cost_strategy =
            CostStrategy::transaction(&self.cost_table, GasUnits::new(gas.max_gas_amount()));
        let mut session = self.vm.new_session(&self.state);

        let result = self
            ._publish_module(&mut session, module, sender, &mut cost_strategy)
            .and_then(|_| session.finish().map(|(ws, e)| (ws, e, vec![])));

        self.handle_vm_result(sender, cost_strategy, gas, result, dry_run)
    }

    fn publish_module_package(
        &self,
        gas: Gas,
        package: PublishPackageTx,
        dry_run: bool,
    ) -> VmResult {
        let (modules, sender) = package.into_inner();
        let mut cost_strategy =
            CostStrategy::transaction(&self.cost_table, GasUnits::new(gas.max_gas_amount()));

        // We need to create a new vm to publish module packages.
        // Because during batch publishing, the cache mutates.
        // This is not the correct behavior for the dry_run case or for rolling back a transaction.
        let vm = MoveVM::new();
        let mut session = vm.new_session(&self.state);

        for module in modules {
            if let Err(err) = self._publish_module(&mut session, module, sender, &mut cost_strategy)
            {
                return self.handle_vm_result(sender, cost_strategy, gas, Err(err), dry_run);
            }
        }
        self.handle_vm_result(
            sender,
            cost_strategy,
            gas,
            session.finish().map(|(ws, e)| (ws, e, vec![])),
            dry_run,
        )
    }

    fn execute_script(
        &self,
        gas: Gas,
        context: ExecutionContext,
        tx: ScriptTx,
        dry_run: bool,
    ) -> VmResult {
        let state_session = self
            .state
            .state_session(Some(context), &self.master_of_coin);
        let mut vm_session = self.vm.new_session(&state_session);

        let (script, args, type_args, senders) = tx.into_inner();
        let sender = senders.get(0).cloned().unwrap_or(AccountAddress::ZERO);

        let mut cost_strategy =
            CostStrategy::transaction(&self.cost_table, GasUnits::new(gas.max_gas_amount()));

        let exec_result = vm_session
            .execute_script(
                script,
                type_args,
                args,
                senders,
                &mut cost_strategy,
                &NoContextLog::new(),
            )
            .and_then(|_| {
                Self::charge_global_write_gas_usage(&mut cost_strategy, &mut vm_session, &sender)
            })
            .and_then(|_| vm_session.finish())
            .and_then(|vm_effects| state_session.finish(vm_effects));

        self.handle_vm_result(sender, cost_strategy, gas, exec_result, dry_run)
    }

    fn clear(&self) {
        self.vm.clear();
        self.master_of_coin.clear();
    }
}
