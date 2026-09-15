//! Field-profile and recommendation types.

use serde::{Deserialize, Serialize};

/// The inferred data type of a field.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FieldKind {
    /// A discrete label/category (e.g. "status", "region").
    Categorical,
    /// A continuous or discrete measurement.
    Numeric,
    /// A date or timestamp.
    Temporal,
    /// Effectively unique per row (e.g. a UUID or database id) — not
    /// meaningful to chart directly.
    Identifier,
}

/// A lightweight profile of one field in a dataset, as a chart-type
/// recommender would receive from a data-inspection pass.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldProfile {
    /// Field name.
    pub name: String,
    /// Inferred kind.
    pub kind: FieldKind,
    /// Number of distinct values observed.
    pub distinct_count: usize,
    /// Total number of rows in the dataset.
    pub row_count: usize,
}

impl FieldProfile {
    /// Construct a new field profile.
    pub fn new(name: impl Into<String>, kind: FieldKind, distinct_count: usize, row_count: usize) -> Self {
        Self { name: name.into(), kind, distinct_count, row_count }
    }

    /// True if nearly every value is distinct (a strong identifier signal),
    /// even if the field wasn't explicitly typed as `Identifier`.
    pub fn looks_like_identifier(&self) -> bool {
        self.row_count > 0 && self.distinct_count as f64 / self.row_count as f64 > 0.95
    }
}

/// A chart type a dashboard builder might suggest for a set of fields.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecommendedChart {
    /// Trend over time.
    LineChart,
    /// Comparison across discrete categories.
    BarChart,
    /// Proportional breakdown of a small number of categories.
    PieChart,
    /// Relationship between two numeric measures.
    ScatterPlot,
    /// Distribution of a single numeric field.
    Histogram,
    /// Raw tabular listing, when nothing chartable is present.
    Table,
}

/// A recommendation with the reasoning behind it, so a caller (or a UI)
/// can explain the suggestion rather than present it as unexplained magic.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Recommendation {
    /// The suggested chart type.
    pub chart: RecommendedChart,
    /// A short, human-readable explanation.
    pub reason: String,
}
