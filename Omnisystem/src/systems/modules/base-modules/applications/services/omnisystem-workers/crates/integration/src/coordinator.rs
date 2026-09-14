/// System Coordinator - Omnisystem Integration

use std::sync::Arc;
use tracing::info;

pub struct SystemCoordinator {
    orchestrator: Arc<super::orchestrator::WorkerOrchestrator>,
}

impl SystemCoordinator {
    pub fn new() -> Self {
        SystemCoordinator {
            orchestrator: Arc::new(super::orchestrator::WorkerOrchestrator::new()),
        }
    }

    pub fn initialize_omnisystem_integration(&self) {
        info!("Initializing Process Workers for Omnisystem");

        // Initialize all workers
        self.orchestrator.initialize();

        info!(
            "Process Workers System ready: {} worker types registered",
            self.orchestrator.get_registry().total_workers()
        );
    }

    pub fn get_orchestrator(&self) -> Arc<super::orchestrator::WorkerOrchestrator> {
        Arc::clone(&self.orchestrator)
    }

    pub fn display_worker_summary(&self) {
        let registry = self.orchestrator.get_registry();
        let workers = registry.list_all_workers();

        info!("=== Process Workers Summary ===");
        info!("Total worker types: {}", registry.total_workers());
        for worker in workers {
            info!("  - {}", worker);
        }
    }
}

impl Default for SystemCoordinator {
    fn default() -> Self {
        Self::new()
    }
}
