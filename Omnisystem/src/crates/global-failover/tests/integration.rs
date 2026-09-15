use global_failover::Manager;

#[test]
fn end_to_end_failover_and_failback() {
    let manager = Manager::new("us-east", 5, true);
    manager.register_region("us-east", 0);
    manager.register_region("us-west", 0);
    manager.register_region("eu-central", 0);

    let initial = manager.evaluate(0);
    assert_eq!(initial.active_primary.as_deref(), Some("us-east"));
    assert!(!initial.failed_over);

    // us-east goes dark; the others keep heartbeating.
    manager.heartbeat("us-west", 10).unwrap();
    manager.heartbeat("eu-central", 10).unwrap();
    let failed_over = manager.evaluate(12);
    assert_eq!(failed_over.active_primary.as_deref(), Some("eu-central"));
    assert!(failed_over.failed_over);
    assert_eq!(failed_over.failover_count, 1);
    assert_eq!(manager.current_active_primary().as_deref(), Some("eu-central"));

    // us-east recovers; auto-failback returns traffic to it.
    manager.heartbeat("us-east", 20).unwrap();
    manager.heartbeat("us-west", 20).unwrap();
    manager.heartbeat("eu-central", 20).unwrap();
    let failed_back = manager.evaluate(20);
    assert_eq!(failed_back.active_primary.as_deref(), Some("us-east"));
    assert!(!failed_back.failed_over);
}

#[test]
fn total_outage_leaves_no_active_primary() {
    let manager = Manager::new("us-east", 5, false);
    manager.register_region("us-east", 0);
    manager.register_region("us-west", 0);
    manager.evaluate(0);

    let status = manager.evaluate(1000);
    assert_eq!(status.active_primary, None);
    assert_eq!(status.alive_regions, 0);
}

#[test]
fn heartbeat_on_unregistered_region_is_an_error() {
    let manager = Manager::new("us-east", 5, false);
    assert!(manager.heartbeat("atlantis", 0).is_err());
}
