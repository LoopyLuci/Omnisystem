//! Backup/restore types: a chain of full and incremental backups.

use serde::{Deserialize, Serialize};

/// What kind of backup a [`BackupRecord`] is.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BackupKind {
    /// A standalone, self-sufficient backup.
    Full,
    /// A delta backup that requires its `parent` to restore.
    Incremental {
        /// The id of the backup this one deltas against.
        parent: String,
    },
}

/// One backup snapshot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupRecord {
    /// Unique backup id.
    pub id: String,
    /// Day index the backup was taken on (integer for deterministic tests,
    /// not wall-clock).
    pub day: i64,
    /// Full or incremental.
    pub kind: BackupKind,
}

/// A retention policy: keep the `keep_full` most recent full backups (and
/// any incrementals that chain to a kept full backup); prune the rest.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct RetentionPolicy {
    /// Number of most-recent full backups to retain.
    pub keep_full: usize,
}
