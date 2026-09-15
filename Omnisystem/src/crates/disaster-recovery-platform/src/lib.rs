//! disaster-recovery-platform: RPO/RTO-aware recovery plan registration,
//! snapshot tracking, and step-by-step drill/recovery execution with
//! validation of whether the observed recovery met its plan's targets.
//!
//! A [`manager::Manager`] holds recovery plans (each with an RPO target,
//! RTO target, and ordered list of steps), records snapshots taken of a
//! system over time, and drives drill execution: `start_drill` declares a
//! disaster at a given tick, `advance_step` executes steps in order, and
//! `complete_drill` -- once all steps have run -- computes the actual
//! recovery time and data-loss window against the plan's targets.

#![warn(missing_docs)]

/// Module-specific error types
pub mod error;
/// Recovery plan registration, snapshot tracking, and drill execution
pub mod manager;
/// Core types and data structures
pub mod types;

pub use error::{Error, Result};
pub use manager::Manager;
pub use types::{DrillResult, RecoveryPlan};
