// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Core types for Move.

#![cfg_attr(not(feature = "std"), no_std)]

#[macro_use]
extern crate alloc;

pub mod account_address;
pub mod gas_schedule;
pub mod identifier;
pub mod language_storage;
pub mod value;
pub mod vm_status;
