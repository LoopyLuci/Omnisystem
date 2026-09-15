//! Error types for statistical data-shaping.

/// Errors that can occur while summarizing or bucketing a dataset.
#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    /// The dataset had no values.
    EmptyDataset,
    /// A histogram with zero buckets was requested.
    InvalidBucketCount,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::EmptyDataset => write!(f, "dataset is empty"),
            Error::InvalidBucketCount => write!(f, "bucket count must be greater than zero"),
        }
    }
}

impl std::error::Error for Error {}

/// Result type for visualization data-prep operations.
pub type Result<T> = std::result::Result<T, Error>;
