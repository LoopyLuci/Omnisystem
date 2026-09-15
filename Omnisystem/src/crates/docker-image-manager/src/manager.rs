use crate::error::{Error, Result};
use crate::types::{Image, Layer};
use std::collections::HashMap;
use std::sync::RwLock;

fn tag_key(registry: &str, repository: &str, tag: &str) -> String {
    format!("{registry}/{repository}:{tag}")
}

/// Tracks images by tag and by digest. Pushing an image under a tag that
/// already exists *moves* the tag to the new image (matching real Docker:
/// tags are mutable pointers), while pushing an identical digest twice is
/// rejected as a duplicate (digests are content-addressed and immutable).
pub struct Manager {
    /// registry/repository:tag -> digest
    tags: RwLock<HashMap<String, String>>,
    /// digest -> image
    images: RwLock<HashMap<String, Image>>,
}

impl Manager {
    /// Create an empty manager.
    pub fn new() -> Self {
        Self { tags: RwLock::new(HashMap::new()), images: RwLock::new(HashMap::new()) }
    }

    /// Push an image built from raw layers under `reference_str`, computing
    /// a synthetic digest from the layer digests (real Docker computes this
    /// from the manifest; here we just join the layer digests).
    pub fn push(&self, reference_str: &str, layers: Vec<Layer>) -> Result<Image> {
        let reference = crate::parse::parse(reference_str)?;
        let digest = reference
            .digest
            .clone()
            .unwrap_or_else(|| format!("sha256:{}", layers.iter().map(|l| l.digest.as_str()).collect::<Vec<_>>().join("+")));

        // Pushing the exact same tag pointing at the exact same content
        // again is a genuine no-op duplicate; reject it. Pushing identical
        // *content* under a *different* tag is fine (that's just adding
        // another tag alias for the same digest, same as real Docker).
        if let Some(tag) = &reference.tag {
            let key = tag_key(&reference.registry, &reference.repository, tag);
            if self.tags.read().unwrap().get(&key) == Some(&digest) {
                return Err(Error::AlreadyExists(reference_str.to_string()));
            }
        }

        let image = Image { reference: reference.clone(), layers };
        self.images.write().unwrap().insert(digest.clone(), image.clone());

        if let Some(tag) = &reference.tag {
            self.tags
                .write()
                .unwrap()
                .insert(tag_key(&reference.registry, &reference.repository, tag), digest);
        }

        Ok(image)
    }

    /// Resolve `registry/repository:tag` to the image it currently points to.
    pub fn resolve_tag(&self, registry: &str, repository: &str, tag: &str) -> Result<Image> {
        let key = tag_key(registry, repository, tag);
        let digest = self
            .tags
            .read()
            .unwrap()
            .get(&key)
            .cloned()
            .ok_or_else(|| Error::ImageNotFound(key.clone()))?;
        self.images
            .read()
            .unwrap()
            .get(&digest)
            .cloned()
            .ok_or_else(|| Error::ImageNotFound(digest))
    }

    /// Look up an image by exact digest.
    pub fn get_by_digest(&self, digest: &str) -> Result<Image> {
        self.images.read().unwrap().get(digest).cloned().ok_or_else(|| Error::ImageNotFound(digest.to_string()))
    }

    /// Number of distinct digests tracked (i.e. distinct image content, not
    /// distinct tags -- several tags can point at the same digest).
    pub fn image_count(&self) -> usize {
        self.images.read().unwrap().len()
    }
}

impl Default for Manager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn layers() -> Vec<Layer> {
        vec![
            Layer { digest: "sha256:base".to_string(), size_bytes: 1000 },
            Layer { digest: "sha256:app".to_string(), size_bytes: 200 },
        ]
    }

    #[test]
    fn push_and_resolve_tag() {
        let m = Manager::new();
        m.push("myorg/myapp:1.0", layers()).unwrap();
        let img = m.resolve_tag("docker.io", "myorg/myapp", "1.0").unwrap();
        assert_eq!(img.total_size_bytes(), 1200);
    }

    #[test]
    fn retagging_moves_the_tag_pointer() {
        let m = Manager::new();
        m.push("myorg/myapp:latest", vec![Layer { digest: "sha256:v1".to_string(), size_bytes: 100 }]).unwrap();
        m.push("myorg/myapp:latest", vec![Layer { digest: "sha256:v2".to_string(), size_bytes: 300 }]).unwrap();

        let img = m.resolve_tag("docker.io", "myorg/myapp", "latest").unwrap();
        assert_eq!(img.total_size_bytes(), 300);
        // Both digests remain individually addressable.
        assert_eq!(m.image_count(), 2);
    }

    #[test]
    fn pushing_identical_digest_twice_is_rejected() {
        let m = Manager::new();
        m.push("myorg/myapp:1.0", layers()).unwrap();
        let err = m.push("myorg/myapp:1.0", layers()).unwrap_err();
        assert!(matches!(err, Error::AlreadyExists(_)));
    }

    #[test]
    fn resolving_unknown_tag_errors() {
        let m = Manager::new();
        assert!(matches!(m.resolve_tag("docker.io", "myorg/myapp", "latest").unwrap_err(), Error::ImageNotFound(_)));
    }

    #[test]
    fn invalid_reference_on_push_errors() {
        let m = Manager::new();
        assert!(m.push("", layers()).is_err());
    }

    #[test]
    fn multiple_tags_can_point_at_same_digest() {
        let m = Manager::new();
        let l = vec![Layer { digest: "sha256:shared".to_string(), size_bytes: 50 }];
        m.push("myorg/myapp:1.0", l.clone()).unwrap();
        m.push("myorg/myapp:stable", l).unwrap();

        let a = m.resolve_tag("docker.io", "myorg/myapp", "1.0").unwrap();
        let b = m.resolve_tag("docker.io", "myorg/myapp", "stable").unwrap();
        assert_eq!(a.layers, b.layers); // both tags resolve to the same underlying image content
        assert_eq!(m.image_count(), 1);
    }
}
