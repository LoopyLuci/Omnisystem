/// CameraWorker - Camera capture and processing

use omnisystem_workers_core::prelude::*;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameInfo {
    pub width: u32,
    pub height: u32,
    pub format: String,
    pub timestamp_ms: u64,
}

pub struct CameraWorker;

pub enum CameraOperation {
    StartCapture { width: u32, height: u32, fps: u32 },
    CaptureFrame,
    StopCapture,
    GetProperties,
}

#[async_trait]
impl Worker for CameraWorker {
    type Input = CameraOperation;
    type Output = String;

    async fn execute(&self, op: Self::Input) -> WorkerResult<Self::Output> {
        match op {
            CameraOperation::StartCapture { width, height, fps } => {
                Ok(format!("Camera capture started: {}x{} @ {}fps", width, height, fps))
            },
            CameraOperation::CaptureFrame => {
                Ok("Frame captured".to_string())
            },
            CameraOperation::StopCapture => {
                Ok("Camera stopped".to_string())
            },
            CameraOperation::GetProperties => {
                Ok("Camera ready".to_string())
            }
        }
    }

    fn name(&self) -> &str {
        "CameraWorker"
    }

    fn timeout(&self) -> Duration {
        Duration::from_secs(10)
    }

    fn priority(&self) -> Priority {
        Priority::High
    }
}
