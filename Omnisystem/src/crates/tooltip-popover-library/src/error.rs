//! Error types for popover placement.

/// Errors that can occur while computing popover placement.
#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    /// The viewport was too small to fit the popover on any side.
    NoFit,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::NoFit => write!(f, "popover does not fit in the viewport on any side"),
        }
    }
}

impl std::error::Error for Error {}

/// Result type for placement operations.
pub type Result<T> = std::result::Result<T, Error>;
