//! Settings configuration types: a layered override cascade
//! (default -> project -> user) with a simple type schema.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// The scalar type a setting's value must be.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum SettingType {
    /// A string value.
    String,
    /// A boolean value.
    Bool,
    /// A numeric value.
    Number,
}

/// A concrete setting value, tagged with its type.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SettingValue {
    /// A string value.
    Str(String),
    /// A boolean value.
    Bool(bool),
    /// A numeric value.
    Number(f64),
}

impl SettingValue {
    /// The [`SettingType`] this value belongs to.
    pub fn kind(&self) -> SettingType {
        match self {
            SettingValue::Str(_) => SettingType::String,
            SettingValue::Bool(_) => SettingType::Bool,
            SettingValue::Number(_) => SettingType::Number,
        }
    }
}

/// One override layer, from lowest to highest priority: `Default`,
/// `Project`, `User`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Layer {
    /// The built-in default layer (lowest priority).
    Default,
    /// Project-level overrides.
    Project,
    /// User-level overrides (highest priority).
    User,
}

/// The declared type for every known setting key.
pub type Schema = HashMap<String, SettingType>;

/// One layer's key -> value overrides.
pub type LayerValues = HashMap<String, SettingValue>;
