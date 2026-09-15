//! Design-token resolution: cascade a token lookup through a per-call local
//! override, the theme's per-component overrides, the theme's own tokens,
//! and finally its parent theme chain. Deliberately rendering-agnostic —
//! this crate resolves *values*, a UI layer applies them to styles.

#![warn(missing_docs)]

pub mod error;
pub mod types;

pub use error::{Error, Result};
pub use types::*;

use std::collections::HashMap;

/// A registry of named themes.
#[derive(Debug, Clone, Default)]
pub struct ThemeRegistry {
    themes: HashMap<String, Theme>,
}

impl ThemeRegistry {
    /// Construct an empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a theme.
    pub fn register(&mut self, theme: Theme) {
        self.themes.insert(theme.name.clone(), theme);
    }

    /// Resolve a token's value, cascading in priority order:
    /// 1. `local_overrides` (caller-supplied, e.g. inline component props)
    /// 2. the theme's override for `component`, if any
    /// 3. the theme's own base token
    /// 4. the parent theme chain, repeating steps 2-3 at each level
    ///
    /// A cyclic parent chain is detected and treated as "no more parents"
    /// rather than looping forever.
    pub fn resolve(
        &self,
        theme_name: &str,
        component: &str,
        token: &str,
        local_overrides: &HashMap<String, String>,
    ) -> Result<String> {
        if let Some(v) = local_overrides.get(token) {
            return Ok(v.clone());
        }

        let mut current = self.themes.get(theme_name).ok_or_else(|| Error::UnknownTheme(theme_name.to_string()))?;
        let mut visited = vec![current.name.clone()];

        loop {
            if let Some(v) = current.component_overrides.get(component).and_then(|m| m.get(token)) {
                return Ok(v.clone());
            }
            if let Some(v) = current.tokens.get(token) {
                return Ok(v.clone());
            }
            match &current.parent {
                Some(parent_name) if !visited.contains(parent_name) => {
                    current = self.themes.get(parent_name).ok_or_else(|| Error::UnknownTheme(parent_name.clone()))?;
                    visited.push(current.name.clone());
                }
                _ => break,
            }
        }

        Err(Error::UnresolvedToken(token.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn registry() -> ThemeRegistry {
        let mut r = ThemeRegistry::new();
        r.register(
            Theme::new("base")
                .with_token("color.primary", "#3366ff")
                .with_token("spacing.md", "8px"),
        );
        r.register(
            Theme::new("dark")
                .extending("base")
                .with_token("color.primary", "#88aaff")
                .with_component_override("Button", "spacing.md", "10px"),
        );
        r
    }

    #[test]
    fn resolves_own_token_before_parent() {
        let r = registry();
        let empty = HashMap::new();
        assert_eq!(r.resolve("dark", "Card", "color.primary", &empty).unwrap(), "#88aaff");
    }

    #[test]
    fn falls_back_to_parent_when_own_theme_lacks_token() {
        let r = registry();
        let empty = HashMap::new();
        // "dark" doesn't define spacing.md itself (only a Button override), base does.
        assert_eq!(r.resolve("dark", "Card", "spacing.md", &empty).unwrap(), "8px");
    }

    #[test]
    fn component_override_beats_theme_and_parent_tokens() {
        let r = registry();
        let empty = HashMap::new();
        assert_eq!(r.resolve("dark", "Button", "spacing.md", &empty).unwrap(), "10px");
    }

    #[test]
    fn local_override_beats_everything() {
        let r = registry();
        let mut local = HashMap::new();
        local.insert("spacing.md".to_string(), "999px".to_string());
        assert_eq!(r.resolve("dark", "Button", "spacing.md", &local).unwrap(), "999px");
    }

    #[test]
    fn unknown_theme_errors() {
        let r = registry();
        let empty = HashMap::new();
        assert_eq!(r.resolve("nope", "Card", "color.primary", &empty).unwrap_err(), Error::UnknownTheme("nope".into()));
    }

    #[test]
    fn undefined_token_errors_after_exhausting_chain() {
        let r = registry();
        let empty = HashMap::new();
        assert_eq!(
            r.resolve("dark", "Card", "color.nonexistent", &empty).unwrap_err(),
            Error::UnresolvedToken("color.nonexistent".into())
        );
    }

    #[test]
    fn cyclic_parent_chain_does_not_infinite_loop() {
        let mut r = ThemeRegistry::new();
        r.register(Theme::new("a").extending("b"));
        r.register(Theme::new("b").extending("a"));
        let empty = HashMap::new();
        // Neither theme defines the token; must terminate with an error, not hang.
        assert!(r.resolve("a", "X", "missing", &empty).is_err());
    }

    #[test]
    fn base_theme_with_no_parent_resolves_its_own_tokens() {
        let r = registry();
        let empty = HashMap::new();
        assert_eq!(r.resolve("base", "Anything", "spacing.md", &empty).unwrap(), "8px");
    }
}
