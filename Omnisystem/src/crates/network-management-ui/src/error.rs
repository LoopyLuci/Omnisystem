//! Error types for network resource shaping.

/// Errors that can occur while shaping network state for display.
#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    /// A route referenced an interface name that isn't in the interface
    /// list, so it can't be attributed for display.
    UnknownInterface(String),
    /// Two interfaces were given the same name.
    DuplicateInterface(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::UnknownInterface(n) => write!(f, "route references unknown interface: {n}"),
            Error::DuplicateInterface(n) => write!(f, "duplicate interface name: {n}"),
        }
    }
}

impl std::error::Error for Error {}

/// Result type for network resource shaping.
pub type Result<T> = std::result::Result<T, Error>;
