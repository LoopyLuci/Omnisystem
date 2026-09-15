use docker_container_lifecycle::{ContainerState, Manager};

#[test]
fn multiple_containers_track_independent_state() {
    let manager = Manager::new();
    manager.create("web").unwrap();
    manager.create("db").unwrap();

    manager.start("web").unwrap();
    assert_eq!(manager.state_of("web").unwrap(), ContainerState::Running);
    assert_eq!(manager.state_of("db").unwrap(), ContainerState::Created);
    assert_eq!(manager.running_count(), 1);

    manager.start("db").unwrap();
    manager.pause("db").unwrap();
    assert_eq!(manager.running_count(), 1);
    assert_eq!(manager.state_of("db").unwrap(), ContainerState::Paused);
}

#[test]
fn invalid_transitions_are_rejected_end_to_end() {
    let manager = Manager::new();
    manager.create("web").unwrap();

    // Can't pause before starting.
    assert!(manager.pause("web").is_err());
    // Can't unpause a never-paused container.
    assert!(manager.unpause("web").is_err());

    manager.start("web").unwrap();
    manager.stop("web").unwrap();

    // Can't pause a stopped container.
    assert!(manager.pause("web").is_err());
    // Removed containers vanish from tracking.
    manager.remove("web").unwrap();
    assert!(manager.state_of("web").is_err());
    // Double-remove errors because it's no longer tracked at all.
    assert!(manager.remove("web").is_err());
}

#[test]
fn full_restart_cycle() {
    let manager = Manager::new();
    manager.create("worker").unwrap();
    manager.start("worker").unwrap();
    manager.stop("worker").unwrap();
    manager.start("worker").unwrap();
    assert_eq!(manager.state_of("worker").unwrap(), ContainerState::Running);
    manager.stop("worker").unwrap();
    manager.remove("worker").unwrap();
}
