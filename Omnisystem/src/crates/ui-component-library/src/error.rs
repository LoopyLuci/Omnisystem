//! Error types for design-token resolution.

/// Errors that can occur while resolving a design token.
#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    /// The token isn't defined in the default theme, any override, or the
    /// component's own local override.
    UnresolvedToken(String),
    /// A theme name was referenced that doesn't exist in the registry.
    UnknownTheme(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::UnresolvedToken(name) => write!(f, "unresolved design token: {}", name),
            Error::UnknownTheme(name) => write!(f, "unknown theme: {}", name),
        }
    }
}

impl std::error::Error for Error {}

/// Result type for token resolution.
pub type Result<T> = std::result::Result<T, Error>;
