/// FirewallWorker - Firewall rule management and enforcement

use omnisystem_workers_core::prelude::*;
use async_trait::async_trait;
use std::time::Duration;

pub struct FirewallWorker {
    timeout: Duration,
}

pub struct FirewallRequest {
    pub rule: FirewallRule,
    pub action: RuleAction,
}

#[derive(Debug, Clone)]
pub struct FirewallRule {
    pub source_ip: String,
    pub dest_ip: String,
    pub port: u16,
    pub protocol: String,
}

#[derive(Debug, Clone)]
pub enum RuleAction {
    Allow,
    Block,
    Log,
    Drop,
}

#[derive(Debug)]
pub enum FirewallResult {
    RuleAdded,
    RuleRemoved,
    RuleBlocked,
    Logged,
}

#[async_trait]
impl Worker for FirewallWorker {
    type Input = FirewallRequest;
    type Output = FirewallResult;

    async fn execute(&self, input: Self::Input) -> WorkerResult<Self::Output> {
        match input.action {
            RuleAction::Allow => Ok(FirewallResult::RuleAdded),
            RuleAction::Block => Ok(FirewallResult::RuleBlocked),
            RuleAction::Log => Ok(FirewallResult::Logged),
            RuleAction::Drop => Ok(FirewallResult::RuleRemoved),
        }
    }

    fn name(&self) -> &str {
        "FirewallWorker"
    }

    fn timeout(&self) -> Duration {
        Duration::from_secs(10)
    }

    fn priority(&self) -> Priority {
        Priority::High
    }
}

impl FirewallWorker {
    pub fn new() -> Self {
        FirewallWorker {
            timeout: Duration::from_secs(10),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_add_rule() {
        let worker = FirewallWorker::new();
        let request = FirewallRequest {
            rule: FirewallRule {
                source_ip: "192.168.1.1".to_string(),
                dest_ip: "10.0.0.1".to_string(),
                port: 443,
                protocol: "TCP".to_string(),
            },
            action: RuleAction::Allow,
        };
        let result = worker.execute(request).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_block_rule() {
        let worker = FirewallWorker::new();
        let request = FirewallRequest {
            rule: FirewallRule {
                source_ip: "0.0.0.0".to_string(),
                dest_ip: "0.0.0.0".to_string(),
                port: 0,
                protocol: "ALL".to_string(),
            },
            action: RuleAction::Block,
        };
        let result = worker.execute(request).await;
        assert!(result.is_ok());
    }
}
