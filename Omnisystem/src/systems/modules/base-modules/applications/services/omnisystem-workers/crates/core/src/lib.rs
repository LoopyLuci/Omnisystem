/// Omnisystem Workers - Core Framework
/// Universal task and process execution system

pub mod worker;
pub mod pool;
pub mod queue;
pub mod scheduler;
pub mod health;
pub mod metrics;
pub mod error;

pub use worker::{Worker, WorkerResult, Priority};
pub use pool::WorkerPool;
pub use queue::{TaskQueue, Task};
pub use scheduler::TaskScheduler;
pub use health::HealthMonitor;
pub use metrics::MetricsCollector;
pub use error::{WorkerError, Result};

/// Core re-exports
pub mod prelude {
    pub use crate::{
        Worker, WorkerPool, TaskQueue, Task, Priority,
        TaskScheduler, HealthMonitor, MetricsCollector,
        WorkerResult, WorkerError,
    };
}
