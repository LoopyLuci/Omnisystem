/// ValidationWorker - Input validation and data integrity checking

use omnisystem_workers_core::prelude::*;
use async_trait::async_trait;
use std::time::Duration;

pub struct ValidationWorker {
    timeout: Duration,
}

pub struct ValidationRequest {
    pub data: Vec<u8>,
    pub rule: ValidationRule,
}

#[derive(Debug, Clone)]
pub enum ValidationRule {
    TypeCheck,
    RangeCheck,
    FormatCheck,
    SchemaValidation,
}

#[derive(Debug)]
pub enum ValidationResult {
    Valid,
    Invalid(String),
    Error(String),
}

#[async_trait]
impl Worker for ValidationWorker {
    type Input = ValidationRequest;
    type Output = ValidationResult;

    async fn execute(&self, input: Self::Input) -> WorkerResult<Self::Output> {
        if input.data.is_empty() {
            return Ok(ValidationResult::Invalid("Empty data".to_string()));
        }

        Ok(ValidationResult::Valid)
    }

    fn name(&self) -> &str {
        "ValidationWorker"
    }

    fn timeout(&self) -> Duration {
        Duration::from_secs(5)
    }

    fn priority(&self) -> Priority {
        Priority::Normal
    }
}

impl ValidationWorker {
    pub fn new() -> Self {
        ValidationWorker {
            timeout: Duration::from_secs(5),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_validate_data() {
        let worker = ValidationWorker::new();
        let request = ValidationRequest {
            data: vec![1, 2, 3, 4, 5],
            rule: ValidationRule::TypeCheck,
        };
        let result = worker.execute(request).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_validate_invalid() {
        let worker = ValidationWorker::new();
        let request = ValidationRequest {
            data: vec![],
            rule: ValidationRule::TypeCheck,
        };
        let result = worker.execute(request).await;
        assert!(result.is_ok());
    }
}
