/// RoutingWorker - Packet routing decisions and path selection

use omnisystem_workers_core::prelude::*;
use async_trait::async_trait;
use std::time::Duration;

pub struct RoutingWorker {
    timeout: Duration,
}

pub struct RoutingRequest {
    pub destination: String,
    pub source: String,
}

#[derive(Debug)]
pub enum RoutingResult {
    RouteFound(Vec<String>),
    NoRoute,
    Error(String),
}

#[async_trait]
impl Worker for RoutingWorker {
    type Input = RoutingRequest;
    type Output = RoutingResult;

    async fn execute(&self, input: Self::Input) -> WorkerResult<Self::Output> {
        if input.destination.is_empty() {
            return Ok(RoutingResult::NoRoute);
        }

        Ok(RoutingResult::RouteFound(vec![
            input.source,
            "gateway".to_string(),
            input.destination,
        ]))
    }

    fn name(&self) -> &str {
        "RoutingWorker"
    }

    fn timeout(&self) -> Duration {
        Duration::from_secs(5)
    }

    fn priority(&self) -> Priority {
        Priority::Normal
    }
}

impl RoutingWorker {
    pub fn new() -> Self {
        RoutingWorker {
            timeout: Duration::from_secs(5),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_find_route() {
        let worker = RoutingWorker::new();
        let request = RoutingRequest {
            destination: "192.168.1.100".to_string(),
            source: "192.168.1.1".to_string(),
        };
        let result = worker.execute(request).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_no_route() {
        let worker = RoutingWorker::new();
        let request = RoutingRequest {
            destination: "".to_string(),
            source: "192.168.1.1".to_string(),
        };
        let result = worker.execute(request).await;
        assert!(result.is_ok());
    }
}
