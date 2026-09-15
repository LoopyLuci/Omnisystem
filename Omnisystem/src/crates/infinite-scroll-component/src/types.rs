//! Virtualized list types.

use serde::{Deserialize, Serialize};

/// The computed visible window into a virtualized list.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct VisibleRange {
    /// Index of the first item to render (inclusive).
    pub start_index: usize,
    /// Index of the last item to render (inclusive).
    pub end_index: usize,
    /// Pixel offset of `start_index`'s top edge from the list's top.
    pub offset_top: f64,
}
