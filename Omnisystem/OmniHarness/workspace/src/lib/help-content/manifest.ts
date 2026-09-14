/**
 * Help manual manifest — the sidebar tree for the in-app manual.
 *
 * Content itself lives as plain `.md` files in this directory (and its
 * `languages/`/`systems/` subdirectories), loaded as raw text at build time
 * via `import.meta.glob(..., { query: '?raw', import: 'default', eager: true })`.
 * Editing a manual page means editing the `.md` file directly — this file
 * only defines the navigation tree (id, title, which file backs it) and
 * page ordering. See `00-overview.md` for the manual's own honesty policy.
 */

const rawModules = import.meta.glob('./**/*.md', {
  query: '?raw',
  import: 'default',
  eager: true,
}) as Record<string, string>;

function loadRaw(relativePath: string): string {
  const key = `./${relativePath}`;
  const content = rawModules[key];
  if (content === undefined) {
    return `# Missing page\n\nExpected help content at \`${relativePath}\` but it was not found at build time.`;
  }
  return content;
}

export interface HelpPage {
  id: string;
  title: string;
  file: string;
}

export interface HelpSection {
  id: string;
  title: string;
  icon: string;
  pages: HelpPage[];
}

export const helpSections: HelpSection[] = [
  {
    id: 'start',
    title: 'Start Here',
    icon: '📖',
    pages: [{ id: 'overview', title: 'Manual overview', file: '00-overview.md' }],
  },
  {
    id: 'languages',
    title: 'The 7 Omni-Languages',
    icon: '🧬',
    pages: [
      { id: 'languages-overview', title: 'Languages overview', file: '01-languages-overview.md' },
      { id: 'titan', title: 'Titan — systems / core', file: 'languages/titan.md' },
      { id: 'sylva', title: 'Sylva — Python-like / ML', file: 'languages/sylva.md' },
      { id: 'aether', title: 'Aether — databases / distributed', file: 'languages/aether.md' },
      { id: 'vera', title: 'Vera — UI', file: 'languages/vera.md' },
      { id: 'nexus', title: 'Nexus — layout', file: 'languages/nexus.md' },
      { id: 'helix', title: 'Helix — GPU / compute', file: 'languages/helix.md' },
      { id: 'axiom', title: 'Axiom — formal verification', file: 'languages/axiom.md' },
    ],
  },
  {
    id: 'omnicc',
    title: 'Cross-Language Tooling',
    icon: '🔀',
    pages: [{ id: 'omnicc', title: 'OmniCC (real v1)', file: '02-omnicc.md' }],
  },
  {
    id: 'systems',
    title: 'Core Systems',
    icon: '🧱',
    pages: [
      { id: 'kernel', title: 'Kernel (gRPC substrate)', file: 'systems/kernel.md' },
      { id: 'workspace', title: 'Workspace (this app)', file: 'systems/workspace.md' },
      { id: 'orchestrator', title: 'Orchestrators (Python + Clojure)', file: 'systems/orchestrator.md' },
      { id: 'vscode-extension', title: 'VS Code Extension', file: 'systems/vscode-extension.md' },
      { id: 'uosc', title: 'UOSC microkernel', file: 'systems/uosc.md' },
      { id: 'crates', title: 'Crate ecosystem', file: 'systems/crates.md' },
    ],
  },
];

export interface ResolvedHelpPage extends HelpPage {
  sectionId: string;
  sectionTitle: string;
  content: string;
}

/** Flat list of every page with its Markdown content loaded, for rendering and search. */
export const allHelpPages: ResolvedHelpPage[] = helpSections.flatMap((section) =>
  section.pages.map((page) => ({
    ...page,
    sectionId: section.id,
    sectionTitle: section.title,
    content: loadRaw(page.file),
  })),
);

export function getHelpPage(pageId: string): ResolvedHelpPage | undefined {
  return allHelpPages.find((p) => p.id === pageId);
}
