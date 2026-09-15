use serde::{Deserialize, Serialize};

/// Outcome of completing an in-flight batch.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum BatchResult {
    /// More instances remain to be updated.
    Remaining(usize),
    /// All instances have been updated.
    Done,
}
