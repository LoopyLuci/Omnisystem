//! End-to-end: extract headings from a realistic multi-section doc, build
//! its TOC, and check reading stats over the whole pipeline.

use documentation_viewer_ui::{build_toc, extract_headings, reading_stats};

const DOC: &str = r#"# API Reference

This guide covers the public API.

## Authentication

Use an API key.

```
curl -H "Authorization: Bearer TOKEN" https://example.com
```

## Endpoints

### List items

Returns all items.

### Create item

Creates a new item.

## Errors

Standard HTTP status codes.
"#;

#[test]
fn full_document_pipeline() {
    let headings = extract_headings(DOC);
    assert_eq!(headings.len(), 6);

    let toc = build_toc(&headings).expect("well-formed heading hierarchy");
    assert_eq!(toc.len(), 1); // single h1 root
    assert_eq!(toc[0].children.len(), 3); // Authentication, Endpoints, Errors

    let endpoints = &toc[0].children[1];
    assert_eq!(endpoints.heading.text, "Endpoints");
    assert_eq!(endpoints.children.len(), 2); // List items, Create item

    let stats = reading_stats(DOC);
    // The curl command inside the fenced block must not count as prose words.
    assert!(stats.word_count < 40);
    assert!(!DOC.is_empty());
}
