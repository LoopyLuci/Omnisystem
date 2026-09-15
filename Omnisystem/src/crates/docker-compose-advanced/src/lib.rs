//! docker-compose-advanced: a real multi-service compose file model
//! (services/networks/volumes/depends_on) with structural validation --
//! dependency-cycle detection and missing service/network/volume reference
//! checks.

#![warn(missing_docs)]

/// Module-specific error types
pub mod error;
/// Core types and data structures
pub mod types;
/// Compose file validation
pub mod validate;

pub use error::{Error, Result};
pub use types::{ComposeFile, Service, ValidationIssue};
pub use validate::validate;
