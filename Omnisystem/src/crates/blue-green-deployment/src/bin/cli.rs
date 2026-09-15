//! CLI demo: deploy a new version to standby, health-check it, promote it
//! live, then roll back.

use blue_green_deployment::Manager;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manager = Manager::new();

    println!("active slot: {:?} ({})", manager.active_environment(), manager.active_state().version);

    manager.deploy_to_standby("1.1.0".to_string())?;
    println!("deployed 1.1.0 to standby slot {:?}", manager.standby_state().environment);

    manager.mark_standby_healthy()?;
    let event = manager.promote()?;
    println!("promoted {:?} -> {:?} (version {})", event.from, event.to, event.version);

    let rollback = manager.rollback()?;
    println!("rolled back {:?} -> {:?}", rollback.from, rollback.to);

    Ok(())
}
