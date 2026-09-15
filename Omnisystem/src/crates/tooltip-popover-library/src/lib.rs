//! Popover/tooltip placement: pick a side relative to an anchor element,
//! flipping to the opposite side when the preferred one doesn't fit in the
//! viewport, and clamping along the cross axis so the popover never spills
//! off-screen. Deliberately DOM-agnostic — this crate computes coordinates,
//! not a rendered overlay.

#![warn(missing_docs)]

pub mod error;
pub mod types;

pub use error::{Error, Result};
pub use types::*;

/// Compute where to place a popover of size `(width, height)` relative to
/// `anchor`, preferring `preferred_side`, with `gap` pixels of separation
/// from the anchor, constrained to stay within `viewport`.
pub fn place(anchor: Rect, size: (f64, f64), viewport: Rect, preferred_side: Side, gap: f64) -> Result<Placement> {
    let (w, h) = size;

    let fits = |side: Side| -> bool {
        let (x, y) = position_for(anchor, size, side, gap);
        x >= viewport.x && y >= viewport.y && x + w <= viewport.right() && y + h <= viewport.bottom()
    };

    let side = if fits(preferred_side) {
        preferred_side
    } else if fits(preferred_side.opposite()) {
        preferred_side.opposite()
    } else {
        // Neither preferred side nor its opposite fits without clamping;
        // fall back to whichever gives the least overflow, then clamp.
        preferred_side
    };

    let (raw_x, raw_y) = position_for(anchor, size, side, gap);
    let clamped_x = raw_x.clamp(viewport.x, (viewport.right() - w).max(viewport.x));
    let clamped_y = raw_y.clamp(viewport.y, (viewport.bottom() - h).max(viewport.y));

    if w > viewport.w || h > viewport.h {
        return Err(Error::NoFit);
    }

    Ok(Placement { side, x: clamped_x, y: clamped_y })
}

fn position_for(anchor: Rect, size: (f64, f64), side: Side, gap: f64) -> (f64, f64) {
    let (w, h) = size;
    match side {
        Side::Top => (anchor.x + (anchor.w - w) / 2.0, anchor.y - h - gap),
        Side::Bottom => (anchor.x + (anchor.w - w) / 2.0, anchor.bottom() + gap),
        Side::Left => (anchor.x - w - gap, anchor.y + (anchor.h - h) / 2.0),
        Side::Right => (anchor.right() + gap, anchor.y + (anchor.h - h) / 2.0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn viewport() -> Rect {
        Rect::new(0.0, 0.0, 800.0, 600.0)
    }

    #[test]
    fn places_below_when_it_fits() {
        let anchor = Rect::new(100.0, 100.0, 50.0, 20.0);
        let placement = place(anchor, (100.0, 40.0), viewport(), Side::Bottom, 8.0).unwrap();
        assert_eq!(placement.side, Side::Bottom);
        assert_eq!(placement.y, anchor.bottom() + 8.0);
    }

    #[test]
    fn flips_to_bottom_when_top_would_overflow() {
        // Anchor near the very top of the viewport: no room above it.
        let anchor = Rect::new(100.0, 5.0, 50.0, 20.0);
        let placement = place(anchor, (100.0, 40.0), viewport(), Side::Top, 8.0).unwrap();
        assert_eq!(placement.side, Side::Bottom);
    }

    #[test]
    fn flips_to_top_when_bottom_would_overflow() {
        let anchor = Rect::new(100.0, 590.0, 50.0, 5.0);
        let placement = place(anchor, (100.0, 40.0), viewport(), Side::Bottom, 8.0).unwrap();
        assert_eq!(placement.side, Side::Top);
    }

    #[test]
    fn flips_left_right_symmetrically() {
        let anchor = Rect::new(5.0, 300.0, 20.0, 20.0);
        let placement = place(anchor, (100.0, 40.0), viewport(), Side::Left, 8.0).unwrap();
        assert_eq!(placement.side, Side::Right);
    }

    #[test]
    fn cross_axis_position_is_centered_on_anchor() {
        let anchor = Rect::new(400.0, 100.0, 40.0, 20.0);
        let placement = place(anchor, (100.0, 40.0), viewport(), Side::Bottom, 8.0).unwrap();
        // Centered: anchor center x = 420, popover half-width = 50 -> x = 370.
        assert_eq!(placement.x, 370.0);
    }

    #[test]
    fn clamps_cross_axis_when_centering_would_overflow_viewport() {
        // Anchor near the left edge; a wide popover centered on it would
        // spill past x=0.
        let anchor = Rect::new(2.0, 100.0, 10.0, 20.0);
        let placement = place(anchor, (200.0, 40.0), viewport(), Side::Bottom, 8.0).unwrap();
        assert!(placement.x >= 0.0);
    }

    #[test]
    fn errors_when_popover_exceeds_viewport_dimensions() {
        let anchor = Rect::new(100.0, 100.0, 50.0, 20.0);
        let result = place(anchor, (1000.0, 40.0), viewport(), Side::Bottom, 8.0);
        assert_eq!(result, Err(Error::NoFit));
    }

    #[test]
    fn opposite_side_pairs_are_correct() {
        assert_eq!(Side::Top.opposite(), Side::Bottom);
        assert_eq!(Side::Bottom.opposite(), Side::Top);
        assert_eq!(Side::Left.opposite(), Side::Right);
        assert_eq!(Side::Right.opposite(), Side::Left);
    }

    #[test]
    fn respects_requested_gap_distance() {
        let anchor = Rect::new(100.0, 100.0, 50.0, 20.0);
        let close = place(anchor, (50.0, 20.0), viewport(), Side::Bottom, 4.0).unwrap();
        let far = place(anchor, (50.0, 20.0), viewport(), Side::Bottom, 20.0).unwrap();
        assert!(far.y > close.y);
        assert_eq!(far.y - close.y, 16.0);
    }
}
