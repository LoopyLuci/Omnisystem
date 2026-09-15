//! Form builder types: a declarative field schema with layout ordering and
//! conditional visibility, deliberately separate from `form-components`'
//! validation-rule domain.

use serde::{Deserialize, Serialize};

/// A condition gating a field's visibility on another field's value.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShowIf {
    /// The controlling field's key.
    pub field: String,
    /// The value the controlling field must equal for this field to show.
    pub equals: String,
}

/// One field in a form schema.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldSchema {
    /// Unique field key.
    pub key: String,
    /// Row index this field is laid out in (fields sharing a row render
    /// side by side, ordered by `col`).
    pub row: u32,
    /// Column index within its row.
    pub col: u32,
    /// Default value shown before the user edits the field.
    pub default: String,
    /// Optional visibility condition; `None` means always visible.
    pub show_if: Option<ShowIf>,
}

/// A form schema: an unordered bag of fields (layout is derived by
/// `row`/`col`, not authoring order).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FormSchema {
    /// The fields that make up the form.
    pub fields: Vec<FieldSchema>,
}
