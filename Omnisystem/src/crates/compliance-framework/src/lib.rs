//! compliance-framework: real compliance control evaluation.
//!
//! Models compliance requirements as [`types::Control`] definitions (an id,
//! a description, and a list of required evidence types), accepts submitted
//! [`types::Evidence`], and evaluates it into a per-control
//! [`types::ControlStatus`] (Pass/Partial/Fail) via [`engine::ComplianceEngine`].

pub mod engine;
pub mod error;
pub mod types;

pub use engine::ComplianceEngine;
pub use error::{Error, Result};
pub use types::*;

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_engine() -> ComplianceEngine {
        let mut e = ComplianceEngine::new();
        e.define_control(
            Control::new("AC-2", "Account management")
                .requires("access_review_log")
                .requires("mfa_enforcement_config"),
        );
        e.define_control(Control::new("PHYS-1", "Physical security has no evidence requirement"));
        e
    }

    #[test]
    fn control_with_all_valid_evidence_passes() {
        let engine = sample_engine();
        let evidence = vec![
            Evidence { control_id: "AC-2".into(), evidence_type: "access_review_log".into(), valid: true },
            Evidence { control_id: "AC-2".into(), evidence_type: "mfa_enforcement_config".into(), valid: true },
        ];
        let report = engine.evaluate(&evidence);
        let r = report.results.iter().find(|r| r.control_id == "AC-2").unwrap();
        assert_eq!(r.status, ControlStatus::Pass);
    }

    #[test]
    fn control_with_no_evidence_fails() {
        let engine = sample_engine();
        let report = engine.evaluate(&[]);
        let r = report.results.iter().find(|r| r.control_id == "AC-2").unwrap();
        assert_eq!(r.status, ControlStatus::Fail);
        assert_eq!(r.missing_evidence.len(), 2);
    }

    #[test]
    fn control_with_partial_evidence_is_partial() {
        let engine = sample_engine();
        let evidence =
            vec![Evidence { control_id: "AC-2".into(), evidence_type: "access_review_log".into(), valid: true }];
        let report = engine.evaluate(&evidence);
        let r = report.results.iter().find(|r| r.control_id == "AC-2").unwrap();
        assert_eq!(r.status, ControlStatus::Partial);
        assert_eq!(r.missing_evidence, vec!["mfa_enforcement_config".to_string()]);
    }

    #[test]
    fn invalid_evidence_does_not_count() {
        let engine = sample_engine();
        let evidence = vec![
            Evidence { control_id: "AC-2".into(), evidence_type: "access_review_log".into(), valid: false },
            Evidence { control_id: "AC-2".into(), evidence_type: "mfa_enforcement_config".into(), valid: true },
        ];
        let report = engine.evaluate(&evidence);
        let r = report.results.iter().find(|r| r.control_id == "AC-2").unwrap();
        assert_eq!(r.status, ControlStatus::Partial);
    }

    #[test]
    fn control_with_no_required_evidence_always_passes() {
        let engine = sample_engine();
        let report = engine.evaluate(&[]);
        let r = report.results.iter().find(|r| r.control_id == "PHYS-1").unwrap();
        assert_eq!(r.status, ControlStatus::Pass);
    }

    #[test]
    fn report_summary_counts_by_status() {
        let engine = sample_engine();
        let report = engine.evaluate(&[]);
        let (pass, partial, fail) = report.summary();
        assert_eq!((pass, partial, fail), (1, 0, 1));
        assert!(!report.fully_compliant());
    }

    #[test]
    fn evaluate_control_errors_on_unknown_id() {
        let engine = sample_engine();
        assert!(matches!(engine.evaluate_control("GHOST-1", &[]), Err(Error::UnknownControl(_))));
    }

    #[test]
    fn evaluate_control_returns_single_result() {
        let engine = sample_engine();
        let result = engine.evaluate_control("PHYS-1", &[]).unwrap();
        assert_eq!(result.status, ControlStatus::Pass);
    }
}
