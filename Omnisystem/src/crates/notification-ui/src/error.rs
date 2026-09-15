//! Error types for notification queue processing.

/// Errors produced by this crate.
#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    /// Two notifications shared the same id.
    DuplicateId(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::DuplicateId(id) => write!(f, "duplicate notification id '{id}'"),
        }
    }
}

impl std::error::Error for Error {}

/// Result type for this crate.
pub type Result<T> = std::result::Result<T, Error>;
