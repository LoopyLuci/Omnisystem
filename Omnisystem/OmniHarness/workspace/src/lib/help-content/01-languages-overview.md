# The 7 Omni-Languages — Overview

Omnisystem defines 7 domain-specific languages, each aimed at a different
part of the stack. The project's own design intent for each is recorded in
`Omnisystem/src/compiler/OMNI_LANGUAGE_CAPABILITY_MAP.md` — that document
also carries an important correction worth repeating here, because it
explains why this manual points you at a different set of files than you
might expect:

> The other six Omni-Languages are still narrow domain DSLs at the
> frontend-survey level... none has a runnable substrate yet, only a
> parser-level DSL grammar.

That line is now out of date in the *pessimistic* direction — real,
runnable interpreters/checkers for all 6 non-Titan languages were built
after it was written (2026-07-04). What it gets right, and what still
matters: the frontends under `src/compiler/frontend/*` are a separate,
inert parallel effort. **The real, runnable implementation of every one of
the 7 languages lives in its own `bootstrap*` / `bootstrap*-rs` directory
pair** at the repo root, not under `src/compiler/`.

| Language | Domain | Real implementation | Tests (verified this session) |
|---|---|---|---|
| **Titan** | Systems / core | `bootstrap-rs` (Rust) + `bootstrap` (TypeScript) — two independent, parity-maintained implementations | 16/16 (both) |
| **Sylva** | Python-like / ML / dynamic scripting | `bootstrap-sylva-rs` | 4/4 |
| **Aether** | Databases / distributed / actors | `bootstrap-aether-rs` | 3/3 |
| **Vera** | UI / component rendering | `bootstrap-vera-rs` | 2/2 |
| **Nexus** | Layout / constraint solving | `bootstrap-nexus-rs` | 3/3 |
| **Helix** | GPU / compute-kernel | `bootstrap-helix-rs` | 5/5 |
| **Axiom** | Formal verification | `bootstrap-axiom-rs` | 2/2 |

All 7 counts above were produced by building each `bootstrap*-rs` crate with
`cargo build --release` and running that binary's own `test` subcommand
against its sibling `bootstrap*/tests/` fixture directory, in this session,
immediately before this manual was written. None of these numbers were
copied from another document.

## Why each language is structurally different, not just re-skinned

A running theme across all 7 real implementations is that each one made a
deliberate, load-bearing syntactic/semantic choice that makes it genuinely
distinct from the others, not a copy with different keywords:

- **Titan** — brace-delimited blocks, static-shaped syntax (types are parsed
  but not enforced by a separate checker), `Result`/`Option`/`?`.
- **Sylva** — real significant whitespace (Indent/Dedent tokens, not just
  "ignore braces"), dynamic typing throughout, real catchable exceptions.
- **Aether** — `do`/`end` keyword-delimited blocks (a third distinct block
  style), multi-clause pattern-matched function definitions tried in order
  (genuine Erlang/Elixir semantics), atoms, `|>` pipe.
- **Vera** — markup embedded directly in the language grammar (not a string
  template, not a builder API); `<` is unambiguous by construction because
  the markup parser is only ever invoked from specific grammar positions.
- **Nexus** — not a general-purpose language at all: no functions, no
  control flow. A declarative constraint/layout solver with real
  dependency-cycle detection and real constraint-violation checking.
- **Helix** — a data-parallel dispatch model (`dispatch(Kernel, buffer, n)`)
  plus two *actually enforced* restrictions: no recursion, no dynamic loop
  bounds — both checked, not just implied by the domain.
- **Axiom** — bounded-exhaustive verification (checks every combination of a
  quantified variable's explicit finite range) rather than a program
  interpreter; `theorem`s are genuinely proven or disproven with a real
  counterexample.

## What "done" means for a language capability

The capability map's own verification gate (worth repeating, because it's a
good bar): a capability only counts as done when the frontend parses it,
it's exercised by a real runtime/interpreter test (not just accepted by the
grammar and silently ignored), and it's documented. A keyword the parser
accepts but does nothing with does not count as "supported" anywhere in
this manual.

See each language's own page for what specifically works today versus
what's an explicit, honest gap.
