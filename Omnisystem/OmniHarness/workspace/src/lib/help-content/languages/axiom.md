# Axiom — formal verification

**Absorbs the design space of:** Coq, Agda, Idris, Lean, TLA+, Dafny, F*,
Alloy, MiniZinc.

## Real implementation

`Omnisystem/bootstrap-axiom-rs`. Binary: `axiom-seed`. Axiom is a genuine
**checker**, not a program interpreter — `theorem`/`invariant` statements
are actually verified, not example-tested. The core technique is
**bounded-exhaustive verification** (Alloy's real, legitimate finite-scope
model-finding approach): a `theorem Name forall x in lo..hi, ...` checks
every combination of the quantified variables' explicit finite ranges.

## Verified test status (this session)

```
cargo build --release   (bootstrap-axiom-rs)
./target/release/axiom-seed.exe test ../bootstrap-axiom/tests
→ 2/2 passed
```

Built and run in this session, immediately before this page was written.

## Real example — from the fixture suite

`Omnisystem/bootstrap-axiom/tests/01_bounded_verification.axiom`, verbatim:

```axiom
theorem CommuteAdd forall x in 0..10, y in 0..10 {
  x + y == y + x
}

theorem PositiveSquare forall x in -5..5 {
  x * x >= 0
}

theorem AlwaysUnderThree forall x in 0..5 {
  x < 3
}

axiom BaseCase {
  0 + 0 == 0
}

theorem UsesAxiom forall x in 0..5 {
  BaseCase => x + 0 == x
}

invariant CounterNonNegative over states (
  { count: 0 },
  { count: 5 },
  { count: -1 }
) {
  count >= 0
}
```

This single fixture verifies both proof and *disproof*: `CommuteAdd` is
proven over 100 cases, `PositiveSquare` over 10 cases (including
negatives), but `AlwaysUnderThree` is correctly and automatically
**disproven** with the real, reproducible counterexample `x=3` — not a
guess or a hardcoded failure. `invariant CounterNonNegative` similarly
correctly flags the `{count: -1}` state as `VIOLATED` while passing the
non-negative ones. The expected exit code for this fixture is `1`, because
a real disproof/violation occurred.

## What works today (verified against real fixtures)

- `theorem ... forall ... in lo..hi` — bounded-exhaustive proof or
  disproof, with a real counterexample on disproof
- `axiom` declarations — assumed ground facts, referenceable by name inside
  theorems via `=>` (material implication)
- `invariant ... over states (...)` — checks a proposition across an
  explicit, enumerated state list (TLA+ heritage, simplified to a fixed
  list rather than reachability search from a transition relation — an
  intentional scope boundary, not an oversight)
- an unbounded free variable in a theorem is a real, reportable error
  (`undefined name 'x'`), not silently assumed universal — this is what
  makes the bounded-exhaustive approach honest rather than silently wrong

## What's explicitly not implemented

- dependent types (Agda/Idris-style), refinement types, indexed types,
  `Prop`/`Type` universes
- a tactic language, proof terms, `Qed`, goals/subgoals
- `requires`/`ensures` contracts, ghost/spec variables
- temporal logic (□/◇), transition-relation reachability search,
  liveness/fairness — `invariant` only checks an explicit fixed state list
- real SMT solving (Z3-class unbounded solving) or unification
- inductive datatypes, structural recursion + termination checking, Horn
  clauses/backtracking, Datalog fixpoint

## In short

Axiom's bounded-exhaustive checker is real and can genuinely disprove false
claims with a real counterexample — that's a meaningfully higher bar than a
"pass/fail on one example" test runner. It is not a general-purpose proof
assistant: there's no tactic language, no unbounded solving, and
`invariant` checking is over an explicit finite state list, not a real
reachability search.
