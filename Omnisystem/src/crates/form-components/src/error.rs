//! Error types for form field validation.

/// Errors surfaced while validating a form.
#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    /// A referenced field name is not part of the form definition.
    UnknownField(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::UnknownField(name) => write!(f, "unknown field: {}", name),
        }
    }
}

impl std::error::Error for Error {}

/// Result type for form operations.
pub type Result<T> = std::result::Result<T, Error>;
