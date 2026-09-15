//! Form builder: validates a [`FormSchema`]'s layout (no duplicate keys, no
//! two fields sharing a grid slot, every `show_if` controller exists),
//! derives a row-major layout, default values, and which fields are
//! currently visible given a set of answers.

#![warn(missing_docs)]

use std::collections::{BTreeMap, HashMap};

pub mod error;
pub mod types;

pub use error::{Error, Result};
pub use types::*;

/// Validate a schema: unique keys, no `(row, col)` collisions, and every
/// `show_if.field` refers to a real field.
pub fn validate(schema: &FormSchema) -> Result<()> {
    let mut seen = std::collections::HashSet::new();
    for f in &schema.fields {
        if !seen.insert(f.key.clone()) {
            return Err(Error::DuplicateField(f.key.clone()));
        }
    }
    let mut slots: HashMap<(u32, u32), &str> = HashMap::new();
    for f in &schema.fields {
        if let Some(other) = slots.insert((f.row, f.col), &f.key) {
            return Err(Error::LayoutCollision {
                row: f.row,
                col: f.col,
                fields: (other.to_string(), f.key.clone()),
            });
        }
    }
    for f in &schema.fields {
        if let Some(cond) = &f.show_if {
            if !seen.contains(&cond.field) {
                return Err(Error::UnknownControllingField {
                    field: f.key.clone(),
                    controller: cond.field.clone(),
                });
            }
        }
    }
    Ok(())
}

/// Lay the schema out row-major: rows sorted ascending, fields within a
/// row sorted by column.
pub fn layout(schema: &FormSchema) -> Result<Vec<Vec<String>>> {
    validate(schema)?;
    let mut rows: BTreeMap<u32, Vec<&FieldSchema>> = BTreeMap::new();
    for f in &schema.fields {
        rows.entry(f.row).or_default().push(f);
    }
    let mut out = Vec::with_capacity(rows.len());
    for (_, mut fields) in rows {
        fields.sort_by_key(|f| f.col);
        out.push(fields.into_iter().map(|f| f.key.clone()).collect());
    }
    Ok(out)
}

/// The schema's default values as a key -> value map.
pub fn default_values(schema: &FormSchema) -> HashMap<String, String> {
    schema.fields.iter().map(|f| (f.key.clone(), f.default.clone())).collect()
}

/// Keys of fields currently visible given a set of submitted answers
/// (falling back to defaults for fields not yet answered).
pub fn visible_fields(schema: &FormSchema, answers: &HashMap<String, String>) -> Result<Vec<String>> {
    validate(schema)?;
    let defaults = default_values(schema);
    let mut out = Vec::new();
    for f in &schema.fields {
        let visible = match &f.show_if {
            None => true,
            Some(cond) => {
                let value = answers.get(&cond.field).or_else(|| defaults.get(&cond.field));
                value.map(|v| v == &cond.equals).unwrap_or(false)
            }
        };
        if visible {
            out.push(f.key.clone());
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn field(key: &str, row: u32, col: u32, show_if: Option<ShowIf>) -> FieldSchema {
        FieldSchema { key: key.into(), row, col, default: String::new(), show_if }
    }

    #[test]
    fn validate_rejects_duplicate_keys() {
        let schema = FormSchema { fields: vec![field("a", 0, 0, None), field("a", 1, 0, None)] };
        assert_eq!(validate(&schema), Err(Error::DuplicateField("a".into())));
    }

    #[test]
    fn validate_rejects_layout_collision() {
        let schema = FormSchema { fields: vec![field("a", 0, 0, None), field("b", 0, 0, None)] };
        assert!(matches!(validate(&schema), Err(Error::LayoutCollision { .. })));
    }

    #[test]
    fn validate_rejects_unknown_controller() {
        let schema = FormSchema {
            fields: vec![field("a", 0, 0, Some(ShowIf { field: "ghost".into(), equals: "x".into() }))],
        };
        assert!(matches!(validate(&schema), Err(Error::UnknownControllingField { .. })));
    }

    #[test]
    fn layout_groups_by_row_sorted_by_col() {
        let schema = FormSchema {
            fields: vec![field("b", 0, 1, None), field("a", 0, 0, None), field("c", 1, 0, None)],
        };
        let rows = layout(&schema).unwrap();
        assert_eq!(rows, vec![vec!["a".to_string(), "b".to_string()], vec!["c".to_string()]]);
    }

    #[test]
    fn default_values_maps_key_to_default() {
        let mut f = field("a", 0, 0, None);
        f.default = "hello".into();
        let schema = FormSchema { fields: vec![f] };
        assert_eq!(default_values(&schema).get("a"), Some(&"hello".to_string()));
    }

    #[test]
    fn visible_fields_always_shows_unconditional_field() {
        let schema = FormSchema { fields: vec![field("a", 0, 0, None)] };
        let answers = HashMap::new();
        assert_eq!(visible_fields(&schema, &answers).unwrap(), vec!["a".to_string()]);
    }

    #[test]
    fn visible_fields_hides_when_condition_unmet() {
        let schema = FormSchema {
            fields: vec![
                field("mode", 0, 0, None),
                field("custom_url", 1, 0, Some(ShowIf { field: "mode".into(), equals: "custom".into() })),
            ],
        };
        let mut answers = HashMap::new();
        answers.insert("mode".to_string(), "standard".to_string());
        let visible = visible_fields(&schema, &answers).unwrap();
        assert!(!visible.contains(&"custom_url".to_string()));
    }

    #[test]
    fn visible_fields_shows_when_condition_met() {
        let schema = FormSchema {
            fields: vec![
                field("mode", 0, 0, None),
                field("custom_url", 1, 0, Some(ShowIf { field: "mode".into(), equals: "custom".into() })),
            ],
        };
        let mut answers = HashMap::new();
        answers.insert("mode".to_string(), "custom".to_string());
        let visible = visible_fields(&schema, &answers).unwrap();
        assert!(visible.contains(&"custom_url".to_string()));
    }

    #[test]
    fn visible_fields_falls_back_to_default_when_unanswered() {
        let mut mode = field("mode", 0, 0, None);
        mode.default = "custom".into();
        let schema = FormSchema {
            fields: vec![
                mode,
                field("custom_url", 1, 0, Some(ShowIf { field: "mode".into(), equals: "custom".into() })),
            ],
        };
        let visible = visible_fields(&schema, &HashMap::new()).unwrap();
        assert!(visible.contains(&"custom_url".to_string()));
    }
}
