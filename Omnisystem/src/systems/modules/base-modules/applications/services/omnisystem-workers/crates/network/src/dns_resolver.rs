/// DNSResolverWorker - DNS query resolution (AETHER DNS integration)

use omnisystem_workers_core::prelude::*;
use async_trait::async_trait;
use std::time::Duration;

pub struct DNSResolverWorker;

#[async_trait]
impl Worker for DNSResolverWorker {
    type Input = String;
    type Output = Vec<String>;

    async fn execute(&self, domain: Self::Input) -> WorkerResult<Self::Output> {
        // In production, would integrate with AETHER DNS
        match tokio::net::lookup_host(&format!("{}:80", domain)).await {
            Ok(addrs) => {
                let ips: Vec<String> = addrs
                    .map(|addr| addr.ip().to_string())
                    .collect();
                Ok(ips)
            }
            Err(e) => Err(WorkerError::ExecutionFailed(e.to_string())),
        }
    }

    fn name(&self) -> &str {
        "DNSResolverWorker"
    }

    fn timeout(&self) -> Duration {
        Duration::from_secs(5)
    }
}
