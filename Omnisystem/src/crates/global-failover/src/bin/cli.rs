//! CLI

use global_failover::Manager;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manager = Manager::new("us-east", 5, true);
    manager.register_region("us-east", 0);
    manager.register_region("us-west", 0);
    manager.register_region("eu-central", 0);

    let status = manager.evaluate(0);
    println!(
        "active primary: {:?} ({} / {} regions alive, failovers: {})",
        status.active_primary, status.alive_regions, status.total_regions, status.failover_count
    );

    Ok(())
}
