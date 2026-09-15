//! Demo CLI: extract headings from a small document and print its TOC and
//! reading stats.

use documentation_viewer_ui::{build_toc, extract_headings, reading_stats};

const DOC: &str = "# Getting Started\n\nSome intro text.\n\n## Installation\n\nRun the installer.\n\n## Usage\n\n### Basic\n\nDo the thing.\n";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let headings = extract_headings(DOC);
    let toc = build_toc(&headings)?;
    for entry in &toc {
        print_entry(entry, 0);
    }
    let stats = reading_stats(DOC);
    println!("~{} min read ({} words)", stats.reading_minutes, stats.word_count);
    Ok(())
}

fn print_entry(entry: &documentation_viewer_ui::TocEntry, depth: usize) {
    println!("{}{} (#{})", "  ".repeat(depth), entry.heading.text, entry.heading.slug);
    for child in &entry.children {
        print_entry(child, depth + 1);
    }
}
