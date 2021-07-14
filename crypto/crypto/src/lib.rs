// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]
#![deny(missing_docs)]
//! This feature gets turned on only if diem-crypto is compiled via MIRAI in a nightly build.
#![cfg_attr(mirai, allow(incomplete_features), feature(const_generics))]
#![cfg_attr(not(feature = "std"), no_std)]

#[macro_use]
extern crate alloc;

#[cfg(test)]
mod unit_tests;

pub mod ed25519;
pub mod hash;
pub mod serde_name;
#[cfg(test)]
pub mod test_utils;
pub mod traits;

pub use self::traits::*;
pub use hash::HashValue;

///Reexport once_cell and serde_name for use in CryptoHasher Derive implementation.
#[doc(hidden)]
pub use cell::{Lazy, OnceCell};
