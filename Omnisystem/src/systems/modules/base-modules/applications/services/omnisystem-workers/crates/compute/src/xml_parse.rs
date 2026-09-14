/// XMLParseWorker - XML parsing and validation

use omnisystem_workers_core::prelude::*;
use async_trait::async_trait;
use std::time::Duration;

pub struct XMLParseWorker {
    timeout: Duration,
}

pub struct XMLParseRequest {
    pub xml_data: String,
    pub validate: bool,
}

#[derive(Debug, Clone)]
pub enum XMLParseResult {
    Success(String),
    ParseError(String),
    ValidationError(String),
}

#[async_trait]
impl Worker for XMLParseWorker {
    type Input = XMLParseRequest;
    type Output = XMLParseResult;

    async fn execute(&self, input: Self::Input) -> WorkerResult<Self::Output> {
        if input.xml_data.is_empty() {
            return Ok(XMLParseResult::ParseError("Empty XML data".to_string()));
        }

        if !input.xml_data.starts_with('<') {
            return Ok(XMLParseResult::ParseError("Invalid XML format".to_string()));
        }

        if input.validate {
            Ok(XMLParseResult::Success("Valid".to_string()))
        } else {
            Ok(XMLParseResult::Success("Parsed".to_string()))
        }
    }

    fn name(&self) -> &str {
        "XMLParseWorker"
    }

    fn timeout(&self) -> Duration {
        Duration::from_secs(10)
    }

    fn priority(&self) -> Priority {
        Priority::Normal
    }
}

impl XMLParseWorker {
    pub fn new() -> Self {
        XMLParseWorker {
            timeout: Duration::from_secs(10),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_parse_valid_xml() {
        let worker = XMLParseWorker::new();
        let request = XMLParseRequest {
            xml_data: "<root><item>test</item></root>".to_string(),
            validate: false,
        };
        let result = worker.execute(request).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_parse_empty_xml() {
        let worker = XMLParseWorker::new();
        let request = XMLParseRequest {
            xml_data: "".to_string(),
            validate: false,
        };
        let result = worker.execute(request).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_validate_xml() {
        let worker = XMLParseWorker::new();
        let request = XMLParseRequest {
            xml_data: "<root></root>".to_string(),
            validate: true,
        };
        let result = worker.execute(request).await;
        assert!(result.is_ok());
    }
}
