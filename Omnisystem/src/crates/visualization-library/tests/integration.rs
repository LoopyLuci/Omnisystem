//! End-to-end: summarize a dataset, then bucket it, and check the two views
//! agree (histogram total equals summary count, buckets span min..max).

use visualization_library::{histogram, summarize};

#[test]
fn summary_and_histogram_agree_on_the_same_dataset() {
    let values: Vec<f64> = (0..100).map(|i| (i as f64 * 1.7).sin() * 50.0 + 100.0).collect();

    let summary = summarize(&values).unwrap();
    assert_eq!(summary.count, values.len());

    let buckets = histogram(&values, 10).unwrap();
    let total: usize = buckets.iter().map(|b| b.count).sum();
    assert_eq!(total, values.len());

    assert!((buckets.first().unwrap().lower - summary.min).abs() < 1e-9);
    assert!((buckets.last().unwrap().upper - summary.max).abs() < 1e-9);
}
