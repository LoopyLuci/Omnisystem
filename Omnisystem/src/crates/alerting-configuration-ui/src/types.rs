//! Alerting configuration types.

use serde::{Deserialize, Serialize};

/// Comparator used by an [`AlertRule`] to test a metric value.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Comparator {
    /// Fires when the sample is greater than the threshold.
    GreaterThan,
    /// Fires when the sample is less than the threshold.
    LessThan,
    /// Fires when the sample equals the threshold (within float epsilon).
    Equal,
}

/// Severity of an alert rule, ordered least to most urgent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Severity {
    /// Informational only.
    Info,
    /// Needs attention soon.
    Warning,
    /// Needs immediate attention.
    Critical,
}

/// A configured alert rule: fire when `metric` compares to `threshold` for
/// at least `for_samples` consecutive samples.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertRule {
    /// Rule identifier, e.g. `high-cpu`.
    pub name: String,
    /// Metric name this rule watches, e.g. `cpu.utilization`.
    pub metric: String,
    /// Comparator applied to each sample.
    pub comparator: Comparator,
    /// Threshold value compared against each sample.
    pub threshold: f64,
    /// Number of consecutive breaching samples required to fire.
    pub for_samples: usize,
    /// Severity assigned when the rule fires.
    pub severity: Severity,
}

/// One timestamped sample for a metric.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Sample {
    /// Sequence index (monotonic, not wall-clock — keeps tests deterministic).
    pub seq: u64,
    /// The observed value.
    pub value: f64,
}

/// The outcome of evaluating one rule against a series.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AlertEvaluation {
    /// The rule that was evaluated.
    pub rule_name: String,
    /// Whether the rule's condition is currently satisfied (fired).
    pub firing: bool,
    /// Sequence number of the sample that caused the rule to fire, if any.
    pub since_seq: Option<u64>,
    /// Severity of the rule, echoed for convenience.
    pub severity: Severity,
}
