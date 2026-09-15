//! Demo CLI: evaluate a high-CPU alert rule against a sample series.

use alerting_configuration_ui::{evaluate_all, AlertRule, Comparator, Sample, Severity};
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rules = vec![AlertRule {
        name: "high-cpu".into(),
        metric: "cpu".into(),
        comparator: Comparator::GreaterThan,
        threshold: 90.0,
        for_samples: 2,
        severity: Severity::Critical,
    }];
    let mut metrics = HashMap::new();
    metrics.insert(
        "cpu".to_string(),
        vec![Sample { seq: 0, value: 50.0 }, Sample { seq: 1, value: 95.0 }, Sample { seq: 2, value: 97.0 }],
    );
    for eval in evaluate_all(&rules, &metrics)? {
        println!("{}: firing={} severity={:?}", eval.rule_name, eval.firing, eval.severity);
    }
    Ok(())
}
