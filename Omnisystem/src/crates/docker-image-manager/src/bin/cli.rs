//! CLI for docker-image-manager.

use docker_image_manager::{Layer, Manager};

fn main() -> docker_image_manager::Result<()> {
    let manager = Manager::new();
    let image = manager.push(
        "myorg/myapp:1.0",
        vec![
            Layer { digest: "sha256:base".to_string(), size_bytes: 25_000_000 },
            Layer { digest: "sha256:app".to_string(), size_bytes: 4_500_000 },
        ],
    )?;
    println!("pushed {} ({} bytes)", image.reference, image.total_size_bytes());

    let resolved = manager.resolve_tag("docker.io", "myorg/myapp", "1.0")?;
    println!("resolved: {} layers, {} bytes total", resolved.layers.len(), resolved.total_size_bytes());

    Ok(())
}
