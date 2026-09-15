//! Image management: registry-tag bookkeeping — grouping tags that share a
//! content digest (dedup candidates), and computing which tags a
//! "keep the N most recent per repository" retention policy would prune.

#![warn(missing_docs)]

use std::collections::BTreeMap;

pub mod error;
pub mod types;

pub use error::{Error, Result};
pub use types::*;

/// Validate that no `(repository, tag)` pair is duplicated.
pub fn validate(images: &[ImageTag]) -> Result<()> {
    let mut seen = std::collections::HashSet::new();
    for img in images {
        if !seen.insert((img.repository.clone(), img.tag.clone())) {
            return Err(Error::DuplicateTag { repository: img.repository.clone(), tag: img.tag.clone() });
        }
    }
    Ok(())
}

/// Group tags by content digest, keeping only digests referenced by 2+
/// tags — these are dedup candidates for storage.
pub fn duplicate_digest_groups(images: &[ImageTag]) -> Result<BTreeMap<String, Vec<TagRef>>> {
    validate(images)?;
    let mut groups: BTreeMap<String, Vec<TagRef>> = BTreeMap::new();
    for img in images {
        groups
            .entry(img.digest.clone())
            .or_default()
            .push(TagRef { repository: img.repository.clone(), tag: img.tag.clone() });
    }
    groups.retain(|_, tags| tags.len() > 1);
    for tags in groups.values_mut() {
        tags.sort();
    }
    Ok(groups)
}

/// For each repository, compute the tags to prune under a "keep the
/// `keep_n` most recently pushed tags" policy. Returns pruned refs sorted
/// by repository then tag.
pub fn tags_to_prune(images: &[ImageTag], keep_n: usize) -> Result<Vec<TagRef>> {
    validate(images)?;
    let mut by_repo: BTreeMap<&str, Vec<&ImageTag>> = BTreeMap::new();
    for img in images {
        by_repo.entry(img.repository.as_str()).or_default().push(img);
    }
    let mut prune = Vec::new();
    for tags in by_repo.values_mut() {
        tags.sort_by(|a, b| b.pushed_day.cmp(&a.pushed_day));
        for img in tags.iter().skip(keep_n) {
            prune.push(TagRef { repository: img.repository.clone(), tag: img.tag.clone() });
        }
    }
    prune.sort();
    Ok(prune)
}

/// Count of distinct content digests vs. total tags — a rough measure of
/// storage dedup potential (`unique_bytes / total_tags`).
pub fn dedup_ratio(images: &[ImageTag]) -> Result<f64> {
    validate(images)?;
    if images.is_empty() {
        return Ok(1.0);
    }
    let unique: std::collections::HashSet<&str> = images.iter().map(|i| i.digest.as_str()).collect();
    Ok(unique.len() as f64 / images.len() as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tag(repo: &str, tag: &str, digest: &str, day: i64) -> ImageTag {
        ImageTag { repository: repo.into(), tag: tag.into(), digest: digest.into(), pushed_day: day }
    }

    #[test]
    fn validate_rejects_duplicate_repo_tag() {
        let images = vec![tag("api", "v1", "d1", 1), tag("api", "v1", "d2", 2)];
        assert!(matches!(validate(&images), Err(Error::DuplicateTag { .. })));
    }

    #[test]
    fn duplicate_digest_groups_finds_shared_digests() {
        let images = vec![
            tag("api", "v1", "sha256:abc", 1),
            tag("api", "latest", "sha256:abc", 2),
            tag("api", "v2", "sha256:def", 3),
        ];
        let groups = duplicate_digest_groups(&images).unwrap();
        assert_eq!(groups.len(), 1);
        assert_eq!(groups["sha256:abc"].len(), 2);
    }

    #[test]
    fn duplicate_digest_groups_excludes_unique_digests() {
        let images = vec![tag("api", "v1", "sha256:abc", 1)];
        assert!(duplicate_digest_groups(&images).unwrap().is_empty());
    }

    #[test]
    fn tags_to_prune_keeps_most_recent_n_per_repo() {
        let images = vec![
            tag("api", "v1", "d1", 1),
            tag("api", "v2", "d2", 2),
            tag("api", "v3", "d3", 3),
        ];
        let prune = tags_to_prune(&images, 2).unwrap();
        assert_eq!(prune, vec![TagRef { repository: "api".into(), tag: "v1".into() }]);
    }

    #[test]
    fn tags_to_prune_is_scoped_per_repository() {
        let images = vec![tag("api", "v1", "d1", 1), tag("web", "v1", "d2", 1)];
        let prune = tags_to_prune(&images, 1).unwrap();
        assert!(prune.is_empty());
    }

    #[test]
    fn tags_to_prune_keep_zero_prunes_everything() {
        let images = vec![tag("api", "v1", "d1", 1)];
        let prune = tags_to_prune(&images, 0).unwrap();
        assert_eq!(prune, vec![TagRef { repository: "api".into(), tag: "v1".into() }]);
    }

    #[test]
    fn dedup_ratio_of_all_unique_digests_is_one() {
        let images = vec![tag("api", "v1", "d1", 1), tag("api", "v2", "d2", 2)];
        assert_eq!(dedup_ratio(&images).unwrap(), 1.0);
    }

    #[test]
    fn dedup_ratio_reflects_shared_digest() {
        let images = vec![tag("api", "v1", "d1", 1), tag("api", "latest", "d1", 2)];
        assert_eq!(dedup_ratio(&images).unwrap(), 0.5);
    }

    #[test]
    fn dedup_ratio_of_empty_registry_is_one() {
        assert_eq!(dedup_ratio(&[]).unwrap(), 1.0);
    }
}
