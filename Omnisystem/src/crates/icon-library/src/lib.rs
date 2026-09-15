//! Icon registry: name/alias resolution, category browsing, and
//! nearest-size variant fallback. Deliberately rendering-agnostic — this
//! crate resolves *which path data* to use, not how to draw it.

#![warn(missing_docs)]

pub mod error;
pub mod types;

pub use error::{Error, Result};
pub use types::*;

use std::collections::HashMap;

/// A registry of icon definitions, indexed by name and alias.
#[derive(Debug, Clone, Default)]
pub struct Registry {
    icons: HashMap<String, IconDef>,
    /// Maps alias -> canonical name.
    aliases: HashMap<String, String>,
}

impl Registry {
    /// Construct an empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Register an icon, indexing it by its canonical name and all aliases.
    /// Registering the same canonical name twice replaces the prior
    /// definition and its old aliases are dropped.
    pub fn register(&mut self, icon: IconDef) {
        // Drop any stale aliases from a prior registration of this same
        // canonical name before adding the new set.
        self.aliases.retain(|_, canonical| canonical != &icon.name);
        for alias in &icon.aliases {
            self.aliases.insert(alias.clone(), icon.name.clone());
        }
        self.icons.insert(icon.name.clone(), icon);
    }

    /// Resolve a name or alias to its icon definition.
    pub fn resolve(&self, name: &str) -> Result<&IconDef> {
        if let Some(icon) = self.icons.get(name) {
            return Ok(icon);
        }
        if let Some(canonical) = self.aliases.get(name) {
            if let Some(icon) = self.icons.get(canonical) {
                return Ok(icon);
            }
        }
        Err(Error::UnknownIcon(name.to_string()))
    }

    /// Resolve the path data for `name` at exactly `size`.
    pub fn path_at(&self, name: &str, size: u32) -> Result<&str> {
        let icon = self.resolve(name)?;
        icon.variants
            .get(&size)
            .map(|s| s.as_str())
            .ok_or_else(|| Error::UnknownSize(icon.name.clone(), size))
    }

    /// Resolve the path data for `name` at the variant size closest to
    /// `size` (ties broken toward the smaller size). Errors only if the
    /// icon has no variants registered at all.
    pub fn nearest_path(&self, name: &str, size: u32) -> Result<(u32, &str)> {
        let icon = self.resolve(name)?;
        icon.variants
            .iter()
            .min_by_key(|(&s, _)| {
                let diff = (s as i64 - size as i64).unsigned_abs();
                (diff, s)
            })
            .map(|(&s, path)| (s, path.as_str()))
            .ok_or_else(|| Error::UnknownSize(icon.name.clone(), size))
    }

    /// All icons in a given category, sorted by name.
    pub fn by_category(&self, category: &str) -> Vec<&IconDef> {
        let mut out: Vec<&IconDef> = self.icons.values().filter(|i| i.category == category).collect();
        out.sort_by(|a, b| a.name.cmp(&b.name));
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_registry() -> Registry {
        let mut r = Registry::new();
        r.register(
            IconDef::new("home", "navigation")
                .with_alias("house")
                .with_variant(16, "M0 0h16v16H0z-16")
                .with_variant(32, "M0 0h32v32H0z-32"),
        );
        r.register(IconDef::new("search", "navigation").with_variant(24, "M0 0h24v24H0z-24"));
        r.register(IconDef::new("play", "media").with_variant(16, "M0 0h16v16H0z-play"));
        r
    }

    #[test]
    fn resolves_by_canonical_name() {
        let r = sample_registry();
        assert_eq!(r.resolve("home").unwrap().name, "home");
    }

    #[test]
    fn resolves_by_alias() {
        let r = sample_registry();
        assert_eq!(r.resolve("house").unwrap().name, "home");
    }

    #[test]
    fn unknown_icon_errors() {
        let r = sample_registry();
        assert_eq!(r.resolve("nope").unwrap_err(), Error::UnknownIcon("nope".into()));
    }

    #[test]
    fn exact_size_lookup_succeeds() {
        let r = sample_registry();
        assert_eq!(r.path_at("home", 16).unwrap(), "M0 0h16v16H0z-16");
    }

    #[test]
    fn exact_size_lookup_fails_for_missing_variant() {
        let r = sample_registry();
        assert_eq!(r.path_at("home", 64).unwrap_err(), Error::UnknownSize("home".into(), 64));
    }

    #[test]
    fn nearest_size_picks_closest_variant() {
        let r = sample_registry();
        // home has 16 and 32; requesting 20 should pick 16 (closer).
        let (size, _) = r.nearest_path("home", 20).unwrap();
        assert_eq!(size, 16);
        // requesting 30 should pick 32.
        let (size, _) = r.nearest_path("home", 30).unwrap();
        assert_eq!(size, 32);
    }

    #[test]
    fn nearest_size_breaks_exact_midpoint_tie_toward_smaller() {
        let r = sample_registry();
        // midpoint between 16 and 32 is 24; ties break toward the smaller size.
        let (size, _) = r.nearest_path("home", 24).unwrap();
        assert_eq!(size, 16);
    }

    #[test]
    fn nearest_size_errors_when_no_variants_exist() {
        let mut r = Registry::new();
        r.register(IconDef::new("blank", "misc"));
        assert!(r.nearest_path("blank", 16).is_err());
    }

    #[test]
    fn category_listing_is_sorted_and_scoped() {
        let r = sample_registry();
        let nav: Vec<&str> = r.by_category("navigation").iter().map(|i| i.name.as_str()).collect();
        assert_eq!(nav, vec!["home", "search"]);
        assert_eq!(r.by_category("media").len(), 1);
        assert!(r.by_category("nonexistent").is_empty());
    }

    #[test]
    fn re_registering_replaces_definition_and_old_aliases() {
        let mut r = sample_registry();
        r.register(IconDef::new("home", "navigation").with_variant(16, "new-path"));
        assert_eq!(r.path_at("home", 16).unwrap(), "new-path");
        // Old alias "house" no longer resolves since the new def has none.
        assert!(r.resolve("house").is_err());
    }
}
