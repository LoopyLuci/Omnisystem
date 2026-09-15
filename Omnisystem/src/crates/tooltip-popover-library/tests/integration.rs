//! End-to-end: place a popover near each edge of a viewport and check it
//! always stays fully within bounds.

use tooltip_popover_library::{place, Rect, Side};

#[test]
fn popover_stays_within_viewport_near_every_edge() {
    let viewport = Rect::new(0.0, 0.0, 1000.0, 800.0);
    let size = (180.0, 60.0);
    let anchors = [
        Rect::new(10.0, 10.0, 40.0, 20.0),     // top-left corner
        Rect::new(950.0, 10.0, 40.0, 20.0),    // top-right corner
        Rect::new(10.0, 780.0, 40.0, 20.0),    // bottom-left corner
        Rect::new(950.0, 780.0, 40.0, 20.0),   // bottom-right corner
        Rect::new(500.0, 400.0, 40.0, 20.0),   // center
    ];

    for anchor in anchors {
        for preferred in [Side::Top, Side::Bottom, Side::Left, Side::Right] {
            let placement = place(anchor, size, viewport, preferred, 6.0).unwrap();
            assert!(placement.x >= viewport.x, "x={} escaped left edge for anchor {:?}", placement.x, anchor);
            assert!(placement.y >= viewport.y, "y={} escaped top edge for anchor {:?}", placement.y, anchor);
            assert!(placement.x + size.0 <= viewport.right() + 1e-9, "popover escaped right edge");
            assert!(placement.y + size.1 <= viewport.bottom() + 1e-9, "popover escaped bottom edge");
        }
    }
}
