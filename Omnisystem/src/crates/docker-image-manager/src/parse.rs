//! Image reference parsing: `[registry/]repository[:tag][@digest]`.

use crate::error::{Error, Result};
use crate::types::ImageReference;

/// Parse a Docker image reference string.
///
/// Rules (matching real Docker reference-parsing behavior):
/// - A leading component before the first `/` is treated as the registry
///   only if it contains a `.` or `:`, or is exactly `localhost` -- this
///   disambiguates `myorg/myapp` (no registry, default `docker.io`) from
///   `registry.example.com/myapp` or `localhost:5000/myapp`.
/// - A digest, if present, is introduced by `@` and always comes last.
/// - A tag, if present, is introduced by the last `:` that appears after
///   the last `/` (so a registry port like `localhost:5000` is never
///   mistaken for a tag separator).
/// - If no tag and no digest are given, the tag defaults to `latest`.
pub fn parse(s: &str) -> Result<ImageReference> {
    if s.is_empty() {
        return Err(Error::InvalidReference(s.to_string()));
    }

    let (main, digest) = match s.find('@') {
        Some(idx) => (&s[..idx], Some(s[idx + 1..].to_string())),
        None => (s, None),
    };
    if main.is_empty() {
        return Err(Error::InvalidReference(s.to_string()));
    }

    let (registry, rest) = match main.split_once('/') {
        Some((first, remainder)) if first.contains('.') || first.contains(':') || first == "localhost" => {
            (first.to_string(), remainder)
        }
        _ => ("docker.io".to_string(), main),
    };
    if rest.is_empty() {
        return Err(Error::InvalidReference(s.to_string()));
    }

    let last_slash = rest.rfind('/');
    let (repository, tag) = match rest.rfind(':') {
        Some(idx) if last_slash.is_none() || idx > last_slash.unwrap() => {
            (rest[..idx].to_string(), Some(rest[idx + 1..].to_string()))
        }
        _ => (rest.to_string(), None),
    };
    if repository.is_empty() {
        return Err(Error::InvalidReference(s.to_string()));
    }

    let tag = if digest.is_some() { tag } else { Some(tag.unwrap_or_else(|| "latest".to_string())) };

    Ok(ImageReference { registry, repository, tag, digest })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bare_name_defaults_registry_and_tag() {
        let r = parse("nginx").unwrap();
        assert_eq!(r.registry, "docker.io");
        assert_eq!(r.repository, "nginx");
        assert_eq!(r.tag.as_deref(), Some("latest"));
        assert_eq!(r.digest, None);
    }

    #[test]
    fn org_and_repo_with_explicit_tag() {
        let r = parse("myorg/myapp:1.2.3").unwrap();
        assert_eq!(r.registry, "docker.io");
        assert_eq!(r.repository, "myorg/myapp");
        assert_eq!(r.tag.as_deref(), Some("1.2.3"));
    }

    #[test]
    fn registry_with_port_is_not_confused_with_a_tag() {
        let r = parse("localhost:5000/myapp").unwrap();
        assert_eq!(r.registry, "localhost:5000");
        assert_eq!(r.repository, "myapp");
        assert_eq!(r.tag.as_deref(), Some("latest"));
    }

    #[test]
    fn dotted_registry_host_with_tag() {
        let r = parse("registry.example.com/team/app:v2").unwrap();
        assert_eq!(r.registry, "registry.example.com");
        assert_eq!(r.repository, "team/app");
        assert_eq!(r.tag.as_deref(), Some("v2"));
    }

    #[test]
    fn digest_pinned_reference_has_no_default_tag() {
        let r = parse("nginx@sha256:abc123").unwrap();
        assert_eq!(r.repository, "nginx");
        assert_eq!(r.tag, None);
        assert_eq!(r.digest.as_deref(), Some("sha256:abc123"));
    }

    #[test]
    fn tag_and_digest_together() {
        let r = parse("nginx:1.25@sha256:abc123").unwrap();
        assert_eq!(r.tag.as_deref(), Some("1.25"));
        assert_eq!(r.digest.as_deref(), Some("sha256:abc123"));
    }

    #[test]
    fn empty_string_is_invalid() {
        assert!(parse("").is_err());
    }

    #[test]
    fn registry_only_with_trailing_slash_is_invalid() {
        assert!(parse("registry.example.com/").is_err());
    }
}
