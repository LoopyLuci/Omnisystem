//! Analytics viewer types: a tiny in-memory tabular dataset.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// One row of numeric measures keyed by column name, plus a categorical
/// dimension used for grouping.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Row {
    /// Grouping dimension, e.g. a region or product name.
    pub dimension: String,
    /// Numeric columns, e.g. `{"revenue": 120.5, "units": 4.0}`.
    pub measures: HashMap<String, f64>,
}

/// Supported aggregation functions for [`crate::group_by`].
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Aggregate {
    /// Sum of all values in the group.
    Sum,
    /// Arithmetic mean of all values in the group.
    Avg,
    /// Number of rows in the group.
    Count,
    /// Smallest value in the group.
    Min,
    /// Largest value in the group.
    Max,
}

/// One aggregated group produced by [`crate::group_by`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GroupResult {
    /// The dimension value this group was formed from.
    pub dimension: String,
    /// The aggregated value for the requested measure.
    pub value: f64,
    /// Number of source rows folded into this group.
    pub row_count: usize,
}
