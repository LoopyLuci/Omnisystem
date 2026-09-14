# Nexus — layout

**Absorbs the design space of:** CSS Flexbox/Grid, Cassowary constraint
layout, Apple Auto Layout, Android ConstraintLayout.

## Real implementation

`Omnisystem/bootstrap-nexus-rs`. Binary: `nexus-seed`. Nexus is genuinely
different *in kind* from the other 6 languages, not just in syntax: it is a
**declarative constraint/layout solver**, not a general-purpose imperative
language — there are no functions, no control flow, no statements.

## Verified test status (this session)

```
cargo build --release   (bootstrap-nexus-rs)
./target/release/nexus-seed.exe test ../bootstrap-nexus/tests
→ 3/3 passed
```

Built and run in this session, immediately before this page was written.
The 3 fixtures deliberately cover the happy path *and both failure modes*
(a violated constraint, and a dependency cycle) — not just success cases.

## Real example — from the fixture suite

`Omnisystem/bootstrap-nexus/tests/01_row_flow_and_constraints.nexus`,
verbatim:

```nexus
box Sidebar {
  width: 200
}

box Content {
  width: Main.width - Sidebar.width
}

layout Main {
  width: 800
  height: 600
  direction: row
  children: [Sidebar, Content]
}

constrain Sidebar.width + Content.width == Main.width
constrain Sidebar.height == Content.height
```

Expected output includes both `constrain` checks reporting `OK` with the
actual resolved numbers (`800.0 == 800.0`), and each box's final resolved
`x/y/width/height` — the solver genuinely computes `Content.width` as `600`
by resolving `Main.width - Sidebar.width` (`800 - 200`), not by any
hardcoded value.

## What works today (verified against real fixtures)

- box/layout properties as equations, resolved lazily and memoized
  (spreadsheet-cell style)
- real dependency-cycle detection — a genuine `A.width` ← `B.width` ←
  `A.width` cycle reports a clear error instead of infinite-looping
- `layout` blocks run a real row/column flow algorithm: cumulative
  main-axis positioning + cross-axis stretch
- top-level `constrain` statements are genuinely checked — a violated
  constraint (e.g. `A.width <= 100` when `A.width` solves to `500`) reports
  `FAILED` and the run exits with a non-zero status; solving can really fail

## What's explicitly not implemented

- CSS Grid: template rows/cols, areas, auto-placement, spanning,
  `minmax`/`fr`, subgrid
- flexbox depth: wrap, grow/shrink/basis, justify/align, gap, order
- a real Cassowary solver (priorities/soft constraints), intrinsic sizing
- responsive design: breakpoints, media/container queries, fluid units,
  aspect ratio
- box model depth: margin/padding/border, absolute/relative/sticky/fixed
  positioning, z-order, overflow
- typography flow: text flow, baseline alignment, columns, line-breaking

## In short

What exists is a real, small, genuinely-checked constraint solver with
row/column flow layout — not a CSS engine. Nothing beyond `x`/`y`/`width`/
`height` and row/column flow is implemented.
