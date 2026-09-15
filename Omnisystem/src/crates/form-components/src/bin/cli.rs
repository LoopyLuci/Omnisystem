//! Demo CLI: validate a hard-coded submission against a signup form.

use form_components::{Field, Form, Rule};
use std::collections::HashMap;

fn main() {
    let form = Form::new(vec![
        Field::new("username", vec![Rule::Required, Rule::MinLength(3)]),
        Field::new("email", vec![Rule::Required, Rule::Email]),
    ]);
    let mut values = HashMap::new();
    values.insert("username".to_string(), "al".to_string());
    values.insert("email".to_string(), "not-an-email".to_string());

    for err in form.validate(&values) {
        println!("{}: {}", err.field, err.message);
    }
}
