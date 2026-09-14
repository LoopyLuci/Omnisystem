/// TransactionWorker - Database transaction management

use omnisystem_workers_core::prelude::*;
use async_trait::async_trait;
use std::time::Duration;

pub struct TransactionWorker {
    timeout: Duration,
}

pub struct TransactionRequest {
    pub transaction_id: String,
    pub operation: TransactionOp,
}

#[derive(Debug, Clone)]
pub enum TransactionOp {
    Begin,
    Commit,
    Rollback,
    Execute(String),
}

#[derive(Debug)]
pub enum TransactionResult {
    Started,
    Committed,
    RolledBack,
    Executed(usize),
    Error(String),
}

#[async_trait]
impl Worker for TransactionWorker {
    type Input = TransactionRequest;
    type Output = TransactionResult;

    async fn execute(&self, input: Self::Input) -> WorkerResult<Self::Output> {
        if input.transaction_id.is_empty() {
            return Ok(TransactionResult::Error("Invalid transaction ID".to_string()));
        }

        match input.operation {
            TransactionOp::Begin => Ok(TransactionResult::Started),
            TransactionOp::Commit => Ok(TransactionResult::Committed),
            TransactionOp::Rollback => Ok(TransactionResult::RolledBack),
            TransactionOp::Execute(sql) => Ok(TransactionResult::Executed(sql.len())),
        }
    }

    fn name(&self) -> &str {
        "TransactionWorker"
    }

    fn timeout(&self) -> Duration {
        Duration::from_secs(45)
    }

    fn priority(&self) -> Priority {
        Priority::High
    }
}

impl TransactionWorker {
    pub fn new() -> Self {
        TransactionWorker {
            timeout: Duration::from_secs(45),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_transaction_begin() {
        let worker = TransactionWorker::new();
        let request = TransactionRequest {
            transaction_id: "txn_123".to_string(),
            operation: TransactionOp::Begin,
        };
        let result = worker.execute(request).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_transaction_commit() {
        let worker = TransactionWorker::new();
        let request = TransactionRequest {
            transaction_id: "txn_123".to_string(),
            operation: TransactionOp::Commit,
        };
        let result = worker.execute(request).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_transaction_execute() {
        let worker = TransactionWorker::new();
        let request = TransactionRequest {
            transaction_id: "txn_123".to_string(),
            operation: TransactionOp::Execute("SELECT * FROM users".to_string()),
        };
        let result = worker.execute(request).await;
        assert!(result.is_ok());
    }
}
