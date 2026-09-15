//! Error types for the analytics viewer.

/// Errors produced by this crate.
#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    /// The requested measure column was not present on any row.
    UnknownMeasure(String),
    /// `group_by` was called with no rows.
    EmptyDataset,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::UnknownMeasure(name) => write!(f, "unknown measure column '{name}'"),
            Error::EmptyDataset => write!(f, "dataset has no rows"),
        }
    }
}

impl std::error::Error for Error {}

/// Result type for this crate.
pub type Result<T> = std::result::Result<T, Error>;
