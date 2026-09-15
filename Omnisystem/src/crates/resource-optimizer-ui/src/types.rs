//! Resource optimization types.

use serde::{Deserialize, Serialize};

/// A resource being tracked for right-sizing (e.g. one VM's CPU).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResourceUsage {
    /// Resource identifier, e.g. `vm-42:cpu`.
    pub name: String,
    /// Currently provisioned capacity (arbitrary unit, e.g. vCPUs).
    pub provisioned: f64,
    /// Utilization samples as fractions of provisioned capacity
    /// (`0.0..=1.0`), most recent last.
    pub samples: Vec<f64>,
}

/// A sizing recommendation for one resource.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Recommendation {
    /// Peak utilization is consistently near capacity; provision more.
    ScaleUp,
    /// Peak utilization is well below capacity; provision less.
    ScaleDown,
    /// Utilization is in a healthy band; no change recommended.
    Keep,
}

/// A recommendation with the numbers that justify it, for display.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SizingAdvice {
    /// Resource this advice applies to.
    pub resource: String,
    /// Average utilization across samples.
    pub avg_utilization: f64,
    /// Peak (maximum) utilization across samples.
    pub peak_utilization: f64,
    /// The recommendation.
    pub recommendation: Recommendation,
}
