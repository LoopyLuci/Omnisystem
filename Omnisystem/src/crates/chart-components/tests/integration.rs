//! End-to-end: build a chart, derive its scale and ticks, and check a
//! renderer-facing invariant (every point maps inside the pixel range).

use chart_components::{Chart, ChartType, LinearScale, Series};

#[test]
fn full_chart_to_scaled_points_pipeline() {
    let mut chart = Chart::new("Latency (ms)", ChartType::Line);
    let mut series = Series::new("p50");
    for (i, y) in [12.0, 18.0, 9.0, 40.0, 22.0].into_iter().enumerate() {
        series.push(i as f64, y);
    }
    chart.add_series(series);

    let (min, max) = chart.domain().expect("non-empty chart has a domain");
    let scale = LinearScale::new((min, max), (0.0, 400.0)).expect("valid domain");

    for point in &chart.series[0].points {
        let px = scale.map(point.y);
        assert!((0.0..=400.0).contains(&px), "point {point:?} mapped out of range: {px}");
    }

    let ticks = chart_components::nice_ticks(min, max, 4).expect("valid tick request");
    assert!(!ticks.is_empty());
    assert!(*ticks.first().unwrap() <= min);
    assert!(*ticks.last().unwrap() >= max);
}

#[test]
fn pie_chart_pipeline_covers_full_circle() {
    let mut chart = Chart::new("Traffic sources", ChartType::Pie);
    let mut series = Series::new("sources");
    for y in [40.0, 30.0, 20.0, 10.0] {
        series.push(0.0, y);
    }
    chart.add_series(series);

    let slices = chart.to_pie_slices().expect("positive totals");
    let total_fraction: f64 = slices.iter().map(|s| s.fraction).sum();
    assert!((total_fraction - 1.0).abs() < 1e-9);
    assert!((slices.last().unwrap().end_angle - 360.0).abs() < 1e-9);
}
