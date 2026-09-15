//! Analytics viewer: groups tabular [`Row`]s by their `dimension` and
//! aggregates a chosen numeric measure across each group.

#![warn(missing_docs)]

use std::collections::BTreeMap;

pub mod error;
pub mod types;

pub use error::{Error, Result};
pub use types::*;

/// Group `rows` by `dimension` and aggregate `measure` with `agg`, sorted
/// by dimension name ascending.
pub fn group_by(rows: &[Row], measure: &str, agg: Aggregate) -> Result<Vec<GroupResult>> {
    if rows.is_empty() {
        return Err(Error::EmptyDataset);
    }
    if !rows.iter().any(|r| r.measures.contains_key(measure)) {
        return Err(Error::UnknownMeasure(measure.to_string()));
    }
    let mut buckets: BTreeMap<String, Vec<f64>> = BTreeMap::new();
    for row in rows {
        if let Some(&v) = row.measures.get(measure) {
            buckets.entry(row.dimension.clone()).or_default().push(v);
        }
    }
    let mut out = Vec::with_capacity(buckets.len());
    for (dimension, values) in buckets {
        let row_count = values.len();
        let value = match agg {
            Aggregate::Sum => values.iter().sum(),
            Aggregate::Avg => values.iter().sum::<f64>() / row_count as f64,
            Aggregate::Count => row_count as f64,
            Aggregate::Min => values.iter().cloned().fold(f64::INFINITY, f64::min),
            Aggregate::Max => values.iter().cloned().fold(f64::NEG_INFINITY, f64::max),
        };
        out.push(GroupResult { dimension, value, row_count });
    }
    Ok(out)
}

/// Rank groups by their aggregated value, descending, returning the top `n`.
pub fn top_n(mut groups: Vec<GroupResult>, n: usize) -> Vec<GroupResult> {
    groups.sort_by(|a, b| b.value.partial_cmp(&a.value).unwrap_or(std::cmp::Ordering::Equal));
    groups.truncate(n);
    groups
}

/// Total across all rows for a measure, regardless of grouping.
pub fn total(rows: &[Row], measure: &str) -> Result<f64> {
    if rows.is_empty() {
        return Err(Error::EmptyDataset);
    }
    if !rows.iter().any(|r| r.measures.contains_key(measure)) {
        return Err(Error::UnknownMeasure(measure.to_string()));
    }
    Ok(rows.iter().filter_map(|r| r.measures.get(measure)).sum())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(dim: &str, measures: &[(&str, f64)]) -> Row {
        Row { dimension: dim.into(), measures: measures.iter().map(|(k, v)| (k.to_string(), *v)).collect() }
    }

    fn dataset() -> Vec<Row> {
        vec![
            row("east", &[("revenue", 100.0)]),
            row("east", &[("revenue", 50.0)]),
            row("west", &[("revenue", 200.0)]),
        ]
    }

    #[test]
    fn group_by_sum_aggregates_correctly() {
        let groups = group_by(&dataset(), "revenue", Aggregate::Sum).unwrap();
        assert_eq!(groups.iter().find(|g| g.dimension == "east").unwrap().value, 150.0);
        assert_eq!(groups.iter().find(|g| g.dimension == "west").unwrap().value, 200.0);
    }

    #[test]
    fn group_by_avg_aggregates_correctly() {
        let groups = group_by(&dataset(), "revenue", Aggregate::Avg).unwrap();
        assert_eq!(groups.iter().find(|g| g.dimension == "east").unwrap().value, 75.0);
    }

    #[test]
    fn group_by_count_counts_rows() {
        let groups = group_by(&dataset(), "revenue", Aggregate::Count).unwrap();
        assert_eq!(groups.iter().find(|g| g.dimension == "east").unwrap().row_count, 2);
    }

    #[test]
    fn group_by_min_max() {
        let groups_min = group_by(&dataset(), "revenue", Aggregate::Min).unwrap();
        let groups_max = group_by(&dataset(), "revenue", Aggregate::Max).unwrap();
        assert_eq!(groups_min.iter().find(|g| g.dimension == "east").unwrap().value, 50.0);
        assert_eq!(groups_max.iter().find(|g| g.dimension == "east").unwrap().value, 100.0);
    }

    #[test]
    fn group_by_errors_on_empty_dataset() {
        assert_eq!(group_by(&[], "revenue", Aggregate::Sum), Err(Error::EmptyDataset));
    }

    #[test]
    fn group_by_errors_on_unknown_measure() {
        assert_eq!(
            group_by(&dataset(), "profit", Aggregate::Sum),
            Err(Error::UnknownMeasure("profit".into()))
        );
    }

    #[test]
    fn top_n_returns_highest_values_first() {
        let groups = group_by(&dataset(), "revenue", Aggregate::Sum).unwrap();
        let top = top_n(groups, 1);
        assert_eq!(top.len(), 1);
        assert_eq!(top[0].dimension, "west");
    }

    #[test]
    fn total_sums_across_all_rows() {
        assert_eq!(total(&dataset(), "revenue").unwrap(), 350.0);
    }

    #[test]
    fn total_errors_on_empty_dataset() {
        assert_eq!(total(&[], "revenue"), Err(Error::EmptyDataset));
    }
}
