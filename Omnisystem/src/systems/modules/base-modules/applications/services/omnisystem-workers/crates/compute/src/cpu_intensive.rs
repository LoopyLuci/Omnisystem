/// CPUIntensiveWorker - CPU-bound computation

use omnisystem_workers_core::prelude::*;
use async_trait::async_trait;
use std::time::Duration;

pub struct CPUIntensiveWorker;

#[async_trait]
impl Worker for CPUIntensiveWorker {
    type Input = Vec<u64>;
    type Output = u64;

    async fn execute(&self, data: Self::Input) -> WorkerResult<Self::Output> {
        // Simulate CPU-intensive work
        let result = data.iter()
            .map(|x| {
                let mut n = *x;
                let mut count = 0;
                while n > 1 {
                    if n % 2 == 0 {
                        n /= 2;
                    } else {
                        n = 3 * n + 1;
                    }
                    count += 1;
                }
                count
            })
            .sum();

        Ok(result)
    }

    fn name(&self) -> &str {
        "CPUIntensiveWorker"
    }

    fn timeout(&self) -> Duration {
        Duration::from_secs(60)
    }

    fn priority(&self) -> Priority {
        Priority::Normal
    }
}
