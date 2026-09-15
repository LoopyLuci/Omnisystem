//! docker-registry-integration: registry auth/push/pull protocol and state
//! modeling. This crate never makes real network calls or handles real
//! credentials -- authentication outcomes and transfer progress are driven
//! by caller-supplied (in tests: mocked) responses, and the crate's job is
//! to enforce the real state machine around them (no transfer without a
//! valid, unexpired token; no advancing a completed/failed transfer).

#![warn(missing_docs)]

/// Module-specific error types
pub mod error;
/// Auth state and transfer state machine
pub mod manager;
/// Core types and data structures
pub mod types;

pub use error::{Error, Result};
pub use manager::{AuthOutcome, Manager};
pub use types::{AuthState, Direction, Transfer, TransferState};
