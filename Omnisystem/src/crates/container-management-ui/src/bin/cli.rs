//! Demo CLI: register a container, run it, and check resource limits.

use container_management_ui::{over_limit_running, ContainerState, Registry, ResourceLimits};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut registry = Registry::new();
    registry.register("web", ResourceLimits { cpu_limit: 100.0, cpu_used: 120.0, memory_limit: 512.0, memory_used: 200.0 })?;
    registry.transition("web", ContainerState::Running)?;
    println!("over-limit running containers: {:?}", over_limit_running(&registry));
    Ok(())
}
