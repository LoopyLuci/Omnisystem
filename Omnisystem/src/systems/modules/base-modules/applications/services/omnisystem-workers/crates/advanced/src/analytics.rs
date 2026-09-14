/// AnalyticsWorker - Data analysis and reporting

use omnisystem_workers_core::prelude::*;
use async_trait::async_trait;
use std::time::Duration;

pub struct AnalyticsWorker;

#[async_trait]
impl Worker for AnalyticsWorker {
    type Input = Vec<u64>;
    type Output = String;

    async fn execute(&self, data: Self::Input) -> WorkerResult<Self::Output> {
        if data.is_empty() {
            return Ok("No data".to_string());
        }

        let sum: u64 = data.iter().sum();
        let avg = sum / data.len() as u64;
        let min = data.iter().min().copied().unwrap_or(0);
        let max = data.iter().max().copied().unwrap_or(0);

        Ok(format!(
            "Sum: {}, Avg: {}, Min: {}, Max: {}",
            sum, avg, min, max
        ))
    }

    fn name(&self) -> &str {
        "AnalyticsWorker"
    }

    fn timeout(&self) -> Duration {
        Duration::from_secs(15)
    }
}
