//! omnidocker-state-manager: bridges Docker-side observed state into
//! Omnisystem's own desired-state model and reconciles the two, finding
//! real drift -- resources Omnisystem wants that were never observed,
//! resources observed that Omnisystem never declared, and resources whose
//! observed state doesn't match what was declared.
//!
//! Unlike the `docker-*` crates (which model Docker's own protocol/state
//! directly), this crate is specifically the omnisystem-bridge: it doesn't
//! know how to start a container or allocate an IP, it only tracks the
//! desired/observed pairing and computes drift between them.

#![warn(missing_docs)]

/// Module-specific error types
pub mod error;
/// Desired/observed state tracking and reconciliation
pub mod manager;
/// Core types and data structures
pub mod types;

pub use error::{Error, Result};
pub use manager::Manager;
pub use types::{Drift, ResourceId, ResourceKind};
