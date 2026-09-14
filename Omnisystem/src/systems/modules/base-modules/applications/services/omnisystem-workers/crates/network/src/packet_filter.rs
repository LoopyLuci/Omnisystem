/// PacketFilterWorker - Network packet filtering

use omnisystem_workers_core::prelude::*;
use async_trait::async_trait;
use std::time::Duration;

pub struct PacketFilterWorker {
    timeout: Duration,
}

pub struct PacketFilterRequest {
    pub packet_data: Vec<u8>,
    pub action: FilterAction,
}

#[derive(Debug, Clone)]
pub enum FilterAction {
    Accept,
    Reject,
    Inspect,
    Drop,
}

#[derive(Debug)]
pub enum PacketFilterResult {
    Accepted,
    Rejected,
    Inspected(usize),
    Dropped,
}

#[async_trait]
impl Worker for PacketFilterWorker {
    type Input = PacketFilterRequest;
    type Output = PacketFilterResult;

    async fn execute(&self, input: Self::Input) -> WorkerResult<Self::Output> {
        match input.action {
            FilterAction::Accept => Ok(PacketFilterResult::Accepted),
            FilterAction::Reject => Ok(PacketFilterResult::Rejected),
            FilterAction::Inspect => Ok(PacketFilterResult::Inspected(input.packet_data.len())),
            FilterAction::Drop => Ok(PacketFilterResult::Dropped),
        }
    }

    fn name(&self) -> &str {
        "PacketFilterWorker"
    }

    fn timeout(&self) -> Duration {
        Duration::from_secs(5)
    }

    fn priority(&self) -> Priority {
        Priority::High
    }
}

impl PacketFilterWorker {
    pub fn new() -> Self {
        PacketFilterWorker {
            timeout: Duration::from_secs(5),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_accept_packet() {
        let worker = PacketFilterWorker::new();
        let request = PacketFilterRequest {
            packet_data: vec![1, 2, 3, 4, 5],
            action: FilterAction::Accept,
        };
        let result = worker.execute(request).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_inspect_packet() {
        let worker = PacketFilterWorker::new();
        let request = PacketFilterRequest {
            packet_data: vec![1, 2, 3],
            action: FilterAction::Inspect,
        };
        let result = worker.execute(request).await;
        assert!(result.is_ok());
    }
}
