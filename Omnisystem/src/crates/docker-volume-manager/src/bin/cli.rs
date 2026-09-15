//! CLI for docker-volume-manager.

use docker_volume_manager::Manager;

fn main() -> docker_volume_manager::Result<()> {
    let manager = Manager::new();
    manager.create("app-data")?;
    manager.mount("app-data", "web-1", "/var/lib/app-data")?;
    println!("state: {:?}", manager.state_of("app-data")?);
    manager.unmount("app-data", "web-1")?;
    manager.remove("app-data")?;
    println!("removed app-data");
    Ok(())
}
