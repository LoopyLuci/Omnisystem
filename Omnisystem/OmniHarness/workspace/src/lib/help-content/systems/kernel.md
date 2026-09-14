# OmniHarness Kernel — gRPC substrate

`Omnisystem/OmniHarness/kernel` — Rust crate `omniharness-kernel`, binary
`omniharness-kernel`.

## What it is

A Rust gRPC server (built on `tonic` 0.14) that acts as a shared substrate
service for the rest of OmniHarness. Its source files describe its real
scope directly:

- `grpc_server.rs` — the gRPC service implementation
- `auth.rs` — authentication
- `event_store.rs` — event storage (backs things like the Workspace app's
  Time Travel panel — see the Workspace page)
- `mesh.rs` — networking/mesh concerns
- `model_router.rs` — routing requests to model backends
- `sandbox.rs` — sandboxed execution
- `session_store.rs` — session persistence
- `substrate.rs` — the core substrate abstraction this crate is named for
- `tool_registry.rs` — registered tool metadata
- `vector_store.rs` — vector storage (embeddings/RAG-adjacent)

The proto contract it serves is generated from
`OmniHarness/proto/omniharness.proto` — both this kernel and the Clojure
orchestrator (`clj-orchestrator`) generate stubs from that same proto file,
which is how the two communicate: the Clojure side's `project.clj` notes
its gRPC stubs are regenerated from `../proto/omniharness.proto` via
`tools/gen-proto.sh`.

## How it relates to the other two orchestrators

Omnisystem has three separate backend-language implementations under
`OmniHarness/`: this Rust kernel, a Python orchestrator (FastAPI), and a
Clojure orchestrator. They are not three implementations of the same
thing — see the Orchestrator page for what the Python and Clojure sides
specifically own (HTTP model gateway + agents/RAG/swarm for Python; HTN
planning, policy engine, and patch management for Clojure). This kernel is
the Rust-side substrate they can optionally bridge to over gRPC.

## What wasn't independently re-verified for this manual

This manual did not run the kernel's own test suite or start the server to
confirm the gRPC service surface matches the proto file exactly — that
would require standing up the service and a client. Treat the module list
above (from real source file names, read directly) as accurate, but treat
any specific claim about request/response behavior as **unverified**
unless you've checked `grpc_server.rs` yourself.
