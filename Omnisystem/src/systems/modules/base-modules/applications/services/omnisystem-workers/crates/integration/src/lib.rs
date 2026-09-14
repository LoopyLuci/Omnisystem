/// Omnisystem Workers Integration
/// Central hub for all worker types and Omnisystem coordination

pub mod registry;
pub mod orchestrator;
pub mod coordinator;

pub use registry::WorkerRegistry;
pub use orchestrator::WorkerOrchestrator;
pub use coordinator::SystemCoordinator;

/// Worker registry initialization
pub fn initialize_all_workers() -> WorkerRegistry {
    let registry = WorkerRegistry::new();

    // Register all worker types
    registry.register_io_workers();
    registry.register_network_workers();
    registry.register_compute_workers();
    registry.register_device_workers();
    registry.register_advanced_workers();

    registry
}
