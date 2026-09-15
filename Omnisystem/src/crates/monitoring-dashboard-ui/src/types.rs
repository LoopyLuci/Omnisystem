//! Monitoring dashboard types: widgets with warn/critical thresholds whose
//! statuses roll up into one overall dashboard status.

use serde::{Deserialize, Serialize};

/// Health status of a widget or the dashboard as a whole, ordered from
/// least to most severe.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Status {
    /// Metric is within normal bounds.
    Ok,
    /// Metric has crossed the warning threshold.
    Warning,
    /// Metric has crossed the critical threshold.
    Critical,
    /// No data has been reported for this widget yet.
    Unknown,
}

/// A single monitoring widget: watches one metric against warn/critical
/// thresholds (both are lower bounds — the metric is "bad" when high).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Widget {
    /// Widget/metric name.
    pub name: String,
    /// Latest observed value, if any has been reported.
    pub latest_value: Option<f64>,
    /// Value at/above which the widget is `Warning`.
    pub warn_threshold: f64,
    /// Value at/above which the widget is `Critical`.
    pub critical_threshold: f64,
}
