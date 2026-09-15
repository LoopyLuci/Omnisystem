//! Demo CLI: compute the visible window over a 1000-row uniform list.

use infinite_scroll_component::Virtualizer;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let heights = vec![24.0; 1000];
    let v = Virtualizer::new(&heights);
    let range = v.visible_range(2400.0, 600.0, 3)?;
    println!("visible: {:?}", range);
    println!("near bottom? {}", v.near_bottom(2400.0, 600.0, 200.0));
    Ok(())
}
