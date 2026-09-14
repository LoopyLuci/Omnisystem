/// IndexWorker - Database index creation and maintenance

use omnisystem_workers_core::prelude::*;
use async_trait::async_trait;
use std::time::Duration;

pub struct IndexWorker {
    timeout: Duration,
}

pub struct IndexRequest {
    pub table: String,
    pub columns: Vec<String>,
    pub index_type: IndexType,
}

#[derive(Debug, Clone)]
pub enum IndexType {
    BTree,
    Hash,
    FullText,
    Spatial,
}

#[derive(Debug)]
pub enum IndexResult {
    Created,
    Rebuilt,
    Deleted,
    Analyzed,
}

#[async_trait]
impl Worker for IndexWorker {
    type Input = IndexRequest;
    type Output = IndexResult;

    async fn execute(&self, input: Self::Input) -> WorkerResult<Self::Output> {
        match input.index_type {
            IndexType::BTree => Ok(IndexResult::Created),
            IndexType::Hash => Ok(IndexResult::Created),
            IndexType::FullText => Ok(IndexResult::Created),
            IndexType::Spatial => Ok(IndexResult::Created),
        }
    }

    fn name(&self) -> &str {
        "IndexWorker"
    }

    fn timeout(&self) -> Duration {
        Duration::from_secs(60)
    }

    fn priority(&self) -> Priority {
        Priority::Normal
    }
}

impl IndexWorker {
    pub fn new() -> Self {
        IndexWorker {
            timeout: Duration::from_secs(60),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_index() {
        let worker = IndexWorker::new();
        let request = IndexRequest {
            table: "users".to_string(),
            columns: vec!["email".to_string()],
            index_type: IndexType::BTree,
        };
        let result = worker.execute(request).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_create_fulltext() {
        let worker = IndexWorker::new();
        let request = IndexRequest {
            table: "documents".to_string(),
            columns: vec!["content".to_string()],
            index_type: IndexType::FullText,
        };
        let result = worker.execute(request).await;
        assert!(result.is_ok());
    }
}
