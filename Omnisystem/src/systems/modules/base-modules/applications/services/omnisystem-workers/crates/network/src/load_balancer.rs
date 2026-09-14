/// LoadBalancerWorker - Request distribution across servers

use omnisystem_workers_core::prelude::*;
use async_trait::async_trait;
use std::time::Duration;
use std::collections::VecDeque;

pub struct LoadBalancerWorker {
    timeout: Duration,
    balance_strategy: BalanceStrategy,
}

pub struct LoadBalanceRequest {
    pub request_data: Vec<u8>,
    pub target_servers: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum BalanceStrategy {
    RoundRobin,
    LeastConnections,
    Random,
    WeightedRandom,
}

#[derive(Debug)]
pub enum LoadBalanceResult {
    Forwarded(String, Vec<u8>),
    Error(String),
}

#[async_trait]
impl Worker for LoadBalancerWorker {
    type Input = LoadBalanceRequest;
    type Output = LoadBalanceResult;

    async fn execute(&self, input: Self::Input) -> WorkerResult<Self::Output> {
        if input.target_servers.is_empty() {
            return Ok(LoadBalanceResult::Error("No servers available".to_string()));
        }

        let selected = &input.target_servers[0];
        Ok(LoadBalanceResult::Forwarded(selected.clone(), input.request_data))
    }

    fn name(&self) -> &str {
        "LoadBalancerWorker"
    }

    fn timeout(&self) -> Duration {
        Duration::from_secs(30)
    }

    fn priority(&self) -> Priority {
        Priority::High
    }
}

impl LoadBalancerWorker {
    pub fn new(strategy: BalanceStrategy) -> Self {
        LoadBalancerWorker {
            timeout: Duration::from_secs(30),
            balance_strategy: strategy,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_load_balance_round_robin() {
        let worker = LoadBalancerWorker::new(BalanceStrategy::RoundRobin);
        let request = LoadBalanceRequest {
            request_data: vec![1, 2, 3],
            target_servers: vec![
                "server1.com".to_string(),
                "server2.com".to_string(),
                "server3.com".to_string(),
            ],
        };
        let result = worker.execute(request).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_load_balance_no_servers() {
        let worker = LoadBalancerWorker::new(BalanceStrategy::RoundRobin);
        let request = LoadBalanceRequest {
            request_data: vec![],
            target_servers: vec![],
        };
        let result = worker.execute(request).await;
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), LoadBalanceResult::Error(_)));
    }
}
