//! Environment builder: layered variable composition (base -> environment
//! -> local overrides, applied in order) plus schema validation of the
//! merged result (required vars present, values within an allowed set) and
//! diffing between two built environments.

#![warn(missing_docs)]

pub mod error;
pub mod types;

pub use error::{Error, Result};
pub use types::*;

use std::collections::BTreeMap;

/// Merge layers in order, later layers overriding earlier ones key-by-key.
pub fn merge_layers(layers: &[Layer]) -> BTreeMap<String, String> {
    let mut merged = BTreeMap::new();
    for layer in layers {
        for (k, v) in &layer.vars {
            merged.insert(k.clone(), v.clone());
        }
    }
    merged
}

/// Validate a merged environment against a schema, collecting every
/// violation rather than stopping at the first (so a UI can show all
/// problems at once).
pub fn validate(merged: &BTreeMap<String, String>, schema: &[VarSpec]) -> Result<()> {
    let mut errors = Vec::new();
    for spec in schema {
        match merged.get(&spec.name) {
            None if spec.required => errors.push(Error::MissingRequiredVar(spec.name.clone())),
            Some(value) if !spec.allowed_values.is_empty() && !spec.allowed_values.contains(value) => {
                errors.push(Error::InvalidVarValue { name: spec.name.clone(), value: value.clone() })
            }
            _ => {}
        }
    }
    match errors.into_iter().next() {
        Some(e) => Err(e),
        None => Ok(()),
    }
}

/// Validate a merged environment, returning every violation (not just the
/// first) for display in a form.
pub fn validate_all(merged: &BTreeMap<String, String>, schema: &[VarSpec]) -> Vec<Error> {
    let mut errors = Vec::new();
    for spec in schema {
        match merged.get(&spec.name) {
            None if spec.required => errors.push(Error::MissingRequiredVar(spec.name.clone())),
            Some(value) if !spec.allowed_values.is_empty() && !spec.allowed_values.contains(value) => {
                errors.push(Error::InvalidVarValue { name: spec.name.clone(), value: value.clone() })
            }
            _ => {}
        }
    }
    errors
}

/// One difference between two environments for a given key.
#[derive(Debug, Clone, PartialEq)]
pub enum VarDiff {
    /// Present only in the left (`from`) environment.
    Removed(String),
    /// Present only in the right (`to`) environment.
    Added(String, String),
    /// Present in both with different values.
    Changed(String, String, String),
}

/// Diff two merged environments, key by key.
pub fn diff(from: &BTreeMap<String, String>, to: &BTreeMap<String, String>) -> Vec<VarDiff> {
    let mut out = Vec::new();
    for (k, v) in from {
        match to.get(k) {
            None => out.push(VarDiff::Removed(k.clone())),
            Some(nv) if nv != v => out.push(VarDiff::Changed(k.clone(), v.clone(), nv.clone())),
            _ => {}
        }
    }
    for (k, v) in to {
        if !from.contains_key(k) {
            out.push(VarDiff::Added(k.clone(), v.clone()));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn layers() -> Vec<Layer> {
        vec![
            Layer::new("base").with("LOG_LEVEL", "info").with("REGION", "us-east"),
            Layer::new("staging").with("REGION", "us-west"),
            Layer::new("local").with("LOG_LEVEL", "debug"),
        ]
    }

    #[test]
    fn later_layers_override_earlier_ones() {
        let merged = merge_layers(&layers());
        assert_eq!(merged.get("LOG_LEVEL").unwrap(), "debug");
        assert_eq!(merged.get("REGION").unwrap(), "us-west");
    }

    #[test]
    fn merge_of_empty_layers_is_empty() {
        assert!(merge_layers(&[]).is_empty());
    }

    #[test]
    fn validate_passes_when_required_present() {
        let merged = merge_layers(&layers());
        let schema = vec![VarSpec { name: "LOG_LEVEL".into(), required: true, allowed_values: vec![] }];
        assert!(validate(&merged, &schema).is_ok());
    }

    #[test]
    fn validate_fails_on_missing_required() {
        let merged = merge_layers(&layers());
        let schema = vec![VarSpec { name: "API_KEY".into(), required: true, allowed_values: vec![] }];
        assert_eq!(validate(&merged, &schema), Err(Error::MissingRequiredVar("API_KEY".into())));
    }

    #[test]
    fn validate_fails_on_disallowed_value() {
        let merged = merge_layers(&layers());
        let schema = vec![VarSpec {
            name: "LOG_LEVEL".into(),
            required: false,
            allowed_values: vec!["info".into(), "warn".into()],
        }];
        assert!(matches!(validate(&merged, &schema), Err(Error::InvalidVarValue { .. })));
    }

    #[test]
    fn validate_all_collects_every_violation() {
        let merged = merge_layers(&layers());
        let schema = vec![
            VarSpec { name: "API_KEY".into(), required: true, allowed_values: vec![] },
            VarSpec { name: "REGION".into(), required: false, allowed_values: vec!["eu".into()] },
        ];
        let errors = validate_all(&merged, &schema);
        assert_eq!(errors.len(), 2);
    }

    #[test]
    fn diff_detects_changed_added_removed() {
        let from = merge_layers(&[Layer::new("a").with("X", "1").with("Y", "2")]);
        let to = merge_layers(&[Layer::new("b").with("X", "9").with("Z", "3")]);
        let d = diff(&from, &to);
        assert!(d.contains(&VarDiff::Changed("X".into(), "1".into(), "9".into())));
        assert!(d.contains(&VarDiff::Removed("Y".into())));
        assert!(d.contains(&VarDiff::Added("Z".into(), "3".into())));
    }

    #[test]
    fn diff_of_identical_environments_is_empty() {
        let env = merge_layers(&layers());
        assert!(diff(&env, &env).is_empty());
    }
}
