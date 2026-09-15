//! Demo CLI: build a small bar chart and print its domain, axis ticks, and
//! pie-slice breakdown.

use chart_components::{Chart, ChartType, Series};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut chart = Chart::new("Revenue", ChartType::Bar);
    let mut series = Series::new("Q1-Q4");
    for (i, value) in [10.0, 25.0, 5.0, 60.0].into_iter().enumerate() {
        series.push(i as f64, value);
    }
    chart.add_series(series);

    let (min, max) = chart.domain()?;
    println!("domain: [{min}, {max}]");

    let ticks = chart_components::nice_ticks(min, max, 5)?;
    println!("ticks: {ticks:?}");

    for slice in chart.to_pie_slices()? {
        println!(
            "slice: value={:.1} fraction={:.2} angle=[{:.1}, {:.1}]",
            slice.value, slice.fraction, slice.start_angle, slice.end_angle
        );
    }
    Ok(())
}
