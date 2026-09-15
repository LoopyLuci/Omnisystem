//! Error types for table sorting, filtering, and pagination.

/// Errors that can occur when operating on a data table.
#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    /// A referenced column name does not exist in the table.
    UnknownColumn(String),
    /// A page size of zero was requested.
    InvalidPageSize,
    /// A page index past the end of the data was requested.
    PageOutOfRange(usize, usize),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::UnknownColumn(name) => write!(f, "unknown column: {}", name),
            Error::InvalidPageSize => write!(f, "page size must be greater than zero"),
            Error::PageOutOfRange(requested, total) => {
                write!(f, "page {} out of range (0..{})", requested, total)
            }
        }
    }
}

impl std::error::Error for Error {}

/// Result type for table operations.
pub type Result<T> = std::result::Result<T, Error>;
