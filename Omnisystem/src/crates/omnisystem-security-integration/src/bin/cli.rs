//! Demo CLI: ingest findings from a couple of sources and print posture.

use omnisystem_security_integration::{SecurityFinding, SecurityIntegrationBridge, SecuritySource, Severity};

fn main() {
    let mut bridge = SecurityIntegrationBridge::new();
    bridge.ingest(SecurityFinding {
        id: "CVE-2024-0001".into(),
        source: SecuritySource::ContainerScan,
        severity: Severity::Critical,
        summary: "openssl vulnerable to X".into(),
        resource: "demo-app:1.4.0".into(),
    });
    bridge.ingest(SecurityFinding {
        id: "RBAC-001".into(),
        source: SecuritySource::Rbac,
        severity: Severity::Medium,
        summary: "wildcard grant on billing resource".into(),
        resource: "role:billing_admin".into(),
    });
    println!("{:?}", bridge.posture());
}
