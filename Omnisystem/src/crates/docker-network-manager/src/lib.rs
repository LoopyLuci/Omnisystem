//! docker-network-manager: real network mode modeling
//! (bridge/overlay/host), CIDR subnet parsing, sequential IP allocation
//! without collision, and container-to-network attachment tracking.

#![warn(missing_docs)]

/// Module-specific error types
pub mod error;
/// Network creation, IP allocation, and attachment tracking
pub mod manager;
/// Core types and data structures
pub mod types;

pub use error::{Error, Result};
pub use manager::{parse_subnet, Manager};
pub use types::{Network, NetworkMode, Subnet};
