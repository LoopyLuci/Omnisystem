//! Error types for icon registry lookups.

/// Errors that can occur when resolving an icon.
#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    /// No icon is registered under this name or any of its aliases.
    UnknownIcon(String),
    /// The requested size variant isn't defined for this icon.
    UnknownSize(String, u32),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::UnknownIcon(name) => write!(f, "unknown icon: {}", name),
            Error::UnknownSize(name, size) => write!(f, "icon '{}' has no {}px variant", name, size),
        }
    }
}

impl std::error::Error for Error {}

/// Result type for icon lookups.
pub type Result<T> = std::result::Result<T, Error>;
