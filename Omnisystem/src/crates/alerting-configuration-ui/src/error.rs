//! Error types for alert rule configuration and evaluation.

/// Errors produced by this crate.
#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    /// A rule's `for_samples` was zero, which can never fire deterministically.
    InvalidForSamples {
        /// The offending rule's name.
        rule: String,
    },
    /// Two rules shared the same name, which would make evaluations ambiguous.
    DuplicateRuleName(String),
    /// A rule referenced a metric with no samples supplied for evaluation.
    NoSamplesForMetric {
        /// Rule name.
        rule: String,
        /// Metric name that had no samples.
        metric: String,
    },
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::InvalidForSamples { rule } => {
                write!(f, "rule '{rule}' has for_samples == 0")
            }
            Error::DuplicateRuleName(name) => write!(f, "duplicate rule name '{name}'"),
            Error::NoSamplesForMetric { rule, metric } => {
                write!(f, "rule '{rule}' references metric '{metric}' with no samples")
            }
        }
    }
}

impl std::error::Error for Error {}

/// Result type for this crate.
pub type Result<T> = std::result::Result<T, Error>;
