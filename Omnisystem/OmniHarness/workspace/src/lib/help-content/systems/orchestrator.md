# Orchestrators — Python and Clojure

OmniHarness has two separate orchestrator implementations in two different
languages, each with a distinct real job — they are not redundant with each
other.

## Python orchestrator (`OmniHarness/orchestrator`)

A FastAPI service, per its own README: a single gateway to any language
model — local (Ollama, LM Studio, llama.cpp) or API (Anthropic, OpenAI,
Google, Groq, Mistral, Cohere, OpenRouter, Together, Fireworks) — plus
ReAct agents, memory, RAG, ensembles, a multi-agent swarm substrate, and an
optional gRPC bridge to the Rust kernel (see the Kernel page).

Its README states it's normally started automatically by the VS Code
extension (which also installs its dependencies on first run and
auto-restarts it), and can be run manually:

```bash
python -m pip install -r requirements.txt
python -m uvicorn omniharness.server:app --host 0.0.0.0 --port 8080
```

Key HTTP endpoints, per its own documented table:

| Method | Path | Purpose |
|---|---|---|
| GET | `/api/health` | Reachability + provider health |
| GET | `/api/models` | Aggregated model catalogue |
| POST | `/api/chat` | Chat completion (+ tool calling) |
| POST | `/api/chat/stream` | Streaming chat (SSE) |
| POST | `/api/swarm/run` | Multi-agent swarm |
| POST | `/api/ensemble/run` | Multi-model ensemble |
| POST | `/api/rag/*` | Retrieval-augmented generation |

This manual did not independently start the server and exercise these
endpoints — the table above is transcribed from the orchestrator's own
README, not separately verified against a running instance. Treat specific
endpoint behavior as **unverified**.

## Clojure orchestrator (`OmniHarness/clj-orchestrator`)

A distinct service with a distinct job, per its own `project.clj`
description: **"HTN planner, policy engine, patch manager."** It is not a
second implementation of the Python orchestrator's model-gateway role.

Real, verifiable facts about it from its own build file:
- Leiningen project, main namespace `omniharness.core`, AOT-compiled.
- Talks gRPC via `grpc-netty-shaded`/`grpc-protobuf`/`grpc-stub` 1.68.1,
  with Java stubs generated into `gen-java/` from the same
  `../proto/omniharness.proto` the Rust kernel serves — this is the
  concrete mechanism by which the Clojure side and the Rust kernel speak a
  shared contract.
- Also exposes an HTTP surface via `http-kit` + `ring` + `compojure`.
- A code comment in `project.clj` notes a real, specific build fix: the
  `@javax.annotation.Generated` annotation used by the generated gRPC-Java
  stubs was removed from the JDK after Java 8, so
  `javax.annotation-api` is pulled in explicitly to supply it.

This manual did not run `lein` against this project in this session to
independently reconfirm it currently builds — per this repo's own memory
notes, it was reported building cleanly (full gRPC Java stub compilation)
as of a prior session, but that was not re-verified here. Treat "currently
builds" as **unverified** until you run `lein compile` yourself.

## In short

Python orchestrator = model gateway + agents/RAG/swarm, reachable over
HTTP. Clojure orchestrator = HTN planning + policy + patch management,
reachable over gRPC (and HTTP), sharing one proto contract with the Rust
kernel. Three different languages, three different jobs, one shared proto
file connecting two of them.
