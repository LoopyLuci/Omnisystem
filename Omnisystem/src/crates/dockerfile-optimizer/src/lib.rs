//! dockerfile-optimizer: real Dockerfile parsing and anti-pattern
//! detection -- combinable consecutive `RUN`s, unpinned base image tags,
//! missing multi-stage build opportunities, and `ADD` where `COPY` would do.

#![warn(missing_docs)]

/// Anti-pattern detection over parsed instructions
pub mod analyze;
/// Module-specific error types
pub mod error;
/// Dockerfile parsing
pub mod parse;
/// Core types and data structures
pub mod types;

pub use analyze::analyze;
pub use error::{Error, Result};
pub use parse::parse;
pub use types::{Instruction, Suggestion};
