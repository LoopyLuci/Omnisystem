//! Demo CLI: group revenue rows by region and print the top region.

use analytics_viewer_ui::{group_by, top_n, Aggregate, Row};
use std::collections::HashMap;

fn row(dim: &str, revenue: f64) -> Row {
    let mut measures = HashMap::new();
    measures.insert("revenue".to_string(), revenue);
    Row { dimension: dim.into(), measures }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rows = vec![row("east", 100.0), row("east", 50.0), row("west", 200.0)];
    let groups = group_by(&rows, "revenue", Aggregate::Sum)?;
    for top in top_n(groups, 1) {
        println!("top region: {} (${:.2})", top.dimension, top.value);
    }
    Ok(())
}
