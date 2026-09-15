//! Demo CLI: run a fixture scan result through the default policy gate.

use container_security_platform::{Finding, PolicyGate, ScanResult, Severity};

fn main() {
    let scan = ScanResult {
        image: "demo-app:1.4.0".into(),
        findings: vec![Finding {
            cve_id: "CVE-2024-0001".into(),
            severity: Severity::Critical,
            affected_package: "openssl".into(),
            installed_version: "1.1.1".into(),
            fixed_version: Some("1.1.1n".into()),
        }],
    };
    let outcome = PolicyGate::block_critical().evaluate(&scan);
    println!("{:?}", outcome);
}
