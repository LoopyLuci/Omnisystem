//! Demo CLI: place a tooltip below a button, showing the flip when it
//! would overflow the viewport.

use tooltip_popover_library::{place, Rect, Side};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let viewport = Rect::new(0.0, 0.0, 1024.0, 768.0);
    let anchor = Rect::new(500.0, 10.0, 80.0, 32.0); // near the top edge
    let placement = place(anchor, (160.0, 48.0), viewport, Side::Top, 8.0)?;
    println!("placed on {:?} at ({}, {})", placement.side, placement.x, placement.y);
    Ok(())
}
