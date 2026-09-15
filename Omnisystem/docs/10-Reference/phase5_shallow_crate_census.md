# Phase 5 — Shallow-Crate Census & First Remediation Increment

Status: census complete, first build-out increment complete (this session).
Continues the backlog item deferred from the roadmap at
`C:\Users\limpi\.claude\plans\recursive-conjuring-panda.md`, picked up after
Phase 4 (crate-duplication reconciliation, commit `bcf78fb62`) established
that de-duplication and "is this crate shallow" are separate questions.

See `[[feedback-prefer-building-out]]` — this project's standing preference
is to build shallow/orphaned crates out into real, wired logic rather than
archive or delete them, when they have a coherent name/position implying
real purpose. This census exists to make that call cheaply, workspace-wide,
without hand-reading 374 crates.

## Method

1. Enumerated the root `Cargo.toml` `[workspace] members` list: **374 crates**
   (`Omnisystem/Cargo.toml`, current as of commit `043682e04`).
2. For every member, computed: total `src/**/*.rs` LOC, file count, `#[test]`
   count, `pub fn` count, and a grep hit-count for known decorative
   doc-comment headers.
3. Classified a crate as **SCAFFOLD** (shallow/decorative) if it matched any
   of three independently-verified generator signatures found by spot-reading
   representative files:
   - **Header signature** — first line of `src/lib.rs` is exactly one of:
     `//! Enterprise Module`, `//! Advanced Module`,
     `//! OmniDocker component: Auto-generated implementation`,
     `//! Feature UI Module`, `//! Component Library`, `//! Web UI Module`
     (98 crates).
   - **CRUD signature** — `src/types.rs` defines a generic `pub struct Record`
     with only `id/created_at/updated_at/created_by/updated_by` fields, paired
     with a `Manager` doing plain create/get/update/delete/list/count over a
     `DashMap`, with domain-agnostic `CreateRequest`/`UpdateRequest` types
     (36 crates — e.g. `blue-green-deployment`'s original `types.rs`/`manager.rs`
     had zero blue/green-specific logic).
   - **process/analyze signature** — `src/lib.rs` defines a bare struct with
     `async fn process`/`analyze`/`predict` that just echoes or hardcodes a
     value (e.g. `Ok(format!("Analyzed: {}", data))`, `Ok(0.95)`) (18 crates,
     plus 9 more found by spot-checking loc-cluster outliers that used
     slightly different wording, e.g. `"Processed: {}"` / `"Analysis: {}"`).
   - Union of all three, minus exact overlaps: **143 crates**.
4. Confirmed by negative sampling: crates that only *start* their `lib.rs`
   with `pub mod error;`/`mod error;` (90 total) were checked for whether that
   alone was a false-positive signal. Spot-checked `audit-logging`,
   `circuit-breaker`, `cost-analyzer`, `deployment-lifecycle` — all four have
   multiple substantive domain modules (200-330 LOC files with real state
   machines) and are correctly **not** flagged. 57 of the 90 are in this
   "real, just also starts with `mod error;`" bucket.
5. **Legitimately-minimal check**: crates with `loc <= 100` that were *not*
   caught by the scaffold signatures (9 found) were hand-read individually —
   all 9 turned out to be additional scaffold instances (the process/analyze
   variant above), not legitimately-small crates; they were folded into the
   SCAFFOLD count in step 3. No crate was found in this pass that is small
   *and* legitimately so (a genuine thin types-only crate) — none exists at
   this LOC tier in the current member list, though larger legitimately-lean
   crates likely exist and were not individually vetted.

## Method limits (be explicit about what this census does NOT prove)

- This is a **heuristic**, not a full read of ~226,000 lines of Rust. A crate
  could have a bespoke doc-comment (dodging the header signature) while still
  being logic-free; conversely a crate matching a signature could have real
  code below the boilerplate top (not observed in any sample so far, but not
  exhaustively ruled out for all 143).
- "SCAFFOLD" and "real-or-minimal" is a binary split; the real-or-minimal
  bucket is not itself broken down into "real" vs "legitimately minimal" per
  crate — that finer distinction still needs doing for the 240 crates in it
  before this backlog item is "done" in the way the original 51% audit
  implied.
- Reference/wiring analysis (which SCAFFOLD crates are actually called from
  elsewhere) was only done for the 8 crates considered for this session's
  build-out slice, not all 143 — a repo-wide reverse-dependency pass is still
  open work (see Next Steps).

## Current counts

- Total workspace members: **374**
- SCAFFOLD (shallow/decorative): **143 (38.2%)**
- real-or-minimal (not yet further subdivided): **240 (64.2%)** — includes
  everything from genuinely deep crates (e.g. `sylva` at 3,106 LOC, `app-menu`
  at 2,888 LOC) down to crates that only look large because of generated
  boilerplate not matching the three signatures above; some fraction of this
  bucket is very likely still shallow under a different template and was not
  caught. Treat 240 as an upper bound on "genuinely fine," not a proof.
- This 38.2% is measurably down from the stale 51% baseline the original
  cross-session audit reported, consistent with Phase 0's 148-directory
  archival and Phase 4's audit-logging reconciliation having already removed
  some of what that audit counted — but it is **not a clean apples-to-apples
  comparison** since the original audit's exact method/scope was not
  reproduced here, only its intent.

Full per-crate data (374 rows: path, LOC, file count, decorative-header grep
hits, test count, pub-fn count, final classification): see
`phase5_shallow_crate_census.tsv` next to this file. The 143-name SCAFFOLD
list alone, one per line, sorted, is reproducible from that TSV via:
`awk -F'\t' '$7=="SCAFFOLD"{print $1}' phase5_shallow_crate_census.tsv`.

## SCAFFOLD crates by rough domain (keyword grouping, not mutually exclusive)

| Domain (keyword match on crate name) | Count |
|---|---|
| UI/component/dashboard/viewer/library | 29 |
| Analytics/AI/intelligent/optimization/recommendation | 19 |
| Monitoring/logging/alerting/event | 13 |
| omnisystem-*/omnidocker-* integration bridges | 10 |
| Security/compliance/audit/rbac/secret | 10 |
| Docker-* (image/network/volume/registry/compose/lifecycle) | 8 |
| Deployment/rollout/HA/failover (**this session's target**) | 8 |
| Storage/distributed/replication/sharding/scale | 7 |
| Healthcare/clinical/patient/HIPAA | 6 |
| (remaining, no strong keyword cluster) | ~33 |

## This session's build-out increment (6 crates)

Selected the deployment-reliability cluster: clear name-implied purpose,
pre-existing real `Cargo.toml` + module skeleton (`error.rs`/`types.rs`/
`manager.rs`/bin CLI) to build on, and a coherent shared domain (release
orchestration) that made the increment reviewable as one unit. None of these
8 candidate crates were referenced from another crate's `Cargo.toml`
dependencies (checked via `grep -rl "\"<crate>\"" --include=Cargo.toml`) —
consistent with this repo's pattern of many standalone domain crates each
shipping their own demo CLI binary rather than a single integrated caller
graph. Built out 6 of the 8; the remaining 2 (`global-failover`,
`disaster-recovery-platform`) are left for the next session (see below).

| Crate | Real logic implemented | Tests (unit + integration) |
|---|---|---|
| `blue-green-deployment` | Active/standby slot tracking, health-gated promote, rollback to prior active | 10 |
| `canary-deployment` | Staged percentage rollout (5/25/50/100%), error-rate-threshold gate per stage, auto-rollback | 10 |
| `rolling-updates` | Batch rollout bounded by `max_unavailable`, pause/resume, failed-batch rollback without progress loss | 10 |
| `zero-downtime-deployment` | Per-instance ready/drain/in-flight-request tracking, terminate only when safe | 9 |
| `health-check-engine` | Consecutive-failure/success threshold state machine (Healthy/Degraded/Unhealthy), multi-check aggregate status | 8 |
| `high-availability-controller` | Tick-driven cluster membership, majority quorum, deterministic leader election with failover | 6 |
| **Total** | | **53 real, passing tests** |

All six replaced the generic `Record`/`Manager` CRUD scaffold (or the
`process`/`analyze` echo-stub) with domain-specific state machines, deleted
the now-unused `api.rs`/`database.rs` axum/postgres-stub files that had no
real implementation behind them, and rewrote each crate's `tests/integration.rs`
(which had the same generic CRUD scaffold as the `lib.rs` unit tests) to
exercise the new logic end-to-end.

### Verification

`cargo test` per crate — real passing output (see full logs in session
transcript; summarized here):

```
blue-green-deployment:          7 unit + 3 integration = 10 passed, 0 failed
canary-deployment:               7 unit + 3 integration = 10 passed, 0 failed
rolling-updates:                 7 unit + 3 integration = 10 passed, 0 failed
zero-downtime-deployment:        6 unit + 3 integration =  9 passed, 0 failed
health-check-engine:             8 unit                 =  8 passed, 0 failed
high-availability-controller:    6 unit                 =  6 passed, 0 failed
```

`cargo check --workspace` after all six changes: **0 errors** (repo's
standing bar). Only pre-existing warnings (unused imports/variables in
unrelated crates like `extensions`, `failure-finder`) remain, none introduced
by this work.

## Archival candidates

None. All 8 deployment-cluster crates considered had a coherent,
non-overlapping purpose implied by their name and a real skeleton to build
on — none were flagged as "genuinely unnecessary." No archival candidates
were identified in this pass; that question was not asked of the other 137
SCAFFOLD crates not touched this session.

## Next steps for a future session

1. **Finish the deployment-reliability cluster** (2 crates left):
   `global-failover` (multi-region primary/secondary with automatic
   failover/failback — same shape as `high-availability-controller` but at
   region granularity) and `disaster-recovery-platform` (RPO/RTO tracking,
   snapshot-based recovery plan execution, drill validation). Same scaffold
   shape as the 6 already done; both currently unreferenced elsewhere.
2. **Docker-* cluster (8 crates)**: `docker-compose-advanced`,
   `docker-container-lifecycle`, `docker-image-manager`,
   `docker-network-manager`, `docker-registry-integration`,
   `docker-volume-manager`, `dockerfile-optimizer`, plus
   `omnidocker-state-manager` from the omnisystem-bridge cluster — likely the
   next-best coherent slice (shared domain, likely shared vocabulary with
   `omnidocker-state-manager`).
3. **Security/compliance cluster (10 crates)**: `rbac-authorization-engine`,
   `compliance-framework`, `audit-logging-platform` (note: distinct from the
   already-canonical `audit-logging` from Phase 4 — needs its own dedup check
   before being built out, not just built out blind), `secret-management-integration`,
   `container-security-platform`, etc.
4. **Do the reverse-dependency pass** this session skipped for the other 137
   SCAFFOLD crates: `grep -rl "\"<crate-name>\""  --include=Cargo.toml` for
   each, to find any that ARE wired from a real caller (higher priority to
   build out for real — a caller is depending on real behavior it isn't
   getting) versus fully standalone (lower urgency, same as this session's
   picks).
5. **Subdivide the 240-crate real-or-minimal bucket.** This census treated
   "not matching a known scaffold signature" as good enough for the sake of
   scoping this session, but per the method limits above, an unknown number
   of those 240 may be shallow under a signature this pass didn't catch.
   A cheap next check: grep for other repeated-first-line patterns beyond the
   6 already found (the "374 crates, sorted first-lines, count duplicates"
   trick used in this session's terminal history surfaces new ones fast).
6. **UI/component cluster (29 crates)** and **analytics/AI cluster (19
   crates)** are the two largest remaining groups — worth their own sessions
   given their size relative to the 5-15-crate increment this backlog item
   is meant to be worked in.
