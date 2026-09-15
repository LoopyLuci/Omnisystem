//! Integration test: full triage lifecycle across mixed sources changes
//! posture as expected.

use omnisystem_security_integration::{SecurityFinding, SecurityIntegrationBridge, SecuritySource, Severity};

#[test]
fn full_lifecycle_changes_posture() {
    let mut bridge = SecurityIntegrationBridge::new();

    bridge.ingest(SecurityFinding {
        id: "CVE-1".into(),
        source: SecuritySource::ContainerScan,
        severity: Severity::Critical,
        summary: "critical CVE in base image".into(),
        resource: "app:2.0".into(),
    });
    bridge.ingest(SecurityFinding {
        id: "SECRET-1".into(),
        source: SecuritySource::SecretManagement,
        severity: Severity::High,
        summary: "secret overdue for rotation".into(),
        resource: "db-password".into(),
    });
    bridge.ingest(SecurityFinding {
        id: "CTRL-1".into(),
        source: SecuritySource::Compliance,
        severity: Severity::Low,
        summary: "control partially satisfied".into(),
        resource: "AC-2".into(),
    });

    let initial = bridge.posture();
    assert_eq!(initial.risk_score, Severity::Critical.weight() + Severity::High.weight() + Severity::Low.weight());

    // Fix the CVE for real.
    bridge.resolve("CVE-1").unwrap();
    // Accept the secret rotation lag as a documented risk this quarter.
    bridge.waive("SECRET-1", "rotation scheduled next maintenance window").unwrap();
    // Acknowledge the compliance gap but it's still open work.
    bridge.acknowledge("CTRL-1", "compliance-team").unwrap();

    let after = bridge.posture();
    assert_eq!(after.risk_score, Severity::Low.weight());
    assert_eq!(bridge.open_findings().len(), 1);
    assert_eq!(bridge.open_findings()[0].finding.id, "CTRL-1");
}
