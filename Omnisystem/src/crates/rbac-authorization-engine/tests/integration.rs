//! Integration test: a realistic multi-role, multi-user scenario.

use rbac_authorization_engine::{Permission, RbacEngine, Role};

#[test]
fn realistic_org_hierarchy_authorization() {
    let mut engine = RbacEngine::new();
    engine.define_role(Role::new("intern"));
    engine.define_role(Role::new("engineer"));
    engine.define_role(Role::new("lead"));
    engine.define_role(Role::new("billing_admin"));

    engine.grant("intern", Permission::new("read", "code")).unwrap();
    engine.grant("engineer", Permission::new("write", "code")).unwrap();
    engine.grant("lead", Permission::new("merge", "code")).unwrap();
    engine.grant("billing_admin", Permission::new("*", "billing")).unwrap();

    engine.add_parent("engineer", "intern").unwrap();
    engine.add_parent("lead", "engineer").unwrap();

    engine.assign_role("carol", "lead").unwrap();
    engine.assign_role("dave", "intern").unwrap();
    engine.assign_role("erin", "billing_admin").unwrap();

    // Lead inherits everything below it.
    assert!(engine.is_allowed("carol", "read", "code"));
    assert!(engine.is_allowed("carol", "write", "code"));
    assert!(engine.is_allowed("carol", "merge", "code"));
    // But leads have no billing access.
    assert!(!engine.is_allowed("carol", "read", "billing"));

    // Intern only has read.
    assert!(engine.is_allowed("dave", "read", "code"));
    assert!(!engine.is_allowed("dave", "write", "code"));

    // billing_admin wildcard action covers anything on billing, nothing else.
    assert!(engine.is_allowed("erin", "delete", "billing"));
    assert!(!engine.is_allowed("erin", "read", "code"));

    // Promote dave to lead: he immediately gains the whole chain.
    engine.assign_role("dave", "lead").unwrap();
    assert!(engine.is_allowed("dave", "merge", "code"));

    // Demote by revoking: falls back to remaining assigned roles only.
    engine.revoke_role("dave", "lead");
    assert!(engine.is_allowed("dave", "read", "code"));
    assert!(!engine.is_allowed("dave", "merge", "code"));
}
