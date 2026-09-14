/// EventLoopWorker - Asynchronous event handling

use omnisystem_workers_core::prelude::*;
use async_trait::async_trait;
use std::time::Duration;
use std::collections::VecDeque;

pub struct EventLoopWorker {
    timeout: Duration,
}

pub struct EventRequest {
    pub event_type: EventType,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone)]
pub enum EventType {
    Register,
    Dispatch,
    Wait,
    Cancel,
}

#[derive(Debug)]
pub enum EventResult {
    Registered,
    Dispatched(usize),
    Waited,
    Cancelled,
}

#[async_trait]
impl Worker for EventLoopWorker {
    type Input = EventRequest;
    type Output = EventResult;

    async fn execute(&self, input: Self::Input) -> WorkerResult<Self::Output> {
        match input.event_type {
            EventType::Register => Ok(EventResult::Registered),
            EventType::Dispatch => Ok(EventResult::Dispatched(input.data.len())),
            EventType::Wait => Ok(EventResult::Waited),
            EventType::Cancel => Ok(EventResult::Cancelled),
        }
    }

    fn name(&self) -> &str {
        "EventLoopWorker"
    }

    fn timeout(&self) -> Duration {
        Duration::from_secs(30)
    }

    fn priority(&self) -> Priority {
        Priority::High
    }
}

impl EventLoopWorker {
    pub fn new() -> Self {
        EventLoopWorker {
            timeout: Duration::from_secs(30),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_event_register() {
        let worker = EventLoopWorker::new();
        let request = EventRequest {
            event_type: EventType::Register,
            data: vec![],
        };
        let result = worker.execute(request).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_event_dispatch() {
        let worker = EventLoopWorker::new();
        let request = EventRequest {
            event_type: EventType::Dispatch,
            data: vec![1, 2, 3, 4, 5],
        };
        let result = worker.execute(request).await;
        assert!(result.is_ok());
    }
}
