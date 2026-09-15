//! secret-management-integration: secret lifecycle metadata, rotation-due
//! computation, and access-grant tracking.
//!
//! Models only secret *metadata* — name, version, age, rotation policy,
//! and who has been granted read/rotate access — never secret values. No
//! real secret store, vault, or credential is contacted anywhere in this
//! crate, including in tests, which use obviously-fake placeholder names
//! (e.g. `"prod/db/password"` as a *name*, never a value).

pub mod error;
pub mod registry;
pub mod types;

pub use error::{Error, Result};
pub use registry::SecretRegistry;
pub use types::*;

#[cfg(test)]
mod tests {
    use super::*;

    fn secret(name: &str, age: u32, max_age: u32) -> SecretMetadata {
        SecretMetadata { name: name.to_string(), version: 1, age_days: age, rotation_policy: RotationPolicy::every_days(max_age) }
    }

    #[test]
    fn fresh_secret_not_due() {
        let s = secret("svc/api-key", 5, 90);
        assert!(!s.rotation_due());
        assert_eq!(s.days_until_due(), 85);
    }

    #[test]
    fn overdue_secret_is_flagged() {
        let s = secret("svc/api-key", 100, 90);
        assert!(s.rotation_due());
        assert_eq!(s.days_until_due(), 0);
    }

    #[test]
    fn unknown_secret_lookup_errors() {
        let registry = SecretRegistry::new();
        assert!(matches!(registry.get("ghost"), Err(Error::UnknownSecret(_))));
    }

    #[test]
    fn rotate_resets_age_and_bumps_version() {
        let mut registry = SecretRegistry::new();
        registry.register(secret("db/password", 95, 90));
        let updated = registry.rotate("db/password").unwrap();
        assert_eq!(updated.version, 2);
        assert_eq!(updated.age_days, 0);
        assert!(!updated.rotation_due());
    }

    #[test]
    fn overdue_secrets_lists_only_due_ones() {
        let mut registry = SecretRegistry::new();
        registry.register(secret("a", 10, 90));
        registry.register(secret("b", 200, 90));
        registry.register(secret("c", 91, 90));
        let overdue: Vec<_> = registry.overdue_secrets().into_iter().map(|s| s.name.clone()).collect();
        assert_eq!(overdue, vec!["b".to_string(), "c".to_string()]);
    }

    #[test]
    fn grant_requires_registered_secret() {
        let mut registry = SecretRegistry::new();
        let grant = AccessGrant { secret_name: "ghost".into(), principal: "svc-a".into(), can_read: true, can_rotate: false };
        assert!(matches!(registry.grant_access(grant), Err(Error::UnknownSecret(_))));
    }

    #[test]
    fn can_read_reflects_grant() {
        let mut registry = SecretRegistry::new();
        registry.register(secret("db/password", 1, 90));
        assert!(!registry.can_read("db/password", "svc-a"));
        registry
            .grant_access(AccessGrant { secret_name: "db/password".into(), principal: "svc-a".into(), can_read: true, can_rotate: false })
            .unwrap();
        assert!(registry.can_read("db/password", "svc-a"));
    }

    #[test]
    fn re_granting_replaces_prior_grant() {
        let mut registry = SecretRegistry::new();
        registry.register(secret("db/password", 1, 90));
        registry
            .grant_access(AccessGrant { secret_name: "db/password".into(), principal: "svc-a".into(), can_read: true, can_rotate: true })
            .unwrap();
        registry
            .grant_access(AccessGrant { secret_name: "db/password".into(), principal: "svc-a".into(), can_read: false, can_rotate: true })
            .unwrap();
        assert_eq!(registry.grants_for("db/password").len(), 1);
        assert!(!registry.can_read("db/password", "svc-a"));
    }

    #[test]
    fn revoke_removes_grant() {
        let mut registry = SecretRegistry::new();
        registry.register(secret("db/password", 1, 90));
        registry
            .grant_access(AccessGrant { secret_name: "db/password".into(), principal: "svc-a".into(), can_read: true, can_rotate: false })
            .unwrap();
        registry.revoke_access("db/password", "svc-a").unwrap();
        assert!(!registry.can_read("db/password", "svc-a"));
    }

    #[test]
    fn revoke_unknown_grant_errors() {
        let mut registry = SecretRegistry::new();
        registry.register(secret("db/password", 1, 90));
        assert!(matches!(registry.revoke_access("db/password", "nobody"), Err(Error::UnknownGrant { .. })));
    }
}
