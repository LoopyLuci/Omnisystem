//! Error types

/// Errors produced by recovery-plan and drill operations.
#[derive(Debug, Clone)]
pub enum Error {
    /// No recovery plan is registered under this name.
    UnknownPlan(String),
    /// A drill/recovery is already running for this plan.
    DrillAlreadyRunning(String),
    /// No drill/recovery is currently running for this plan.
    NoDrillInProgress(String),
    /// `advance_step` or `complete` was called but the plan's steps are
    /// already exhausted, or `complete` was called before all steps ran.
    StepsNotComplete {
        /// The plan whose steps were incomplete.
        plan: String,
        /// Steps completed so far.
        completed: usize,
        /// Total steps in the plan.
        total: usize,
    },
    /// Other error
    Other(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::UnknownPlan(name) => write!(f, "no recovery plan named '{}'", name),
            Error::DrillAlreadyRunning(name) => write!(f, "a drill is already running for plan '{}'", name),
            Error::NoDrillInProgress(name) => write!(f, "no drill in progress for plan '{}'", name),
            Error::StepsNotComplete { plan, completed, total } => write!(
                f,
                "plan '{}' has only completed {}/{} steps",
                plan, completed, total
            ),
            Error::Other(msg) => write!(f, "Error: {}", msg),
        }
    }
}

impl std::error::Error for Error {}

/// Result type
pub type Result<T> = std::result::Result<T, Error>;
