//! CLI for docker-registry-integration. Uses a fake mocked auth outcome --
//! this crate never talks to a real registry.

use docker_registry_integration::{AuthOutcome, Direction, Manager};

fn main() -> docker_registry_integration::Result<()> {
    let manager = Manager::new();
    manager.authenticate(AuthOutcome::Granted { token: "fake-token".to_string(), ttl_ticks: 100 }, 0);

    manager.start_transfer("t1", "myorg/app:1.0", Direction::Push, 0)?;
    manager.advance_transfer("t1", 1)?;
    let final_state = manager.advance_transfer("t1", 2)?;
    println!("transfer t1 finished in state: {:?}", final_state);

    Ok(())
}
