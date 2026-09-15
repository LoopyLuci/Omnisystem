//! Demo CLI: auto-pack a small widget set into a 12-column grid.

use dashboard_engine::{Grid, WidgetSpec};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let grid = Grid::new(12);
    let widgets = vec![
        WidgetSpec::new("revenue-chart", 6, 3),
        WidgetSpec::new("active-users", 3, 2),
        WidgetSpec::new("error-rate", 3, 2),
        WidgetSpec::new("logs", 12, 4),
    ];
    let placed = grid.auto_pack(&widgets)?;
    for p in &placed {
        println!("{}: col={} row={} ({}x{})", p.id, p.col, p.row, p.width, p.height);
    }
    println!("content height: {}", grid.content_height(&placed));
    Ok(())
}
