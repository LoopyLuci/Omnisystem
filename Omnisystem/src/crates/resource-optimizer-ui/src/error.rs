//! Error types for resource optimization shaping.

/// Errors that can occur while analyzing resource utilization samples.
#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    /// A utilization sample was outside the valid `0.0..=1.0` range.
    UtilizationOutOfRange {
        /// Resource name the sample belongs to.
        resource: String,
        /// The offending value.
        value: f64,
    },
    /// No samples were provided for a resource that recommendations were
    /// requested for.
    NoSamples(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::UtilizationOutOfRange { resource, value } => {
                write!(f, "{resource}: utilization {value} outside 0.0..=1.0")
            }
            Error::NoSamples(resource) => write!(f, "{resource}: no utilization samples"),
        }
    }
}

impl std::error::Error for Error {}

/// Result type for resource optimization analysis.
pub type Result<T> = std::result::Result<T, Error>;
