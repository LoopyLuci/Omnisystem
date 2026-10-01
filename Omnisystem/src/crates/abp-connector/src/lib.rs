//! AgenticBotPlatform (ABP) inside Omnisystem.
//!
//! [`AbpModule`] is an Omnisystem [`ModuleInterface`] (an `AppModule`): load it like any other module and every
//! part of Omnisystem can use ABP through `execute(command, args)`. [`AbpClient`] is the same over plain async
//! calls, and `abp-omni` (src/main.rs) is the command line.
//!
//! ABP is reached at `ABP_URL` (default `http://127.0.0.1:8787`) with an integration key in `ABP_KEY`: mint one in
//! ABP with the "omnisystem" preset (least privilege: status, bots read-only, asking a bot, swarms, ABP's model
//! gateway, its modules, its local AI and Neural Lab read-only — no bot lifecycle, no Docker, no config, no
//! credentials). The key is sent as `Authorization: Bearer`, never logged, never printed.
//!
//! Commands (`execute`, and `abp-omni <command> [args]`):
//!
//! | command | args | ABP call |
//! |---|---|---|
//! | `status` | | `GET /healthz`, `GET /api/integrations/whoami` |
//! | `bots` | | `GET /api/bots` |
//! | `ask` | `<from-bot> <to-bot> <prompt…>` | `POST /api/agent/ask` |
//! | `chat` | `<model> <prompt…>` | `POST /api/browser/v1/chat/completions` (ABP's gateway; model `auto` lets ABP route) |
//! | `models` | | `GET /api/models` |
//! | `modules` | | `GET /api/modules` |
//! | `call` | `<module> <operation> [json]` | `POST /api/modules/<module>/call` |
//! | `swarms` | | `GET /api/swarms` |
//! | `swarm` | `<id> <goal…>` | `POST /api/swarms/<id>/run` |
//! | `localai` | | `GET /api/localai/models` |
//! | `lab` | | `GET /api/lab` |
//! | `advice` | `<transfer src dst|llm model|memory|stability>` | `GET /api/lab/systune/advice` |
//! | `raw` | `<GET|POST> <path> [json]` | any route the key's scopes allow |

use async_trait::async_trait;
use module_interfaces::{
    HealthStatus, ModuleConfig, ModuleDependency, ModuleError, ModuleInterface, ModuleMetadata, ModuleStatus, ModuleType,
};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub const DEFAULT_URL: &str = "http://127.0.0.1:8787";

#[derive(Debug, thiserror::Error)]
pub enum AbpError {
    #[error("ABP is not reachable at {url}: {source}")]
    Unreachable { url: String, source: reqwest::Error },
    #[error("ABP answered {status} for {path}: {body}")]
    Http { status: u16, path: String, body: String },
    #[error("{0}")]
    Usage(String),
    #[error("ABP's answer was not JSON: {0}")]
    BadJson(String),
}

impl From<AbpError> for ModuleError {
    fn from(e: AbpError) -> Self {
        match e {
            AbpError::Usage(m) => ModuleError::ConfigurationError(m),
            AbpError::Http { status: 401 | 403, .. } => ModuleError::PermissionDenied(e.to_string()),
            other => ModuleError::ExecutionFailed(other.to_string()),
        }
    }
}

/// ABP's HTTP API with an integration key.
#[derive(Clone)]
pub struct AbpClient {
    base: String,
    key: Option<String>,
    http: reqwest::Client,
}

impl std::fmt::Debug for AbpClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AbpClient").field("base", &self.base).field("key", &self.key.as_ref().map(|_| "<set>")).finish()
    }
}

impl AbpClient {
    pub fn new(base: impl Into<String>, key: Option<String>) -> Self {
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(300))
            .user_agent(concat!("omnisystem-abp-connector/", env!("CARGO_PKG_VERSION")))
            .build()
            .expect("an HTTP client");
        Self { base: base.into().trim_end_matches('/').to_string(), key: key.filter(|k| !k.trim().is_empty()), http }
    }

    /// From ABP_URL and ABP_KEY.
    pub fn from_env() -> Self {
        Self::new(std::env::var("ABP_URL").unwrap_or_else(|_| DEFAULT_URL.into()), std::env::var("ABP_KEY").ok())
    }

    pub fn base_url(&self) -> &str {
        &self.base
    }

    pub async fn request(&self, method: &str, path: &str, body: Option<&Value>, query: &[(&str, &str)]) -> Result<Value, AbpError> {
        if !path.starts_with('/') {
            return Err(AbpError::Usage(format!("a path starts with / (got {path})")));
        }
        let m = reqwest::Method::from_bytes(method.to_ascii_uppercase().as_bytes())
            .map_err(|_| AbpError::Usage(format!("unknown HTTP method {method}")))?;
        let mut req = self.http.request(m, format!("{}{}", self.base, path)).query(query);
        if let Some(k) = &self.key {
            req = req.bearer_auth(k);
        }
        if let Some(b) = body {
            req = req.json(b);
        }
        let resp = req.send().await.map_err(|source| AbpError::Unreachable { url: self.base.clone(), source })?;
        let status = resp.status().as_u16();
        let text = resp.text().await.map_err(|source| AbpError::Unreachable { url: self.base.clone(), source })?;
        if !(200..300).contains(&status) {
            let detail = serde_json::from_str::<Value>(&text)
                .ok()
                .and_then(|v| v.get("detail").or_else(|| v.get("error")).cloned())
                .map(|d| d.as_str().map(str::to_string).unwrap_or_else(|| d.to_string()))
                .unwrap_or_else(|| text.chars().take(500).collect());
            return Err(AbpError::Http { status, path: path.to_string(), body: detail });
        }
        if text.trim().is_empty() {
            return Ok(Value::Null);
        }
        serde_json::from_str(&text).map_err(|e| AbpError::BadJson(format!("{e}: {}", text.chars().take(200).collect::<String>())))
    }

    pub async fn get(&self, path: &str) -> Result<Value, AbpError> {
        self.request("GET", path, None, &[]).await
    }

    pub async fn post(&self, path: &str, body: &Value) -> Result<Value, AbpError> {
        self.request("POST", path, Some(body), &[]).await
    }

    pub async fn status(&self) -> Result<Value, AbpError> {
        let health = self.get("/healthz").await?;
        let who = self.get("/api/integrations/whoami").await.unwrap_or(Value::Null);
        Ok(json!({"url": self.base, "health": health, "key": who}))
    }

    pub async fn chat(&self, model: &str, prompt: &str) -> Result<String, AbpError> {
        let r = self
            .post("/api/browser/v1/chat/completions", &json!({"model": model, "messages": [{"role": "user", "content": prompt}]}))
            .await?;
        Ok(r.pointer("/choices/0/message/content").and_then(Value::as_str).unwrap_or_default().to_string())
    }

    /// Run one command (the table in the crate docs); returns ABP's answer as JSON.
    pub async fn run(&self, command: &str, args: &[String]) -> Result<Value, AbpError> {
        let rest = |from: usize| args.get(from..).map(|a| a.join(" ")).unwrap_or_default();
        let need = |n: usize, usage: &str| -> Result<(), AbpError> {
            if args.len() < n {
                Err(AbpError::Usage(format!("usage: {command} {usage}")))
            } else {
                Ok(())
            }
        };
        match command {
            "status" => self.status().await,
            "bots" => self.get("/api/bots").await,
            "models" => self.get("/api/models").await,
            "modules" => self.get("/api/modules").await,
            "swarms" => self.get("/api/swarms").await,
            "localai" => self.get("/api/localai/models").await,
            "lab" => self.get("/api/lab").await,
            "ask" => {
                need(3, "<from-bot> <to-bot> <prompt>")?;
                self.post("/api/agent/ask", &json!({"source_instance": args[0], "target_instance": args[1], "prompt": rest(2)}))
                    .await
            }
            "chat" => {
                need(2, "<model|auto> <prompt>")?;
                Ok(json!({"model": args[0], "reply": self.chat(&args[0], &rest(1)).await?}))
            }
            "call" => {
                need(2, "<module> <operation> [json inputs]")?;
                let inputs: Value = if args.len() > 2 {
                    serde_json::from_str(&rest(2)).map_err(|e| AbpError::Usage(format!("inputs must be JSON: {e}")))?
                } else {
                    json!({})
                };
                self.post(&format!("/api/modules/{}/call", seg(&args[0])?), &json!({"operation": args[1], "args": inputs})).await
            }
            "swarm" => {
                need(2, "<swarm id> <goal>")?;
                let id: u64 = args[0].parse().map_err(|_| AbpError::Usage("a swarm id is a number".into()))?;
                self.post(&format!("/api/swarms/{id}/run"), &json!({"prompt": rest(1)})).await
            }
            "advice" => {
                need(1, "<transfer <src> <dst> | llm <model> | memory | stability>")?;
                let mut q: Vec<(&str, &str)> = vec![("kind", args[0].as_str())];
                match args[0].as_str() {
                    "transfer" => {
                        need(3, "transfer <src> <dst>")?;
                        q.push(("src", &args[1]));
                        q.push(("dst", &args[2]));
                    }
                    "llm" => {
                        need(2, "llm <model>")?;
                        q.push(("model", &args[1]));
                    }
                    "memory" | "stability" => {}
                    other => return Err(AbpError::Usage(format!("advice is transfer, llm, memory or stability (got {other})"))),
                }
                self.request("GET", "/api/lab/systune/advice", None, &q).await
            }
            "raw" => {
                need(2, "<GET|POST> <path> [json]")?;
                let body: Option<Value> = if args.len() > 2 {
                    Some(serde_json::from_str(&rest(2)).map_err(|e| AbpError::Usage(format!("the body must be JSON: {e}")))?)
                } else {
                    None
                };
                self.request(&args[0], &args[1], body.as_ref(), &[]).await
            }
            other => Err(AbpError::Usage(format!("unknown command {other} (one of: {})", COMMANDS.join(", ")))),
        }
    }
}

pub const COMMANDS: &[&str] = &[
    "status", "bots", "ask", "chat", "models", "modules", "call", "swarms", "swarm", "localai", "lab", "advice", "raw",
];

/// A path segment: letters, digits, ., _ and - only (no way to reach another route).
fn seg(s: &str) -> Result<&str, AbpError> {
    if !s.is_empty() && s.len() <= 128 && s.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-')) {
        Ok(s)
    } else {
        Err(AbpError::Usage(format!("{s:?} is not a module id")))
    }
}

/// Split `execute`'s args the way a shell would for simple cases: whitespace, with "double quotes" grouping.
pub fn split_args(args: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut quoted = false;
    let mut started = false;
    for c in args.chars() {
        match c {
            '"' => {
                quoted = !quoted;
                started = true;
            }
            c if c.is_whitespace() && !quoted => {
                if started {
                    out.push(std::mem::take(&mut cur));
                    started = false;
                }
            }
            c => {
                cur.push(c);
                started = true;
            }
        }
    }
    if started {
        out.push(cur);
    }
    out
}

fn now() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

/// ABP as an Omnisystem module.
pub struct AbpModule {
    client: AbpClient,
    meta: ModuleMetadata,
    status: ModuleStatus,
    config: ModuleConfig,
    started: u64,
    healthy: AtomicBool,
    checked: AtomicU64,
    errors: AtomicU64,
}

impl AbpModule {
    pub fn new(client: AbpClient) -> Self {
        let mut extra = HashMap::new();
        extra.insert("abp_url".into(), json!(client.base_url()));
        extra.insert("key_preset".into(), json!("omnisystem"));
        Self {
            meta: ModuleMetadata {
                id: "abp".into(),
                name: "AgenticBotPlatform".into(),
                version: env!("CARGO_PKG_VERSION").into(),
                description: "ABP's agents, bots, swarms, model gateway, modules, local AI and Neural Lab".into(),
                module_type: ModuleType::AppModule,
                author: Some("LoopyLuci".into()),
                license: Some("Apache-2.0".into()),
                capabilities: COMMANDS.iter().map(|c| format!("abp.{c}")).collect(),
                dependencies: vec![],
                tags: vec!["agents".into(), "ai".into(), "llm".into(), "automation".into()],
                metadata: extra,
            },
            client,
            status: ModuleStatus::Unloaded,
            config: ModuleConfig { settings: HashMap::new() },
            started: 0,
            healthy: AtomicBool::new(false),
            checked: AtomicU64::new(0),
            errors: AtomicU64::new(0),
        }
    }

    pub fn from_env() -> Self {
        Self::new(AbpClient::from_env())
    }

    pub fn client(&self) -> &AbpClient {
        &self.client
    }

    fn observe<T>(&self, r: &Result<T, AbpError>) {
        self.checked.store(now(), Ordering::Relaxed);
        match r {
            Ok(_) | Err(AbpError::Usage(_)) | Err(AbpError::Http { .. }) => self.healthy.store(true, Ordering::Relaxed),
            Err(_) => {
                self.healthy.store(false, Ordering::Relaxed);
                self.errors.fetch_add(1, Ordering::Relaxed);
            }
        }
    }
}

#[async_trait]
impl ModuleInterface for AbpModule {
    fn id(&self) -> &str {
        &self.meta.id
    }
    fn version(&self) -> &str {
        &self.meta.version
    }
    fn name(&self) -> &str {
        &self.meta.name
    }
    fn description(&self) -> &str {
        &self.meta.description
    }
    fn module_type(&self) -> ModuleType {
        ModuleType::AppModule
    }
    fn capabilities(&self) -> Vec<String> {
        self.meta.capabilities.clone()
    }
    fn dependencies(&self) -> Vec<ModuleDependency> {
        vec![]
    }

    async fn initialize(&mut self) -> Result<(), ModuleError> {
        self.status = ModuleStatus::Loading;
        let r = self.client.get("/healthz").await;
        self.observe(&r);
        match r {
            Ok(_) => {
                self.status = ModuleStatus::Running;
                self.started = now();
                Ok(())
            }
            Err(e) => {
                self.status = ModuleStatus::Error;
                Err(ModuleError::InitializationFailed(e.to_string()))
            }
        }
    }

    async fn execute(&self, command: &str, args: &str) -> Result<String, ModuleError> {
        let r = self.client.run(command, &split_args(args)).await;
        self.observe(&r);
        Ok(serde_json::to_string_pretty(&r?).unwrap_or_default())
    }

    async fn shutdown(&mut self) -> Result<(), ModuleError> {
        self.status = ModuleStatus::Unloaded;
        Ok(())
    }

    fn status(&self) -> ModuleStatus {
        self.status
    }
    fn metadata(&self) -> &ModuleMetadata {
        &self.meta
    }

    /// The last thing seen of ABP (initialize and every command check it); no network call here, since the trait's
    /// health check is synchronous.
    fn health_check(&self) -> HealthStatus {
        HealthStatus {
            healthy: self.healthy.load(Ordering::Relaxed) && self.status == ModuleStatus::Running,
            last_check: self.checked.load(Ordering::Relaxed),
            uptime_seconds: if self.started > 0 { now().saturating_sub(self.started) } else { 0 },
            error_count: self.errors.load(Ordering::Relaxed),
        }
    }

    async fn configure(&mut self, config: ModuleConfig) -> Result<(), ModuleError> {
        let url = config.settings.get("abp_url").and_then(Value::as_str).map(str::to_string);
        let key = config.settings.get("abp_key").and_then(Value::as_str).map(str::to_string);
        if url.is_some() || key.is_some() {
            let base = url.unwrap_or_else(|| self.client.base.clone());
            self.client = AbpClient::new(base.clone(), key.or_else(|| self.client.key.clone()));
            self.meta.metadata.insert("abp_url".into(), json!(base));
        }
        let mut stored = config;
        stored.settings.remove("abp_key"); // a secret is used, never kept where get_config would hand it out
        self.config = stored;
        Ok(())
    }

    fn get_config(&self) -> ModuleConfig {
        self.config.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn args_split_like_a_shell() {
        assert_eq!(split_args(r#"main helper "two words" x"#), vec!["main", "helper", "two words", "x"]);
        assert_eq!(split_args("  "), Vec::<String>::new());
        assert_eq!(split_args(r#""""#), vec![""]);
    }

    #[test]
    fn module_ids_cannot_escape_the_route() {
        assert!(seg("kotmoe").is_ok());
        assert!(seg("../config").is_err());
        assert!(seg("a/b").is_err());
        assert!(seg("").is_err());
    }

    #[test]
    fn the_key_is_never_shown() {
        let c = AbpClient::new("http://127.0.0.1:1/", Some("unused".into()));
        let shown = format!("{c:?}");
        assert!(!shown.contains("unused") && shown.contains("<set>"));
        assert_eq!(c.base_url(), "http://127.0.0.1:1");
    }

    #[tokio::test]
    async fn usage_errors_say_what_to_type() {
        let c = AbpClient::new("http://127.0.0.1:1", None);
        let e = c.run("ask", &["only-one".into()]).await.unwrap_err().to_string();
        assert!(e.contains("<from-bot> <to-bot> <prompt>"), "{e}");
        let e = c.run("nope", &[]).await.unwrap_err().to_string();
        assert!(e.contains("unknown command") && e.contains("status"), "{e}");
        let e = c.run("raw", &["GET".into(), "api".into()]).await.unwrap_err().to_string();
        assert!(e.contains("starts with /"), "{e}");
    }

    #[tokio::test]
    async fn an_unreachable_abp_makes_the_module_unhealthy() {
        let mut m = AbpModule::new(AbpClient::new("http://127.0.0.1:1", None));
        assert!(m.initialize().await.is_err());
        assert_eq!(m.status(), ModuleStatus::Error);
        let h = m.health_check();
        assert!(!h.healthy && h.error_count == 1 && h.last_check > 0);
    }

    #[tokio::test]
    async fn configure_keeps_no_secret() {
        let mut m = AbpModule::new(AbpClient::new("http://127.0.0.1:1", None));
        let mut s = HashMap::new();
        s.insert("abp_url".to_string(), json!("http://127.0.0.1:2"));
        s.insert("abp_key".to_string(), json!("unused"));
        m.configure(ModuleConfig { settings: s }).await.unwrap();
        assert_eq!(m.client().base_url(), "http://127.0.0.1:2");
        assert!(!m.get_config().settings.contains_key("abp_key"));
    }

    /// Against a real ABP: set ABP_URL and ABP_KEY (an "omnisystem" integration key), then
    /// `cargo test -p abp-connector -- --ignored`.
    #[tokio::test]
    #[ignore]
    async fn live_abp() {
        let mut m = AbpModule::from_env();
        m.initialize().await.expect("ABP reachable");
        let st: Value = serde_json::from_str(&m.execute("status", "").await.unwrap()).unwrap();
        assert!(st.get("health").is_some());
        let bots: Value = serde_json::from_str(&m.execute("bots", "").await.unwrap()).unwrap();
        assert!(bots.is_array() || bots.is_object());
        assert!(m.health_check().healthy);
    }
}
