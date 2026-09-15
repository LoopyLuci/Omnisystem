//! rbac-authorization-engine: real role-based access control.
//!
//! Models roles as named sets of `(action, resource)` permission grants
//! that can inherit from parent roles (role hierarchy), users as holders
//! of zero or more directly-assigned roles, and exposes a single
//! authorization primitive, [`engine::RbacEngine::can`], that walks a
//! user's effective (direct + inherited) roles to decide whether an
//! action on a resource is allowed.

pub mod engine;
pub mod error;
pub mod types;

pub use engine::RbacEngine;
pub use error::{Error, Result};
pub use types::*;

#[cfg(test)]
mod tests {
    use super::*;

    fn engine_with_hierarchy() -> RbacEngine {
        let mut e = RbacEngine::new();
        e.define_role(Role::new("viewer"));
        e.define_role(Role::new("editor"));
        e.define_role(Role::new("admin"));
        e.grant("viewer", Permission::new("read", "documents")).unwrap();
        e.grant("editor", Permission::new("write", "documents")).unwrap();
        e.grant("admin", Permission::new("delete", "documents")).unwrap();
        e.add_parent("editor", "viewer").unwrap();
        e.add_parent("admin", "editor").unwrap();
        e
    }

    #[test]
    fn unknown_role_grant_errors() {
        let mut e = RbacEngine::new();
        assert!(matches!(e.grant("ghost", Permission::new("read", "x")), Err(Error::UnknownRole(_))));
    }

    #[test]
    fn unassigned_user_is_denied() {
        let e = engine_with_hierarchy();
        assert!(!e.is_allowed("nobody", "read", "documents"));
    }

    #[test]
    fn direct_permission_allows() {
        let mut e = engine_with_hierarchy();
        e.assign_role("alice", "viewer").unwrap();
        assert!(e.is_allowed("alice", "read", "documents"));
        assert!(!e.is_allowed("alice", "write", "documents"));
    }

    #[test]
    fn inherited_permission_allows_via_hierarchy() {
        let mut e = engine_with_hierarchy();
        e.assign_role("bob", "admin").unwrap();
        // admin inherits editor inherits viewer: all three grants apply.
        assert!(e.is_allowed("bob", "read", "documents"));
        assert!(e.is_allowed("bob", "write", "documents"));
        assert!(e.is_allowed("bob", "delete", "documents"));
    }

    #[test]
    fn decision_names_granting_role() {
        let mut e = engine_with_hierarchy();
        e.assign_role("bob", "admin").unwrap();
        match e.can("bob", "read", "documents") {
            Decision::Allow { via_role, .. } => assert_eq!(via_role, "viewer"),
            Decision::Deny => panic!("expected allow"),
        }
    }

    #[test]
    fn wildcard_action_matches_anything() {
        let mut e = RbacEngine::new();
        e.define_role(Role::new("superuser"));
        e.grant("superuser", Permission::new("*", "*")).unwrap();
        e.assign_role("root", "superuser").unwrap();
        assert!(e.is_allowed("root", "delete", "billing"));
    }

    #[test]
    fn revoke_removes_access() {
        let mut e = engine_with_hierarchy();
        e.assign_role("alice", "viewer").unwrap();
        assert!(e.is_allowed("alice", "read", "documents"));
        e.revoke_role("alice", "viewer");
        assert!(!e.is_allowed("alice", "read", "documents"));
    }

    #[test]
    fn cycle_in_hierarchy_is_rejected() {
        let mut e = RbacEngine::new();
        e.define_role(Role::new("a"));
        e.define_role(Role::new("b"));
        e.add_parent("b", "a").unwrap();
        assert!(matches!(e.add_parent("a", "b"), Err(Error::HierarchyCycle(_))));
    }

    #[test]
    fn effective_roles_includes_ancestors() {
        let mut e = engine_with_hierarchy();
        e.assign_role("bob", "admin").unwrap();
        let roles = e.effective_roles("bob");
        assert!(roles.contains("admin"));
        assert!(roles.contains("editor"));
        assert!(roles.contains("viewer"));
    }
}
