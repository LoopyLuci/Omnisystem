//! Image management types: tagged container images in a registry.

use serde::{Deserialize, Serialize};

/// One tagged image in the registry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageTag {
    /// Repository name, e.g. `omnisystem/api`.
    pub repository: String,
    /// Tag, e.g. `v1.2.3` or `latest`.
    pub tag: String,
    /// Content digest — two tags with the same digest are the same bytes.
    pub digest: String,
    /// Day index the tag was pushed (integer for deterministic tests).
    pub pushed_day: i64,
}

/// A `(repository, tag)` reference, used to identify a tag for pruning.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct TagRef {
    /// Repository name.
    pub repository: String,
    /// Tag name.
    pub tag: String,
}
