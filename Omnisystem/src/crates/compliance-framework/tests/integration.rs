//! Integration test: multi-control report with mixed pass/partial/fail.

use compliance_framework::{ComplianceEngine, Control, ControlStatus, Evidence};

#[test]
fn multi_control_report_reflects_mixed_compliance() {
    let mut engine = ComplianceEngine::new();
    engine.define_control(Control::new("A", "control a").requires("log_a"));
    engine.define_control(Control::new("B", "control b").requires("log_b1").requires("log_b2"));
    engine.define_control(Control::new("C", "control c"));

    let evidence = vec![
        Evidence { control_id: "A".into(), evidence_type: "log_a".into(), valid: true },
        Evidence { control_id: "B".into(), evidence_type: "log_b1".into(), valid: true },
        // log_b2 missing entirely -> B is Partial.
    ];

    let report = engine.evaluate(&evidence);
    assert_eq!(report.results.len(), 3);

    let a = report.results.iter().find(|r| r.control_id == "A").unwrap();
    let b = report.results.iter().find(|r| r.control_id == "B").unwrap();
    let c = report.results.iter().find(|r| r.control_id == "C").unwrap();

    assert_eq!(a.status, ControlStatus::Pass);
    assert_eq!(b.status, ControlStatus::Partial);
    assert_eq!(c.status, ControlStatus::Pass);

    let (pass, partial, fail) = report.summary();
    assert_eq!((pass, partial, fail), (2, 1, 0));
    assert!(!report.fully_compliant());
}
