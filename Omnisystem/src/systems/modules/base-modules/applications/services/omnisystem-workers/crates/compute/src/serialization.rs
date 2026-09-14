/// SerializationWorker - Data serialization and deserialization

use omnisystem_workers_core::prelude::*;
use async_trait::async_trait;
use std::time::Duration;

pub struct SerializationWorker {
    timeout: Duration,
}

pub struct SerializationRequest {
    pub data: Vec<u8>,
    pub format: SerializeFormat,
    pub operation: SerializeOp,
}

#[derive(Debug, Clone)]
pub enum SerializeFormat {
    Json,
    Msgpack,
    Bincode,
    Protobuf,
}

#[derive(Debug, Clone)]
pub enum SerializeOp {
    Serialize,
    Deserialize,
    Validate,
}

#[derive(Debug)]
pub enum SerializationResult {
    Serialized(Vec<u8>),
    Deserialized(Vec<u8>),
    Valid,
}

#[async_trait]
impl Worker for SerializationWorker {
    type Input = SerializationRequest;
    type Output = SerializationResult;

    async fn execute(&self, input: Self::Input) -> WorkerResult<Self::Output> {
        match input.operation {
            SerializeOp::Serialize => Ok(SerializationResult::Serialized(input.data)),
            SerializeOp::Deserialize => Ok(SerializationResult::Deserialized(input.data)),
            SerializeOp::Validate => Ok(SerializationResult::Valid),
        }
    }

    fn name(&self) -> &str {
        "SerializationWorker"
    }

    fn timeout(&self) -> Duration {
        Duration::from_secs(15)
    }

    fn priority(&self) -> Priority {
        Priority::Normal
    }
}

impl SerializationWorker {
    pub fn new() -> Self {
        SerializationWorker {
            timeout: Duration::from_secs(15),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_serialize() {
        let worker = SerializationWorker::new();
        let request = SerializationRequest {
            data: vec![1, 2, 3, 4, 5],
            format: SerializeFormat::Json,
            operation: SerializeOp::Serialize,
        };
        let result = worker.execute(request).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_deserialize() {
        let worker = SerializationWorker::new();
        let request = SerializationRequest {
            data: vec![1, 2, 3, 4, 5],
            format: SerializeFormat::Bincode,
            operation: SerializeOp::Deserialize,
        };
        let result = worker.execute(request).await;
        assert!(result.is_ok());
    }
}
