use docker_registry_integration::{AuthOutcome, Direction, Manager, TransferState};

#[test]
fn end_to_end_authenticated_push_and_pull() {
    let manager = Manager::new();
    manager.authenticate(AuthOutcome::Granted { token: "fake".to_string(), ttl_ticks: 50 }, 0);

    manager.start_transfer("push-1", "myorg/app:1.0", Direction::Push, 0).unwrap();
    manager.start_transfer("pull-1", "myorg/other:2.0", Direction::Pull, 0).unwrap();

    manager.advance_transfer("push-1", 1).unwrap();
    manager.advance_transfer("push-1", 2).unwrap();
    assert_eq!(manager.transfer_state("push-1").unwrap(), TransferState::Completed);

    manager.advance_transfer("pull-1", 3).unwrap();
    assert_eq!(manager.transfer_state("pull-1").unwrap(), TransferState::InProgress);
}

#[test]
fn rejected_auth_never_allows_any_transfer() {
    let manager = Manager::new();
    manager.authenticate(AuthOutcome::Denied, 0);
    assert!(manager.start_transfer("t1", "myorg/app:1.0", Direction::Push, 0).is_err());
}

#[test]
fn token_expiry_mid_transfer_blocks_completion() {
    let manager = Manager::new();
    manager.authenticate(AuthOutcome::Granted { token: "fake".to_string(), ttl_ticks: 2 }, 0);
    manager.start_transfer("t1", "myorg/app:1.0", Direction::Push, 0).unwrap();
    manager.advance_transfer("t1", 1).unwrap(); // still valid (expires at tick 2)
    assert!(manager.advance_transfer("t1", 5).is_err()); // token now expired
}

#[test]
fn reauthenticating_after_expiry_allows_progress_to_resume() {
    let manager = Manager::new();
    manager.authenticate(AuthOutcome::Granted { token: "fake".to_string(), ttl_ticks: 2 }, 0);
    manager.start_transfer("t1", "img", Direction::Pull, 0).unwrap();
    assert!(manager.advance_transfer("t1", 10).is_err());

    manager.authenticate(AuthOutcome::Granted { token: "fake2".to_string(), ttl_ticks: 50 }, 10);
    assert!(manager.advance_transfer("t1", 10).is_ok());
}
