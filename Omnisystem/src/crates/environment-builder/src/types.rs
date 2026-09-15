//! Environment layering types.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// One named layer of environment variable overrides (e.g. `base`,
/// `staging`, `local`). Later layers, applied in order, override earlier
/// ones key-by-key.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Layer {
    /// Layer name, for diagnostics.
    pub name: String,
    /// Variables this layer sets or overrides.
    pub vars: BTreeMap<String, String>,
}

impl Layer {
    /// Construct a named, empty layer.
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into(), vars: BTreeMap::new() }
    }

    /// Set a variable on this layer, builder-style.
    pub fn with(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.vars.insert(key.into(), value.into());
        self
    }
}

/// A schema constraint on one variable: whether it's required, and an
/// optional allowed-value set.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VarSpec {
    /// Variable name this spec constrains.
    pub name: String,
    /// Whether the variable must have a value after all layers merge.
    pub required: bool,
    /// If non-empty, the value must be one of these after merging.
    pub allowed_values: Vec<String>,
}
