/// VPNWorker - VPN connection management and tunneling

use omnisystem_workers_core::prelude::*;
use async_trait::async_trait;
use std::time::Duration;

pub struct VPNWorker {
    timeout: Duration,
}

pub struct VPNRequest {
    pub server: String,
    pub operation: VPNOperation,
}

#[derive(Debug, Clone)]
pub enum VPNOperation {
    Connect,
    Disconnect,
    Status,
    SetProtocol(String),
}

#[derive(Debug)]
pub enum VPNResult {
    Connected,
    Disconnected,
    Active,
    ProtocolSet,
}

#[async_trait]
impl Worker for VPNWorker {
    type Input = VPNRequest;
    type Output = VPNResult;

    async fn execute(&self, input: Self::Input) -> WorkerResult<Self::Output> {
        if input.server.is_empty() {
            return Err(WorkerError::ExecutionFailed("No server specified".to_string()));
        }

        match input.operation {
            VPNOperation::Connect => Ok(VPNResult::Connected),
            VPNOperation::Disconnect => Ok(VPNResult::Disconnected),
            VPNOperation::Status => Ok(VPNResult::Active),
            VPNOperation::SetProtocol(_) => Ok(VPNResult::ProtocolSet),
        }
    }

    fn name(&self) -> &str {
        "VPNWorker"
    }

    fn timeout(&self) -> Duration {
        Duration::from_secs(30)
    }

    fn priority(&self) -> Priority {
        Priority::High
    }
}

impl VPNWorker {
    pub fn new() -> Self {
        VPNWorker {
            timeout: Duration::from_secs(30),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_vpn_connect() {
        let worker = VPNWorker::new();
        let request = VPNRequest {
            server: "vpn.example.com".to_string(),
            operation: VPNOperation::Connect,
        };
        let result = worker.execute(request).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_vpn_status() {
        let worker = VPNWorker::new();
        let request = VPNRequest {
            server: "vpn.example.com".to_string(),
            operation: VPNOperation::Status,
        };
        let result = worker.execute(request).await;
        assert!(result.is_ok());
    }
}
