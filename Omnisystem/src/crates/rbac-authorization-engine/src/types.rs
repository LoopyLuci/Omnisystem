//! RBAC domain types: roles, permissions, and bindings.

use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

/// An action a permission grants, e.g. "read", "write", "delete", or "*"
/// for all actions.
pub type Action = String;

/// A resource (or resource pattern) a permission applies to, e.g.
/// "documents", "billing/invoices", or "*" for all resources.
pub type Resource = String;

/// A single grant: the right to perform `action` on `resource`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Permission {
    /// Action this permission grants (supports "*" wildcard).
    pub action: Action,
    /// Resource this permission applies to (supports "*" wildcard).
    pub resource: Resource,
}

impl Permission {
    /// Build a new permission.
    pub fn new(action: impl Into<Action>, resource: impl Into<Resource>) -> Self {
        Self { action: action.into(), resource: resource.into() }
    }

    /// Whether this permission covers the given `(action, resource)` pair,
    /// honoring the "*" wildcard on either field.
    pub fn matches(&self, action: &str, resource: &str) -> bool {
        (self.action == "*" || self.action == action)
            && (self.resource == "*" || self.resource == resource)
    }
}

/// A named role: a set of directly-granted permissions plus zero or more
/// parent roles it inherits permissions from (role hierarchy).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Role {
    /// Unique role name.
    pub name: String,
    /// Permissions granted directly by this role.
    pub permissions: BTreeSet<Permission>,
    /// Names of roles this role inherits permissions from. Inheritance is
    /// transitive: if `admin` inherits from `editor`, and `editor` inherits
    /// from `viewer`, `admin` has `viewer`'s permissions too.
    pub parents: BTreeSet<String>,
}

impl Role {
    /// Create a new role with no permissions and no parents.
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into(), permissions: BTreeSet::new(), parents: BTreeSet::new() }
    }
}

/// An assignment of a role to a user.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserRoleAssignment {
    /// Opaque user identifier.
    pub user_id: String,
    /// Role name assigned to the user.
    pub role: String,
}

/// The outcome of an authorization check, with the reason recorded so
/// callers/audit logs can explain a decision, not just its boolean value.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Decision {
    /// Access allowed, naming the role and permission that granted it.
    Allow { via_role: String, permission: Permission },
    /// Access denied: the user has no role (direct or inherited) granting
    /// the requested action on the requested resource.
    Deny,
}

impl Decision {
    /// True if this decision allows the action.
    pub fn is_allowed(&self) -> bool {
        matches!(self, Decision::Allow { .. })
    }
}
