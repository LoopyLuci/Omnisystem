//! Automation builder types: a workflow as a DAG of named steps.

use serde::{Deserialize, Serialize};

/// One step in a workflow, depending on zero or more other steps by name.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Step {
    /// Unique step name within the workflow.
    pub name: String,
    /// Names of steps that must complete before this one runs.
    pub depends_on: Vec<String>,
}

/// A workflow: an ordered collection of [`Step`]s (order as authored, not
/// necessarily execution order).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Workflow {
    /// The steps that make up the workflow.
    pub steps: Vec<Step>,
}
