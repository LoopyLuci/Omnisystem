//! Error types for grid widget placement.

/// Errors that can occur while placing widgets on a dashboard grid.
#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    /// A widget's span was wider than the grid itself.
    WidgetTooWide(String, u32, u32),
    /// Two widgets were placed with overlapping cells.
    Overlap(String, String),
    /// A widget id was referenced but not present on the grid.
    UnknownWidget(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::WidgetTooWide(id, w, cols) => {
                write!(f, "widget '{}' is {} columns wide but the grid only has {}", id, w, cols)
            }
            Error::Overlap(a, b) => write!(f, "widgets '{}' and '{}' overlap", a, b),
            Error::UnknownWidget(id) => write!(f, "unknown widget: {}", id),
        }
    }
}

impl std::error::Error for Error {}

/// Result type for grid operations.
pub type Result<T> = std::result::Result<T, Error>;
