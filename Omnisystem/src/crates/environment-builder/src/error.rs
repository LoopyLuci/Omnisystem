//! Error types for environment configuration building.

/// Errors that can occur while composing or validating an environment.
#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    /// A variable declared `required` in the schema has no value after
    /// every layer has been applied.
    MissingRequiredVar(String),
    /// A variable's final value failed its schema-declared validator
    /// (e.g. not one of an allowed set).
    InvalidVarValue {
        /// Variable name.
        name: String,
        /// The value that failed validation.
        value: String,
    },
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::MissingRequiredVar(name) => write!(f, "missing required variable: {name}"),
            Error::InvalidVarValue { name, value } => {
                write!(f, "invalid value for {name}: {value}")
            }
        }
    }
}

impl std::error::Error for Error {}

/// Result type for environment building.
pub type Result<T> = std::result::Result<T, Error>;
