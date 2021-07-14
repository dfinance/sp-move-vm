// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use alloc::string::{String, ToString};
use move_core_types::vm_status::{StatusCode, StatusType};
use move_vm_types::natives::function::PartialVMError;
use vm::errors::VMError;

// Trait used by the VM to log interesting data.
// Clients are responsible for the implementation of alert.
pub trait LogContext: Clone {
    // Alert is called on critical errors
    fn alert(&self);
}

// Helper `Logger` implementation that does nothing
#[derive(Clone)]
pub struct NoContextLog {
    name: String,
}

impl NoContextLog {
    pub fn new() -> Self {
        Self {
            name: "test".to_string(),
        }
    }
}

impl LogContext for NoContextLog {
    fn alert(&self) {}
}

//
// Utility functions
//
pub fn expect_no_verification_errors(err: VMError, _log_context: &impl LogContext) -> VMError {
    match err.status_type() {
        status_type @ StatusType::Deserialization | status_type @ StatusType::Verification => {
            let message = format!(
                "Unexpected verifier/deserialization error! This likely means there is code \
                stored on chain that is unverifiable!\nError: {:?}",
                &err
            );
            let (_old_status, _old_sub_status, _old_message, location, indices, offsets) =
                err.all_data();
            let major_status = match status_type {
                StatusType::Deserialization => StatusCode::UNEXPECTED_DESERIALIZATION_ERROR,
                StatusType::Verification => StatusCode::UNEXPECTED_VERIFIER_ERROR,
                _ => unreachable!(),
            };

            error!("[VM] {}", message);
            PartialVMError::new(major_status)
                .with_message(message)
                .at_indices(indices)
                .at_code_offsets(offsets)
                .finish(location)
        }
        _ => err,
    }
}
