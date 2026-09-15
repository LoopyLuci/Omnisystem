//! Error types for the monitoring dashboard.

/// Errors produced by this crate.
#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    /// A widget's `warn_threshold` was not strictly less than its
    /// `critical_threshold`, which would make status derivation ambiguous.
    InvalidThresholds {
        /// Widget name.
        widget: String,
    },
    /// `dashboard_status` was called with no widgets.
    NoWidgets,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::InvalidThresholds { widget } => {
                write!(f, "widget '{widget}' has warn_threshold >= critical_threshold")
            }
            Error::NoWidgets => write!(f, "dashboard has no widgets"),
        }
    }
}

impl std::error::Error for Error {}

/// Result type for this crate.
pub type Result<T> = std::result::Result<T, Error>;
