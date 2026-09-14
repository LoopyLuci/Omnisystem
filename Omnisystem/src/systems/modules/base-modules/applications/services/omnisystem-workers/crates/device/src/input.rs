/// InputWorker - Keyboard, mouse, touch input

use omnisystem_workers_core::prelude::*;
use async_trait::async_trait;
use std::time::Duration;

pub struct InputWorker;

pub enum InputEvent {
    Keyboard(String),
    Mouse { x: i32, y: i32 },
    Touch { x: f32, y: f32 },
}

#[async_trait]
impl Worker for InputWorker {
    type Input = InputEvent;
    type Output = String;

    async fn execute(&self, event: Self::Input) -> WorkerResult<Self::Output> {
        match event {
            InputEvent::Keyboard(key) => Ok(format!("Key: {}", key)),
            InputEvent::Mouse { x, y } => Ok(format!("Mouse: ({}, {})", x, y)),
            InputEvent::Touch { x, y } => Ok(format!("Touch: ({:.2}, {:.2})", x, y)),
        }
    }

    fn name(&self) -> &str {
        "InputWorker"
    }

    fn timeout(&self) -> Duration {
        Duration::from_millis(100)
    }

    fn priority(&self) -> Priority {
        Priority::High
    }
}
