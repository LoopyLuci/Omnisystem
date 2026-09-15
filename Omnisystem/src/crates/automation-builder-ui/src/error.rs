//! Error types for the automation/workflow builder.

/// Errors produced by this crate.
#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    /// Two steps in the same workflow shared a name.
    DuplicateStep(String),
    /// A step depended on a name that doesn't exist in the workflow.
    UnknownDependency {
        /// The step with the dangling dependency.
        step: String,
        /// The missing dependency name.
        depends_on: String,
    },
    /// The dependency graph contains a cycle, so no valid execution order exists.
    CycleDetected(Vec<String>),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::DuplicateStep(name) => write!(f, "duplicate step name '{name}'"),
            Error::UnknownDependency { step, depends_on } => {
                write!(f, "step '{step}' depends on unknown step '{depends_on}'")
            }
            Error::CycleDetected(cycle) => write!(f, "dependency cycle detected: {}", cycle.join(" -> ")),
        }
    }
}

impl std::error::Error for Error {}

/// Result type for this crate.
pub type Result<T> = std::result::Result<T, Error>;
