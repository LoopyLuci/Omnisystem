//! Error types for chart-type recommendation.

/// Errors that can occur while analyzing a dataset for recommendation.
#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    /// No fields were supplied to analyze.
    NoFields,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::NoFields => write!(f, "no fields supplied to analyze"),
        }
    }
}

impl std::error::Error for Error {}

/// Result type for recommendation operations.
pub type Result<T> = std::result::Result<T, Error>;
