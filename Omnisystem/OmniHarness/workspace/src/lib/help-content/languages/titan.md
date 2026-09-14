# Titan — systems / core

**Absorbs the design space of:** Rust, C, C++, Zig, Go, Ada, D, and similar
systems languages (per `OMNI_LANGUAGE_CAPABILITY_MAP.md`).

## Real implementation

Titan is the only Omni-Language with **two** independently maintained, real
implementations, kept at parity on purpose:

- `Omnisystem/bootstrap-rs` — a Rust-hosted lexer, parser, and tree-walking
  interpreter. Binary: `titan`.
- `Omnisystem/bootstrap` — a TypeScript implementation of the same language,
  run via `node src/cli.ts`.

Both are driven by the same `run|test <file|dir>` CLI shape and share the
same fixture suite at `Omnisystem/bootstrap/tests/*.titan`
(`//@ expect-stdout:` assertions).

**Why two implementations, not one deprecated in favor of the other:** git
history shows both were added together in one commit
(`c5772a97d feat: add 7-language bootstrap interpreters (Rust+TS) + sample
applications + validation suite`), and a later commit
(`31b310ede fix(bootstrap): bring TS Titan interpreter to 16/16 fixture
parity with bootstrap-rs`) specifically brought the TS one back into parity
with the Rust one rather than removing it. They're maintained as two
independent hosts for the same language, not a primary/legacy pair.

## Verified test status (this session)

```
cargo build --release   (bootstrap-rs)
./target/release/titan.exe test ../bootstrap/tests
→ 16/16 passed

node src/cli.ts test tests   (bootstrap, from the bootstrap/ directory)
→ 16/16 passed
```

Both builds and both test runs were executed in this session, immediately
before this page was written.

## Real example — from the fixture suite

`Omnisystem/bootstrap/tests/02_fib.titan`, verbatim:

```titan
//@ expect-stdout: fib(10) = 55
fn fib(n: i32) -> i32 {
    if n < 2 { return n }
    fib(n - 1) + fib(n - 2)
}
pub fn main() {
    println!("fib(10) = {}", fib(10))
}
```

## What works today (verified against real fixtures)

- struct / enum / trait / generics
- closures
- `?` operator, including `?` + `From` auto-conversion across differing
  `Result<_, E>` error types (single-`From`-impl case)
- `if let` / `while let` / match guards
- associated types (`type Item`, `Self::Item`) and custom trait-based
  iterator implementations
- `dyn` trait objects
- range patterns in `match` (`1..=9`, open-ended `..=69`, char ranges)
- loop labels (`'outer: for … break 'outer / continue 'outer`)
- hashmaps, strings, nested generics, basic same-type `?` propagation

## What's explicitly not implemented (not stubbed, not faked)

Per the capability map, these are real, acknowledged gaps, not silently
half-working features:

- borrow checker, lifetimes, RAII, arenas/regions, `defer`/`errdefer`
- macros (declarative or procedural), `comptime`, const-eval
- raw pointers, inline asm, `extern "C"` FFI, SIMD intrinsics, `#[repr]`
- OS threads, channels, atomics, `async`/`await` execution (keywords parse;
  no real concurrency runtime)
- const generics, full type inference, newtypes, higher-kinded types
- multi-source `From` dispatch (only the single-impl case works), `panic`/
  `recover`, error sets
- `cfg` conditional compilation, features, a package/workspace system,
  `goto`

## Type annotations are consumed, not enforced

Titan source can carry type annotations (`n: i32`), but `bootstrap-rs`'s
parser consumes and discards them — there is no separate static
type-checking pass. This matters if you're relying on Titan to catch type
errors at "compile" time: it currently does not. (See the OmniCC page for
how this affects Titan → Rust lowering specifically.)
