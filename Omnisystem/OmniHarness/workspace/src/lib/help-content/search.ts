/**
 * Simple client-side search over the help manual's Markdown content.
 *
 * Deliberately not fancy: case-insensitive substring matching over each
 * page's title and body, with a small relevance score (title hits rank
 * above body hits) and a plain-text snippet around the first match. This
 * is intentionally a pure function (no DOM, no Svelte) so it's easy to
 * unit-test in isolation from the panel component.
 */

import type { ResolvedHelpPage } from './manifest';

export interface HelpSearchResult {
  page: ResolvedHelpPage;
  score: number;
  snippet: string;
}

const SNIPPET_RADIUS = 60;

function stripMarkdown(text: string): string {
  return text
    .replace(/```[\s\S]*?```/g, ' ')
    .replace(/`([^`]*)`/g, '$1')
    .replace(/^#{1,6}\s+/gm, '')
    .replace(/\*\*([^*]*)\*\*/g, '$1')
    .replace(/\[([^\]]*)\]\([^)]*\)/g, '$1')
    .replace(/\s+/g, ' ')
    .trim();
}

function buildSnippet(plainText: string, queryLower: string): string {
  const idx = plainText.toLowerCase().indexOf(queryLower);
  if (idx === -1) return plainText.slice(0, SNIPPET_RADIUS * 2).trim();
  const start = Math.max(0, idx - SNIPPET_RADIUS);
  const end = Math.min(plainText.length, idx + queryLower.length + SNIPPET_RADIUS);
  const prefix = start > 0 ? '…' : '';
  const suffix = end < plainText.length ? '…' : '';
  return `${prefix}${plainText.slice(start, end).trim()}${suffix}`;
}

/**
 * Search `pages` for `query`, returning matches ordered by relevance
 * (title matches first, then by number of body occurrences).
 * Returns an empty array for an empty/whitespace-only query.
 */
export function searchHelpPages(pages: ResolvedHelpPage[], query: string): HelpSearchResult[] {
  const q = query.trim().toLowerCase();
  if (!q) return [];

  const results: HelpSearchResult[] = [];

  for (const page of pages) {
    const titleLower = page.title.toLowerCase();
    const plainBody = stripMarkdown(page.content);
    const bodyLower = plainBody.toLowerCase();

    const titleHit = titleLower.includes(q);
    let bodyHits = 0;
    let searchFrom = 0;
    for (;;) {
      const found = bodyLower.indexOf(q, searchFrom);
      if (found === -1) break;
      bodyHits += 1;
      searchFrom = found + q.length;
    }

    if (!titleHit && bodyHits === 0) continue;

    const score = (titleHit ? 1000 : 0) + bodyHits;
    const snippet = titleHit && bodyHits === 0
      ? stripMarkdown(page.content).slice(0, SNIPPET_RADIUS * 2).trim()
      : buildSnippet(plainBody, q);

    results.push({ page, score, snippet });
  }

  results.sort((a, b) => b.score - a.score);
  return results;
}
