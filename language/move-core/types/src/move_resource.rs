// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    identifier::Identifier,
    language_storage::{StructTag, TypeTag},
};
use alloc::vec::Vec;

pub trait MoveResource {
    const MODULE_NAME: &'static str;
    const STRUCT_NAME: &'static str;

    fn module_identifier() -> Identifier {
        Identifier::new(Self::MODULE_NAME).expect("failed to get IdentStr for Move module")
    }

    fn struct_identifier() -> Identifier {
        Identifier::new(Self::STRUCT_NAME).expect("failed to get IdentStr for Move struct")
    }

    fn type_params() -> Vec<TypeTag> {
        vec![]
    }

    fn struct_tag() -> StructTag {
        StructTag {
            address: crate::language_storage::CORE_CODE_ADDRESS,
            name: Self::struct_identifier(),
            module: Self::module_identifier(),
            type_params: Self::type_params(),
        }
    }

    fn resource_path() -> Vec<u8> {
        Self::struct_tag().access_vector()
    }
}
