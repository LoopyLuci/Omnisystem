//! Error types for volume placement planning.

/// Errors produced by this crate.
#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    /// No disk had enough free capacity for a volume.
    NoCapacityFor(String),
    /// A volume requested a non-positive size.
    InvalidSize(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::NoCapacityFor(name) => write!(f, "no disk has enough free capacity for volume '{name}'"),
            Error::InvalidSize(name) => write!(f, "volume '{name}' has a non-positive size"),
        }
    }
}

impl std::error::Error for Error {}

/// Result type for this crate.
pub type Result<T> = std::result::Result<T, Error>;
