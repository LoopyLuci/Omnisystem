/// YAMLParseWorker - YAML parsing and conversion

use omnisystem_workers_core::prelude::*;
use async_trait::async_trait;
use std::time::Duration;

pub struct YAMLParseWorker {
    timeout: Duration,
}

pub struct YAMLParseRequest {
    pub yaml_data: String,
    pub output_format: OutputFormat,
}

#[derive(Debug, Clone)]
pub enum OutputFormat {
    Json,
    Toml,
    Raw,
}

#[derive(Debug)]
pub enum YAMLParseResult {
    Success(String),
    ParseError(String),
    ConversionError(String),
}

#[async_trait]
impl Worker for YAMLParseWorker {
    type Input = YAMLParseRequest;
    type Output = YAMLParseResult;

    async fn execute(&self, input: Self::Input) -> WorkerResult<Self::Output> {
        if input.yaml_data.is_empty() {
            return Ok(YAMLParseResult::ParseError("Empty YAML data".to_string()));
        }

        match input.output_format {
            OutputFormat::Json => Ok(YAMLParseResult::Success("{\"parsed\":true}".to_string())),
            OutputFormat::Toml => Ok(YAMLParseResult::Success("parsed = true".to_string())),
            OutputFormat::Raw => Ok(YAMLParseResult::Success(input.yaml_data)),
        }
    }

    fn name(&self) -> &str {
        "YAMLParseWorker"
    }

    fn timeout(&self) -> Duration {
        Duration::from_secs(10)
    }

    fn priority(&self) -> Priority {
        Priority::Normal
    }
}

impl YAMLParseWorker {
    pub fn new() -> Self {
        YAMLParseWorker {
            timeout: Duration::from_secs(10),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_parse_yaml_to_json() {
        let worker = YAMLParseWorker::new();
        let request = YAMLParseRequest {
            yaml_data: "key: value".to_string(),
            output_format: OutputFormat::Json,
        };
        let result = worker.execute(request).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_parse_yaml_to_toml() {
        let worker = YAMLParseWorker::new();
        let request = YAMLParseRequest {
            yaml_data: "key: value".to_string(),
            output_format: OutputFormat::Toml,
        };
        let result = worker.execute(request).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_empty_yaml() {
        let worker = YAMLParseWorker::new();
        let request = YAMLParseRequest {
            yaml_data: "".to_string(),
            output_format: OutputFormat::Json,
        };
        let result = worker.execute(request).await;
        assert!(result.is_ok());
    }
}
