//! Demo CLI: prune all but the 2 most recent tags in a repository.

use image_management_ui::{tags_to_prune, ImageTag};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let images = vec![
        ImageTag { repository: "api".into(), tag: "v1".into(), digest: "sha256:1".into(), pushed_day: 1 },
        ImageTag { repository: "api".into(), tag: "v2".into(), digest: "sha256:2".into(), pushed_day: 2 },
        ImageTag { repository: "api".into(), tag: "v3".into(), digest: "sha256:3".into(), pushed_day: 3 },
    ];
    for r in tags_to_prune(&images, 2)? {
        println!("prune {}:{}", r.repository, r.tag);
    }
    Ok(())
}
