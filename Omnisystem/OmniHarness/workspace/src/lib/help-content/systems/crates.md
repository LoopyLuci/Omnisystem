# The Cargo crate ecosystem

Omnisystem has real Rust code spread across two separate crate trees, plus
several standalone workspaces. This page is an orientation, not an
exhaustive per-crate listing — that would go stale immediately and isn't
maintainable by hand.

## Two crate trees, counted directly (this session)

- **`OmniHarness/crates/`** — **41** directories containing a
  `Cargo.toml` at the time this page was written. Includes: `actors`,
  `capability-registry`, `cas` (content-addressed storage), `chess`,
  `coordinator`, `crdt`, `credits`, `extensions`, `fabric`,
  `failure-finder`, `hnsw` (vector search), `ir` (the real OmniCC v1 — see
  its own page), `knowledge`, `mailbox`, `marketplace`, `mcp-server`,
  `omnisystem-core`, `p2p-core`/`p2p-crypto`, `package`, `profiler`,
  `sandbox`, `skill-compiler`, `sns`, `swarm`, `universe` (backs Time
  Travel — see the Workspace page), `verify`, and more.
- **`src/crates/`** — **336** directories containing a `Cargo.toml`. A
  much larger, older tree.
- Standalone workspaces outside both trees: `kernel` and `mcp-server`
  each declare their own `[workspace]` table (per commit `107177d7a`'s own
  investigation), so they're intentionally excluded from the root
  workspace rather than accidentally missing.

## How crates get included in builds

The root `Cargo.toml`'s `[workspace] members` list is the actual source of
truth for what builds as part of the main workspace — a crate directory
existing on disk does not by itself mean it's built or tested by default.
Recent history includes real fixes to this list: commit `107177d7a` found
`omnisystem-core` and `universe` were real, compiling crates that had been
left out of `members` by oversight, and added them; `a59a24df3` "rewired
53 crates' dead `cli.rs` binaries to their real `lib.rs` APIs" — evidence
that "present in the tree" and "actually wired up correctly" have been two
different questions here before, worth checking independently if you're
relying on a specific crate.

## How to explore further yourself

- `cargo metadata --no-deps` from the repo root lists every workspace
  member Cargo actually knows about — the ground truth for "is this crate
  really part of the build."
- `cargo build -p <crate-name>` (or `cargo check -p <crate-name>`) builds
  one crate in isolation without pulling in the whole workspace.
- A crate under `_Archive/` or with `-dead` in its path (see the UOSC page
  for one example) is, by this repo's own convention, not part of the
  active build — check `members` before assuming otherwise.

## What wasn't independently re-verified for this manual

The counts above (41 / 336) are directory counts from a `Cargo.toml`
presence check, not a confirmation that every one of those ~377 crates
currently builds cleanly — that would require a full workspace build,
which was not run for this manual (the 7 language `bootstrap*-rs` crates
were built individually and are covered on their own pages). Treat "this
crate builds" as unverified for any specific crate not covered elsewhere
in this manual until you build it yourself.
