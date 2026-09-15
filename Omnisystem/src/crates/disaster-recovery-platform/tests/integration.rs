use disaster_recovery_platform::{Manager, RecoveryPlan};

fn sample_plan() -> RecoveryPlan {
    RecoveryPlan {
        name: "region-recovery".to_string(),
        rpo_ticks: 15,
        rto_ticks: 60,
        steps: vec![
            "spin up standby region".to_string(),
            "restore latest snapshot".to_string(),
            "repoint traffic".to_string(),
            "validate health checks".to_string(),
        ],
    }
}

#[test]
fn end_to_end_drill_meets_targets() {
    let manager = Manager::new();
    manager.register_plan(sample_plan());
    manager.record_snapshot("primary-region", 990);
    manager.record_snapshot("primary-region", 995); // most recent before disaster wins.

    manager.start_drill("region-recovery", 1000).unwrap();
    let mut executed = vec![];
    loop {
        match manager.advance_step("region-recovery") {
            Ok(step) => executed.push(step),
            Err(_) => break,
        }
    }
    assert_eq!(executed.len(), 4);

    let result = manager.complete_drill("region-recovery", "primary-region", 1040).unwrap();
    assert_eq!(result.actual_rto_ticks, 40);
    assert!(result.rto_met);
    assert_eq!(result.actual_rpo_ticks, Some(5)); // 1000 - 995
    assert!(result.rpo_met);
    assert_eq!(result.steps_completed, 4);
    assert_eq!(result.steps_total, 4);
}

#[test]
fn drill_missing_recent_snapshot_fails_rpo_but_still_completes() {
    let manager = Manager::new();
    manager.register_plan(sample_plan());
    manager.record_snapshot("primary-region", 900); // 100 ticks stale, RPO target is 15.

    manager.start_drill("region-recovery", 1000).unwrap();
    for _ in 0..4 {
        manager.advance_step("region-recovery").unwrap();
    }
    let result = manager.complete_drill("region-recovery", "primary-region", 1010).unwrap();

    assert!(result.rto_met);
    assert_eq!(result.actual_rpo_ticks, Some(100));
    assert!(!result.rpo_met);
}

#[test]
fn cannot_complete_a_drill_that_never_started() {
    let manager = Manager::new();
    manager.register_plan(sample_plan());
    assert!(manager.complete_drill("region-recovery", "primary-region", 100).is_err());
}

#[test]
fn cannot_register_and_run_unknown_plan() {
    let manager = Manager::new();
    assert!(manager.start_drill("nonexistent", 0).is_err());
}
