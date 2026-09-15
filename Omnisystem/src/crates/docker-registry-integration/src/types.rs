//! Data types
use serde::{Deserialize, Serialize};

/// A registry connection's credential state, modeled without ever holding
/// or transmitting real secrets -- this crate is protocol/state modeling,
/// not a real Docker registry client.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuthState {
    /// No authentication attempted yet.
    Unauthenticated,
    /// A bearer token was obtained and has not yet expired (tracked by a
    /// caller-supplied logical tick, not wall-clock time).
    Authenticated {
        /// Opaque token identifier (never a real credential in this crate).
        token: String,
        /// Logical tick at which the token expires.
        expires_at_tick: u64,
    },
    /// The last authentication attempt was rejected by the registry.
    Rejected,
}

/// Which direction a transfer request is going.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Direction {
    /// Uploading a local image to the registry.
    Push,
    /// Downloading an image from the registry.
    Pull,
}

/// Progress state of a single push/pull transfer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransferState {
    /// Transfer requested but not yet started.
    Requested,
    /// Transfer actively in progress.
    InProgress,
    /// Transfer finished successfully.
    Completed,
    /// Transfer failed (e.g. auth rejected mid-transfer, or registry error).
    Failed,
}

/// A tracked push/pull transfer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transfer {
    /// Image reference being transferred.
    pub reference: String,
    /// Push or pull.
    pub direction: Direction,
    /// Current state.
    pub state: TransferState,
}
