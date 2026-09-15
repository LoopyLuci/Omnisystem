//! The RBAC engine: role/permission storage and the `can` authorization
//! check, with transitive role-hierarchy inheritance.

use crate::error::{Error, Result};
use crate::types::{Decision, Permission, Role};
use std::collections::{BTreeMap, BTreeSet};

/// In-memory RBAC engine.
#[derive(Debug, Default)]
pub struct RbacEngine {
    roles: BTreeMap<String, Role>,
    /// user_id -> directly-assigned role names.
    assignments: BTreeMap<String, BTreeSet<String>>,
}

impl RbacEngine {
    /// Create an empty engine.
    pub fn new() -> Self {
        Self::default()
    }

    /// Define (or replace) a role.
    pub fn define_role(&mut self, role: Role) {
        self.roles.insert(role.name.clone(), role);
    }

    /// Add `parent` as a parent of `role`, validating both exist and that
    /// the edge does not introduce a cycle in the inheritance graph.
    pub fn add_parent(&mut self, role: &str, parent: &str) -> Result<()> {
        if !self.roles.contains_key(role) {
            return Err(Error::UnknownRole(role.to_string()));
        }
        if !self.roles.contains_key(parent) {
            return Err(Error::UnknownRole(parent.to_string()));
        }
        // Cycle check: would `role` become reachable from `parent`'s own
        // ancestor chain (i.e. is `role` already an ancestor of `parent`)?
        if role == parent || self.ancestors(parent).contains(role) {
            return Err(Error::HierarchyCycle(role.to_string()));
        }
        self.roles.get_mut(role).unwrap().parents.insert(parent.to_string());
        Ok(())
    }

    /// Grant a permission directly to a role.
    pub fn grant(&mut self, role: &str, permission: Permission) -> Result<()> {
        self.roles
            .get_mut(role)
            .ok_or_else(|| Error::UnknownRole(role.to_string()))?
            .permissions
            .insert(permission);
        Ok(())
    }

    /// Assign a role to a user.
    pub fn assign_role(&mut self, user_id: &str, role: &str) -> Result<()> {
        if !self.roles.contains_key(role) {
            return Err(Error::UnknownRole(role.to_string()));
        }
        self.assignments.entry(user_id.to_string()).or_default().insert(role.to_string());
        Ok(())
    }

    /// Revoke a role from a user. No-op (not an error) if not assigned.
    pub fn revoke_role(&mut self, user_id: &str, role: &str) {
        if let Some(set) = self.assignments.get_mut(user_id) {
            set.remove(role);
        }
    }

    /// All roles reachable from `role` via the parent chain, including
    /// `role` itself is NOT included (ancestors only).
    fn ancestors(&self, role: &str) -> BTreeSet<String> {
        let mut seen = BTreeSet::new();
        let mut stack = vec![role.to_string()];
        while let Some(r) = stack.pop() {
            if let Some(def) = self.roles.get(&r) {
                for p in &def.parents {
                    if seen.insert(p.clone()) {
                        stack.push(p.clone());
                    }
                }
            }
        }
        seen
    }

    /// All roles effectively held by a user: directly assigned plus every
    /// ancestor in the hierarchy.
    pub fn effective_roles(&self, user_id: &str) -> BTreeSet<String> {
        let direct = self.assignments.get(user_id).cloned().unwrap_or_default();
        let mut all = direct.clone();
        for r in &direct {
            all.extend(self.ancestors(r));
        }
        all
    }

    /// The core authorization check: can `user_id` perform `action` on
    /// `resource`? Walks the user's direct roles and their full ancestor
    /// chain, returning the first matching grant (deterministic role-name
    /// order) or `Decision::Deny` if none match.
    pub fn can(&self, user_id: &str, action: &str, resource: &str) -> Decision {
        for role_name in self.effective_roles(user_id) {
            if let Some(role) = self.roles.get(&role_name) {
                if let Some(perm) = role.permissions.iter().find(|p| p.matches(action, resource)) {
                    return Decision::Allow { via_role: role_name, permission: perm.clone() };
                }
            }
        }
        Decision::Deny
    }

    /// Convenience boolean form of `can`.
    pub fn is_allowed(&self, user_id: &str, action: &str, resource: &str) -> bool {
        self.can(user_id, action, resource).is_allowed()
    }
}
