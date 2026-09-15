//! Document structure types.

use serde::{Deserialize, Serialize};

/// One heading extracted from a document.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Heading {
    /// Heading level, 1-6 (`#` through `######`).
    pub level: u8,
    /// Heading text, with markdown emphasis markers stripped.
    pub text: String,
    /// URL-safe anchor slug derived from the text.
    pub slug: String,
    /// 1-indexed source line number.
    pub line: usize,
}

/// A table-of-contents entry with nested children.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TocEntry {
    /// The heading this entry represents.
    pub heading: Heading,
    /// Nested entries at a deeper level, in document order.
    pub children: Vec<TocEntry>,
}

/// Reading-time and length statistics for a document.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ReadingStats {
    /// Word count, excluding code block contents.
    pub word_count: usize,
    /// Estimated reading time in whole minutes (rounded up), at 200 wpm.
    pub reading_minutes: u32,
}
