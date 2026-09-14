# VS Code Extension

`Omnisystem/vscode-omnisystem` — published as `omnisystem`, display name
"Omnisystem — 7-Language IDE", version `2.11.0` at the time this page was
written (from its own `package.json`).

## What it declares itself to be

Per its own `package.json` description: "Complete IDE for Omnisystem:
TITAN, VERA, HELIX, AETHER, AXIOM, SYLVA, NEXUS — with OmniOS, OmniPM,
Omnisystem Ecosystem, visual tooling, and cross-language debugging."

That's the extension's self-description, not an independently re-verified
claim — see the language pages in this manual for what's actually real and
tested per language before assuming "Complete IDE" support means every
language has equal depth of tooling.

## Real, structural facts (counted directly from `package.json`)

- **217** registered VS Code commands (`contributes.commands` entries) —
  a rough measure of the extension's real command surface.
- Includes an **OmniHarness AI** chat view contribution (icon +
  contextual title "OmniHarness — AI assistant for VS Code"), which is
  this manual's sibling AI-panel integration — see project memory notes on
  `omniharness-vscode-panel` for that feature's own history.
- Command categories visible directly in the manifest include: build/run
  tooling (Build Project, Build Release, Build WASM, Build Linux, Run
  Tests, Run Benchmarks, Profile Build), language-server commands (Restart
  Language Server, Format Document, Organize Imports, Show Type
  Hierarchy, Toggle Inlay Hints), an "OmniOS" sandbox surface (Boot OmniOS
  in Sandbox, View System Services, Show Kernel Log, Package OmniOS Image),
  a package-manager surface ("OmniPM": Install Dependencies, Add Package),
  and formal-verification tooling ("Verify Formal Proofs (AXIOM)").

## Three separate "OmniCC"-named things — this extension owns one of them

`vscode-omnisystem/src/omnicc/` is a large TypeScript implementation
distinct from `OmniHarness/crates/ir`'s real v1 (covered on the OmniCC
page) and from the confirmed-fake `Omnisystem/bin/omnicc.js`. This
extension's own `src/omnicc/` was **not independently re-verified** for
this manual — its `CAPABILITY_COVERAGE.md` makes claims about IR coverage
that weren't re-checked against real code or a real test run here. Treat
its actual depth as **unverified**, not confirmed.

## What wasn't independently re-verified for this manual

- Whether all 217 commands are wired to working implementations versus
  registered-but-stubbed handlers — not individually checked.
- The "OmniOS" sandbox boot/kernel-log/package-image commands — not
  exercised in this session.
- The language-server/formal-verification command set's actual behavior
  against real Omni-Language source — not exercised in this session
  (though the underlying `bootstrap*-rs` interpreters/checkers these
  commands likely shell out to were independently verified — see the
  language pages).
