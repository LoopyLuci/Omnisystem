import { describe, expect, it } from 'vitest';
import { searchHelpPages } from './search';
import type { ResolvedHelpPage } from './manifest';

function page(id: string, title: string, content: string): ResolvedHelpPage {
  return { id, title, file: `${id}.md`, sectionId: 'test', sectionTitle: 'Test', content };
}

describe('searchHelpPages', () => {
  const pages: ResolvedHelpPage[] = [
    page('titan', 'Titan — systems / core', 'Titan is the systems language. `bootstrap-rs` is real Rust. 16/16 passed.'),
    page('sylva', 'Sylva — Python-like / ML', 'Sylva has real significant whitespace and dynamic typing. 4/4 passed.'),
    page('omnicc', 'OmniCC (real v1)', 'omnicc compiles Titan and Sylva-subset to Rust source.'),
  ];

  it('returns an empty array for an empty query', () => {
    expect(searchHelpPages(pages, '')).toEqual([]);
    expect(searchHelpPages(pages, '   ')).toEqual([]);
  });

  it('matches on title, case-insensitively', () => {
    const results = searchHelpPages(pages, 'TITAN');
    expect(results.length).toBeGreaterThan(0);
    expect(results[0].page.id).toBe('titan');
  });

  it('matches on body content when title does not match', () => {
    const results = searchHelpPages(pages, 'whitespace');
    expect(results).toHaveLength(1);
    expect(results[0].page.id).toBe('sylva');
  });

  it('ranks a title match above a body-only match', () => {
    const results = searchHelpPages(pages, 'sylva');
    // "sylva" appears in the Sylva page's title AND in the OmniCC page's body.
    expect(results.length).toBe(2);
    expect(results[0].page.id).toBe('sylva');
  });

  it('returns no results when nothing matches', () => {
    expect(searchHelpPages(pages, 'nonexistent-term-xyz')).toEqual([]);
  });

  it('produces a snippet containing the matched text', () => {
    const results = searchHelpPages(pages, 'real Rust');
    expect(results).toHaveLength(1);
    expect(results[0].snippet.toLowerCase()).toContain('real rust');
  });
});
