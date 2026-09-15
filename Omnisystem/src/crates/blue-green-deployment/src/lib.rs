//! blue-green-deployment: real blue/green release orchestration.
//!
//! Tracks two deployment slots (blue and green). A new version is deployed
//! to whichever slot is not currently serving traffic ("standby"), health
//! checked, then promoted with an atomic active-pointer flip. The most
//! recent promotion can be undone with `rollback`.

pub mod error;
pub mod manager;
pub mod types;

pub use error::{Error, Result};
pub use manager::Manager;
pub use types::*;
