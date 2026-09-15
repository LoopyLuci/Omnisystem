//! Dashboard grid layout: a skyline-based auto-packer that places widgets
//! into a fixed-column-count grid in first-fit order, plus an overlap
//! validator for manually authored layouts. Deliberately rendering-agnostic
//! — this crate computes grid coordinates, not pixels or DOM nodes.

#![warn(missing_docs)]

pub mod error;
pub mod types;

pub use error::{Error, Result};
pub use types::*;

/// A fixed-width dashboard grid.
#[derive(Debug, Clone)]
pub struct Grid {
    columns: u32,
}

impl Grid {
    /// Construct a grid with the given column count.
    pub fn new(columns: u32) -> Self {
        Self { columns }
    }

    /// Automatically pack `widgets` into this grid using a skyline
    /// algorithm: for each widget in order, find the lowest row at which a
    /// contiguous run of `width` columns is free (leftmost among ties),
    /// place it there, and raise the skyline under it.
    pub fn auto_pack(&self, widgets: &[WidgetSpec]) -> Result<Vec<PlacedWidget>> {
        let mut skyline = vec![0u32; self.columns as usize];
        let mut placed = Vec::with_capacity(widgets.len());

        for spec in widgets {
            if spec.width > self.columns {
                return Err(Error::WidgetTooWide(spec.id.clone(), spec.width, self.columns));
            }
            let (col, row) = self.best_position(&skyline, spec.width);
            for c in col..col + spec.width {
                skyline[c as usize] = row + spec.height;
            }
            placed.push(PlacedWidget { id: spec.id.clone(), col, row, width: spec.width, height: spec.height });
        }
        Ok(placed)
    }

    /// Find the leftmost column achieving the lowest possible starting row
    /// for a widget of the given width.
    fn best_position(&self, skyline: &[u32], width: u32) -> (u32, u32) {
        let mut best_col = 0u32;
        let mut best_row = u32::MAX;
        for start in 0..=(self.columns - width) {
            let window_max = skyline[start as usize..(start + width) as usize].iter().copied().max().unwrap_or(0);
            if window_max < best_row {
                best_row = window_max;
                best_col = start;
            }
        }
        (best_col, best_row)
    }

    /// Validate a manually authored layout: every widget must fit within
    /// the grid's column count, and no two widgets may overlap.
    pub fn validate(&self, placements: &[PlacedWidget]) -> Result<()> {
        for p in placements {
            if p.col + p.width > self.columns {
                return Err(Error::WidgetTooWide(p.id.clone(), p.width, self.columns));
            }
        }
        for i in 0..placements.len() {
            for j in (i + 1)..placements.len() {
                if Self::rects_overlap(&placements[i], &placements[j]) {
                    return Err(Error::Overlap(placements[i].id.clone(), placements[j].id.clone()));
                }
            }
        }
        Ok(())
    }

    fn rects_overlap(a: &PlacedWidget, b: &PlacedWidget) -> bool {
        a.col < b.col + b.width && b.col < a.col + a.width && a.row < b.row + b.height && b.row < a.row + a.height
    }

    /// The total number of rows spanned by a set of placed widgets (the
    /// grid's resulting content height).
    pub fn content_height(&self, placements: &[PlacedWidget]) -> u32 {
        placements.iter().map(|p| p.row + p.height).max().unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn packs_widgets_left_to_right_on_first_row() {
        let grid = Grid::new(12);
        let widgets = vec![WidgetSpec::new("a", 4, 2), WidgetSpec::new("b", 4, 2)];
        let placed = grid.auto_pack(&widgets).unwrap();
        assert_eq!(placed[0].col, 0);
        assert_eq!(placed[1].col, 4);
        assert_eq!(placed[0].row, 0);
        assert_eq!(placed[1].row, 0);
    }

    #[test]
    fn wraps_to_next_row_when_no_space_remains() {
        let grid = Grid::new(12);
        let widgets = vec![WidgetSpec::new("a", 8, 2), WidgetSpec::new("b", 8, 2)];
        let placed = grid.auto_pack(&widgets).unwrap();
        assert_eq!(placed[1].row, 2);
        assert_eq!(placed[1].col, 0);
    }

    #[test]
    fn fills_gaps_beside_shorter_widgets() {
        let grid = Grid::new(12);
        // "a" is tall and narrow; "b" and "c" are short and should stack
        // beside it before wrapping past "a"'s height.
        let widgets = vec![
            WidgetSpec::new("a", 4, 4),
            WidgetSpec::new("b", 8, 2),
            WidgetSpec::new("c", 8, 2),
        ];
        let placed = grid.auto_pack(&widgets).unwrap();
        let b = &placed[1];
        let c = &placed[2];
        assert_eq!(b.col, 4);
        assert_eq!(b.row, 0);
        assert_eq!(c.col, 4);
        assert_eq!(c.row, 2);
    }

    #[test]
    fn auto_pack_rejects_widget_wider_than_grid() {
        let grid = Grid::new(6);
        let widgets = vec![WidgetSpec::new("a", 10, 1)];
        assert!(matches!(grid.auto_pack(&widgets), Err(Error::WidgetTooWide(_, 10, 6))));
    }

    #[test]
    fn auto_pack_produces_no_overlaps() {
        let grid = Grid::new(12);
        let widgets = vec![
            WidgetSpec::new("a", 5, 3),
            WidgetSpec::new("b", 7, 2),
            WidgetSpec::new("c", 4, 4),
            WidgetSpec::new("d", 8, 1),
            WidgetSpec::new("e", 3, 5),
        ];
        let placed = grid.auto_pack(&widgets).unwrap();
        assert!(grid.validate(&placed).is_ok());
    }

    #[test]
    fn validate_detects_overlap() {
        let grid = Grid::new(12);
        let placements = vec![
            PlacedWidget { id: "a".into(), col: 0, row: 0, width: 6, height: 2 },
            PlacedWidget { id: "b".into(), col: 3, row: 0, width: 6, height: 2 },
        ];
        assert!(matches!(grid.validate(&placements), Err(Error::Overlap(_, _))));
    }

    #[test]
    fn validate_accepts_adjacent_non_overlapping_widgets() {
        let grid = Grid::new(12);
        let placements = vec![
            PlacedWidget { id: "a".into(), col: 0, row: 0, width: 6, height: 2 },
            PlacedWidget { id: "b".into(), col: 6, row: 0, width: 6, height: 2 },
        ];
        assert!(grid.validate(&placements).is_ok());
    }

    #[test]
    fn validate_rejects_widget_extending_past_grid_edge() {
        let grid = Grid::new(12);
        let placements = vec![PlacedWidget { id: "a".into(), col: 10, row: 0, width: 4, height: 1 }];
        assert!(matches!(grid.validate(&placements), Err(Error::WidgetTooWide(_, 4, 12))));
    }

    #[test]
    fn content_height_is_the_tallest_bottom_edge() {
        let grid = Grid::new(12);
        let placements = vec![
            PlacedWidget { id: "a".into(), col: 0, row: 0, width: 4, height: 2 },
            PlacedWidget { id: "b".into(), col: 4, row: 0, width: 4, height: 5 },
        ];
        assert_eq!(grid.content_height(&placements), 5);
    }

    #[test]
    fn empty_layout_has_zero_content_height() {
        let grid = Grid::new(12);
        assert_eq!(grid.content_height(&[]), 0);
    }
}
