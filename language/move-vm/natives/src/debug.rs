// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use alloc::collections::VecDeque;
use alloc::vec::Vec;
use move_core_types::gas_schedule::ONE_GAS_UNIT;
#[allow(unused_imports)]
use move_vm_types::values::{values_impl::debug::print_reference, Reference};
use move_vm_types::{
    loaded_data::runtime_types::Type,
    natives::function::{NativeContext, NativeResult},
    values::Value,
};
use smallvec::smallvec;
use vm::errors::PartialVMResult;

#[allow(unused_mut)]
#[allow(unused_variables)]
pub fn native_print(
    context: &mut impl NativeContext,
    mut ty_args: Vec<Type>,
    mut args: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    debug_assert!(ty_args.len() == 1);
    debug_assert!(args.len() == 1);

    // No-op if the feature flag is not present.
    #[cfg(feature = "debug_module")]
    {
        use crate::u256::U256;
        use move_core_types::identifier::Identifier;
        use move_core_types::language_storage::StructTag;
        use move_core_types::language_storage::{TypeTag, CORE_CODE_ADDRESS};
        use move_vm_types::values::{Container, ValueImpl};
        let ty = ty_args.pop().unwrap();
        let r = pop_arg!(args, Reference);

        if let Ok(type_tag) = context.type_to_type_tag(&ty) {
            if type_tag
                == TypeTag::Struct(StructTag {
                    address: CORE_CODE_ADDRESS,
                    module: Identifier::new("U256").unwrap(),
                    name: Identifier::new("U256").unwrap(),
                    type_params: vec![],
                })
            {
                if let Container::Struct(fields) = r.read_ref()?.value_as::<Container>()? {
                    if let Some(ValueImpl::Container(Container::VecU8(r))) = fields.borrow().get(0)
                    {
                        let cell = r.as_ref().clone();
                        println!("[debug] {}", U256::from_little_endian(&cell.into_inner()));
                    }
                }
                return Ok(NativeResult::ok(ONE_GAS_UNIT, smallvec![]));
            }
        }

        let mut buf = String::new();
        print_reference(&mut buf, &r)?;
        println!("[debug] {}", buf);
    }

    Ok(NativeResult::ok(ONE_GAS_UNIT, smallvec![]))
}

#[allow(unused_variables)]
pub fn native_print_stack_trace(
    context: &mut impl NativeContext,
    ty_args: Vec<Type>,
    args: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    debug_assert!(ty_args.is_empty());
    debug_assert!(args.is_empty());

    #[cfg(feature = "debug_module")]
    {
        let mut s = String::new();
        context.print_stack_trace(&mut s)?;
        println!("{}", s);
    }

    Ok(NativeResult::ok(ONE_GAS_UNIT, smallvec![]))
}
