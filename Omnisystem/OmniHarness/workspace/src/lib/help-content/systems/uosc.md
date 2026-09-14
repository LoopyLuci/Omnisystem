# UOSC — Universal Operating System Core

## A note on how this page was corrected

An earlier draft of this page said UOSC "does not currently exist in this
repo in a buildable, tested form." That was wrong, and it's worth
explaining why the mistake happened, because the underlying cause is a
real, separate bug worth knowing about.

`Omnisystem/src/systems/UOSC` is tracked in git as a **gitlink**
(submodule-style commit pointer, mode `160000`) with **no `.gitmodules`
file** registering it anywhere in the repo. That means: in the long-lived
checkout this project has actually been developed in, the directory has
real content, checked out once and left in place — but in a **fresh
clone, or any fresh `git worktree add`**, that directory checks out
completely empty, with no error or warning. Two independent passes at
writing this manual were run from fresh worktrees, saw an empty
directory, and reasonably (but incorrectly) concluded the code doesn't
exist. It does — it just doesn't travel with the repo the way normal
tracked files do. That's a real packaging bug in this repo, separate from
UOSC's own content, and it should be fixed by registering a proper
`.gitmodules` entry.

## What's actually there, verified directly, right now

Re-run in this session, in the checkout that has the content:

```
cd Omnisystem/src/systems/UOSC/reference-rs
cargo test --release
```
&rarr; **64 passed, 0 failed, 0 ignored.**

`Omnisystem/src/systems/UOSC/` is itself a real, independent nested git
repository (its own `README.md`, `CONTRIBUTING.md`, `LICENSE`, and
directories: `docs/`, `drivers/`, `future-work/`, `hypercalls/`,
`kernel/`, `kernel-x86_64/`, `kernel-x86_64-builder/`, plus the
`reference-rs/` crate the test above lives in). Its own README is
unusually candid for this project — it explicitly separates what's real
and verified from what's still specification, and documents a prior
version of itself that made unbacked "production ready" claims it no
longer stands behind. That real README is the one to trust for this
subsystem, not the one described below.

## A decoy document — do not confuse this with the real README

`Omnisystem/src/systems/desktop/docs/production-docs/UOSC_README.md` is a
**different file**, unrelated to the nested repo above, that describes an
aspirational UOSC: badges for "build: passing" and "Formal Verification"
with nothing behind them, a claim of "~10,000 LOC of core kernel," and a
`git clone` URL pointing at the placeholder `github.com/your-org/uosc`. It
was never filled in and doesn't reflect anything real. If you find it
while exploring the repo, treat it the same way this whole manual treats
every other unverified "complete" doc: as a claim to check, not a fact.

## Also worth knowing about

- `Omnisystem/_Archive/src-crates-dead/uosc-core/` — a separate, earlier
  Rust crate attempt (capability/core/hypervisor/ipc/memory/process/
  scheduler/security modules) that predates the current
  `src/systems/UOSC/reference-rs` work and was archived as dead/unwired.
  It's not the current implementation; the real one is the nested repo
  above.
- `src/compiler/languages/titan/uosc/uosc-core/` — a `module.ti` stub
  under the inert Titan frontend tree (see the Languages Overview page
  for why that tree doesn't execute) — also not the real implementation.

## Honest summary

UOSC's real, working microkernel code exists, is tested, and passes
64/64 real tests in this checkout today. The catch isn't that it's fake
— it's that it currently can't be reliably obtained by cloning this
repository fresh, because of a missing `.gitmodules` registration. That's
a real, fixable infrastructure gap, not a false claim about the code
itself, and it's worth fixing so the next person who clones this repo
doesn't lose 64 passing tests' worth of real kernel work without so much
as an error message.
