/// KeyboardWorker - Keyboard input and key event handling

use omnisystem_workers_core::prelude::*;
use async_trait::async_trait;
use std::time::Duration;

pub struct KeyboardWorker {
    timeout: Duration,
}

pub struct KeyboardRequest {
    pub event: KeyEvent,
}

#[derive(Debug, Clone)]
pub enum KeyEvent {
    KeyDown(char),
    KeyUp(char),
    Macro(String),
    LedControl(u8),
}

#[derive(Debug)]
pub enum KeyboardResult {
    KeyProcessed(char),
    KeyReleased(char),
    MacroExecuted,
    LedSet,
}

#[async_trait]
impl Worker for KeyboardWorker {
    type Input = KeyboardRequest;
    type Output = KeyboardResult;

    async fn execute(&self, input: Self::Input) -> WorkerResult<Self::Output> {
        match input.event {
            KeyEvent::KeyDown(ch) => Ok(KeyboardResult::KeyProcessed(ch)),
            KeyEvent::KeyUp(ch) => Ok(KeyboardResult::KeyReleased(ch)),
            KeyEvent::Macro(_) => Ok(KeyboardResult::MacroExecuted),
            KeyEvent::LedControl(_) => Ok(KeyboardResult::LedSet),
        }
    }

    fn name(&self) -> &str {
        "KeyboardWorker"
    }

    fn timeout(&self) -> Duration {
        Duration::from_secs(5)
    }

    fn priority(&self) -> Priority {
        Priority::High
    }
}

impl KeyboardWorker {
    pub fn new() -> Self {
        KeyboardWorker {
            timeout: Duration::from_secs(5),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_key_down() {
        let worker = KeyboardWorker::new();
        let request = KeyboardRequest {
            event: KeyEvent::KeyDown('A'),
        };
        let result = worker.execute(request).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_key_up() {
        let worker = KeyboardWorker::new();
        let request = KeyboardRequest {
            event: KeyEvent::KeyUp('A'),
        };
        let result = worker.execute(request).await;
        assert!(result.is_ok());
    }
}
