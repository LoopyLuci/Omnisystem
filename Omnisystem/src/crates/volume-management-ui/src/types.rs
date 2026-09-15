//! Volume management types: storage volumes to be placed on disks.

use serde::{Deserialize, Serialize};

/// A storage volume requesting a given size.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Volume {
    /// Volume name.
    pub name: String,
    /// Requested size in GB.
    pub size_gb: f64,
}

/// A disk with a total capacity and current usage.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Disk {
    /// Disk name.
    pub name: String,
    /// Total capacity in GB.
    pub capacity_gb: f64,
    /// Already-used capacity in GB (before any new allocations).
    pub used_gb: f64,
}

impl Disk {
    /// Capacity not yet used.
    pub fn free_gb(&self) -> f64 {
        self.capacity_gb - self.used_gb
    }

    /// Fraction of capacity used, `0.0..=1.0`.
    pub fn used_fraction(&self) -> f64 {
        if self.capacity_gb <= 0.0 {
            1.0
        } else {
            self.used_gb / self.capacity_gb
        }
    }
}

/// One volume's placement onto a disk.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Placement {
    /// The volume being placed.
    pub volume: String,
    /// The disk it was placed on.
    pub disk: String,
}
