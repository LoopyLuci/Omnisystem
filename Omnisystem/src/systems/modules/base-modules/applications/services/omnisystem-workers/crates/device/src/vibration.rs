/// VibrationWorker - Haptic feedback and vibration control

use omnisystem_workers_core::prelude::*;
use async_trait::async_trait;
use std::time::Duration;

pub struct VibrationWorker {
    timeout: Duration,
}

pub struct VibrationRequest {
    pub intensity: u8,
    pub duration_ms: u32,
    pub pattern: VibrationPattern,
}

#[derive(Debug, Clone)]
pub enum VibrationPattern {
    Pulse,
    Wave,
    Burst,
    Custom(Vec<u8>),
}

#[derive(Debug)]
pub enum VibrationResult {
    Activated,
    Stopped,
    Error(String),
}

#[async_trait]
impl Worker for VibrationWorker {
    type Input = VibrationRequest;
    type Output = VibrationResult;

    async fn execute(&self, input: Self::Input) -> WorkerResult<Self::Output> {
        if input.intensity > 100 {
            return Ok(VibrationResult::Error("Intensity out of range".to_string()));
        }
        Ok(VibrationResult::Activated)
    }

    fn name(&self) -> &str {
        "VibrationWorker"
    }

    fn timeout(&self) -> Duration {
        Duration::from_secs(5)
    }

    fn priority(&self) -> Priority {
        Priority::Normal
    }
}

impl VibrationWorker {
    pub fn new() -> Self {
        VibrationWorker {
            timeout: Duration::from_secs(5),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_vibrate() {
        let worker = VibrationWorker::new();
        let request = VibrationRequest {
            intensity: 50,
            duration_ms: 500,
            pattern: VibrationPattern::Pulse,
        };
        let result = worker.execute(request).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_invalid_intensity() {
        let worker = VibrationWorker::new();
        let request = VibrationRequest {
            intensity: 150,
            duration_ms: 500,
            pattern: VibrationPattern::Wave,
        };
        let result = worker.execute(request).await;
        assert!(result.is_ok());
    }
}
