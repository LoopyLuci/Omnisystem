//! End-to-end: a three-level theme chain (base -> brand -> brand-dark)
//! resolved for several components and tokens.

use std::collections::HashMap;
use ui_component_library::{Theme, ThemeRegistry};

#[test]
fn three_level_theme_chain_resolves_correctly() {
    let mut registry = ThemeRegistry::new();
    registry.register(
        Theme::new("base")
            .with_token("color.text", "#000000")
            .with_token("radius.md", "4px"),
    );
    registry.register(
        Theme::new("brand")
            .extending("base")
            .with_token("color.primary", "#ff6600"),
    );
    registry.register(
        Theme::new("brand-dark")
            .extending("brand")
            .with_token("color.text", "#ffffff")
            .with_component_override("Card", "radius.md", "12px"),
    );

    let empty = HashMap::new();

    // Inherited from "base" through two levels.
    assert_eq!(registry.resolve("brand-dark", "Card", "radius.md", &empty).unwrap(), "12px");
    assert_eq!(registry.resolve("brand-dark", "Button", "radius.md", &empty).unwrap(), "4px");

    // Overridden at the leaf theme.
    assert_eq!(registry.resolve("brand-dark", "Any", "color.text", &empty).unwrap(), "#ffffff");

    // Defined only at the middle theme, inherited by the leaf.
    assert_eq!(registry.resolve("brand-dark", "Any", "color.primary", &empty).unwrap(), "#ff6600");

    // A local override still wins over all of the above.
    let mut local = HashMap::new();
    local.insert("color.text".to_string(), "#123456".to_string());
    assert_eq!(registry.resolve("brand-dark", "Any", "color.text", &local).unwrap(), "#123456");
}
