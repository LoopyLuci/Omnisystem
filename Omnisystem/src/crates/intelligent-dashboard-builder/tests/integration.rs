//! End-to-end: run recommendation over several realistic field combos drawn
//! from one hypothetical analytics dataset, checking each lands on a
//! sensible, distinct chart type.

use intelligent_dashboard_builder::{recommend, FieldKind, FieldProfile, RecommendedChart};

#[test]
fn dataset_wide_recommendation_pass() {
    let event_id = FieldProfile::new("event_id", FieldKind::Identifier, 10_000, 10_000);
    let timestamp = FieldProfile::new("timestamp", FieldKind::Temporal, 10_000, 10_000);
    let latency = FieldProfile::new("latency_ms", FieldKind::Numeric, 8_500, 10_000);
    let region = FieldProfile::new("region", FieldKind::Categorical, 5, 10_000);
    let status_code = FieldProfile::new("status_code", FieldKind::Categorical, 40, 10_000);
    let bytes_sent = FieldProfile::new("bytes_sent", FieldKind::Numeric, 9_000, 10_000);

    assert_eq!(recommend(&[event_id.clone()]).unwrap().chart, RecommendedChart::Table);
    assert_eq!(
        recommend(&[timestamp.clone(), latency.clone()]).unwrap().chart,
        RecommendedChart::LineChart
    );
    assert_eq!(recommend(&[region.clone(), latency.clone()]).unwrap().chart, RecommendedChart::PieChart);
    assert_eq!(
        recommend(&[status_code.clone(), latency.clone()]).unwrap().chart,
        RecommendedChart::BarChart
    );
    assert_eq!(recommend(&[latency.clone(), bytes_sent.clone()]).unwrap().chart, RecommendedChart::ScatterPlot);
    assert_eq!(recommend(&[latency]).unwrap().chart, RecommendedChart::Histogram);
}
