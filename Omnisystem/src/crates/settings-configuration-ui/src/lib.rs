//! Settings configuration: validates each layer's values against a schema,
//! then resolves a final value per key by cascading `Default -> Project ->
//! User`, the highest-priority layer that sets a key winning.

#![warn(missing_docs)]

use std::collections::HashMap;

pub mod error;
pub mod types;

pub use error::{Error, Result};
pub use types::*;

/// Validate one layer's values against the schema: every key must be
/// declared and every value must match its declared type.
pub fn validate_layer(schema: &Schema, values: &LayerValues) -> Result<()> {
    for (key, value) in values {
        match schema.get(key) {
            None => return Err(Error::UnknownKey { key: key.clone() }),
            Some(expected) if *expected != value.kind() => {
                return Err(Error::TypeMismatch { key: key.clone(), expected: *expected, actual: value.kind() })
            }
            _ => {}
        }
    }
    Ok(())
}

/// Resolve the effective value for one key: the highest layer (by
/// `Layer` ordering) that sets it wins; `None` if no layer sets it.
pub fn resolve_key<'a>(key: &str, layers: &[(Layer, &'a LayerValues)]) -> Option<&'a SettingValue> {
    let mut sorted: Vec<&(Layer, &LayerValues)> = layers.iter().collect();
    sorted.sort_by_key(|(l, _)| *l);
    sorted.iter().rev().find_map(|(_, values)| values.get(key))
}

/// Validate every layer, then resolve every schema key to its effective
/// value across all layers.
pub fn resolve_all(schema: &Schema, layers: &[(Layer, &LayerValues)]) -> Result<HashMap<String, SettingValue>> {
    for (_, values) in layers {
        validate_layer(schema, values)?;
    }
    let mut out = HashMap::new();
    for key in schema.keys() {
        if let Some(v) = resolve_key(key, layers) {
            out.insert(key.clone(), v.clone());
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn schema() -> Schema {
        let mut s = Schema::new();
        s.insert("theme".to_string(), SettingType::String);
        s.insert("notifications_enabled".to_string(), SettingType::Bool);
        s
    }

    #[test]
    fn validate_layer_rejects_undeclared_key() {
        let mut values = LayerValues::new();
        values.insert("ghost".to_string(), SettingValue::Bool(true));
        assert!(matches!(validate_layer(&schema(), &values), Err(Error::UnknownKey { .. })));
    }

    #[test]
    fn validate_layer_rejects_type_mismatch() {
        let mut values = LayerValues::new();
        values.insert("theme".to_string(), SettingValue::Bool(true));
        assert!(matches!(validate_layer(&schema(), &values), Err(Error::TypeMismatch { .. })));
    }

    #[test]
    fn validate_layer_accepts_matching_types() {
        let mut values = LayerValues::new();
        values.insert("theme".to_string(), SettingValue::Str("dark".into()));
        assert!(validate_layer(&schema(), &values).is_ok());
    }

    #[test]
    fn resolve_key_picks_highest_layer_that_sets_it() {
        let mut defaults = LayerValues::new();
        defaults.insert("theme".to_string(), SettingValue::Str("light".into()));
        let mut user = LayerValues::new();
        user.insert("theme".to_string(), SettingValue::Str("dark".into()));
        let layers: Vec<(Layer, &LayerValues)> = vec![(Layer::Default, &defaults), (Layer::User, &user)];
        assert_eq!(resolve_key("theme", &layers), Some(&SettingValue::Str("dark".into())));
    }

    #[test]
    fn resolve_key_falls_back_when_higher_layer_silent() {
        let mut defaults = LayerValues::new();
        defaults.insert("theme".to_string(), SettingValue::Str("light".into()));
        let user = LayerValues::new();
        let layers: Vec<(Layer, &LayerValues)> = vec![(Layer::Default, &defaults), (Layer::User, &user)];
        assert_eq!(resolve_key("theme", &layers), Some(&SettingValue::Str("light".into())));
    }

    #[test]
    fn resolve_key_none_when_no_layer_sets_it() {
        let defaults = LayerValues::new();
        let layers: Vec<(Layer, &LayerValues)> = vec![(Layer::Default, &defaults)];
        assert_eq!(resolve_key("theme", &layers), None);
    }

    #[test]
    fn resolve_all_validates_and_resolves_every_key() {
        let mut defaults = LayerValues::new();
        defaults.insert("theme".to_string(), SettingValue::Str("light".into()));
        defaults.insert("notifications_enabled".to_string(), SettingValue::Bool(true));
        let mut project = LayerValues::new();
        project.insert("theme".to_string(), SettingValue::Str("dark".into()));
        let layers: Vec<(Layer, &LayerValues)> = vec![(Layer::Default, &defaults), (Layer::Project, &project)];
        let resolved = resolve_all(&schema(), &layers).unwrap();
        assert_eq!(resolved.get("theme"), Some(&SettingValue::Str("dark".into())));
        assert_eq!(resolved.get("notifications_enabled"), Some(&SettingValue::Bool(true)));
    }

    #[test]
    fn resolve_all_propagates_validation_error() {
        let mut bad = LayerValues::new();
        bad.insert("ghost".to_string(), SettingValue::Bool(true));
        let layers: Vec<(Layer, &LayerValues)> = vec![(Layer::Default, &bad)];
        assert!(resolve_all(&schema(), &layers).is_err());
    }
}
