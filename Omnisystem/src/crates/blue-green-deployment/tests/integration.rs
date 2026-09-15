use blue_green_deployment::*;

#[test]
fn test_full_release_cycle() {
    let manager = Manager::new();
    assert_eq!(manager.active_environment(), Environment::Blue);

    manager.deploy_to_standby("2.0.0".to_string()).expect("deploy failed");
    assert_eq!(manager.standby_state().version, "2.0.0");
    assert!(!manager.standby_state().healthy);

    manager.mark_standby_healthy().expect("mark healthy failed");
    let promote_event = manager.promote().expect("promote failed");
    assert_eq!(promote_event.to, Environment::Green);
    assert_eq!(manager.active_environment(), Environment::Green);
    assert_eq!(manager.active_state().version, "2.0.0");

    let rollback_event = manager.rollback().expect("rollback failed");
    assert!(rollback_event.is_rollback);
    assert_eq!(manager.active_environment(), Environment::Blue);
}

#[test]
fn test_promotion_requires_healthy_standby() {
    let manager = Manager::new();
    manager.deploy_to_standby("2.0.0".to_string()).unwrap();
    let err = manager.promote().expect_err("promote should require a healthy standby");
    assert!(matches!(err, Error::StandbyNotHealthy));
}

#[test]
fn test_concurrent_health_checks_are_safe() {
    let manager = std::sync::Arc::new(Manager::new());
    manager.deploy_to_standby("2.0.0".to_string()).unwrap();

    let mut handles = vec![];
    for _ in 0..8 {
        let m = manager.clone();
        handles.push(std::thread::spawn(move || {
            let _ = m.mark_standby_healthy();
        }));
    }
    for h in handles {
        h.join().expect("thread panicked");
    }

    assert!(manager.standby_state().healthy);
    let event = manager.promote().expect("promote failed after concurrent health checks");
    assert_eq!(event.version, "2.0.0");
}
