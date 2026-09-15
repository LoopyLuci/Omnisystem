//! Data types
use serde::{Deserialize, Serialize};

/// A parsed Docker image reference:
/// `[registry/]repository[:tag][@digest]`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImageReference {
    /// Registry host, e.g. `docker.io`, `ghcr.io`, `localhost:5000`.
    /// Defaults to `docker.io` when not specified.
    pub registry: String,
    /// Repository path, e.g. `library/nginx` or `myorg/myapp`.
    pub repository: String,
    /// Tag, e.g. `latest`, `1.2.3`. Defaults to `latest` when not specified
    /// and no digest is present.
    pub tag: Option<String>,
    /// Content digest, e.g. `sha256:abc123...`, if pinned.
    pub digest: Option<String>,
}

impl std::fmt::Display for ImageReference {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}", self.registry, self.repository)?;
        if let Some(tag) = &self.tag {
            write!(f, ":{tag}")?;
        }
        if let Some(digest) = &self.digest {
            write!(f, "@{digest}")?;
        }
        Ok(())
    }
}

/// A single filesystem layer within an image, with its content digest and size.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Layer {
    /// Layer content digest.
    pub digest: String,
    /// Compressed size in bytes.
    pub size_bytes: u64,
}

/// A tracked image: its reference plus layer manifest.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Image {
    /// The image's reference (registry/repository/tag/digest).
    pub reference: ImageReference,
    /// Layers making up the image, in order from base to top.
    pub layers: Vec<Layer>,
}

impl Image {
    /// Total size of all layers combined.
    pub fn total_size_bytes(&self) -> u64 {
        self.layers.iter().map(|l| l.size_bytes).sum()
    }
}
