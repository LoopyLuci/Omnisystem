//! Chart-type recommendation: a rule-based heuristic that looks at a
//! dataset's field profiles (kind + cardinality) and suggests the most
//! appropriate chart type, with a human-readable reason. Deliberately not
//! a machine-learned model — this is the same kind of explainable
//! heuristic real BI tools (Tableau's "Show Me", Power BI's autochart)
//! start from.

#![warn(missing_docs)]

pub mod error;
pub mod types;

pub use error::{Error, Result};
pub use types::*;

/// Categories at or below this count are considered small enough for a pie
/// chart to remain readable; above it, a bar chart is preferred.
const PIE_CHART_MAX_CATEGORIES: usize = 6;

/// Recommend a chart type for the given fields (typically 1-2 fields a user
/// has selected to visualize together).
pub fn recommend(fields: &[FieldProfile]) -> Result<Recommendation> {
    if fields.is_empty() {
        return Err(Error::NoFields);
    }

    let categorical: Vec<&FieldProfile> = fields.iter().filter(|f| f.kind == FieldKind::Categorical).collect();
    let numeric: Vec<&FieldProfile> = fields.iter().filter(|f| f.kind == FieldKind::Numeric).collect();
    let temporal: Vec<&FieldProfile> = fields.iter().filter(|f| f.kind == FieldKind::Temporal).collect();
    let identifier_like = fields.iter().any(|f| f.kind == FieldKind::Identifier || f.looks_like_identifier());

    // An identifier-shaped field with nothing else chartable: just list it.
    if identifier_like && numeric.is_empty() && temporal.is_empty() {
        return Ok(Recommendation {
            chart: RecommendedChart::Table,
            reason: "field looks like a unique identifier, not a chartable measure".into(),
        });
    }

    if !temporal.is_empty() && !numeric.is_empty() {
        return Ok(Recommendation {
            chart: RecommendedChart::LineChart,
            reason: format!("'{}' is a time field paired with a numeric measure: trend over time", temporal[0].name),
        });
    }

    if categorical.len() == 1 && numeric.len() == 1 {
        let cat = categorical[0];
        if cat.distinct_count <= PIE_CHART_MAX_CATEGORIES {
            return Ok(Recommendation {
                chart: RecommendedChart::PieChart,
                reason: format!(
                    "'{}' has only {} categories: a proportional breakdown fits",
                    cat.name, cat.distinct_count
                ),
            });
        }
        return Ok(Recommendation {
            chart: RecommendedChart::BarChart,
            reason: format!(
                "'{}' has {} categories, too many for a readable pie chart",
                cat.name, cat.distinct_count
            ),
        });
    }

    if numeric.len() >= 2 {
        return Ok(Recommendation {
            chart: RecommendedChart::ScatterPlot,
            reason: "two or more numeric measures: relationship is best seen as a scatter plot".into(),
        });
    }

    if numeric.len() == 1 && categorical.is_empty() && temporal.is_empty() {
        return Ok(Recommendation {
            chart: RecommendedChart::Histogram,
            reason: "a single numeric measure with no grouping field: show its distribution".into(),
        });
    }

    if categorical.len() > 1 {
        return Ok(Recommendation {
            chart: RecommendedChart::BarChart,
            reason: "multiple categorical fields: grouped bar comparison".into(),
        });
    }

    Ok(Recommendation {
        chart: RecommendedChart::Table,
        reason: "no clear chartable pattern found; showing raw values".into(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn field(name: &str, kind: FieldKind, distinct: usize, rows: usize) -> FieldProfile {
        FieldProfile::new(name, kind, distinct, rows)
    }

    #[test]
    fn rejects_empty_field_list() {
        assert_eq!(recommend(&[]).unwrap_err(), Error::NoFields);
    }

    #[test]
    fn temporal_plus_numeric_suggests_line_chart() {
        let fields = vec![field("date", FieldKind::Temporal, 30, 30), field("revenue", FieldKind::Numeric, 30, 30)];
        let rec = recommend(&fields).unwrap();
        assert_eq!(rec.chart, RecommendedChart::LineChart);
    }

    #[test]
    fn small_category_count_suggests_pie_chart() {
        let fields = vec![field("plan", FieldKind::Categorical, 3, 500), field("count", FieldKind::Numeric, 500, 500)];
        let rec = recommend(&fields).unwrap();
        assert_eq!(rec.chart, RecommendedChart::PieChart);
    }

    #[test]
    fn large_category_count_suggests_bar_chart() {
        let fields = vec![field("country", FieldKind::Categorical, 40, 500), field("count", FieldKind::Numeric, 500, 500)];
        let rec = recommend(&fields).unwrap();
        assert_eq!(rec.chart, RecommendedChart::BarChart);
    }

    #[test]
    fn two_numeric_fields_suggest_scatter_plot() {
        let fields = vec![field("height", FieldKind::Numeric, 100, 100), field("weight", FieldKind::Numeric, 100, 100)];
        let rec = recommend(&fields).unwrap();
        assert_eq!(rec.chart, RecommendedChart::ScatterPlot);
    }

    #[test]
    fn lone_numeric_field_suggests_histogram() {
        let fields = vec![field("latency_ms", FieldKind::Numeric, 950, 1000)];
        let rec = recommend(&fields).unwrap();
        assert_eq!(rec.chart, RecommendedChart::Histogram);
    }

    #[test]
    fn identifier_field_alone_suggests_table() {
        let fields = vec![field("user_id", FieldKind::Identifier, 1000, 1000)];
        let rec = recommend(&fields).unwrap();
        assert_eq!(rec.chart, RecommendedChart::Table);
    }

    #[test]
    fn near_unique_categorical_field_is_treated_as_identifier() {
        // Not explicitly typed as Identifier, but 990/1000 distinct is a
        // strong enough signal to treat it the same way.
        let fields = vec![field("session_token", FieldKind::Categorical, 990, 1000)];
        let rec = recommend(&fields).unwrap();
        assert_eq!(rec.chart, RecommendedChart::Table);
    }

    #[test]
    fn multiple_categorical_fields_suggest_grouped_bar_chart() {
        let fields = vec![
            field("region", FieldKind::Categorical, 4, 200),
            field("tier", FieldKind::Categorical, 3, 200),
        ];
        let rec = recommend(&fields).unwrap();
        assert_eq!(rec.chart, RecommendedChart::BarChart);
    }

    #[test]
    fn pie_chart_boundary_is_inclusive_at_six_categories() {
        let fields = vec![field("status", FieldKind::Categorical, 6, 100), field("count", FieldKind::Numeric, 100, 100)];
        assert_eq!(recommend(&fields).unwrap().chart, RecommendedChart::PieChart);

        let fields = vec![field("status", FieldKind::Categorical, 7, 100), field("count", FieldKind::Numeric, 100, 100)];
        assert_eq!(recommend(&fields).unwrap().chart, RecommendedChart::BarChart);
    }

    #[test]
    fn identifier_signal_yields_to_time_series_when_a_time_field_exists() {
        // A high-cardinality id column shouldn't force Table if there's
        // still a usable time+numeric pairing among the other fields.
        let fields = vec![
            field("request_id", FieldKind::Identifier, 1000, 1000),
            field("timestamp", FieldKind::Temporal, 1000, 1000),
            field("duration_ms", FieldKind::Numeric, 1000, 1000),
        ];
        let rec = recommend(&fields).unwrap();
        assert_eq!(rec.chart, RecommendedChart::LineChart);
    }
}
