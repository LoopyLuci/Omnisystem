//! docker-image-manager: real image reference parsing
//! (`[registry/]repository[:tag][@digest]`), tag resolution, and
//! layer/size tracking.

#![warn(missing_docs)]

/// Module-specific error types
pub mod error;
/// Image tracking, tag resolution, and pushing
pub mod manager;
/// Image reference parsing
pub mod parse;
/// Core types and data structures
pub mod types;

pub use error::{Error, Result};
pub use manager::Manager;
pub use parse::parse;
pub use types::{Image, ImageReference, Layer};
