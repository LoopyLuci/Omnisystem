//! canary-deployment: staged, error-rate-gated canary rollouts.
//!
//! Traffic is shifted to a new version in increasing percentage stages.
//! Each stage advance is gated on an externally-observed error rate; if the
//! rate exceeds the configured threshold at any point, the rollout is
//! immediately reverted to 0% canary traffic.

pub mod error;
pub mod manager;
pub mod types;

pub use error::{Error, Result};
pub use manager::Manager;
pub use types::*;
