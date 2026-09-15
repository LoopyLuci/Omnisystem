//! Error types for the deployment wizard state machine.

/// Errors produced by this crate.
#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    /// The wizard has no steps defined.
    NoSteps,
    /// Tried to advance past the current step while required fields are
    /// missing or empty.
    IncompleteStep {
        /// Step name.
        step: String,
        /// The missing/empty field names.
        missing: Vec<String>,
    },
    /// Tried to advance past the final step, or go back before the first.
    OutOfRange,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::NoSteps => write!(f, "wizard has no steps"),
            Error::IncompleteStep { step, missing } => {
                write!(f, "step '{step}' is missing required fields: {}", missing.join(", "))
            }
            Error::OutOfRange => write!(f, "no step in that direction"),
        }
    }
}

impl std::error::Error for Error {}

/// Result type for this crate.
pub type Result<T> = std::result::Result<T, Error>;
