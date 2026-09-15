//! Notification UI types: a queue of notifications with priority, category,
//! and optional dedup keys.

use serde::{Deserialize, Serialize};

/// Notification priority, ordered least to most urgent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Priority {
    /// Low-urgency, informational.
    Low,
    /// Normal priority.
    Normal,
    /// High priority — surface prominently.
    High,
}

/// One notification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    /// Unique id.
    pub id: String,
    /// Category, e.g. `billing` or `security`.
    pub category: String,
    /// Priority.
    pub priority: Priority,
    /// Sequence number (monotonic, deterministic stand-in for a timestamp).
    pub seq: i64,
    /// Optional dedup key — later notifications sharing a key within a
    /// window supersede earlier ones instead of stacking up.
    pub dedup_key: Option<String>,
    /// Whether the user has read it.
    pub read: bool,
}
