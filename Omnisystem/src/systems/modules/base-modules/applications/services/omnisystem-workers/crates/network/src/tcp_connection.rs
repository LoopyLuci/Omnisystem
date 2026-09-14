/// TCPConnectionWorker - TCP connection lifecycle

use omnisystem_workers_core::prelude::*;
use async_trait::async_trait;
use std::time::Duration;

pub struct TCPConnectionWorker;

pub struct TCPRequest {
    pub host: String,
    pub port: u16,
}

#[async_trait]
impl Worker for TCPConnectionWorker {
    type Input = TCPRequest;
    type Output = String;

    async fn execute(&self, request: Self::Input) -> WorkerResult<Self::Output> {
        match tokio::net::TcpStream::connect(format!("{}:{}", request.host, request.port)).await {
            Ok(_stream) => Ok("Connected".to_string()),
            Err(e) => Err(WorkerError::ExecutionFailed(e.to_string())),
        }
    }

    fn name(&self) -> &str {
        "TCPConnectionWorker"
    }

    fn timeout(&self) -> Duration {
        Duration::from_secs(10)
    }
}
