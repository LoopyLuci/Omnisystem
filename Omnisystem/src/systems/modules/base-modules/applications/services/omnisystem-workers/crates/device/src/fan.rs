/// FanWorker - Cooling system and thermal control

use omnisystem_workers_core::prelude::*;
use async_trait::async_trait;
use std::time::Duration;

pub struct FanWorker {
    timeout: Duration,
}

pub struct FanRequest {
    pub operation: FanOperation,
    pub device_id: u32,
}

#[derive(Debug, Clone)]
pub enum FanOperation {
    SetSpeed(u8),
    GetSpeed,
    Enable,
    Disable,
    Auto,
}

#[derive(Debug)]
pub enum FanResult {
    SpeedSet(u8),
    Speed(u8),
    Enabled,
    Disabled,
    Auto,
}

#[async_trait]
impl Worker for FanWorker {
    type Input = FanRequest;
    type Output = FanResult;

    async fn execute(&self, input: Self::Input) -> WorkerResult<Self::Output> {
        match input.operation {
            FanOperation::SetSpeed(speed) => Ok(FanResult::SpeedSet(speed)),
            FanOperation::GetSpeed => Ok(FanResult::Speed(50)),
            FanOperation::Enable => Ok(FanResult::Enabled),
            FanOperation::Disable => Ok(FanResult::Disabled),
            FanOperation::Auto => Ok(FanResult::Auto),
        }
    }

    fn name(&self) -> &str {
        "FanWorker"
    }

    fn timeout(&self) -> Duration {
        Duration::from_secs(10)
    }

    fn priority(&self) -> Priority {
        Priority::Normal
    }
}

impl FanWorker {
    pub fn new() -> Self {
        FanWorker {
            timeout: Duration::from_secs(10),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_set_fan_speed() {
        let worker = FanWorker::new();
        let request = FanRequest {
            operation: FanOperation::SetSpeed(75),
            device_id: 0,
        };
        let result = worker.execute(request).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_fan_speed() {
        let worker = FanWorker::new();
        let request = FanRequest {
            operation: FanOperation::GetSpeed,
            device_id: 0,
        };
        let result = worker.execute(request).await;
        assert!(result.is_ok());
    }
}
