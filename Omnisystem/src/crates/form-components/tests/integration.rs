//! End-to-end: a multi-field form validated across several submissions.

use form_components::{Field, Form, Rule};
use std::collections::HashMap;

fn contact_form() -> Form {
    Form::new(vec![
        Field::new("name", vec![Rule::Required, Rule::MinLength(2)]),
        Field::new("email", vec![Rule::Required, Rule::Email]),
        Field::new("message", vec![Rule::Required, Rule::MaxLength(500)]),
    ])
}

fn values(pairs: &[(&str, &str)]) -> HashMap<String, String> {
    pairs.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect()
}

#[test]
fn fully_valid_submission_passes() {
    let form = contact_form();
    let v = values(&[("name", "Jo"), ("email", "jo@example.com"), ("message", "hi")]);
    assert!(form.is_valid(&v));
}

#[test]
fn multiple_field_failures_all_reported() {
    let form = contact_form();
    let v = values(&[("name", ""), ("email", "bad"), ("message", "hi")]);
    let errors = form.validate(&v);
    let fields: Vec<&str> = errors.iter().map(|e| e.field.as_str()).collect();
    assert!(fields.contains(&"name"));
    assert!(fields.contains(&"email"));
    assert!(!fields.contains(&"message"));
}
