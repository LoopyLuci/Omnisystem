//! zero-downtime-deployment: connection-draining-aware instance replacement.
//!
//! An instance must be marked ready, then draining (stop receiving new
//! traffic), and only becomes terminable once every in-flight request
//! against it has completed. This prevents killing an instance mid-request.

pub mod error;
pub mod manager;
pub mod types;

pub use error::{Error, Result};
pub use manager::Manager;
pub use types::*;
