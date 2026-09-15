//! Error types for markdown document analysis.

/// Errors that can occur while parsing or analyzing a markdown document.
#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    /// A heading skipped more than one level deeper than its predecessor
    /// (e.g. an `h1` followed directly by an `h3`), which would produce an
    /// invalid nested table of contents.
    SkippedHeadingLevel {
        /// Source line number of the offending heading.
        line: usize,
        /// The parent heading's level.
        from: u8,
        /// The offending heading's level.
        to: u8,
    },
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::SkippedHeadingLevel { line, from, to } => {
                write!(f, "line {}: heading level jumped from h{} to h{}", line, from, to)
            }
        }
    }
}

impl std::error::Error for Error {}

/// Result type for document analysis.
pub type Result<T> = std::result::Result<T, Error>;
