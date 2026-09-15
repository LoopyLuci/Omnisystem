//! Data types
use serde::{Deserialize, Serialize};

/// Volume lifecycle state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VolumeState {
    /// Volume created but not mounted anywhere.
    Created,
    /// Volume mounted into at least one container.
    Mounted,
    /// Volume unmounted from all containers but still exists.
    Unmounted,
    /// Volume removed and no longer exists.
    Removed,
}

/// A tracked volume and the containers/mount-points using it.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Volume {
    /// Volume name, unique within the manager.
    pub name: String,
    /// Current lifecycle state.
    pub state: VolumeState,
}
