//! Grid widget types.

use serde::{Deserialize, Serialize};

/// A widget's requested footprint, in grid cell units.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WidgetSpec {
    /// Unique widget identifier.
    pub id: String,
    /// Width in grid columns.
    pub width: u32,
    /// Height in grid rows.
    pub height: u32,
}

impl WidgetSpec {
    /// Construct a new widget spec.
    pub fn new(id: impl Into<String>, width: u32, height: u32) -> Self {
        Self { id: id.into(), width, height }
    }
}

/// A widget placed at a specific grid position.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlacedWidget {
    /// Widget identifier.
    pub id: String,
    /// Zero-indexed starting column.
    pub col: u32,
    /// Zero-indexed starting row.
    pub row: u32,
    /// Width in grid columns.
    pub width: u32,
    /// Height in grid rows.
    pub height: u32,
}
