//! Demo CLI: resolve a button's spacing token through a dark theme that
//! extends a base theme.

use std::collections::HashMap;
use ui_component_library::{Theme, ThemeRegistry};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut registry = ThemeRegistry::new();
    registry.register(Theme::new("base").with_token("spacing.md", "8px"));
    registry.register(Theme::new("dark").extending("base").with_component_override("Button", "spacing.md", "10px"));

    let value = registry.resolve("dark", "Button", "spacing.md", &HashMap::new())?;
    println!("Button spacing.md = {value}");
    Ok(())
}
