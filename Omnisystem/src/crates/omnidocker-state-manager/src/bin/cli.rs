//! CLI for omnidocker-state-manager.

use omnidocker_state_manager::{Manager, ResourceKind};

fn main() {
    let manager = Manager::new();
    manager.set_desired(ResourceKind::Container, "web-1", "running");
    manager.report_observed(ResourceKind::Container, "web-1", "stopped");

    let drifts = manager.reconcile();
    println!("in sync: {}", manager.is_in_sync());
    for drift in drifts {
        println!("  {:?}", drift);
    }
}
