//! CLI for docker-container-lifecycle.

use docker_container_lifecycle::Manager;

fn main() -> docker_container_lifecycle::Result<()> {
    let manager = Manager::new();
    manager.create("web-1")?;
    manager.start("web-1")?;
    println!("web-1 state: {}", manager.state_of("web-1")?);
    manager.pause("web-1")?;
    println!("web-1 state: {}", manager.state_of("web-1")?);
    manager.unpause("web-1")?;
    manager.stop("web-1")?;
    manager.remove("web-1")?;
    println!("running containers: {}", manager.running_count());
    Ok(())
}
