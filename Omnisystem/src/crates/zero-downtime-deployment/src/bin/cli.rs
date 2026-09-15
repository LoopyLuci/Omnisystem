//! CLI demo: bring an instance up, drain it under load, and terminate it
//! once safe.

use zero_downtime_deployment::Manager;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let m = Manager::new();
    m.register_instance("web-1");
    m.mark_ready("web-1")?;
    m.record_request_start("web-1")?;
    m.begin_drain("web-1")?;
    println!("draining web-1, can_terminate={}", m.can_terminate("web-1")?);

    m.record_request_end("web-1")?;
    println!("request finished, can_terminate={}", m.can_terminate("web-1")?);
    m.terminate("web-1")?;
    println!("web-1 terminated");

    Ok(())
}
