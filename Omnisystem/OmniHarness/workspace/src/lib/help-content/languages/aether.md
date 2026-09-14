# Aether — databases / cloud / distributed

**Absorbs the design space of:** SQL, GraphQL, Cypher, SPARQL, Erlang,
Elixir, Gleam, plus distributed-systems semantics.

## Real implementation

`Omnisystem/bootstrap-aether-rs`. Binary: `aether-seed`. Structurally
distinct from both Titan (braces) and Sylva (indentation): Aether uses
`do`/`end` keyword-delimited blocks, a third distinct block convention.

## Verified test status (this session)

```
cargo build --release   (bootstrap-aether-rs)
./target/release/aether-seed.exe test ../bootstrap-aether/tests
→ 3/3 passed
```

Built and run in this session, immediately before this page was written.

## Real example — from the fixture suite

`Omnisystem/bootstrap-aether/tests/02_cons_patterns_actors.aether`,
verbatim:

```aether
def sum_list([]) do
  0
end

def sum_list([h | t]) do
  h + sum_list(t)
end

actor Counter do
  def start(initial) do
    initial
  end

  receive :inc, state do
    state + 1
  end

  receive {:add, n}, state do
    state + n
  end

  receive :get, state do
    state
  end
end

puts(sum_list([1, 2, 3, 4, 5]))

c = spawn Counter(10)
send(c, :inc)
send(c, :inc)
send(c, {:add, 5})
puts(send(c, :get))
```

This fixture verifies, in one program: cons-cell list pattern matching
(`[h | t]`), multi-clause function definitions tried in order (two
`sum_list` clauses — base case and recursive case, genuine Erlang/Elixir
semantics, not a single body with an internal `match`), atoms (`:inc`,
`:add`), tuple patterns (`{:add, n}`), and a working `spawn` /
`send` / `receive` actor model with real state threading between messages.

## What works today (verified against real fixtures)

- multi-clause pattern-matched function definitions, tried in order
- cons-cell list patterns (`[h | t]`)
- atoms (`:ok`), the pipe operator (`|>`), `#{}` string interpolation
- guards (`when`)
- `case` / `if` as real expressions, not statement-only
- a cooperative (documented as cooperative, not preemptive) actor model
  with `spawn` / `send` / `receive`

## What's explicitly not implemented

- relational query surface: SELECT/JOIN/aggregation/GROUP BY, CTEs, window
  functions, subqueries, DDL/schema, indexes, prepared statements, views
- transactions: ACID, isolation levels, MVCC, savepoints, 2-phase commit
- NoSQL/graph: document store, key-value, graph traversal, time-series,
  vector search
- supervision trees, let-it-crash semantics, hot code reload, links/monitors
  (the actor model has spawn/send/receive but not OTP-style supervision)
- distribution: clustering, consensus (Raft/Paxos), replication, sharding,
  CRDTs, node discovery
- messaging/streaming: pub/sub, message queues, RPC, event sourcing, CQRS
- schema evolution, migrations, constraints, triggers, stored procedures

## In short

The name "Aether" points at databases *and* distributed/actor systems as
one combined domain. Today, only the actor-model half has a real,
tested substrate. The database/query half is entirely unbuilt — there is no
SQL parser, no schema system, and no transaction semantics anywhere in
`bootstrap-aether-rs`.
