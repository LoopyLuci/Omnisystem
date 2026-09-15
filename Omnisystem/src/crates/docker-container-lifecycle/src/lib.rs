//! docker-container-lifecycle: a real container lifecycle state machine
//! (`created -> running -> paused -> stopped -> removed`) with enforcement
//! of only the transitions Docker itself allows.

#![warn(missing_docs)]

/// Module-specific error types
pub mod error;
/// Container lifecycle state machine
pub mod manager;
/// Core types and data structures
pub mod types;

pub use error::{Error, Result};
pub use manager::Manager;
pub use types::{Container, ContainerState};
