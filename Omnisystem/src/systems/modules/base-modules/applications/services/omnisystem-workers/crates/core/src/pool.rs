/// Worker Pool Manager

use crate::queue::TaskQueue;
use dashmap::DashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tracing::{info, debug};

pub struct WorkerPool {
    name: String,
    workers: Arc<DashMap<String, WorkerStats>>,
    task_queue: Arc<TaskQueue>,
    total_tasks: Arc<AtomicU64>,
    completed_tasks: Arc<AtomicU64>,
    failed_tasks: Arc<AtomicU64>,
}

#[derive(Clone, Debug)]
struct WorkerStats {
    worker_id: String,
    tasks_processed: u64,
    tasks_failed: u64,
    total_latency_ms: u64,
    last_active: chrono::DateTime<chrono::Utc>,
    healthy: bool,
}

impl WorkerPool {
    pub fn new(name: &str, queue_size: usize) -> Self {
        info!("Creating worker pool: {}", name);

        WorkerPool {
            name: name.to_string(),
            workers: Arc::new(DashMap::new()),
            task_queue: Arc::new(TaskQueue::new(queue_size)),
            total_tasks: Arc::new(AtomicU64::new(0)),
            completed_tasks: Arc::new(AtomicU64::new(0)),
            failed_tasks: Arc::new(AtomicU64::new(0)),
        }
    }

    pub fn register_worker(&self, worker_id: String) {
        debug!("Registering worker: {}", worker_id);

        self.workers.insert(
            worker_id.clone(),
            WorkerStats {
                worker_id,
                tasks_processed: 0,
                tasks_failed: 0,
                total_latency_ms: 0,
                last_active: chrono::Utc::now(),
                healthy: true,
            },
        );
    }

    pub fn record_task_completion(&self, worker_id: &str, latency_ms: u64) {
        if let Some(mut stats) = self.workers.get_mut(worker_id) {
            stats.tasks_processed += 1;
            stats.total_latency_ms += latency_ms;
            stats.last_active = chrono::Utc::now();
        }

        self.completed_tasks.fetch_add(1, Ordering::Relaxed);
        self.total_tasks.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_task_failure(&self, worker_id: &str) {
        if let Some(mut stats) = self.workers.get_mut(worker_id) {
            stats.tasks_failed += 1;
            stats.last_active = chrono::Utc::now();
        }

        self.failed_tasks.fetch_add(1, Ordering::Relaxed);
        self.total_tasks.fetch_add(1, Ordering::Relaxed);
    }

    pub fn mark_worker_unhealthy(&self, worker_id: &str) {
        if let Some(mut stats) = self.workers.get_mut(worker_id) {
            stats.healthy = false;
        }
    }

    pub fn get_pool_stats(&self) -> PoolStats {
        PoolStats {
            total_workers: self.workers.len(),
            healthy_workers: self.workers.iter().filter(|w| w.healthy).count(),
            total_tasks: self.total_tasks.load(Ordering::Relaxed),
            completed_tasks: self.completed_tasks.load(Ordering::Relaxed),
            failed_tasks: self.failed_tasks.load(Ordering::Relaxed),
            queue_size: self.task_queue.len(),
        }
    }

    pub fn avg_latency_ms(&self) -> f64 {
        let total_latency: u64 = self.workers.iter().map(|w| w.total_latency_ms).sum();
        let total_processed: u64 = self.workers.iter().map(|w| w.tasks_processed).sum();

        if total_processed == 0 {
            0.0
        } else {
            total_latency as f64 / total_processed as f64
        }
    }

    pub fn success_rate(&self) -> f64 {
        let total = self.total_tasks.load(Ordering::Relaxed);
        let completed = self.completed_tasks.load(Ordering::Relaxed);

        if total == 0 {
            0.0
        } else {
            (completed as f64 / total as f64) * 100.0
        }
    }
}

#[derive(Debug, Clone)]
pub struct PoolStats {
    pub total_workers: usize,
    pub healthy_workers: usize,
    pub total_tasks: u64,
    pub completed_tasks: u64,
    pub failed_tasks: u64,
    pub queue_size: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pool_creation() {
        let pool = WorkerPool::new("test_pool", 100);
        assert_eq!(pool.workers.len(), 0);
    }

    #[test]
    fn test_register_worker() {
        let pool = WorkerPool::new("test_pool", 100);
        pool.register_worker("worker1".to_string());
        assert_eq!(pool.workers.len(), 1);
    }

    #[test]
    fn test_task_stats() {
        let pool = WorkerPool::new("test_pool", 100);
        pool.register_worker("worker1".to_string());
        pool.record_task_completion("worker1", 50);

        let stats = pool.get_pool_stats();
        assert_eq!(stats.completed_tasks, 1);
        assert_eq!(pool.success_rate(), 100.0);
    }
}
