//! Demo CLI: lay out a small form schema and print visible fields.

use form_builder::{layout, visible_fields, FieldSchema, FormSchema, ShowIf};
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let schema = FormSchema {
        fields: vec![
            FieldSchema { key: "mode".into(), row: 0, col: 0, default: "standard".into(), show_if: None },
            FieldSchema {
                key: "custom_url".into(),
                row: 1,
                col: 0,
                default: String::new(),
                show_if: Some(ShowIf { field: "mode".into(), equals: "custom".into() }),
            },
        ],
    };
    println!("layout: {:?}", layout(&schema)?);
    let mut answers = HashMap::new();
    answers.insert("mode".to_string(), "custom".to_string());
    println!("visible with mode=custom: {:?}", visible_fields(&schema, &answers)?);
    Ok(())
}
