/// PubSubWorker - Publish-Subscribe message distribution

use omnisystem_workers_core::prelude::*;
use async_trait::async_trait;
use std::time::Duration;

pub struct PubSubWorker {
    timeout: Duration,
}

pub struct PubSubRequest {
    pub topic: String,
    pub message: Vec<u8>,
    pub operation: PubSubOp,
}

#[derive(Debug, Clone)]
pub enum PubSubOp {
    Publish,
    Subscribe,
    Unsubscribe,
    Broadcast,
}

#[derive(Debug)]
pub enum PubSubResult {
    Published(usize),
    Subscribed,
    Unsubscribed,
    Broadcasted(u32),
}

#[async_trait]
impl Worker for PubSubWorker {
    type Input = PubSubRequest;
    type Output = PubSubResult;

    async fn execute(&self, input: Self::Input) -> WorkerResult<Self::Output> {
        match input.operation {
            PubSubOp::Publish => Ok(PubSubResult::Published(input.message.len())),
            PubSubOp::Subscribe => Ok(PubSubResult::Subscribed),
            PubSubOp::Unsubscribe => Ok(PubSubResult::Unsubscribed),
            PubSubOp::Broadcast => Ok(PubSubResult::Broadcasted(1)),
        }
    }

    fn name(&self) -> &str {
        "PubSubWorker"
    }

    fn timeout(&self) -> Duration {
        Duration::from_secs(30)
    }

    fn priority(&self) -> Priority {
        Priority::Normal
    }
}

impl PubSubWorker {
    pub fn new() -> Self {
        PubSubWorker {
            timeout: Duration::from_secs(30),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_publish() {
        let worker = PubSubWorker::new();
        let request = PubSubRequest {
            topic: "events".to_string(),
            message: vec![1, 2, 3],
            operation: PubSubOp::Publish,
        };
        let result = worker.execute(request).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_subscribe() {
        let worker = PubSubWorker::new();
        let request = PubSubRequest {
            topic: "events".to_string(),
            message: vec![],
            operation: PubSubOp::Subscribe,
        };
        let result = worker.execute(request).await;
        assert!(result.is_ok());
    }
}
