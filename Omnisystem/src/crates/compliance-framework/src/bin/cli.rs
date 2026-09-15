//! Demo CLI: define a control, submit partial evidence, print the report.

use compliance_framework::{ComplianceEngine, Control, Evidence};

fn main() {
    let mut engine = ComplianceEngine::new();
    engine.define_control(
        Control::new("AC-2", "Account management").requires("access_review_log").requires("mfa_enforcement_config"),
    );
    let evidence =
        vec![Evidence { control_id: "AC-2".into(), evidence_type: "access_review_log".into(), valid: true }];
    let report = engine.evaluate(&evidence);
    for r in &report.results {
        println!("{}: {:?} (missing: {:?})", r.control_id, r.status, r.missing_evidence);
    }
}
