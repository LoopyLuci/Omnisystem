//! Markdown document analysis: heading extraction, nested table-of-contents
//! construction, and reading-time estimation. Deliberately a plain-text
//! parser, not a full CommonMark implementation — this crate extracts the
//! structure a docs viewer needs to build navigation, not a render tree.

#![warn(missing_docs)]

pub mod error;
pub mod types;

pub use error::{Error, Result};
pub use types::*;

/// Extract every ATX-style heading (`# `..`###### `) from `markdown`,
/// skipping lines inside fenced code blocks (delimited by ` ``` `).
pub fn extract_headings(markdown: &str) -> Vec<Heading> {
    let mut headings = Vec::new();
    let mut in_code_block = false;
    let mut used_slugs: std::collections::HashMap<String, usize> = std::collections::HashMap::new();

    for (idx, raw_line) in markdown.lines().enumerate() {
        let line = raw_line.trim_end();
        if line.trim_start().starts_with("```") {
            in_code_block = !in_code_block;
            continue;
        }
        if in_code_block {
            continue;
        }
        let trimmed = line.trim_start();
        let level = trimmed.chars().take_while(|&c| c == '#').count();
        if level == 0 || level > 6 {
            continue;
        }
        let rest = &trimmed[level..];
        if !rest.starts_with(' ') && !rest.is_empty() {
            continue; // e.g. "#nope" is not a heading
        }
        let text = strip_emphasis(rest.trim());
        if text.is_empty() {
            continue;
        }
        let base_slug = slugify(&text);
        let count = used_slugs.entry(base_slug.clone()).or_insert(0);
        let slug = if *count == 0 { base_slug.clone() } else { format!("{}-{}", base_slug, count) };
        *count += 1;

        headings.push(Heading { level: level as u8, text, slug, line: idx + 1 });
    }
    headings
}

fn strip_emphasis(text: &str) -> String {
    text.chars().filter(|&c| c != '*' && c != '_' && c != '`').collect::<String>().trim().to_string()
}

fn slugify(text: &str) -> String {
    let mut slug = String::new();
    let mut last_was_dash = false;
    for c in text.to_lowercase().chars() {
        if c.is_alphanumeric() {
            slug.push(c);
            last_was_dash = false;
        } else if !last_was_dash && !slug.is_empty() {
            slug.push('-');
            last_was_dash = true;
        }
    }
    while slug.ends_with('-') {
        slug.pop();
    }
    slug
}

/// Build a nested table of contents from a flat heading list, validating
/// that no heading skips more than one level below its parent.
pub fn build_toc(headings: &[Heading]) -> Result<Vec<TocEntry>> {
    let mut roots: Vec<TocEntry> = Vec::new();
    build_toc_recursive(headings, 0, 0, &mut roots)?;
    Ok(roots)
}

fn build_toc_recursive(headings: &[Heading], mut idx: usize, parent_level: u8, out: &mut Vec<TocEntry>) -> Result<usize> {
    while idx < headings.len() {
        let h = &headings[idx];
        if h.level <= parent_level {
            return Ok(idx);
        }
        if h.level > parent_level + 1 && parent_level != 0 {
            return Err(Error::SkippedHeadingLevel { line: h.line, from: parent_level, to: h.level });
        }
        let mut children = Vec::new();
        idx = build_toc_recursive(headings, idx + 1, h.level, &mut children)?;
        out.push(TocEntry { heading: h.clone(), children });
    }
    Ok(idx)
}

/// Compute word count and estimated reading time for `markdown`, excluding
/// fenced code block contents from the word count.
pub fn reading_stats(markdown: &str) -> ReadingStats {
    let mut in_code_block = false;
    let mut word_count = 0usize;
    for line in markdown.lines() {
        if line.trim_start().starts_with("```") {
            in_code_block = !in_code_block;
            continue;
        }
        if in_code_block {
            continue;
        }
        word_count += line.split_whitespace().count();
    }
    const WORDS_PER_MINUTE: usize = 200;
    let reading_minutes = word_count.div_ceil(WORDS_PER_MINUTE) as u32;
    ReadingStats { word_count, reading_minutes }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_headings_at_various_levels() {
        let md = "# Title\n\n## Section\n\n### Sub\n";
        let headings = extract_headings(md);
        assert_eq!(headings.len(), 3);
        assert_eq!(headings[0].level, 1);
        assert_eq!(headings[1].level, 2);
        assert_eq!(headings[2].level, 3);
    }

    #[test]
    fn ignores_headings_inside_fenced_code_blocks() {
        let md = "# Real\n```\n# Not a heading\n```\n## Also Real";
        let headings = extract_headings(md);
        let texts: Vec<&str> = headings.iter().map(|h| h.text.as_str()).collect();
        assert_eq!(texts, vec!["Real", "Also Real"]);
    }

    #[test]
    fn slugs_are_url_safe() {
        let md = "# Hello, World! (v2.0)";
        let headings = extract_headings(md);
        assert_eq!(headings[0].slug, "hello-world-v2-0");
    }

    #[test]
    fn duplicate_headings_get_disambiguated_slugs() {
        let md = "# Overview\n## Details\n# Overview";
        let headings = extract_headings(md);
        assert_eq!(headings[0].slug, "overview");
        assert_eq!(headings[2].slug, "overview-1");
    }

    #[test]
    fn strips_emphasis_markers_from_heading_text() {
        let md = "## **Bold** and _italic_ and `code`";
        let headings = extract_headings(md);
        assert_eq!(headings[0].text, "Bold and italic and code");
    }

    #[test]
    fn hash_without_space_is_not_a_heading() {
        let md = "#nope\n# Real";
        let headings = extract_headings(md);
        assert_eq!(headings.len(), 1);
        assert_eq!(headings[0].text, "Real");
    }

    #[test]
    fn toc_nests_children_under_parents() {
        let md = "# A\n## A.1\n## A.2\n### A.2.1\n# B";
        let headings = extract_headings(md);
        let toc = build_toc(&headings).unwrap();
        assert_eq!(toc.len(), 2); // A, B
        assert_eq!(toc[0].children.len(), 2); // A.1, A.2
        assert_eq!(toc[0].children[1].children.len(), 1); // A.2.1
    }

    #[test]
    fn toc_rejects_a_skipped_level_within_a_subtree() {
        let md = "# A\n## A.1\n#### Too Deep";
        let headings = extract_headings(md);
        let result = build_toc(&headings);
        assert!(matches!(result, Err(Error::SkippedHeadingLevel { .. })));
    }

    #[test]
    fn reading_stats_counts_words_excluding_code() {
        let md = "one two three\n```\nfour five six seven\n```\neight";
        let stats = reading_stats(md);
        assert_eq!(stats.word_count, 4); // "one two three" + "eight"
    }

    #[test]
    fn reading_stats_rounds_minutes_up() {
        let words = vec!["word"; 201].join(" ");
        let stats = reading_stats(&words);
        assert_eq!(stats.reading_minutes, 2); // 201 words at 200wpm rounds up to 2
    }

    #[test]
    fn empty_document_has_zero_reading_time() {
        let stats = reading_stats("");
        assert_eq!(stats.word_count, 0);
        assert_eq!(stats.reading_minutes, 0);
    }
}
