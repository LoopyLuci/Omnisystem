//! Demo CLI: register a couple of fake secrets and list overdue ones.

use secret_management_integration::{RotationPolicy, SecretMetadata, SecretRegistry};

fn main() {
    let mut registry = SecretRegistry::new();
    registry.register(SecretMetadata {
        name: "demo/db-password".into(),
        version: 3,
        age_days: 95,
        rotation_policy: RotationPolicy::every_days(90),
    });
    registry.register(SecretMetadata {
        name: "demo/api-key".into(),
        version: 1,
        age_days: 10,
        rotation_policy: RotationPolicy::every_days(90),
    });
    for s in registry.overdue_secrets() {
        println!("overdue: {} (age {} days)", s.name, s.age_days);
    }
}
