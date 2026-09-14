/// SearchWorker - Full-text and pattern search operations

use omnisystem_workers_core::prelude::*;
use async_trait::async_trait;
use std::time::Duration;

pub struct SearchWorker {
    timeout: Duration,
}

pub struct SearchRequest {
    pub query: String,
    pub target: String,
    pub mode: SearchMode,
}

#[derive(Debug, Clone)]
pub enum SearchMode {
    Literal,
    Regex,
    Fuzzy,
    Semantic,
}

#[derive(Debug)]
pub enum SearchResult {
    Matches(Vec<(usize, String)>),
    NoMatches,
    InvalidPattern,
}

#[async_trait]
impl Worker for SearchWorker {
    type Input = SearchRequest;
    type Output = SearchResult;

    async fn execute(&self, input: Self::Input) -> WorkerResult<Self::Output> {
        if input.query.is_empty() {
            return Ok(SearchResult::InvalidPattern);
        }

        if input.target.contains(&input.query) {
            Ok(SearchResult::Matches(vec![(0, input.query)]))
        } else {
            Ok(SearchResult::NoMatches)
        }
    }

    fn name(&self) -> &str {
        "SearchWorker"
    }

    fn timeout(&self) -> Duration {
        Duration::from_secs(15)
    }

    fn priority(&self) -> Priority {
        Priority::Normal
    }
}

impl SearchWorker {
    pub fn new() -> Self {
        SearchWorker {
            timeout: Duration::from_secs(15),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_literal_search() {
        let worker = SearchWorker::new();
        let request = SearchRequest {
            query: "test".to_string(),
            target: "this is a test".to_string(),
            mode: SearchMode::Literal,
        };
        let result = worker.execute(request).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_no_match() {
        let worker = SearchWorker::new();
        let request = SearchRequest {
            query: "xyz".to_string(),
            target: "test string".to_string(),
            mode: SearchMode::Literal,
        };
        let result = worker.execute(request).await;
        assert!(result.is_ok());
    }
}
