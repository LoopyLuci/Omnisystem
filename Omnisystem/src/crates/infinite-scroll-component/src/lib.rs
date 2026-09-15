//! List virtualization: given per-item (possibly variable) heights and a
//! scroll position, compute which items are visible plus a pagination
//! "near the bottom, fetch more" signal. Deliberately DOM-agnostic — this
//! crate computes indices and offsets, not actual rendering.

#![warn(missing_docs)]

pub mod error;
pub mod types;

pub use error::{Error, Result};
pub use types::*;

/// A virtualizer over a list of items with known (possibly non-uniform)
/// heights.
#[derive(Debug, Clone)]
pub struct Virtualizer {
    /// Cumulative height up to (but not including) item `i`, length
    /// `heights.len() + 1`, with a trailing total at the end.
    prefix: Vec<f64>,
}

impl Virtualizer {
    /// Build a virtualizer from a list of per-item pixel heights.
    pub fn new(heights: &[f64]) -> Self {
        let mut prefix = Vec::with_capacity(heights.len() + 1);
        prefix.push(0.0);
        let mut total = 0.0;
        for h in heights {
            total += h.max(0.0);
            prefix.push(total);
        }
        Self { prefix }
    }

    /// Total number of items.
    pub fn len(&self) -> usize {
        self.prefix.len().saturating_sub(1)
    }

    /// True if there are no items.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Total pixel height of all items combined.
    pub fn total_height(&self) -> f64 {
        *self.prefix.last().unwrap_or(&0.0)
    }

    /// The pixel top offset of item `index`.
    fn offset_of(&self, index: usize) -> f64 {
        self.prefix.get(index).copied().unwrap_or_else(|| self.total_height())
    }

    /// Compute the range of item indices visible in a viewport of
    /// `viewport_height` pixels, scrolled `scroll_offset` pixels from the
    /// top, expanded by `overscan` extra items on each side (to pre-render
    /// just-offscreen rows and avoid pop-in during fast scrolling).
    pub fn visible_range(&self, scroll_offset: f64, viewport_height: f64, overscan: usize) -> Result<VisibleRange> {
        if viewport_height <= 0.0 {
            return Err(Error::InvalidViewportHeight(viewport_height));
        }
        if self.is_empty() {
            return Ok(VisibleRange { start_index: 0, end_index: 0, offset_top: 0.0 });
        }
        let scroll_offset = scroll_offset.clamp(0.0, self.total_height());
        let viewport_end = scroll_offset + viewport_height;

        // First index whose bottom edge is at or past scroll_offset.
        let start = self.prefix.partition_point(|&top| top <= scroll_offset).saturating_sub(1).min(self.len() - 1);
        // Last index whose top edge is before viewport_end.
        let end = self.prefix[..self.len()].partition_point(|&top| top < viewport_end).saturating_sub(1).min(self.len() - 1);

        let start = start.saturating_sub(overscan);
        let end = (end + overscan).min(self.len() - 1);

        Ok(VisibleRange { start_index: start, end_index: end, offset_top: self.offset_of(start) })
    }

    /// True if the given scroll position is within `threshold` pixels of
    /// the bottom of the list — the conventional signal to fetch the next
    /// page.
    pub fn near_bottom(&self, scroll_offset: f64, viewport_height: f64, threshold: f64) -> bool {
        let distance_to_bottom = self.total_height() - (scroll_offset + viewport_height);
        distance_to_bottom <= threshold
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn uniform(count: usize, height: f64) -> Virtualizer {
        Virtualizer::new(&vec![height; count])
    }

    #[test]
    fn total_height_sums_all_items() {
        let v = uniform(10, 20.0);
        assert_eq!(v.total_height(), 200.0);
    }

    #[test]
    fn visible_range_at_top_starts_at_zero() {
        let v = uniform(100, 10.0);
        let range = v.visible_range(0.0, 50.0, 0).unwrap();
        assert_eq!(range.start_index, 0);
        assert_eq!(range.end_index, 4);
    }

    #[test]
    fn visible_range_mid_scroll_offsets_correctly() {
        let v = uniform(100, 10.0);
        // Scrolled 105px into a list of 10px rows: item 10 starts at 100, item 11 at 110.
        let range = v.visible_range(105.0, 30.0, 0).unwrap();
        assert_eq!(range.start_index, 10);
        assert_eq!(range.offset_top, 100.0);
    }

    #[test]
    fn overscan_expands_range_symmetrically() {
        let v = uniform(100, 10.0);
        let tight = v.visible_range(100.0, 30.0, 0).unwrap();
        let padded = v.visible_range(100.0, 30.0, 2).unwrap();
        assert_eq!(padded.start_index, tight.start_index.saturating_sub(2));
        assert_eq!(padded.end_index, (tight.end_index + 2).min(v.len() - 1));
    }

    #[test]
    fn range_clamps_to_list_bounds_near_the_end() {
        let v = uniform(10, 10.0);
        let range = v.visible_range(1000.0, 50.0, 3).unwrap();
        assert_eq!(range.end_index, 9);
    }

    #[test]
    fn variable_heights_are_respected() {
        let v = Virtualizer::new(&[10.0, 100.0, 10.0, 10.0]);
        // Item 1 spans [10, 110); a viewport at offset 50 should include it.
        let range = v.visible_range(50.0, 20.0, 0).unwrap();
        assert_eq!(range.start_index, 1);
    }

    #[test]
    fn rejects_non_positive_viewport_height() {
        let v = uniform(10, 10.0);
        assert!(matches!(v.visible_range(0.0, 0.0, 0), Err(Error::InvalidViewportHeight(_))));
    }

    #[test]
    fn empty_list_has_zero_range() {
        let v = Virtualizer::new(&[]);
        let range = v.visible_range(0.0, 100.0, 0).unwrap();
        assert_eq!(range.start_index, 0);
        assert_eq!(range.end_index, 0);
    }

    #[test]
    fn near_bottom_true_within_threshold() {
        let v = uniform(10, 10.0); // total 100
        assert!(v.near_bottom(50.0, 40.0, 15.0)); // 100 - 90 = 10 <= 15
    }

    #[test]
    fn near_bottom_false_when_far_from_end() {
        let v = uniform(10, 10.0);
        assert!(!v.near_bottom(0.0, 20.0, 5.0)); // 100 - 20 = 80 > 5
    }
}
