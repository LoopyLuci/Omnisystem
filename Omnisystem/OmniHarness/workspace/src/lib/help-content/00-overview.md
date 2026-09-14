# Omnisystem Workspace — Manual

This is the in-app manual for **Omnisystem Workspace**, the Tauri desktop
shell you're running right now. It covers the 7 Omni-Languages, the real
cross-language tooling that exists today, and the other subsystems that make
up the Omnisystem project.

## Read this first: how this manual was written

Omnisystem's history includes a well-documented pattern of status documents
that claimed things were "complete" or "production-ready" when the
underlying code didn't back that up. This manual was written specifically to
avoid repeating that mistake. Every concrete claim in it — every test count,
every "this works" / "this doesn't" line — was checked against real code or
a real command run in the same session this manual was written in, not
copied from an older status document. Where something could not be verified
directly, it's marked **unverified** instead of stated as fact.

You can (and should) be skeptical of documentation in this codebase,
including this manual. If something here turns out to be wrong, the fix is
to correct the `.md` file it lives in — see [How this manual is
built](#how-this-manual-is-built) below.

## What's in here

- **The 7 Omni-Languages** — Titan, Sylva, Aether, Vera, Nexus, Helix, Axiom.
  Each language page states what actually runs today, with real example
  programs pulled from that language's own test fixtures, a real,
  freshly-run test count, and an explicit "not implemented" list.
- **OmniCC** — the real, narrow Titan/Sylva-subset → Rust converter in
  `OmniHarness/crates/ir`. Not to be confused with two other, unrelated
  things that also carry the "OmniCC" name elsewhere in this repo.
- **Core systems** — the gRPC kernel, this Workspace app itself, the Python
  and Clojure orchestrators, the VS Code extension, the UOSC microkernel
  project, and an orientation to the Cargo crate ecosystem.

## How this manual is built

Content lives as plain Markdown files under
`OmniHarness/workspace/src/lib/help-content/`, one file per section. The
`HelpPanel` component loads them, renders them with the same Markdown
pipeline already used for assistant chat messages, and provides simple
substring search across every section's text. Editing a fact in this manual
means editing the relevant `.md` file directly — nothing here is generated.

## A note on "verified"

"Verified" in this manual means: a build was run, a test suite was run, or a
source file was read directly, in the same session that wrote the page,
specifically to check the claim being made. It does not mean formally
proven, exhaustively fuzzed, or reviewed by anyone other than whoever wrote
this page. Treat every number here as "true as of the commit this was
written against" — re-run the underlying command yourself if it matters for
what you're doing.
