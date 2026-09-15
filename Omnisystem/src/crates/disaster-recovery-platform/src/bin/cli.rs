//! CLI for disaster-recovery-platform.

use disaster_recovery_platform::{Manager, RecoveryPlan};

fn main() -> disaster_recovery_platform::Result<()> {
    let manager = Manager::new();
    manager.register_plan(RecoveryPlan {
        name: "db-failover".to_string(),
        rpo_ticks: 10,
        rto_ticks: 30,
        steps: vec![
            "promote replica".to_string(),
            "repoint dns".to_string(),
            "verify writes".to_string(),
        ],
    });
    manager.record_snapshot("db-primary", 95);

    manager.start_drill("db-failover", 100)?;
    while let Ok(step) = manager.advance_step("db-failover") {
        println!("executed step: {step}");
    }
    let result = manager.complete_drill("db-failover", "db-primary", 118)?;
    println!(
        "drill complete: rto {} ticks (met={}), rpo {:?} ticks (met={})",
        result.actual_rto_ticks, result.rto_met, result.actual_rpo_ticks, result.rpo_met
    );

    Ok(())
}
