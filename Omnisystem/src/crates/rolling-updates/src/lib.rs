//! rolling-updates: batched, max-unavailable-bounded rolling deployment.
//!
//! Rolls a fleet of instances forward in batches, never taking more than
//! `max_unavailable` instances out of service at once. Supports pausing,
//! resuming, and rolling a failed batch back into service without counting
//! it as updated.

pub mod error;
pub mod manager;
pub mod types;

pub use error::{Error, Result};
pub use manager::Manager;
pub use types::*;
