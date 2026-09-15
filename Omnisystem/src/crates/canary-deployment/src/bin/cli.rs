//! CLI demo: run a canary rollout to completion, then simulate one that
//! gets rolled back.

use canary_deployment::Manager;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let m = Manager::new();
    println!("starting rollout at {}%", m.start()?);
    for rate in [0.01, 0.02, 0.01, 0.0] {
        let decision = m.record_error_rate(rate)?;
        println!("observed error rate {rate:.2} -> {decision:?}");
    }

    let rollback_demo = Manager::new();
    rollback_demo.start()?;
    let decision = rollback_demo.record_error_rate(0.5)?;
    println!("high error rate -> {decision:?}");

    Ok(())
}
