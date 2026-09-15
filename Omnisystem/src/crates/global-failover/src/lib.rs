//! global-failover: multi-region primary/secondary failover with automatic
//! failover and (optional) automatic failback.
//!
//! Regions join a failover group and heartbeat on a caller-driven logical
//! tick. `evaluate` recomputes which regions are alive (heartbeated within
//! the configured timeout) and which region is currently serving as active
//! primary -- failing over to another alive region when the active primary
//! goes stale, and failing back to the originally-designated primary once it
//! recovers, if auto-failback is enabled.

#![warn(missing_docs)]

/// Module-specific error types
pub mod error;
/// Region membership, failover and failback logic
pub mod manager;
/// Core types and data structures
pub mod types;

pub use error::{Error, Result};
pub use manager::Manager;
pub use types::*;
