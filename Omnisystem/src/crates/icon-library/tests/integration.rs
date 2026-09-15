//! End-to-end: build a small icon set and exercise alias, category, and
//! nearest-size resolution together.

use icon_library::{IconDef, Registry};

#[test]
fn registry_pipeline_alias_and_size_resolution() {
    let mut registry = Registry::new();
    registry.register(
        IconDef::new("chevron-right", "navigation")
            .with_alias("arrow-right")
            .with_variant(12, "p12")
            .with_variant(24, "p24")
            .with_variant(48, "p48"),
    );
    registry.register(IconDef::new("chevron-left", "navigation").with_variant(24, "left24"));
    registry.register(IconDef::new("volume", "media").with_variant(24, "vol24"));

    // Alias resolves to the same definition as the canonical name.
    assert_eq!(registry.resolve("arrow-right").unwrap().name, "chevron-right");

    // Nearest-size fallback picks the closest registered variant.
    let (size, _) = registry.nearest_path("arrow-right", 30).unwrap();
    assert_eq!(size, 24);

    // Category browsing only returns matching, sorted entries.
    let nav_names: Vec<&str> = registry.by_category("navigation").iter().map(|i| i.name.as_str()).collect();
    assert_eq!(nav_names, vec!["chevron-left", "chevron-right"]);
    assert_eq!(registry.by_category("media").len(), 1);
}
