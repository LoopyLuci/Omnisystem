//! Demo CLI: bake a two-second ease-in-out fade and print sampled frames.

use animation_library::{Easing, Keyframe, Timeline};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let timeline = Timeline::new(vec![
        Keyframe::new(0.0, 0.0, Easing::Linear),
        Keyframe::new(2.0, 1.0, Easing::EaseInOut),
    ])?;
    for (t, v) in timeline.bake(5)? {
        println!("t={t:.2} value={v:.3}");
    }
    Ok(())
}
