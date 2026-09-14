/// USBWorker - USB device handling

use omnisystem_workers_core::prelude::*;
use async_trait::async_trait;
use std::time::Duration;

pub struct USBWorker {
    timeout: Duration,
}

pub struct USBRequest {
    pub device_path: String,
    pub operation: USBOperation,
}

#[derive(Debug, Clone)]
pub enum USBOperation {
    Enumerate,
    Open,
    Close,
    Send(Vec<u8>),
    Receive(usize),
    Reset,
}

#[derive(Debug)]
pub enum USBResult {
    Devices(Vec<String>),
    Opened,
    Closed,
    Sent(usize),
    Received(Vec<u8>),
    Reset,
}

#[async_trait]
impl Worker for USBWorker {
    type Input = USBRequest;
    type Output = USBResult;

    async fn execute(&self, input: Self::Input) -> WorkerResult<Self::Output> {
        match input.operation {
            USBOperation::Enumerate => Ok(USBResult::Devices(vec![])),
            USBOperation::Open => Ok(USBResult::Opened),
            USBOperation::Close => Ok(USBResult::Closed),
            USBOperation::Send(data) => Ok(USBResult::Sent(data.len())),
            USBOperation::Receive(size) => Ok(USBResult::Received(vec![0u8; size])),
            USBOperation::Reset => Ok(USBResult::Reset),
        }
    }

    fn name(&self) -> &str {
        "USBWorker"
    }

    fn timeout(&self) -> Duration {
        Duration::from_secs(30)
    }

    fn priority(&self) -> Priority {
        Priority::High
    }
}

impl USBWorker {
    pub fn new() -> Self {
        USBWorker {
            timeout: Duration::from_secs(30),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_usb_enumerate() {
        let worker = USBWorker::new();
        let request = USBRequest {
            device_path: "/dev/usb0".to_string(),
            operation: USBOperation::Enumerate,
        };
        let result = worker.execute(request).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_usb_open() {
        let worker = USBWorker::new();
        let request = USBRequest {
            device_path: "/dev/usb0".to_string(),
            operation: USBOperation::Open,
        };
        let result = worker.execute(request).await;
        assert!(result.is_ok());
    }
}
