/// StreamingWorker - Real-time data streaming and processing

use omnisystem_workers_core::prelude::*;
use async_trait::async_trait;
use std::time::Duration;

pub struct StreamingWorker {
    timeout: Duration,
}

pub struct StreamingRequest {
    pub stream_id: String,
    pub operation: StreamOp,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone)]
pub enum StreamOp {
    StartStream,
    ProcessChunk,
    Flush,
    EndStream,
}

#[derive(Debug)]
pub enum StreamingResult {
    Started,
    Processed(usize),
    Flushed,
    Ended,
}

#[async_trait]
impl Worker for StreamingWorker {
    type Input = StreamingRequest;
    type Output = StreamingResult;

    async fn execute(&self, input: Self::Input) -> WorkerResult<Self::Output> {
        match input.operation {
            StreamOp::StartStream => Ok(StreamingResult::Started),
            StreamOp::ProcessChunk => Ok(StreamingResult::Processed(input.data.len())),
            StreamOp::Flush => Ok(StreamingResult::Flushed),
            StreamOp::EndStream => Ok(StreamingResult::Ended),
        }
    }

    fn name(&self) -> &str {
        "StreamingWorker"
    }

    fn timeout(&self) -> Duration {
        Duration::from_secs(60)
    }

    fn priority(&self) -> Priority {
        Priority::Normal
    }
}

impl StreamingWorker {
    pub fn new() -> Self {
        StreamingWorker {
            timeout: Duration::from_secs(60),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_start_stream() {
        let worker = StreamingWorker::new();
        let request = StreamingRequest {
            stream_id: "stream1".to_string(),
            operation: StreamOp::StartStream,
            data: vec![],
        };
        let result = worker.execute(request).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_process_chunk() {
        let worker = StreamingWorker::new();
        let request = StreamingRequest {
            stream_id: "stream1".to_string(),
            operation: StreamOp::ProcessChunk,
            data: vec![1, 2, 3, 4, 5],
        };
        let result = worker.execute(request).await;
        assert!(result.is_ok());
    }
}
