use omnidocker_state_manager::{Drift, Manager, ResourceKind};

#[test]
fn full_fleet_reconciliation() {
    let manager = Manager::new();

    manager.set_desired(ResourceKind::Container, "web-1", "running");
    manager.set_desired(ResourceKind::Container, "worker-1", "running");
    manager.set_desired(ResourceKind::Volume, "app-data", "mounted");
    manager.set_desired(ResourceKind::Network, "app-net", "active");

    manager.report_observed(ResourceKind::Container, "web-1", "running");
    manager.report_observed(ResourceKind::Container, "worker-1", "stopped"); // drift: mismatch
    manager.report_observed(ResourceKind::Network, "app-net", "active");
    // app-data never observed -> drift: missing
    manager.report_observed(ResourceKind::Image, "myorg/app:1.0", "pulled"); // drift: undeclared

    let drifts = manager.reconcile();
    assert_eq!(drifts.len(), 3);
    assert!(!manager.is_in_sync());

    assert!(drifts.iter().any(|d| matches!(d,
        Drift::StateMismatch { resource, .. } if resource.id == "worker-1")));
    assert!(drifts.iter().any(|d| matches!(d,
        Drift::MissingObservation { resource, .. } if resource.id == "app-data")));
    assert!(drifts.iter().any(|d| matches!(d,
        Drift::UndeclaredResource { resource, .. } if resource.id == "myorg/app:1.0")));
}

#[test]
fn resolving_drift_by_correcting_observation_reaches_sync() {
    let manager = Manager::new();
    manager.set_desired(ResourceKind::Container, "web-1", "running");
    manager.report_observed(ResourceKind::Container, "web-1", "stopped");
    assert!(!manager.is_in_sync());

    manager.report_observed(ResourceKind::Container, "web-1", "running");
    assert!(manager.is_in_sync());
    assert!(manager.reconcile().is_empty());
}
