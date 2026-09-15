//! Error types for form schema layout resolution.

/// Errors produced by this crate.
#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    /// Two fields shared the same key.
    DuplicateField(String),
    /// Two fields in the schema claimed the same `(row, col)` slot.
    LayoutCollision {
        /// Row index.
        row: u32,
        /// Column index.
        col: u32,
        /// The two conflicting field keys.
        fields: (String, String),
    },
    /// A field's `show_if` referenced a field key that doesn't exist.
    UnknownControllingField {
        /// The dependent field.
        field: String,
        /// The missing controller key.
        controller: String,
    },
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::DuplicateField(key) => write!(f, "duplicate field key '{key}'"),
            Error::LayoutCollision { row, col, fields } => {
                write!(f, "fields '{}' and '{}' both claim row {row} col {col}", fields.0, fields.1)
            }
            Error::UnknownControllingField { field, controller } => {
                write!(f, "field '{field}' show_if references unknown field '{controller}'")
            }
        }
    }
}

impl std::error::Error for Error {}

/// Result type for this crate.
pub type Result<T> = std::result::Result<T, Error>;
