//! Statistical data-shaping for visualizations: descriptive summaries and
//! equal-width histogram bucketing. Deliberately renderer-agnostic — this
//! crate prepares the numbers a chart would plot.

#![warn(missing_docs)]

pub mod error;
pub mod types;

pub use error::{Error, Result};
pub use types::*;

/// Compute descriptive statistics for `values`.
pub fn summarize(values: &[f64]) -> Result<Summary> {
    if values.is_empty() {
        return Err(Error::EmptyDataset);
    }
    let count = values.len();
    let mean = values.iter().sum::<f64>() / count as f64;
    let variance = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / count as f64;
    let stddev = variance.sqrt();
    let min = values.iter().cloned().fold(f64::INFINITY, f64::min);
    let max = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let median = if count % 2 == 1 {
        sorted[count / 2]
    } else {
        (sorted[count / 2 - 1] + sorted[count / 2]) / 2.0
    };

    Ok(Summary { count, mean, median, stddev, min, max })
}

/// Bucket `values` into `bucket_count` equal-width histogram buckets
/// spanning `[min, max]`. When every value is identical, a single bucket
/// containing all values is returned.
pub fn histogram(values: &[f64], bucket_count: usize) -> Result<Vec<Bucket>> {
    if values.is_empty() {
        return Err(Error::EmptyDataset);
    }
    if bucket_count == 0 {
        return Err(Error::InvalidBucketCount);
    }
    let min = values.iter().cloned().fold(f64::INFINITY, f64::min);
    let max = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

    if min == max {
        return Ok(vec![Bucket { lower: min, upper: max, count: values.len() }]);
    }

    let width = (max - min) / bucket_count as f64;
    let mut buckets: Vec<Bucket> = (0..bucket_count)
        .map(|i| Bucket { lower: min + width * i as f64, upper: min + width * (i + 1) as f64, count: 0 })
        .collect();

    for &v in values {
        let idx = (((v - min) / width) as usize).min(bucket_count - 1);
        buckets[idx].count += 1;
    }

    Ok(buckets)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn summary_computes_mean_and_extremes() {
        let s = summarize(&[1.0, 2.0, 3.0, 4.0, 5.0]).unwrap();
        assert_eq!(s.mean, 3.0);
        assert_eq!(s.min, 1.0);
        assert_eq!(s.max, 5.0);
        assert_eq!(s.count, 5);
    }

    #[test]
    fn summary_median_odd_count() {
        let s = summarize(&[5.0, 1.0, 3.0]).unwrap();
        assert_eq!(s.median, 3.0);
    }

    #[test]
    fn summary_median_even_count_averages_middle_two() {
        let s = summarize(&[1.0, 2.0, 3.0, 4.0]).unwrap();
        assert_eq!(s.median, 2.5);
    }

    #[test]
    fn summary_stddev_of_constant_dataset_is_zero() {
        let s = summarize(&[7.0, 7.0, 7.0]).unwrap();
        assert_eq!(s.stddev, 0.0);
    }

    #[test]
    fn summary_stddev_matches_known_value() {
        // [2, 4, 4, 4, 5, 5, 7, 9]: population stddev is 2.0 (classic example).
        let s = summarize(&[2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0]).unwrap();
        assert!((s.stddev - 2.0).abs() < 1e-9);
    }

    #[test]
    fn summary_rejects_empty_dataset() {
        assert_eq!(summarize(&[]).unwrap_err(), Error::EmptyDataset);
    }

    #[test]
    fn histogram_buckets_cover_full_range() {
        let values = [0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0];
        let buckets = histogram(&values, 5).unwrap();
        assert_eq!(buckets.len(), 5);
        assert_eq!(buckets.first().unwrap().lower, 0.0);
        assert_eq!(buckets.last().unwrap().upper, 9.0);
    }

    #[test]
    fn histogram_total_count_matches_input_length() {
        let values = [1.0, 2.0, 2.0, 3.0, 8.0, 8.5, 9.0];
        let buckets = histogram(&values, 4).unwrap();
        let total: usize = buckets.iter().map(|b| b.count).sum();
        assert_eq!(total, values.len());
    }

    #[test]
    fn histogram_places_max_value_in_last_bucket() {
        let values = [0.0, 10.0];
        let buckets = histogram(&values, 4).unwrap();
        assert_eq!(buckets.last().unwrap().count, 1);
    }

    #[test]
    fn histogram_of_identical_values_is_single_bucket() {
        let buckets = histogram(&[5.0, 5.0, 5.0], 10).unwrap();
        assert_eq!(buckets.len(), 1);
        assert_eq!(buckets[0].count, 3);
    }

    #[test]
    fn histogram_rejects_zero_buckets() {
        assert_eq!(histogram(&[1.0, 2.0], 0).unwrap_err(), Error::InvalidBucketCount);
    }

    #[test]
    fn histogram_rejects_empty_dataset() {
        assert_eq!(histogram(&[], 5).unwrap_err(), Error::EmptyDataset);
    }
}
