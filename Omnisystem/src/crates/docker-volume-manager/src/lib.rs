//! docker-volume-manager: real volume lifecycle tracking
//! (create/mount/unmount/remove) with host mount-point conflict detection.

#![warn(missing_docs)]

/// Module-specific error types
pub mod error;
/// Volume lifecycle and mount tracking
pub mod manager;
/// Core types and data structures
pub mod types;

pub use error::{Error, Result};
pub use manager::Manager;
pub use types::{Volume, VolumeState};
