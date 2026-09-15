//! Secret lifecycle model: metadata, rotation policy, and access grants.
//!
//! No real secret *values* appear anywhere in this crate or its tests —
//! only metadata (names, versions, timestamps, principals). Test fixtures
//! use obviously-fake placeholder names, never anything resembling a real
//! credential shape.

use serde::{Deserialize, Serialize};

/// How often a secret must be rotated.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RotationPolicy {
    /// Maximum age, in days, before the secret is considered overdue for
    /// rotation.
    pub max_age_days: u32,
}

impl RotationPolicy {
    /// Build a policy with the given max age.
    pub fn every_days(max_age_days: u32) -> Self {
        Self { max_age_days }
    }
}

/// Metadata about a single managed secret (never the secret value itself).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretMetadata {
    /// Secret name/path, e.g. "prod/db/password".
    pub name: String,
    /// Current version number, incremented on each rotation.
    pub version: u32,
    /// Days since this version was created (caller-supplied "now" offset;
    /// this crate does no real wall-clock I/O so tests stay deterministic).
    pub age_days: u32,
    /// Rotation policy governing this secret.
    pub rotation_policy: RotationPolicy,
}

impl SecretMetadata {
    /// Whether this secret is currently overdue for rotation.
    pub fn rotation_due(&self) -> bool {
        self.age_days >= self.rotation_policy.max_age_days
    }

    /// Days remaining until rotation is due (0 if already overdue).
    pub fn days_until_due(&self) -> u32 {
        self.rotation_policy.max_age_days.saturating_sub(self.age_days)
    }
}

/// A grant of access to a secret for a principal (user or service
/// identity).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccessGrant {
    /// Secret this grant applies to.
    pub secret_name: String,
    /// Principal granted access.
    pub principal: String,
    /// Whether the principal can read the secret value.
    pub can_read: bool,
    /// Whether the principal can trigger/perform rotation.
    pub can_rotate: bool,
}
