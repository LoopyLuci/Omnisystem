//! Design token and theme types.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A named theme: a flat set of design tokens (e.g. `"color.primary"` ->
/// `"#3366ff"`), optionally extending a parent theme for tokens it doesn't
/// define itself.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    /// Theme name, used to reference it from a registry.
    pub name: String,
    /// This theme's own token values.
    pub tokens: HashMap<String, String>,
    /// Per-component token overrides scoped to this theme
    /// (component name -> token name -> value).
    pub component_overrides: HashMap<String, HashMap<String, String>>,
    /// Name of a parent theme to fall back to for tokens this theme
    /// doesn't define.
    pub parent: Option<String>,
}

impl Theme {
    /// Construct a new, empty theme.
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into(), tokens: HashMap::new(), component_overrides: HashMap::new(), parent: None }
    }

    /// Set a base token value.
    pub fn with_token(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.tokens.insert(name.into(), value.into());
        self
    }

    /// Set a per-component token override.
    pub fn with_component_override(mut self, component: impl Into<String>, token: impl Into<String>, value: impl Into<String>) -> Self {
        self.component_overrides.entry(component.into()).or_default().insert(token.into(), value.into());
        self
    }

    /// Set this theme's parent (for token inheritance).
    pub fn extending(mut self, parent: impl Into<String>) -> Self {
        self.parent = Some(parent.into());
        self
    }
}
