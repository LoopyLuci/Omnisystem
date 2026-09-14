# Vera — UI

**Absorbs the design space of:** React/JSX, Vue, Svelte, SolidJS, SwiftUI,
Jetpack Compose.

## Real implementation

`Omnisystem/bootstrap-vera-rs`. Binary: `vera-seed`. The genuinely
distinctive structural choice: markup is embedded **directly in the
language grammar** (`<Tag attr={expr}>...</Tag>`) — not a string template,
not a builder-pattern API. The classic JSX `<`-ambiguity (tag-open vs.
less-than comparison) is avoided structurally rather than by lexer
heuristics: the parser only ever calls its markup-node parser from
`render { }`, a tag's own children, or inside `{if}`/`{for}` — never from
general expression context — so `<` is unambiguous by construction.

## Verified test status (this session)

```
cargo build --release   (bootstrap-vera-rs)
./target/release/vera-seed.exe test ../bootstrap-vera/tests
→ 2/2 passed
```

Built and run in this session, immediately before this page was written.

## Real example — from the fixture suite

`Omnisystem/bootstrap-vera/tests/01_reactive_counter.vera`, verbatim:

```vera
component Counter(initial) {
  state count = initial
  computed doubled = count * 2

  fn increment() {
    count = count + 1
  }

  fn decrement() {
    count = count - 1
  }

  render {
    <div class="counter">
      <span>{count}</span>
      <span>{doubled}</span>
    </div>
  }
}

c = mount(Counter, 5)
puts(render(c))

c.increment()
c.increment()
puts(render(c))

c.decrement()
puts(render(c))
```

The expected output for this fixture shows `count` going 5 → 7 → 6 and
`doubled` tracking it (10 → 14 → 12) across `increment()`/`decrement()`
calls and re-renders — a real before/after data-structure diff, not a
visual claim (there is no browser or GPU involved; "real" here means the
render output is a genuinely, verifiably correct tree each time).

## What works today (verified against real fixtures)

- components, props, composition (a tag naming another component mounts
  and recursively renders it in place)
- reactive `state` — every binding (props/state/locals) is stored as a
  shared mutable cell, so a method mutating a cell and a render reading
  that same cell is genuinely reactive, not asserted
- `computed` values, refreshed each render pass (documented as
  whole-pass recomputation, not fine-grained/incrementally memoized like
  Solid/Svelte signals)
- `{if}` / `{for}` conditional and list rendering
- event-handler closures (`fn(){...}`, `|| expr`, `|params| expr`)

## What's explicitly not implemented

- slots, lifecycle hooks, named fragments
- real virtual-DOM diffing, effects
- two-way binding, refs, context/provide-inject
- hooks/composables, custom events, event delegation
- keys, portals, suspense/async boundaries, error boundaries
- scoped styles, CSS-in-JS, themes, transitions/animations, keyframes
- ARIA, focus management, semantic roles, keyboard navigation

## In short

The component/state/render model is real and verified with an actual
before/after tree diff. Everything past that core loop — styling,
accessibility, advanced rendering features — is unbuilt.
