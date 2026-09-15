//! Demo CLI: build a deploy-pipeline workflow and print its execution waves.

use automation_builder_ui::{parallel_waves, Step, Workflow};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let workflow = Workflow {
        steps: vec![
            Step { name: "lint".into(), depends_on: vec![] },
            Step { name: "unit-test".into(), depends_on: vec![] },
            Step { name: "package".into(), depends_on: vec!["lint".into(), "unit-test".into()] },
            Step { name: "deploy".into(), depends_on: vec!["package".into()] },
        ],
    };
    for (i, wave) in parallel_waves(&workflow)?.into_iter().enumerate() {
        println!("wave {i}: {}", wave.join(", "));
    }
    Ok(())
}
