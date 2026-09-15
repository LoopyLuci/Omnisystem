//! Demo CLI: recommend a chart type for a category + count field pair.

use intelligent_dashboard_builder::{recommend, FieldKind, FieldProfile};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let fields = vec![
        FieldProfile::new("plan", FieldKind::Categorical, 4, 800),
        FieldProfile::new("subscribers", FieldKind::Numeric, 800, 800),
    ];
    let rec = recommend(&fields)?;
    println!("{:?}: {}", rec.chart, rec.reason);
    Ok(())
}
