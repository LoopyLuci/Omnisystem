/// Worker Orchestrator

use omnisystem_workers_core::prelude::*;
use std::sync::Arc;

pub struct WorkerOrchestrator {
    pool: Arc<WorkerPool>,
    registry: Arc<super::registry::WorkerRegistry>,
}

impl WorkerOrchestrator {
    pub fn new() -> Self {
        WorkerOrchestrator {
            pool: Arc::new(WorkerPool::new("omnisystem-global", 10000)),
            registry: Arc::new(super::registry::WorkerRegistry::new()),
        }
    }

    pub fn initialize(&self) {
        let registry = super::registry::WorkerRegistry::new();
        registry.register_io_workers();
        registry.register_network_workers();
        registry.register_compute_workers();
        registry.register_device_workers();
        registry.register_advanced_workers();
    }

    pub fn get_pool(&self) -> Arc<WorkerPool> {
        Arc::clone(&self.pool)
    }

    pub fn get_registry(&self) -> Arc<super::registry::WorkerRegistry> {
        Arc::clone(&self.registry)
    }

    pub fn system_stats(&self) -> OrchestrationStats {
        let pool_stats = self.pool.get_pool_stats();

        OrchestrationStats {
            total_workers: pool_stats.total_workers,
            healthy_workers: pool_stats.healthy_workers,
            total_tasks_processed: pool_stats.total_tasks,
            success_rate: self.pool.success_rate(),
            avg_latency_ms: self.pool.avg_latency_ms(),
        }
    }
}

impl Default for WorkerOrchestrator {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct OrchestrationStats {
    pub total_workers: usize,
    pub healthy_workers: usize,
    pub total_tasks_processed: u64,
    pub success_rate: f64,
    pub avg_latency_ms: f64,
}
