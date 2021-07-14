// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use alloc::string::String;
use alloc::string::ToString;
use core::fmt;
use serde::{de, ser};

pub type Result<T, E = Error> = core::result::Result<T, E>;

#[derive(Clone, Debug, PartialEq)]
pub enum Error {
    Eof,
    Io(String),
    ExceededMaxLen(usize),
    ExceededContainerDepthLimit(&'static str),
    ExpectedBoolean,
    ExpectedMapKey,
    ExpectedMapValue,
    NonCanonicalMap,
    ExpectedOption,
    Custom(String),
    MissingLen,
    NotSupported(&'static str),
    RemainingInput,
    Utf8,
    NonCanonicalUleb128Encoding,
    IntegerOverflowDuringUleb128Decoding,
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Eof => write!(f, "unexpected end of input"),
            Error::Io(v) => write!(f, "I/O error: {0}", v),
            Error::ExceededMaxLen(v) => write!(f, "exceeded max sequence length: {0}", v),
            Error::ExceededContainerDepthLimit(v) => {
                write!(f, "exceeded max container depth while entering: {0}", v)
            }
            Error::ExpectedBoolean => write!(f, "expected boolean"),
            Error::ExpectedMapKey => write!(f, "expected map key"),
            Error::ExpectedMapValue => write!(f, "expected map value"),
            Error::NonCanonicalMap => write!(
                f,
                "keys of serialized maps must be unique and in increasing order"
            ),
            Error::ExpectedOption => write!(f, "expected option type"),
            Error::Custom(err) => write!(f, "{0}", err),
            Error::MissingLen => write!(f, "sequence missing length"),
            Error::NotSupported(v) => write!(f, "{0}", v),
            Error::RemainingInput => write!(f, "remaining input"),
            Error::Utf8 => write!(f, "malformed utf8"),
            Error::NonCanonicalUleb128Encoding => {
                write!(f, "ULEB128 encoding was not minimal in size")
            }
            Error::IntegerOverflowDuringUleb128Decoding => {
                write!(f, "ULEB128-encoded integer did not fit in the target size")
            }
        }
    }
}

impl ser::Error for Error {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        Error::Custom(msg.to_string())
    }
}

impl de::Error for Error {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        Error::Custom(msg.to_string())
    }
}
