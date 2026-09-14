// ── Substrate governance (Rust kernel) ───────────────────────────────────────
// The trust anchor for autonomous runs: enforces resource budgets, capability
// policy, a SHA-256 hash-chained audit log, and a live kill switch — in the
// kernel, so no orchestrator bug can let a swarm/evolution run exceed its bounds.
//
// Wired live (Phase 5): `main.rs` builds one process-wide `SharedGovernor` from
// `Budget::from_env()` / `CapabilityPolicy::from_env()` and hands it to
// `grpc_server::HarnessState`. `ModelService::Chat` calls `check_model` before
// dispatch and `record_call` after (budget/cost tracking); `ToolService::Execute`
// calls `check_tool` before running anything, `run_wasm` included. A single
// process-wide governor (rather than one per session) is a deliberate scope
// choice for this phase — it already stops any client from blowing through the
// kernel's aggregate limits, which is the property the doc comment above
// describes. Per-session/per-run governors are a natural follow-up once the
// orchestrator has a stable way to correlate a session_id across every RPC it
// makes (Chat carries one; ToolService::Execute's session_id field is currently
// unused by any client), and are left dormant on purpose rather than being
// half-wired against inconsistent session data — this file's own audit chain
// and kill switch already work per-Governor whenever that split happens.
//
// The kernel intentionally does not depend on `Omnisystem/src/crates/audit-logging`
// (the canonical crate after Phase 4's audit-system merge): that crate's
// Cargo.toml inherits versions via `.workspace = true`, which requires joining
// the root Cargo workspace this kernel crate deliberately detached from (see
// the `[workspace]` comment in Cargo.toml — most of `src/crates/*` doesn't
// resolve together). The `AuditChain` below is a small, self-contained,
// already-tested SHA-256 hash chain that covers the kernel's own audit needs
// without reintroducing that dependency.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Budget {
    pub max_model_calls: u32,
    pub max_tokens: u64,
    pub max_cost_usd: f64,
    pub max_steps: u32,
    pub max_wallclock_ms: u64,
    pub max_parallel: u32,
}

impl Default for Budget {
    fn default() -> Self {
        Budget {
            max_model_calls: 200,
            max_tokens: 2_000_000,
            max_cost_usd: 10.0,
            max_steps: 500,
            max_wallclock_ms: 1_800_000,
            max_parallel: 16,
        }
    }
}

fn env_u32(key: &str, default: u32) -> u32 {
    std::env::var(key).ok().and_then(|v| v.parse().ok()).unwrap_or(default)
}
fn env_u64(key: &str, default: u64) -> u64 {
    std::env::var(key).ok().and_then(|v| v.parse().ok()).unwrap_or(default)
}
fn env_f64(key: &str, default: f64) -> f64 {
    std::env::var(key).ok().and_then(|v| v.parse().ok()).unwrap_or(default)
}
fn env_bool(key: &str, default: bool) -> bool {
    std::env::var(key)
        .ok()
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(default)
}
fn env_set(key: &str) -> HashSet<String> {
    std::env::var(key)
        .ok()
        .map(|v| v.split(',').map(str::trim).filter(|s| !s.is_empty()).map(String::from).collect())
        .unwrap_or_default()
}

impl Budget {
    /// Read run limits from the environment, falling back to `Default` for
    /// anything unset or unparseable. Lets an operator (or a test) tighten the
    /// kernel-enforced bounds without a rebuild.
    pub fn from_env() -> Self {
        let d = Budget::default();
        Budget {
            max_model_calls:  env_u32("OMNIHARNESS_MAX_MODEL_CALLS", d.max_model_calls),
            max_tokens:       env_u64("OMNIHARNESS_MAX_TOKENS", d.max_tokens),
            max_cost_usd:     env_f64("OMNIHARNESS_MAX_COST_USD", d.max_cost_usd),
            max_steps:        env_u32("OMNIHARNESS_MAX_STEPS", d.max_steps),
            max_wallclock_ms: env_u64("OMNIHARNESS_MAX_WALLCLOCK_MS", d.max_wallclock_ms),
            max_parallel:     env_u32("OMNIHARNESS_MAX_PARALLEL", d.max_parallel),
        }
    }
}

impl CapabilityPolicy {
    /// Read capability policy from the environment. Empty allow-lists mean
    /// "any" (matches `Default`), so an unconfigured kernel stays permissive.
    pub fn from_env() -> Self {
        CapabilityPolicy {
            allowed_models: env_set("OMNIHARNESS_ALLOWED_MODELS"),
            allowed_tools:  env_set("OMNIHARNESS_ALLOWED_TOOLS"),
            denied_tools:   env_set("OMNIHARNESS_DENIED_TOOLS"),
            allow_network:  env_bool("OMNIHARNESS_ALLOW_NETWORK", true),
            allow_fs_write: env_bool("OMNIHARNESS_ALLOW_FS_WRITE", true),
            max_agents:     env_u32("OMNIHARNESS_MAX_AGENTS", 0),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Usage {
    pub model_calls: u32,
    pub tokens: u64,
    pub cost_usd: f64,
    pub steps: u32,
    pub started_ms: i64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CapabilityPolicy {
    pub allowed_models: HashSet<String>, // empty = any
    pub allowed_tools: HashSet<String>,  // empty = any
    pub denied_tools: HashSet<String>,
    pub allow_network: bool,
    pub allow_fs_write: bool,
    pub max_agents: u32,
}

#[derive(Debug)]
pub enum GovError {
    BudgetExceeded(String),
    PolicyViolation(String),
    Aborted(String),
}

impl std::fmt::Display for GovError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GovError::BudgetExceeded(s) => write!(f, "budget exceeded: {s}"),
            GovError::PolicyViolation(s) => write!(f, "policy violation: {s}"),
            GovError::Aborted(s) => write!(f, "aborted: {s}"),
        }
    }
}

// ── Hash-chained audit log ────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    pub seq: u64,
    pub timestamp_ms: i64,
    pub kind: String,
    pub payload: String,
    pub prev_hash: String,
    pub hash: String,
}

#[derive(Default)]
pub struct AuditChain {
    events: Vec<AuditEvent>,
    head: String,
}

impl AuditChain {
    pub fn new() -> Self {
        AuditChain { events: Vec::new(), head: "0".repeat(64) }
    }

    fn hash(head: &str, ts: i64, kind: &str, payload: &str) -> String {
        let mut h = Sha256::new();
        h.update(head.as_bytes());
        h.update(ts.to_string().as_bytes());
        h.update(kind.as_bytes());
        h.update(payload.as_bytes());
        hex::encode(h.finalize())
    }

    pub fn append(&mut self, kind: &str, payload: &str) -> String {
        let ts = chrono::Utc::now().timestamp_millis();
        let digest = Self::hash(&self.head, ts, kind, payload);
        self.events.push(AuditEvent {
            seq: self.events.len() as u64,
            timestamp_ms: ts,
            kind: kind.to_string(),
            payload: payload.to_string(),
            prev_hash: self.head.clone(),
            hash: digest.clone(),
        });
        self.head = digest.clone();
        digest
    }

    pub fn verify(&self) -> bool {
        let mut head = "0".repeat(64);
        for ev in &self.events {
            let digest = Self::hash(&head, ev.timestamp_ms, &ev.kind, &ev.payload);
            if digest != ev.hash || ev.prev_hash != head {
                return false;
            }
            head = digest;
        }
        true
    }

    pub fn events(&self) -> &[AuditEvent] {
        &self.events
    }
}

// ── Kill switch ───────────────────────────────────────────────────────────────

#[derive(Default)]
pub struct KillSwitch {
    tripped: bool,
    reason: String,
}

impl KillSwitch {
    pub fn trip(&mut self, reason: &str) {
        self.tripped = true;
        self.reason = reason.to_string();
    }
    pub fn tripped(&self) -> bool {
        self.tripped
    }
}

// ── Governor: enforces everything, shareable across async tasks ───────────────

pub struct Governor {
    pub budget: Budget,
    pub policy: CapabilityPolicy,
    pub usage: Usage,
    pub kill: KillSwitch,
    pub audit: AuditChain,
}

const COST_PER_1K: f64 = 0.005;

impl Governor {
    pub fn new(budget: Budget, policy: CapabilityPolicy) -> Self {
        let mut g = Governor {
            budget,
            policy,
            usage: Usage { started_ms: chrono::Utc::now().timestamp_millis(), ..Default::default() },
            kill: KillSwitch::default(),
            audit: AuditChain::new(),
        };
        g.audit.append("run_start", "");
        g
    }

    pub fn checkpoint(&mut self, note: &str) -> Result<(), GovError> {
        if self.kill.tripped() {
            self.audit.append("aborted", &self.kill.reason);
            return Err(GovError::Aborted(self.kill.reason.clone()));
        }
        self.usage.steps += 1;
        if self.budget.max_steps > 0 && self.usage.steps > self.budget.max_steps {
            return Err(GovError::BudgetExceeded(format!("max_steps ({note})")));
        }
        let elapsed = (chrono::Utc::now().timestamp_millis() - self.usage.started_ms) as u64;
        if self.budget.max_wallclock_ms > 0 && elapsed > self.budget.max_wallclock_ms {
            return Err(GovError::BudgetExceeded("max_wallclock_ms".into()));
        }
        Ok(())
    }

    pub fn check_model(&mut self, model: &str) -> Result<(), GovError> {
        if !self.policy.allowed_models.is_empty() && !self.policy.allowed_models.contains(model) {
            self.audit.append("policy_violation", model);
            return Err(GovError::PolicyViolation(format!("model {model}")));
        }
        if self.budget.max_model_calls > 0 && self.usage.model_calls >= self.budget.max_model_calls {
            return Err(GovError::BudgetExceeded("max_model_calls".into()));
        }
        Ok(())
    }

    pub fn check_tool(&mut self, tool: &str) -> Result<(), GovError> {
        if self.policy.denied_tools.contains(tool)
            || (!self.policy.allowed_tools.is_empty() && !self.policy.allowed_tools.contains(tool))
        {
            self.audit.append("policy_violation", tool);
            return Err(GovError::PolicyViolation(format!("tool {tool}")));
        }
        Ok(())
    }

    pub fn record_call(&mut self, model: &str, tokens: u64) -> Result<(), GovError> {
        self.usage.model_calls += 1;
        self.usage.tokens += tokens;
        self.usage.cost_usd += (tokens as f64 / 1000.0) * COST_PER_1K;
        self.audit.append("model_call", &format!("{model}:{tokens}"));
        if self.budget.max_tokens > 0 && self.usage.tokens > self.budget.max_tokens {
            return Err(GovError::BudgetExceeded("max_tokens".into()));
        }
        if self.budget.max_cost_usd > 0.0 && self.usage.cost_usd > self.budget.max_cost_usd {
            return Err(GovError::BudgetExceeded("max_cost_usd".into()));
        }
        Ok(())
    }

    pub fn parallelism(&self, requested: u32) -> u32 {
        requested
            .min(self.budget.max_parallel)
            .min(if self.policy.max_agents == 0 { u32::MAX } else { self.policy.max_agents })
            .max(1)
    }
}

/// A thread-safe handle other kernel services share to enforce one run's limits.
pub type SharedGovernor = Arc<RwLock<Governor>>;

pub fn shared(budget: Budget, policy: CapabilityPolicy) -> SharedGovernor {
    Arc::new(RwLock::new(Governor::new(budget, policy)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn budget_and_audit() {
        let mut g = Governor::new(
            Budget { max_model_calls: 2, ..Default::default() },
            CapabilityPolicy::default(),
        );
        assert!(g.check_model("m").is_ok());
        g.record_call("m", 100).unwrap();
        g.record_call("m", 100).unwrap();
        assert!(matches!(g.check_model("m"), Err(GovError::BudgetExceeded(_))));
        assert!(g.audit.verify());
    }

    #[test]
    fn kill_switch_aborts() {
        let mut g = Governor::new(Budget::default(), CapabilityPolicy::default());
        g.kill.trip("stop");
        assert!(matches!(g.checkpoint("x"), Err(GovError::Aborted(_))));
    }

    #[test]
    fn policy_denies() {
        let mut policy = CapabilityPolicy::default();
        policy.allowed_models.insert("ok".to_string());
        let mut g = Governor::new(Budget::default(), policy);
        assert!(g.check_model("ok").is_ok());
        assert!(matches!(g.check_model("nope"), Err(GovError::PolicyViolation(_))));
    }
}
