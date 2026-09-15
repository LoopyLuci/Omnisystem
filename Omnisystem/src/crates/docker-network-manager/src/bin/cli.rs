//! CLI for docker-network-manager.

use docker_network_manager::{Manager, NetworkMode};

fn main() -> docker_network_manager::Result<()> {
    let manager = Manager::new();
    manager.create_network("app-net", NetworkMode::Bridge, Some("172.18.0.0/24"))?;

    let ip = manager.attach("app-net", "web-1")?;
    println!("web-1 attached with ip: {:?}", ip);
    println!("containers attached: {}", manager.attached_count("app-net")?);

    Ok(())
}
