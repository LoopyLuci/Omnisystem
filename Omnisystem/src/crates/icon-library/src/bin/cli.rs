//! Demo CLI: register a couple of icons and resolve them.

use icon_library::{IconDef, Registry};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut registry = Registry::new();
    registry.register(
        IconDef::new("home", "navigation")
            .with_alias("house")
            .with_variant(16, "M0 0h16v16H0z")
            .with_variant(32, "M0 0h32v32H0z"),
    );

    println!("resolved via alias: {}", registry.resolve("house")?.name);
    let (size, path) = registry.nearest_path("home", 20)?;
    println!("nearest to 20px: {size}px -> {path}");
    Ok(())
}
