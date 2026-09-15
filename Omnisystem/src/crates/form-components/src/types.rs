//! Field definitions and validation rules.

use serde::{Deserialize, Serialize};

/// A single validation rule applied to a field's raw string value.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Rule {
    /// The value must be non-empty (after trimming whitespace).
    Required,
    /// The value's character count must be at least this many.
    MinLength(usize),
    /// The value's character count must be at most this many.
    MaxLength(usize),
    /// The value must parse as a number within `[min, max]`.
    NumberRange(f64, f64),
    /// The value must look like `local@domain.tld` (a simple structural
    /// check, not full RFC 5322 validation).
    Email,
}

/// One reported validation failure.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FieldError {
    /// The field that failed.
    pub field: String,
    /// Human-readable reason.
    pub message: String,
}

/// A named field with an ordered list of rules to apply, in order.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Field {
    /// Field name (used as the form-value lookup key).
    pub name: String,
    /// Rules checked in order; the first failure short-circuits the rest
    /// for this field.
    pub rules: Vec<Rule>,
}

impl Field {
    /// Construct a new field with the given rules.
    pub fn new(name: impl Into<String>, rules: Vec<Rule>) -> Self {
        Self { name: name.into(), rules }
    }
}

/// A form: an ordered set of fields to validate together.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Form {
    /// The fields making up this form.
    pub fields: Vec<Field>,
}

impl Form {
    /// Construct a new form from its fields.
    pub fn new(fields: Vec<Field>) -> Self {
        Self { fields }
    }
}
