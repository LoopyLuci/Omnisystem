//! Statistical summary and histogram types.

use serde::{Deserialize, Serialize};

/// A single histogram bucket.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Bucket {
    /// Inclusive lower bound.
    pub lower: f64,
    /// Exclusive upper bound (except the final bucket, which is inclusive).
    pub upper: f64,
    /// Number of values falling in `[lower, upper)`.
    pub count: usize,
}

/// Descriptive statistics for a numeric dataset.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Summary {
    /// Number of values.
    pub count: usize,
    /// Arithmetic mean.
    pub mean: f64,
    /// Middle value (average of the two middle values for even counts).
    pub median: f64,
    /// Population standard deviation.
    pub stddev: f64,
    /// Smallest value.
    pub min: f64,
    /// Largest value.
    pub max: f64,
}
