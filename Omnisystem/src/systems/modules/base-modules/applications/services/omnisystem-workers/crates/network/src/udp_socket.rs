/// UDPSocketWorker - UDP packet operations

use omnisystem_workers_core::prelude::*;
use async_trait::async_trait;
use std::time::Duration;

pub struct UDPSocketWorker;

pub struct UDPPacket {
    pub source: String,
    pub destination: String,
    pub port: u16,
    pub data: Vec<u8>,
}

#[async_trait]
impl Worker for UDPSocketWorker {
    type Input = UDPPacket;
    type Output = usize;

    async fn execute(&self, packet: Self::Input) -> WorkerResult<Self::Output> {
        if packet.data.is_empty() {
            return Err(WorkerError::ExecutionFailed("Empty packet".to_string()));
        }

        Ok(packet.data.len())
    }

    fn name(&self) -> &str {
        "UDPSocketWorker"
    }

    fn timeout(&self) -> Duration {
        Duration::from_secs(5)
    }

    fn priority(&self) -> Priority {
        Priority::High
    }
}
