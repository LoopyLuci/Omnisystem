//! Demo CLI: register a couple of agents and drive them through commands.

use agent_control_ui::{AgentRegistry, Command};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut registry = AgentRegistry::new();
    registry.register("scraper-1", "Scraper")?;
    registry.register("indexer-1", "Indexer")?;

    registry.dispatch("scraper-1", Command::Start)?;
    registry.dispatch("indexer-1", Command::Start)?;
    registry.dispatch("scraper-1", Command::Pause)?;

    for (state, count) in registry.state_counts() {
        println!("{state:?}: {count}");
    }
    Ok(())
}
