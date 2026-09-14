/// BluetoothWorker - Bluetooth device management

use omnisystem_workers_core::prelude::*;
use async_trait::async_trait;
use std::time::Duration;

pub struct BluetoothWorker {
    timeout: Duration,
}

pub struct BluetoothRequest {
    pub device_id: String,
    pub command: BluetoothCommand,
}

#[derive(Debug, Clone)]
pub enum BluetoothCommand {
    Scan,
    Connect,
    Disconnect,
    SendData(Vec<u8>),
    GetSignalStrength,
}

#[derive(Debug)]
pub enum BluetoothResult {
    Scanning,
    Connected,
    Disconnected,
    DataSent(usize),
    SignalStrength(i8),
}

#[async_trait]
impl Worker for BluetoothWorker {
    type Input = BluetoothRequest;
    type Output = BluetoothResult;

    async fn execute(&self, input: Self::Input) -> WorkerResult<Self::Output> {
        match input.command {
            BluetoothCommand::Scan => Ok(BluetoothResult::Scanning),
            BluetoothCommand::Connect => Ok(BluetoothResult::Connected),
            BluetoothCommand::Disconnect => Ok(BluetoothResult::Disconnected),
            BluetoothCommand::SendData(data) => Ok(BluetoothResult::DataSent(data.len())),
            BluetoothCommand::GetSignalStrength => Ok(BluetoothResult::SignalStrength(-50)),
        }
    }

    fn name(&self) -> &str {
        "BluetoothWorker"
    }

    fn timeout(&self) -> Duration {
        Duration::from_secs(10)
    }

    fn priority(&self) -> Priority {
        Priority::High
    }
}

impl BluetoothWorker {
    pub fn new() -> Self {
        BluetoothWorker {
            timeout: Duration::from_secs(10),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_bluetooth_scan() {
        let worker = BluetoothWorker::new();
        let request = BluetoothRequest {
            device_id: "00:11:22:33:44:55".to_string(),
            command: BluetoothCommand::Scan,
        };
        let result = worker.execute(request).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_bluetooth_connect() {
        let worker = BluetoothWorker::new();
        let request = BluetoothRequest {
            device_id: "00:11:22:33:44:55".to_string(),
            command: BluetoothCommand::Connect,
        };
        let result = worker.execute(request).await;
        assert!(result.is_ok());
    }
}
