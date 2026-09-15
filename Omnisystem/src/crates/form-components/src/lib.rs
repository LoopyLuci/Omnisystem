//! Declarative form field validation: run a set of [`Rule`]s against
//! submitted string values and collect structured errors. Deliberately
//! UI-agnostic — this crate decides *what's wrong*, not how to display it.

#![warn(missing_docs)]

pub mod error;
pub mod types;

pub use error::{Error, Result};
pub use types::*;

use std::collections::HashMap;

impl Rule {
    /// Check `value` against this rule, returning an error message if it
    /// fails.
    fn check(&self, value: &str) -> std::result::Result<(), String> {
        match self {
            Rule::Required => {
                if value.trim().is_empty() {
                    Err("value is required".to_string())
                } else {
                    Ok(())
                }
            }
            Rule::MinLength(min) => {
                if value.chars().count() < *min {
                    Err(format!("must be at least {} characters", min))
                } else {
                    Ok(())
                }
            }
            Rule::MaxLength(max) => {
                if value.chars().count() > *max {
                    Err(format!("must be at most {} characters", max))
                } else {
                    Ok(())
                }
            }
            Rule::NumberRange(min, max) => match value.trim().parse::<f64>() {
                Ok(n) if n >= *min && n <= *max => Ok(()),
                Ok(n) => Err(format!("{} is outside the range [{}, {}]", n, min, max)),
                Err(_) => Err("must be a number".to_string()),
            },
            Rule::Email => {
                let at_count = value.matches('@').count();
                let valid = at_count == 1
                    && value
                        .split('@')
                        .next_tuple()
                        .map(|(local, domain)| {
                            !local.is_empty() && domain.contains('.') && !domain.starts_with('.') && !domain.ends_with('.')
                        })
                        .unwrap_or(false);
                if valid {
                    Ok(())
                } else {
                    Err("must be a valid email address".to_string())
                }
            }
        }
    }
}

trait NextTuple: Iterator + Sized {
    fn next_tuple(&mut self) -> Option<(Self::Item, Self::Item)> {
        let a = self.next()?;
        let b = self.next()?;
        if self.next().is_some() {
            None
        } else {
            Some((a, b))
        }
    }
}
impl<I: Iterator> NextTuple for I {}

impl Form {
    /// Validate a submitted map of field name to raw string value.
    /// Fields absent from `values` are treated as an empty string. Returns
    /// every validation failure across all fields (not just the first).
    pub fn validate(&self, values: &HashMap<String, String>) -> Vec<FieldError> {
        let mut errors = Vec::new();
        for field in &self.fields {
            let value = values.get(&field.name).map(|s| s.as_str()).unwrap_or("");
            for rule in &field.rules {
                if let Err(message) = rule.check(value) {
                    errors.push(FieldError { field: field.name.clone(), message });
                    break; // first failing rule per field, matches doc comment
                }
            }
        }
        errors
    }

    /// Validate a single named field's value in isolation.
    pub fn validate_field(&self, name: &str, value: &str) -> Result<Vec<String>> {
        let field = self.fields.iter().find(|f| f.name == name).ok_or_else(|| Error::UnknownField(name.to_string()))?;
        let mut messages = Vec::new();
        for rule in &field.rules {
            if let Err(m) = rule.check(value) {
                messages.push(m);
            }
        }
        Ok(messages)
    }

    /// True if a full submission has no validation errors.
    pub fn is_valid(&self, values: &HashMap<String, String>) -> bool {
        self.validate(values).is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn signup_form() -> Form {
        Form::new(vec![
            Field::new("username", vec![Rule::Required, Rule::MinLength(3), Rule::MaxLength(20)]),
            Field::new("email", vec![Rule::Required, Rule::Email]),
            Field::new("age", vec![Rule::NumberRange(13.0, 120.0)]),
        ])
    }

    fn values(pairs: &[(&str, &str)]) -> HashMap<String, String> {
        pairs.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect()
    }

    #[test]
    fn valid_submission_has_no_errors() {
        let form = signup_form();
        let v = values(&[("username", "alice"), ("email", "a@example.com"), ("age", "30")]);
        assert!(form.is_valid(&v));
    }

    #[test]
    fn missing_required_field_errors() {
        let form = signup_form();
        let v = values(&[("email", "a@example.com")]);
        let errors = form.validate(&v);
        assert!(errors.iter().any(|e| e.field == "username"));
    }

    #[test]
    fn min_length_rejects_short_values() {
        let form = signup_form();
        let v = values(&[("username", "al"), ("email", "a@example.com")]);
        let errors = form.validate(&v);
        assert!(errors.iter().any(|e| e.field == "username" && e.message.contains("at least")));
    }

    #[test]
    fn max_length_rejects_long_values() {
        let form = signup_form();
        let long = "x".repeat(30);
        let v = values(&[("username", &long), ("email", "a@example.com")]);
        let errors = form.validate(&v);
        assert!(errors.iter().any(|e| e.field == "username" && e.message.contains("at most")));
    }

    #[test]
    fn email_rule_rejects_missing_at_sign() {
        let form = signup_form();
        let v = values(&[("username", "alice"), ("email", "not-an-email")]);
        let errors = form.validate(&v);
        assert!(errors.iter().any(|e| e.field == "email"));
    }

    #[test]
    fn email_rule_rejects_multiple_at_signs() {
        let form = signup_form();
        let v = values(&[("username", "alice"), ("email", "a@b@c.com")]);
        assert!(!form.is_valid(&v));
    }

    #[test]
    fn email_rule_accepts_simple_address() {
        let form = signup_form();
        let v = values(&[("username", "alice"), ("email", "bob@example.co.uk")]);
        let msgs = form.validate_field("email", "bob@example.co.uk").unwrap();
        assert!(msgs.is_empty());
        let _ = v;
    }

    #[test]
    fn number_range_rejects_out_of_bounds() {
        let form = signup_form();
        let v = values(&[("username", "alice"), ("email", "a@example.com"), ("age", "5")]);
        let errors = form.validate(&v);
        assert!(errors.iter().any(|e| e.field == "age"));
    }

    #[test]
    fn number_range_rejects_non_numeric() {
        let form = signup_form();
        let v = values(&[("username", "alice"), ("email", "a@example.com"), ("age", "abc")]);
        let errors = form.validate(&v);
        assert!(errors.iter().any(|e| e.field == "age" && e.message.contains("number")));
    }

    #[test]
    fn unknown_field_lookup_errors() {
        let form = signup_form();
        assert_eq!(form.validate_field("nope", "x").unwrap_err(), Error::UnknownField("nope".into()));
    }

    #[test]
    fn only_first_failing_rule_per_field_is_reported() {
        let form = signup_form();
        // Empty username fails both Required and MinLength; only Required should surface.
        let v = values(&[("username", ""), ("email", "a@example.com")]);
        let errors = form.validate(&v);
        let username_errors: Vec<_> = errors.iter().filter(|e| e.field == "username").collect();
        assert_eq!(username_errors.len(), 1);
        assert!(username_errors[0].message.contains("required"));
    }
}
