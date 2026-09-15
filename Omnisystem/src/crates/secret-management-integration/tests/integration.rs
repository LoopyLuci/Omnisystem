//! Integration test: registry with mixed rotation states and grants.

use secret_management_integration::{AccessGrant, RotationPolicy, SecretMetadata, SecretRegistry};

#[test]
fn fleet_rotation_and_access_scenario() {
    let mut registry = SecretRegistry::new();
    registry.register(SecretMetadata {
        name: "svc-a/token".into(),
        version: 1,
        age_days: 120,
        rotation_policy: RotationPolicy::every_days(90),
    });
    registry.register(SecretMetadata {
        name: "svc-b/token".into(),
        version: 4,
        age_days: 5,
        rotation_policy: RotationPolicy::every_days(30),
    });

    // svc-a/token is overdue; svc-b/token is not.
    let overdue: Vec<_> = registry.overdue_secrets().into_iter().map(|s| s.name.clone()).collect();
    assert_eq!(overdue, vec!["svc-a/token".to_string()]);

    // Only an on-call rotator has can_rotate.
    registry
        .grant_access(AccessGrant {
            secret_name: "svc-a/token".into(),
            principal: "oncall-bot".into(),
            can_read: true,
            can_rotate: true,
        })
        .unwrap();
    registry
        .grant_access(AccessGrant {
            secret_name: "svc-a/token".into(),
            principal: "readonly-dashboard".into(),
            can_read: true,
            can_rotate: false,
        })
        .unwrap();

    assert_eq!(registry.grants_for("svc-a/token").len(), 2);

    // The on-call bot rotates it.
    let rotated = registry.rotate("svc-a/token").unwrap();
    assert_eq!(rotated.version, 2);
    assert!(!rotated.rotation_due());

    // Now nothing is overdue.
    assert!(registry.overdue_secrets().is_empty());

    // Revoking the dashboard's access leaves only the bot.
    registry.revoke_access("svc-a/token", "readonly-dashboard").unwrap();
    assert_eq!(registry.grants_for("svc-a/token").len(), 1);
    assert!(!registry.can_read("svc-a/token", "readonly-dashboard"));
    assert!(registry.can_read("svc-a/token", "oncall-bot"));
}
