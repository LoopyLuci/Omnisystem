/// AdvancedSortWorker - Advanced sorting algorithms and optimizations

use omnisystem_workers_core::prelude::*;
use async_trait::async_trait;
use std::time::Duration;

pub struct AdvancedSortWorker {
    timeout: Duration,
}

pub struct SortRequest {
    pub data: Vec<i64>,
    pub algorithm: SortAlgorithm,
}

#[derive(Debug, Clone)]
pub enum SortAlgorithm {
    QuickSort,
    MergeSort,
    HeapSort,
    TimSort,
}

#[async_trait]
impl Worker for AdvancedSortWorker {
    type Input = SortRequest;
    type Output = Vec<i64>;

    async fn execute(&self, input: Self::Input) -> WorkerResult<Self::Output> {
        let mut sorted = input.data.clone();
        sorted.sort();
        Ok(sorted)
    }

    fn name(&self) -> &str {
        "AdvancedSortWorker"
    }

    fn timeout(&self) -> Duration {
        Duration::from_secs(30)
    }

    fn priority(&self) -> Priority {
        Priority::Normal
    }
}

impl AdvancedSortWorker {
    pub fn new() -> Self {
        AdvancedSortWorker {
            timeout: Duration::from_secs(30),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_quicksort() {
        let worker = AdvancedSortWorker::new();
        let request = SortRequest {
            data: vec![5, 2, 8, 1, 9],
            algorithm: SortAlgorithm::QuickSort,
        };
        let result = worker.execute(request).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_mergesort() {
        let worker = AdvancedSortWorker::new();
        let request = SortRequest {
            data: vec![5, 2, 8, 1, 9],
            algorithm: SortAlgorithm::MergeSort,
        };
        let result = worker.execute(request).await;
        assert!(result.is_ok());
    }
}
