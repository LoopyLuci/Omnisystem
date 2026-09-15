//! Icon registry data model.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// One icon definition: its canonical path data per available pixel size,
/// plus any alternate names it can be looked up by.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IconDef {
    /// Canonical (primary) name.
    pub name: String,
    /// Category used for grouping/browsing (e.g. "navigation", "media").
    pub category: String,
    /// Alternate names that also resolve to this icon.
    pub aliases: Vec<String>,
    /// SVG path data, keyed by pixel size (e.g. 16, 24, 32).
    pub variants: BTreeMap<u32, String>,
}

impl IconDef {
    /// Construct a new icon definition with no variants or aliases yet.
    pub fn new(name: impl Into<String>, category: impl Into<String>) -> Self {
        Self { name: name.into(), category: category.into(), aliases: Vec::new(), variants: BTreeMap::new() }
    }

    /// Register a size variant's path data.
    pub fn with_variant(mut self, size: u32, path: impl Into<String>) -> Self {
        self.variants.insert(size, path.into());
        self
    }

    /// Register an alias name.
    pub fn with_alias(mut self, alias: impl Into<String>) -> Self {
        self.aliases.push(alias.into());
        self
    }
}
