//! Error types for settings cascade resolution.

use crate::SettingType;

/// Errors produced by this crate.
#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    /// A layer set a key that isn't declared in the schema.
    UnknownKey {
        /// The undeclared key.
        key: String,
    },
    /// A layer's value for a key doesn't match the schema's declared type.
    TypeMismatch {
        /// The offending key.
        key: String,
        /// The type the schema declares.
        expected: SettingType,
        /// The type the value actually was.
        actual: SettingType,
    },
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::UnknownKey { key } => write!(f, "setting '{key}' is not declared in the schema"),
            Error::TypeMismatch { key, expected, actual } => {
                write!(f, "setting '{key}' expected {expected:?}, got {actual:?}")
            }
        }
    }
}

impl std::error::Error for Error {}

/// Result type for this crate.
pub type Result<T> = std::result::Result<T, Error>;
