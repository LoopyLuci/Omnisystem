# Sylva — Python-like / ML / dynamic scripting

**Absorbs the design space of:** Python, JavaScript, TypeScript, Ruby, Lua,
R, Julia, plus NumPy/PyTorch-style numeric semantics.

## Real implementation

`Omnisystem/bootstrap-sylva-rs` — a genuine, from-scratch lexer, parser, and
tree-walking interpreter. Binary: `sylva-seed`. It is **not** a reskin of
Titan's grammar; it was deliberately built with structurally different
choices (see below), per explicit project direction that all 7
Omni-Languages must be unique to themselves, not variations on one grammar.

Note: an older file, `src/compiler/languages/sylva_interpreter.py`, is
**not** a real interpreter — its own `interpret()` method calls Python's
`exec()` directly on raw source, with a comment admitting *"In a real
implementation, this would parse Sylva syntax."* There is no lexer or
parser in that file. Do not confuse it with the real implementation above;
it predates `bootstrap-sylva-rs` and represents the "hollow scaffolding"
this manual is explicitly trying not to add to.

## Verified test status (this session)

```
cargo build --release   (bootstrap-sylva-rs)
./target/release/sylva-seed.exe test ../bootstrap-sylva/tests
→ 4/4 passed
```

Built and run in this session, immediately before this page was written.

## Real example — from the fixture suite

`Omnisystem/bootstrap-sylva/tests/04_tensor_closures_recursion.sylva`,
verbatim:

```sylva
def main():
    a = zeros([2, 2])
    b = ones([2, 2])
    c = a.add(b)
    print(c.sum())
    print(c.mean())

    d = tensor([1.0, 2.0, 3.0, 4.0])
    print(d.sum())

    def fib(n):
        if n <= 1:
            return n
        return fib(n - 1) + fib(n - 2)

    print(fib(10))

    def make_counter():
        count = [0]
        def inc():
            count[0] = count[0] + 1
            return count[0]
        return inc

    counter = make_counter()
    print(counter())
    print(counter())
    print(counter())

main()
```

This one fixture alone verifies: a minimal real `Tensor` type
(zeros/ones/add/sum/mean/tensor), recursion, nested function definitions,
and closures that genuinely capture mutable state by reference (the
`make_counter` pattern above only produces `1, 2, 3` if `count` is a real
shared cell, not a copy).

## What works today (verified against real fixtures)

- real significant whitespace (Indent/Dedent tokens), not brace-delimited
- `def` / `class`, dynamic typing throughout — type hints are parsed and
  discarded, never enforced
- real exceptions: `try` / `except` / `finally` / `raise`. All runtime
  faults (division by zero, index errors, missing attributes) are
  catchable — this was a real bug fix during development (errors originally
  routed through an uncatchable internal error variant, defeating the point
  of having exception handling)
- classes with single inheritance + MRO, `__init__` constructors
- closures with real captured-by-reference mutable state
- f-string interpolation, list/dict comprehensions, tuple-unpacking `for`
  targets (`for k, v in d.items():`), lambda
- a minimal real `Tensor` value type (zeros/ones/tensor/add/mul/sum/mean —
  a small true subset of the much larger `SYLVA_STANDARD_LIBRARY.sylva`
  spec, not faked further than that)
- top-to-bottom script execution — no `main()` requirement, matching real
  Python semantics

## What's explicitly not implemented (flagged, not faked)

- generators / `yield` — real lazy suspension needs a coroutine-capable
  evaluator; this explicitly errors rather than pretending to support it
- `async` / `await` — parses, but evaluates synchronously; there is no
  event loop
- stepped slicing
- multi-source `From`-style dispatch equivalents
- autodiff, tensor broadcasting, dataframes, GPU dispatch, JIT — the wider
  ML-domain capability list is aspirational; only the minimal Tensor
  methods listed above are real

## In short

Sylva is intentionally the least "complete" of the 7 languages relative to
its target design space (Python + ML) — the capability map itself flags
this as the single biggest remaining gap. What exists is real and tested,
not a facade.
