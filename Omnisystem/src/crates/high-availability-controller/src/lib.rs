//! high-availability-controller: cluster membership, quorum, and
//! deterministic leader election.
//!
//! Nodes join the cluster and heartbeat on a caller-driven logical tick.
//! `evaluate` recomputes which nodes are alive (heartbeated within the
//! configured timeout), whether a majority quorum exists, and who the
//! leader is -- re-electing whenever the current leader is no longer alive
//! and quorum permits it.

#![warn(missing_docs)]

/// Module-specific error types
pub mod error;
/// Cluster membership, quorum and leader-election logic
pub mod manager;
/// Core types and data structures
pub mod types;

pub use error::{Error, Result};
pub use manager::Manager;
pub use types::*;
