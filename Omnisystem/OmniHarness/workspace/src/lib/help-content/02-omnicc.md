# OmniCC — the real v1 cross-converter

**This page is specifically about `Omnisystem/OmniHarness/crates/ir` and its
`omnicc` binary.** There are two other, unrelated things in this repo that
also carry the "OmniCC" name — see [Which OmniCC is this?](#which-omnicc-is-this)
below before you go looking for other implementations.

## What it actually does

`omnicc` parses source in one of two real front-end languages into one
shared intermediate representation (`IrModule`), then emits real,
compilable Rust source from it:

```
omnicc compile --from sylva --to rust <file> [-o out.rs]
omnicc compile --from titan --to rust <file> [-o out.rs]
```

This is genuine source-to-source translation — it does **not** generate
machine code (no ELF/PE/Mach-O output). The emitted Rust source is then
compiled by a real Rust toolchain (`rustc`/`cargo`), same as any other Rust
you'd write by hand.

The two front ends:

- **Sylva-subset** — a real recursive-descent parser (`ir::parser`) for a
  deliberately small surface grammar: `fn` definitions, `let`, `if`/`else`,
  lambdas, `spawn`/`send`/`receive`, `@device` annotations, `@sql(...)`
  literals, and a handful of primitive/container types (`Unit`, `Bool`,
  `Int`, `Float`, `Str`, `Bytes`, arrays, `ActorRef<T>`, `DataFrame`,
  `NDArray<T>`). This is a purpose-built subset for the IR pipeline, not the
  full `bootstrap-sylva-rs` language.
- **Titan** — reuses Titan's own real lexer/parser (`titan::parser::parse`,
  the same one `bootstrap-rs` uses) and lowers Titan's real AST
  (`titan::ast`) into the same `IrModule`, via `titan_lower.rs`.

## Exact coverage boundary (from `titan_lower.rs`'s own doc comment)

This is the file that defines precisely what the Titan → Rust path
supports. Quoting it directly rather than summarizing, since precision
matters here:

**Supported:**
- top-level `fn` items (`pub fn` or plain `fn`), each with a single block
  body
- statements: `let NAME = expr;` with a plain identifier bind pattern, and
  bare expression statements (sequenced; the final one is the block's
  value)
- expressions: integer/float/string/bool literals; single-segment variable
  references; unary `-`/`!`; binary arithmetic, comparison, logical and
  bitwise operators; `if { .. } else { .. }` (both branches required — no
  `if let`); `return`; calls to other functions defined in the same file;
  and the `println!`/`print!`/`eprintln!`/`format!` macros

**Explicitly NOT supported (rejected with a `LowerError`, never silently
mishandled or dropped):**
- `struct`/`enum`/`impl`/`trait`/`mod`/`const`/`use` items, and therefore
  also `self`, methods, field access (`a.b`), and `Type::method(...)` paths
- `match`, `loop`, `while`, `for`, labeled break/continue
- closures, arrays, tuples, ranges, casts, `?`, indexing
- any `let` pattern other than a plain identifier (no destructuring)
- multi-segment paths (`Foo::bar`) anywhere

The source's own summary of scope: this covers `01_hello.titan` and
`02_fib.titan` from `bootstrap/tests/` end-to-end. It is a real but
partial subset, not full Titan.

## Why parameter types are a heuristic, not real inference

Titan's AST carries no static types — type annotations in Titan source
(`n: i32`) are consumed and discarded by `bootstrap-rs`'s parser, since
Titan has no separate type-checking pass (see the Titan language page).
UniIR, by contrast, requires a concrete type on every parameter and return.
So `titan_lower.rs` uses a shallow syntactic heuristic: a function's
parameters and return type are inferred `F64` if any float literal appears
anywhere in the function body, otherwise `I64` — correct for
arithmetic-shaped functions like `fib`, and simply wrong for anything
needing a `Str`, `Bool`, or struct-typed parameter, unless the return type
is explicitly annotated in the Titan source (`-> i32`, `-> bool`, `->
String`), which the lowering pass does read from the real `TypeRef`.

## Which OmniCC is this?

There are three unrelated things named "OmniCC" in this repository. This
page covers only the first one:

1. **`OmniHarness/crates/ir` + its `omnicc` binary** (this page) — real,
   tested, narrow. Covers Titan and a Sylva-subset, both → Rust source.
2. **`Omnisystem/bin/omnicc.js`** — a separate script. A prior audit in
   this project's history confirmed this one is fake (does not do real
   compilation).
3. **`Omnisystem/vscode-omnisystem/src/omnicc/`** — a separate, large
   TypeScript implementation inside the VS Code extension. Its actual depth
   was not independently re-verified for this manual — treat it as
   **unverified**, not confirmed working, until someone checks it directly
   against its own test suite (if any).

Do not assume a claim about one of these three applies to another. If you
see "OmniCC" mentioned elsewhere in this codebase or in older docs, check
which of the three it's actually talking about before trusting a
capability claim.
