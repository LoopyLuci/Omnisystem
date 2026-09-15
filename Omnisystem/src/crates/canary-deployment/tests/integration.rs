use canary_deployment::*;

#[test]
fn test_full_rollout_completes() {
    let m = Manager::new();
    assert_eq!(m.start().unwrap(), 5);
    assert_eq!(m.record_error_rate(0.0).unwrap(), CanaryDecision::Advanced(25));
    assert_eq!(m.record_error_rate(0.0).unwrap(), CanaryDecision::Advanced(50));
    assert_eq!(m.record_error_rate(0.0).unwrap(), CanaryDecision::Advanced(100));
    assert_eq!(m.record_error_rate(0.0).unwrap(), CanaryDecision::Completed);
    assert_eq!(m.history().len(), 4);
}

#[test]
fn test_custom_stage_list() {
    let m = Manager::with_stages(vec![10, 100], 0.1);
    assert_eq!(m.start().unwrap(), 10);
    assert_eq!(m.record_error_rate(0.0).unwrap(), CanaryDecision::Advanced(100));
    assert_eq!(m.record_error_rate(0.0).unwrap(), CanaryDecision::Completed);
}

#[test]
fn test_rollback_resets_to_zero_and_allows_restart() {
    let m = Manager::new();
    m.start().unwrap();
    let decision = m.record_error_rate(0.5).unwrap();
    assert_eq!(decision, CanaryDecision::RolledBack);
    assert_eq!(m.current_percent(), 0);

    // A fresh rollout can begin after the rollback.
    assert_eq!(m.start().unwrap(), 5);
}
