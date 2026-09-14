/// PowerWorker - Power management and energy control

use omnisystem_workers_core::prelude::*;
use async_trait::async_trait;
use std::time::Duration;

pub struct PowerWorker {
    timeout: Duration,
}

pub struct PowerRequest {
    pub action: PowerAction,
    pub delay_ms: u64,
}

#[derive(Debug, Clone)]
pub enum PowerAction {
    Suspend,
    Hibernate,
    Shutdown,
    Reboot,
    QueryBattery,
}

#[derive(Debug)]
pub enum PowerResult {
    Suspended,
    Hibernated,
    ShutdownInitiated,
    RebootInitiated,
    BatteryLevel(u8),
}

#[async_trait]
impl Worker for PowerWorker {
    type Input = PowerRequest;
    type Output = PowerResult;

    async fn execute(&self, input: Self::Input) -> WorkerResult<Self::Output> {
        match input.action {
            PowerAction::Suspend => Ok(PowerResult::Suspended),
            PowerAction::Hibernate => Ok(PowerResult::Hibernated),
            PowerAction::Shutdown => Ok(PowerResult::ShutdownInitiated),
            PowerAction::Reboot => Ok(PowerResult::RebootInitiated),
            PowerAction::QueryBattery => Ok(PowerResult::BatteryLevel(85)),
        }
    }

    fn name(&self) -> &str {
        "PowerWorker"
    }

    fn timeout(&self) -> Duration {
        Duration::from_secs(30)
    }

    fn priority(&self) -> Priority {
        Priority::High
    }
}

impl PowerWorker {
    pub fn new() -> Self {
        PowerWorker {
            timeout: Duration::from_secs(30),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_query_battery() {
        let worker = PowerWorker::new();
        let request = PowerRequest {
            action: PowerAction::QueryBattery,
            delay_ms: 0,
        };
        let result = worker.execute(request).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_suspend() {
        let worker = PowerWorker::new();
        let request = PowerRequest {
            action: PowerAction::Suspend,
            delay_ms: 1000,
        };
        let result = worker.execute(request).await;
        assert!(result.is_ok());
    }
}
