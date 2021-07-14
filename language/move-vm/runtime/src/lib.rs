// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

//! The core Move VM logic.
//!
//! It is a design goal for the Move VM to be independent of the Diem blockchain, so that
//! other blockchains can use it as well. The VM isn't there yet, but hopefully will be there
//! soon.
#![cfg_attr(not(feature = "std"), no_std)]

#[macro_use]
extern crate mirai_annotations;
#[macro_use]
extern crate alloc;
#[macro_use]
extern crate log;

pub mod data_cache;
mod interpreter;
pub mod loader;
pub mod logging;
pub mod move_vm;
mod native_functions;
mod runtime;
pub mod session;

#[cfg(test)]
mod unit_tests;
