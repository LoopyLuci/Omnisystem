//! CLI for dockerfile-optimizer.

use dockerfile_optimizer::{analyze, parse};

const SAMPLE: &str = "FROM golang:latest\nRUN go build -o app .\nRUN chmod +x app\nADD app.tar.gz /app\nCMD [\"./app\"]\n";

fn main() -> dockerfile_optimizer::Result<()> {
    let instructions = parse(SAMPLE)?;
    let suggestions = analyze(&instructions);

    println!("{} instruction(s) parsed, {} suggestion(s):", instructions.len(), suggestions.len());
    for s in suggestions {
        println!("  {:?}", s);
    }
    Ok(())
}
