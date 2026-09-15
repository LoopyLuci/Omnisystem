//! container-security-platform: container image vulnerability scan
//! aggregation and a shippability policy gate.
//!
//! Models the finding/scan-result/policy-gate protocol only: no real CVE
//! database or registry is contacted anywhere in this crate, including in
//! tests, which use fixture findings. Distinct from `security-analyzer`
//! (static analysis of source/config) — this crate's unit is a container
//! *image* and its dependency-package vulnerabilities.

pub mod error;
pub mod gate;
pub mod types;

pub use error::{Error, Result};
pub use gate::PolicyGate;
pub use types::*;

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    fn finding(cve: &str, sev: Severity, pkg: &str, fixed: Option<&str>) -> Finding {
        Finding {
            cve_id: cve.to_string(),
            severity: sev,
            affected_package: pkg.to_string(),
            installed_version: "1.0.0".to_string(),
            fixed_version: fixed.map(|s| s.to_string()),
        }
    }

    #[test]
    fn severity_parses_case_insensitively() {
        assert_eq!(Severity::from_str("critical").unwrap(), Severity::Critical);
        assert_eq!(Severity::from_str("HIGH").unwrap(), Severity::High);
    }

    #[test]
    fn severity_parse_rejects_unknown() {
        assert!(matches!(Severity::from_str("ULTRA"), Err(Error::InvalidSeverity(_))));
    }

    #[test]
    fn severity_ordering_is_low_to_critical() {
        assert!(Severity::Critical > Severity::High);
        assert!(Severity::High > Severity::Medium);
        assert!(Severity::Medium > Severity::Low);
    }

    #[test]
    fn severity_counts_tally_correctly() {
        let scan = ScanResult {
            image: "app:1".into(),
            findings: vec![
                finding("CVE-1", Severity::Low, "a", Some("1.0.1")),
                finding("CVE-2", Severity::Critical, "b", None),
                finding("CVE-3", Severity::Critical, "c", Some("2.0.0")),
            ],
        };
        assert_eq!(scan.severity_counts(), (1, 0, 0, 2));
        assert_eq!(scan.unfixable().len(), 1);
    }

    #[test]
    fn gate_blocks_on_critical_by_default() {
        let scan = ScanResult {
            image: "app:1".into(),
            findings: vec![finding("CVE-9", Severity::Critical, "openssl", None)],
        };
        let outcome = PolicyGate::block_critical().evaluate(&scan);
        assert!(!outcome.passed());
    }

    #[test]
    fn gate_passes_when_below_threshold() {
        let scan = ScanResult {
            image: "app:1".into(),
            findings: vec![finding("CVE-9", Severity::Medium, "openssl", Some("1.1.1"))],
        };
        let outcome = PolicyGate::block_critical().evaluate(&scan);
        assert!(outcome.passed());
    }

    #[test]
    fn gate_allowlist_exempts_specific_cve() {
        let scan = ScanResult {
            image: "app:1".into(),
            findings: vec![finding("CVE-EXEMPT", Severity::Critical, "libx", None)],
        };
        let gate = PolicyGate::block_critical().allow("CVE-EXEMPT");
        assert!(gate.evaluate(&scan).passed());
    }

    #[test]
    fn gate_names_violating_cves() {
        let scan = ScanResult {
            image: "app:1".into(),
            findings: vec![
                finding("CVE-A", Severity::Critical, "a", None),
                finding("CVE-B", Severity::Low, "b", None),
            ],
        };
        match PolicyGate::block_critical().evaluate(&scan) {
            GateOutcome::Blocked { violating_cve_ids } => assert_eq!(violating_cve_ids, vec!["CVE-A".to_string()]),
            GateOutcome::Pass => panic!("expected blocked"),
        }
    }
}
