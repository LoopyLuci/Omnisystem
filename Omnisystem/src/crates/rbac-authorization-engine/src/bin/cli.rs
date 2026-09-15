//! Demo CLI: define a small role hierarchy and evaluate a few checks.

use rbac_authorization_engine::{Permission, RbacEngine, Role};

fn main() {
    let mut engine = RbacEngine::new();
    engine.define_role(Role::new("viewer"));
    engine.define_role(Role::new("editor"));
    engine.grant("viewer", Permission::new("read", "documents")).unwrap();
    engine.grant("editor", Permission::new("write", "documents")).unwrap();
    engine.add_parent("editor", "viewer").unwrap();
    engine.assign_role("alice", "editor").unwrap();

    for (action, resource) in [("read", "documents"), ("write", "documents"), ("delete", "documents")] {
        println!(
            "alice can {action} {resource}? {}",
            engine.is_allowed("alice", action, resource)
        );
    }
}
