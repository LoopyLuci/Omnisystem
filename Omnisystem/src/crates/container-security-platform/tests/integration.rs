//! Integration test: aggregate multiple images' scans and gate each.

use container_security_platform::{Finding, GateOutcome, PolicyGate, ScanResult, Severity};

fn finding(cve: &str, sev: Severity) -> Finding {
    Finding {
        cve_id: cve.to_string(),
        severity: sev,
        affected_package: "libexample".to_string(),
        installed_version: "3.2.1".to_string(),
        fixed_version: None,
    }
}

#[test]
fn fleet_of_images_gated_independently() {
    let scans = vec![
        ScanResult { image: "svc-a:1".into(), findings: vec![finding("CVE-A1", Severity::Low)] },
        ScanResult {
            image: "svc-b:1".into(),
            findings: vec![finding("CVE-B1", Severity::Critical), finding("CVE-B2", Severity::High)],
        },
        ScanResult { image: "svc-c:1".into(), findings: vec![] },
    ];

    let gate = PolicyGate::block_critical();
    let mut blocked_images = Vec::new();
    for scan in &scans {
        if let GateOutcome::Blocked { .. } = gate.evaluate(scan) {
            blocked_images.push(scan.image.clone());
        }
    }
    assert_eq!(blocked_images, vec!["svc-b:1".to_string()]);

    // A stricter gate (block at HIGH) blocks svc-b for a different reason
    // and would also block a hypothetical HIGH-only image.
    let strict_gate = PolicyGate::with_threshold(Severity::High);
    let outcome = strict_gate.evaluate(&scans[1]);
    match outcome {
        GateOutcome::Blocked { violating_cve_ids } => {
            assert!(violating_cve_ids.contains(&"CVE-B1".to_string()));
            assert!(violating_cve_ids.contains(&"CVE-B2".to_string()));
        }
        GateOutcome::Pass => panic!("expected blocked"),
    }
}
