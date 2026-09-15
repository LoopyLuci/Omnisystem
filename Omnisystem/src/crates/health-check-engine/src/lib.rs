//! health-check-engine: consecutive-threshold health-status tracking.
//!
//! Each registered check tracks consecutive successes/failures. A single
//! failure degrades a check without failing it outright; `failure_threshold`
//! consecutive failures mark it Unhealthy, and `success_threshold`
//! consecutive successes are required to recover to Healthy. This avoids
//! flapping status on a single flaky probe.

#![warn(missing_docs)]

/// Module-specific error types
pub mod error;
/// Core check-tracking logic
pub mod manager;
/// Core types and data structures
pub mod types;

pub use error::{Error, Result};
pub use manager::Manager;
pub use types::*;
