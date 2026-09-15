//! Error types for scroll virtualization.

/// Errors that can occur while computing a virtualized viewport.
#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    /// The item count didn't match the number of heights supplied.
    ItemCountMismatch(usize, usize),
    /// A viewport height of zero or less was given.
    InvalidViewportHeight(f64),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::ItemCountMismatch(items, heights) => {
                write!(f, "{} items but {} heights supplied", items, heights)
            }
            Error::InvalidViewportHeight(h) => write!(f, "invalid viewport height: {}", h),
        }
    }
}

impl std::error::Error for Error {}

/// Result type for scroll operations.
pub type Result<T> = std::result::Result<T, Error>;
