# UOSC — Universal Operating System Core

## A note before anything else

This manual was asked to point at UOSC's own README as "an unusually honest
model to follow for tone." Having read it directly
(`src/systems/desktop/docs/production-docs/UOSC_README.md`), that's not
actually the document this page can recommend as a model — it's written in
the same grandiose register this manual is explicitly trying to avoid: a
"build: passing" badge with no CI behind it, a "Formal Verification"
badge, a claim of "~10,000 LOC of core kernel," and a `git clone` URL
pointing at `github.com/your-org/uosc` — a placeholder that was never
filled in. Rather than transcribe those claims, this page reports what's
actually on disk.

## What's actually on disk

- **`src/systems/UOSC/`** — the directory this manual was originally
  pointed at — is empty and untracked; there is no code there.
- **`_Archive/src-crates-dead/uosc-core/`** — a real Rust crate (Cargo.toml
  + `src/` with `capability.rs`, `core.rs`, `hypervisor.rs`, `ipc.rs`,
  `memory.rs`, `process.rs`, `scheduler.rs`, `security.rs`, `types.rs`,
  `error.rs`) — but it lives under this repo's own `_Archive/` /
  `-dead` naming convention, which this codebase uses to mark code that
  isn't part of the active, maintained build. Its presence confirms real
  design work happened here at some point; its location confirms it is not
  currently wired into anything.
- **`src/compiler/languages/titan/uosc/uosc-core/`** — a `module.ti`
  (Titan-language) stub plus a test file, under the *inert*
  `src/compiler/frontend`-adjacent tree that the language capability map
  itself warns is a parallel, non-executing effort (see the Languages
  Overview page) — not the real, runnable Titan substrate.

## Honest summary

UOSC as a *working* microkernel does not currently exist in this repo in a
buildable, tested form. What exists is: an aspirational README with
unverifiable claims and a placeholder repo URL, an archived (dead,
unwired) Rust crate with real module-level design work, and a stub in the
inert Titan frontend tree. If you're looking for UOSC to actually boot,
verify a capability proof, or run under QEMU as its README's Quick Start
section describes — none of that was reproducible from this repo as
checked out for this manual.

This is exactly the kind of gap this whole manual was written to be honest
about, rather than repeat.
