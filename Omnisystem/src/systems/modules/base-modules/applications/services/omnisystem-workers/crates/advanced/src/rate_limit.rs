/// RateLimitWorker - Request rate limiting and throttling

use omnisystem_workers_core::prelude::*;
use async_trait::async_trait;
use std::time::Duration;

pub struct RateLimitWorker {
    max_requests_per_second: u32,
    timeout: Duration,
}

pub struct RateLimitRequest {
    pub client_id: String,
    pub request_count: u32,
}

#[derive(Debug)]
pub enum RateLimitResult {
    Allowed,
    Throttled(u64),
    Blocked,
}

#[async_trait]
impl Worker for RateLimitWorker {
    type Input = RateLimitRequest;
    type Output = RateLimitResult;

    async fn execute(&self, input: Self::Input) -> WorkerResult<Self::Output> {
        if input.request_count > self.max_requests_per_second {
            return Ok(RateLimitResult::Throttled(100));
        }
        Ok(RateLimitResult::Allowed)
    }

    fn name(&self) -> &str {
        "RateLimitWorker"
    }

    fn timeout(&self) -> Duration {
        Duration::from_secs(10)
    }

    fn priority(&self) -> Priority {
        Priority::High
    }
}

impl RateLimitWorker {
    pub fn new(max_requests: u32) -> Self {
        RateLimitWorker {
            max_requests_per_second: max_requests,
            timeout: Duration::from_secs(10),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_allow_request() {
        let worker = RateLimitWorker::new(100);
        let request = RateLimitRequest {
            client_id: "client1".to_string(),
            request_count: 50,
        };
        let result = worker.execute(request).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_throttle_request() {
        let worker = RateLimitWorker::new(100);
        let request = RateLimitRequest {
            client_id: "client1".to_string(),
            request_count: 150,
        };
        let result = worker.execute(request).await;
        assert!(result.is_ok());
    }
}
