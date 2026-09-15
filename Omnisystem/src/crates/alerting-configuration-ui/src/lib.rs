//! Alerting configuration: evaluates configured [`AlertRule`]s against
//! metric sample series, requiring `for_samples` consecutive breaching
//! samples (trailing edge) before a rule is considered firing.

#![warn(missing_docs)]

use std::collections::{BTreeMap, HashMap};

pub mod error;
pub mod types;

pub use error::{Error, Result};
pub use types::*;

/// Validate a set of rules: no duplicate names, and `for_samples >= 1`.
pub fn validate_rules(rules: &[AlertRule]) -> Result<()> {
    let mut seen = std::collections::HashSet::new();
    for r in rules {
        if r.for_samples == 0 {
            return Err(Error::InvalidForSamples { rule: r.name.clone() });
        }
        if !seen.insert(r.name.clone()) {
            return Err(Error::DuplicateRuleName(r.name.clone()));
        }
    }
    Ok(())
}

fn breaches(rule: &AlertRule, value: f64) -> bool {
    match rule.comparator {
        Comparator::GreaterThan => value > rule.threshold,
        Comparator::LessThan => value < rule.threshold,
        Comparator::Equal => (value - rule.threshold).abs() < 1e-9,
    }
}

/// Evaluate a single rule against its metric's samples (assumed ordered by
/// `seq` ascending). A rule fires when the *trailing* `for_samples` samples
/// all breach the threshold.
pub fn evaluate_rule(rule: &AlertRule, samples: &[Sample]) -> Result<AlertEvaluation> {
    if samples.is_empty() {
        return Err(Error::NoSamplesForMetric { rule: rule.name.clone(), metric: rule.metric.clone() });
    }
    if samples.len() < rule.for_samples {
        return Ok(AlertEvaluation {
            rule_name: rule.name.clone(),
            firing: false,
            since_seq: None,
            severity: rule.severity,
        });
    }
    let tail = &samples[samples.len() - rule.for_samples..];
    let firing = tail.iter().all(|s| breaches(rule, s.value));
    let since_seq = if firing { Some(tail[0].seq) } else { None };
    Ok(AlertEvaluation { rule_name: rule.name.clone(), firing, since_seq, severity: rule.severity })
}

/// Evaluate every rule against a metric -> samples map, validating the rule
/// set first. Results are returned sorted with the most urgent firing
/// alerts first (`Critical` > `Warning` > `Info`), non-firing rules last.
pub fn evaluate_all(
    rules: &[AlertRule],
    metrics: &HashMap<String, Vec<Sample>>,
) -> Result<Vec<AlertEvaluation>> {
    validate_rules(rules)?;
    let mut out = Vec::with_capacity(rules.len());
    for rule in rules {
        let samples = metrics.get(&rule.metric).map(Vec::as_slice).unwrap_or(&[]);
        if samples.is_empty() {
            return Err(Error::NoSamplesForMetric { rule: rule.name.clone(), metric: rule.metric.clone() });
        }
        out.push(evaluate_rule(rule, samples)?);
    }
    out.sort_by(|a, b| match (a.firing, b.firing) {
        (true, true) => b.severity.cmp(&a.severity),
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        (false, false) => a.rule_name.cmp(&b.rule_name),
    });
    Ok(out)
}

/// Group evaluations by severity, counting only those that are firing.
pub fn firing_counts_by_severity(evals: &[AlertEvaluation]) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    for e in evals.iter().filter(|e| e.firing) {
        let key = format!("{:?}", e.severity);
        *counts.entry(key).or_insert(0usize) += 1;
    }
    counts
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rule(name: &str, for_samples: usize, sev: Severity) -> AlertRule {
        AlertRule {
            name: name.into(),
            metric: "cpu".into(),
            comparator: Comparator::GreaterThan,
            threshold: 90.0,
            for_samples,
            severity: sev,
        }
    }

    fn series(vals: &[f64]) -> Vec<Sample> {
        vals.iter().enumerate().map(|(i, &v)| Sample { seq: i as u64, value: v }).collect()
    }

    #[test]
    fn validate_rejects_zero_for_samples() {
        let rules = vec![rule("r1", 0, Severity::Warning)];
        assert_eq!(validate_rules(&rules), Err(Error::InvalidForSamples { rule: "r1".into() }));
    }

    #[test]
    fn validate_rejects_duplicate_names() {
        let rules = vec![rule("r1", 1, Severity::Warning), rule("r1", 2, Severity::Critical)];
        assert!(matches!(validate_rules(&rules), Err(Error::DuplicateRuleName(_))));
    }

    #[test]
    fn evaluate_rule_fires_when_trailing_samples_all_breach() {
        let r = rule("high-cpu", 2, Severity::Critical);
        let s = series(&[50.0, 95.0, 96.0]);
        let eval = evaluate_rule(&r, &s).unwrap();
        assert!(eval.firing);
        assert_eq!(eval.since_seq, Some(1));
    }

    #[test]
    fn evaluate_rule_does_not_fire_if_one_trailing_sample_recovers() {
        let r = rule("high-cpu", 2, Severity::Critical);
        let s = series(&[95.0, 96.0, 50.0]);
        let eval = evaluate_rule(&r, &s).unwrap();
        assert!(!eval.firing);
    }

    #[test]
    fn evaluate_rule_not_firing_when_too_few_samples() {
        let r = rule("high-cpu", 3, Severity::Critical);
        let s = series(&[95.0, 96.0]);
        let eval = evaluate_rule(&r, &s).unwrap();
        assert!(!eval.firing);
        assert_eq!(eval.since_seq, None);
    }

    #[test]
    fn evaluate_rule_errors_on_empty_series() {
        let r = rule("high-cpu", 1, Severity::Critical);
        assert!(matches!(evaluate_rule(&r, &[]), Err(Error::NoSamplesForMetric { .. })));
    }

    #[test]
    fn evaluate_all_sorts_firing_by_severity_then_non_firing_last() {
        let rules = vec![
            rule("warn-cpu", 1, Severity::Warning),
            rule("crit-cpu", 1, Severity::Critical),
            rule("info-cpu", 1, Severity::Info),
        ];
        let mut metrics = HashMap::new();
        metrics.insert("cpu".to_string(), series(&[95.0]));
        let evals = evaluate_all(&rules, &metrics).unwrap();
        assert_eq!(evals[0].rule_name, "crit-cpu");
        assert_eq!(evals[1].rule_name, "warn-cpu");
        assert_eq!(evals[2].rule_name, "info-cpu");
    }

    #[test]
    fn evaluate_all_errors_when_metric_missing() {
        let rules = vec![rule("high-cpu", 1, Severity::Critical)];
        let metrics = HashMap::new();
        assert!(matches!(evaluate_all(&rules, &metrics), Err(Error::NoSamplesForMetric { .. })));
    }

    #[test]
    fn firing_counts_by_severity_counts_only_firing() {
        let evals = vec![
            AlertEvaluation { rule_name: "a".into(), firing: true, since_seq: Some(0), severity: Severity::Critical },
            AlertEvaluation { rule_name: "b".into(), firing: false, since_seq: None, severity: Severity::Critical },
            AlertEvaluation { rule_name: "c".into(), firing: true, since_seq: Some(0), severity: Severity::Warning },
        ];
        let counts = firing_counts_by_severity(&evals);
        assert_eq!(counts.get("Critical"), Some(&1));
        assert_eq!(counts.get("Warning"), Some(&1));
    }
}
