# Omnisystem Workspace — this app

`Omnisystem/OmniHarness/workspace` — the Tauri desktop application you are
using right now to read this manual.

## Shape of the app

- **Frontend**: Svelte, under `workspace/src/`. The main shell is
  `src/App.svelte`, which composes a large number of feature panels
  (file tree, chat, terminal, sessions, agents, extensions, survival
  system, time travel, and this help manual among them) behind toggle
  buttons and overlay panels.
- **Backend**: Rust, under `workspace/src-tauri/src/`, exposed to the
  frontend via `#[tauri::command]`-annotated functions invoked through
  `@tauri-apps/api/core`'s `invoke()`. A direct count of
  `#[tauri::command]` attributes across `src-tauri/src/` at the time this
  page was written: **652** — a rough measure of how much real
  Tauri-command surface this app exposes, not an exhaustive feature list.

## Two panel-integration patterns, side by side

Reading the existing panels shows two different conventions in active use:

1. **Hand-wired overlay panels** (the older, more common pattern) — a
   boolean `show*` variable in `App.svelte`, a toolbar button that toggles
   it, and an `{#if show*}` block that renders the panel component
   directly (see `AgentsPanel`, `ResourcesPanel`, and most of the panels in
   `src/lib/components/`).
2. **The widget registry** (`src/lib/widgets/registry.ts` +
   `<WidgetHost widgetId="...">`) — the newer, preferred pattern per that
   file's own doc comment. A widget is registered once with an id, title,
   icon, and a dynamic-import thunk; `WidgetHost` handles lazy-loading and
   crash isolation (an import failure or a runtime error thrown after
   mount both fall back to a local "this panel failed" card with a Retry
   button, instead of taking down the whole app shell). `extensions`,
   `self-build`, `survival`, and `model-builder` are migrated to this
   pattern; this manual's own Help panel uses it too.

## Notable real subsystems visible from the panel list

- **Time Travel** (`src/lib/panels/TimeTravelPanel.svelte`) — a real
  timeline/snapshot UI backed by Tauri commands (`get_timeline`,
  `get_snapshots`, `create_snapshot`, `revert_preview_event`,
  `revert_preview_snapshot`). Its own UI is explicit that revert
  *execution* (not just preview) is flagged as a future phase — the panel
  shows a preview of what a revert would change, and labels full execution
  "requires full implementation (Phase 2)" directly in its own markup, a
  good example of the honesty standard this manual is trying to match.
- **Extensions** (`src/lib/panels/ExtensionsPanel.svelte`) — package/
  extension management, migrated to the widget registry pattern.
- **Survival System** — bug discovery/knowledge-base/sandbox monitoring,
  also on the widget registry.

## Markdown rendering already in this app

`src/lib/components/AssistantMessage.svelte` already renders Markdown for
assistant chat messages using `marked` (with a custom code-block renderer
that adds copy buttons) piped through `DOMPurify` for sanitization. This
manual's Help panel reuses that same `marked` + `DOMPurify` pipeline rather
than adding a new Markdown dependency.

## What wasn't independently re-verified for this manual

The exact behavior of most of the 652 Tauri commands was not individually
tested — that count is a structural measurement (a grep for the
`#[tauri::command]` attribute), not a claim that all 652 commands are fully
working end-to-end. Treat specific command behavior as unverified unless
you've exercised it yourself.
