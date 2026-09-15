//! Error types for chart data preparation.

/// Errors that can occur while building or scaling chart data.
#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    /// A series had no data points.
    EmptySeries(String),
    /// A domain (min, max) was degenerate or inverted.
    InvalidDomain(f64, f64),
    /// A pie chart was built from all-zero or negative values.
    NonPositiveTotal,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::EmptySeries(name) => write!(f, "series '{}' has no data points", name),
            Error::InvalidDomain(min, max) => {
                write!(f, "invalid domain: min {} >= max {}", min, max)
            }
            Error::NonPositiveTotal => write!(f, "pie chart total value must be positive"),
        }
    }
}

impl std::error::Error for Error {}

/// Result type for chart operations.
pub type Result<T> = std::result::Result<T, Error>;
