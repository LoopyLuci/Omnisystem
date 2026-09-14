<script lang="ts">
  import { marked, Renderer } from 'marked';
  import DOMPurify from 'dompurify';
  import { helpSections, allHelpPages, type ResolvedHelpPage } from '$lib/help-content/manifest';
  import { searchHelpPages, type HelpSearchResult } from '$lib/help-content/search';

  let activePageId = allHelpPages[0]?.id ?? '';
  let query = '';
  let collapsedSections = new Set<string>();

  $: activePage = allHelpPages.find((p) => p.id === activePageId) ?? allHelpPages[0];
  $: searchResults = searchHelpPages(allHelpPages, query);
  $: showingSearch = query.trim().length > 0;

  function openPage(page: ResolvedHelpPage | { id: string }) {
    activePageId = page.id;
    query = '';
  }

  function toggleSection(sectionId: string) {
    if (collapsedSections.has(sectionId)) {
      collapsedSections.delete(sectionId);
    } else {
      collapsedSections.add(sectionId);
    }
    collapsedSections = collapsedSections; // trigger reactivity
  }

  // Same Markdown pipeline used for assistant chat messages
  // (src/lib/components/AssistantMessage.svelte): marked + DOMPurify.
  const renderer = new Renderer();
  function escapeHtml(s: string): string {
    return s.replace(/&/g, '&amp;').replace(/"/g, '&quot;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
  }
  (renderer as any).code = ({ text, lang }: { text: string; lang?: string }) => {
    const safeLang = lang ? escapeHtml(lang) : '';
    const langClass = safeLang ? ` class="language-${safeLang}"` : '';
    return `<pre><code${langClass}>${escapeHtml(text)}</code></pre>`;
  };
  marked.use({ renderer });

  const PURIFY_CONFIG = {
    ALLOWED_TAGS: [
      'p', 'br', 'strong', 'em', 'code', 'pre', 'ul', 'ol', 'li',
      'blockquote', 'table', 'thead', 'tbody', 'tr', 'th', 'td',
      'a', 'h1', 'h2', 'h3', 'h4', 'div', 'span', 'hr',
    ] as string[],
    ALLOWED_ATTR: ['href', 'class', 'target', 'rel', 'title'] as string[],
    FORCE_BODY: true,
  };

  function renderMarkdown(content: string): string {
    const raw = marked.parse(content) as string;
    return DOMPurify.sanitize(raw, PURIFY_CONFIG) as string;
  }

  $: html = activePage ? renderMarkdown(activePage.content) : '';

  function highlight(text: string, q: string): string {
    if (!q.trim()) return escapeHtml(text);
    const escaped = escapeHtml(text);
    const escapedQuery = escapeHtml(q.trim()).replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
    return escaped.replace(new RegExp(`(${escapedQuery})`, 'ig'), '<mark>$1</mark>');
  }
</script>

<div class="help-panel">
  <aside class="help-sidebar">
    <div class="help-search">
      <input
        type="text"
        placeholder="Search the manual…"
        bind:value={query}
        aria-label="Search help manual"
      />
    </div>

    {#if showingSearch}
      <div class="help-search-results">
        {#if searchResults.length === 0}
          <div class="help-empty">No matches for "{query}".</div>
        {:else}
          {#each searchResults as result (result.page.id)}
            <button
              class="help-search-result"
              class:active={activePageId === result.page.id}
              on:click={() => openPage(result.page)}
            >
              <div class="result-title">{@html highlight(result.page.title, query)}</div>
              <div class="result-section">{result.page.sectionTitle}</div>
              <div class="result-snippet">{@html highlight(result.snippet, query)}</div>
            </button>
          {/each}
        {/if}
      </div>
    {:else}
      <nav class="help-tree">
        {#each helpSections as section (section.id)}
          <div class="help-section">
            <button
              class="section-header"
              on:click={() => toggleSection(section.id)}
              aria-expanded={!collapsedSections.has(section.id)}
            >
              <span class="section-icon">{section.icon}</span>
              <span class="section-title">{section.title}</span>
              <span class="section-caret">{collapsedSections.has(section.id) ? '▸' : '▾'}</span>
            </button>
            {#if !collapsedSections.has(section.id)}
              <div class="section-pages">
                {#each section.pages as page (page.id)}
                  <button
                    class="page-link"
                    class:active={activePageId === page.id}
                    on:click={() => openPage(page)}
                  >
                    {page.title}
                  </button>
                {/each}
              </div>
            {/if}
          </div>
        {/each}
      </nav>
    {/if}
  </aside>

  <div class="help-content">
    {#if activePage}
      <article class="help-article">
        {@html html}
      </article>
    {:else}
      <div class="help-empty">No page selected.</div>
    {/if}
  </div>
</div>

<style>
  .help-panel {
    display: flex;
    height: 100%;
    background: #0f0f1a;
    color: #e2e8f0;
    font-size: 13px;
  }
  .help-sidebar {
    width: 280px;
    flex-shrink: 0;
    display: flex;
    flex-direction: column;
    border-right: 1px solid #2d2d3f;
    overflow-y: auto;
  }
  .help-search {
    padding: 10px 12px;
    border-bottom: 1px solid #1e1e35;
  }
  .help-search input {
    width: 100%;
    background: #1a1a2e;
    border: 1px solid #2d2d3f;
    color: #e2e8f0;
    border-radius: 6px;
    padding: 6px 10px;
    font-size: 12px;
    box-sizing: border-box;
  }
  .help-tree { padding: 6px 0; }
  .help-section { margin-bottom: 2px; }
  .section-header {
    display: flex;
    align-items: center;
    gap: 6px;
    width: 100%;
    background: none;
    border: none;
    color: #cbd5e1;
    padding: 6px 12px;
    cursor: pointer;
    font-size: 12px;
    font-weight: 600;
    text-align: left;
  }
  .section-header:hover { background: #1a1a2e; }
  .section-icon { font-size: 13px; }
  .section-title { flex: 1; }
  .section-caret { color: #64748b; font-size: 10px; }
  .section-pages { display: flex; flex-direction: column; padding-left: 8px; }
  .page-link {
    background: none;
    border: none;
    border-left: 2px solid transparent;
    color: #94a3b8;
    text-align: left;
    padding: 5px 12px 5px 22px;
    cursor: pointer;
    font-size: 12px;
    border-radius: 0 6px 6px 0;
  }
  .page-link:hover { background: #1a1a2e; color: #e2e8f0; }
  .page-link.active {
    background: #1e1e35;
    color: #c9b8ff;
    border-left-color: #c9b8ff;
  }
  .help-search-results { padding: 6px; display: flex; flex-direction: column; gap: 4px; }
  .help-search-result {
    background: #1a1a2e;
    border: 1px solid #2d2d3f;
    border-radius: 8px;
    padding: 8px 10px;
    text-align: left;
    cursor: pointer;
    color: #e2e8f0;
  }
  .help-search-result:hover { background: #1e1e35; }
  .help-search-result.active { border-color: #c9b8ff; }
  .result-title { font-weight: 600; font-size: 12px; margin-bottom: 2px; }
  .result-section { font-size: 10px; color: #64748b; margin-bottom: 4px; }
  .result-snippet { font-size: 11px; color: #94a3b8; line-height: 1.4; }
  .help-empty { color: #4a5568; text-align: center; padding: 32px 16px; font-size: 12px; }

  .help-content { flex: 1; overflow-y: auto; }
  .help-article {
    max-width: 780px;
    margin: 0 auto;
    padding: 28px 32px 60px;
    line-height: 1.6;
  }
  .help-article :global(h1) {
    font-size: 22px;
    margin: 0 0 16px;
    color: #f1f5f9;
    border-bottom: 1px solid #2d2d3f;
    padding-bottom: 10px;
  }
  .help-article :global(h2) {
    font-size: 16px;
    margin: 28px 0 10px;
    color: #e2e8f0;
  }
  .help-article :global(h3) {
    font-size: 14px;
    margin: 20px 0 8px;
    color: #cbd5e1;
  }
  .help-article :global(p) { margin: 0 0 12px; color: #cbd5e1; }
  .help-article :global(ul), .help-article :global(ol) {
    margin: 0 0 12px;
    padding-left: 22px;
    color: #cbd5e1;
  }
  .help-article :global(li) { margin-bottom: 4px; }
  .help-article :global(code) {
    background: #1a1a2e;
    border: 1px solid #2d2d3f;
    border-radius: 4px;
    padding: 1px 5px;
    font-size: 12px;
    color: #c9b8ff;
  }
  .help-article :global(pre) {
    background: #12122a;
    border: 1px solid #2d2d3f;
    border-radius: 8px;
    padding: 12px 14px;
    overflow-x: auto;
    margin: 0 0 14px;
  }
  .help-article :global(pre code) {
    background: none;
    border: none;
    padding: 0;
    color: #e2e8f0;
  }
  .help-article :global(table) {
    border-collapse: collapse;
    width: 100%;
    margin: 0 0 14px;
    font-size: 12px;
  }
  .help-article :global(th), .help-article :global(td) {
    border: 1px solid #2d2d3f;
    padding: 6px 10px;
    text-align: left;
  }
  .help-article :global(th) { background: #1a1a2e; color: #e2e8f0; }
  .help-article :global(blockquote) {
    border-left: 3px solid #4c1d95;
    margin: 0 0 14px;
    padding: 4px 14px;
    color: #a1a1aa;
    background: #16162a;
  }
  .help-article :global(a) { color: #93c5fd; }
  .help-article :global(mark) {
    background: #4c1d95;
    color: #e9d5ff;
    border-radius: 3px;
    padding: 0 2px;
  }
</style>
