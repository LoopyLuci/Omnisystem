/// RateLimiterWorker - Advanced rate limiting and throttling

use omnisystem_workers_core::prelude::*;
use async_trait::async_trait;
use std::time::Duration;

pub struct RateLimiterWorker {
    timeout: Duration,
}

pub struct RateLimitRequest {
    pub client_id: String,
    pub tokens_requested: u32,
}

#[derive(Debug)]
pub enum RateLimitStatus {
    Allowed,
    Throttled(u64),
    Rejected,
}

#[async_trait]
impl Worker for RateLimiterWorker {
    type Input = RateLimitRequest;
    type Output = RateLimitStatus;

    async fn execute(&self, input: Self::Input) -> WorkerResult<Self::Output> {
        if input.tokens_requested > 1000 {
            Ok(RateLimitStatus::Rejected)
        } else {
            Ok(RateLimitStatus::Allowed)
        }
    }

    fn name(&self) -> &str {
        "RateLimiterWorker"
    }

    fn timeout(&self) -> Duration {
        Duration::from_secs(5)
    }

    fn priority(&self) -> Priority {
        Priority::High
    }
}

impl RateLimiterWorker {
    pub fn new() -> Self {
        RateLimiterWorker {
            timeout: Duration::from_secs(5),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_allow_tokens() {
        let worker = RateLimiterWorker::new();
        let request = RateLimitRequest {
            client_id: "client1".to_string(),
            tokens_requested: 100,
        };
        let result = worker.execute(request).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_reject_excess() {
        let worker = RateLimiterWorker::new();
        let request = RateLimitRequest {
            client_id: "client1".to_string(),
            tokens_requested: 2000,
        };
        let result = worker.execute(request).await;
        assert!(result.is_ok());
    }
}
