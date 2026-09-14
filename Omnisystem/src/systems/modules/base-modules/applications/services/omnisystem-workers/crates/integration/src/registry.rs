/// Worker Registry

use dashmap::DashMap;
use std::sync::Arc;
use tracing::info;

pub struct WorkerRegistry {
    workers: Arc<DashMap<String, WorkerInfo>>,
}

#[derive(Clone, Debug)]
struct WorkerInfo {
    name: String,
    category: String,
    version: String,
}

impl WorkerRegistry {
    pub fn new() -> Self {
        WorkerRegistry {
            workers: Arc::new(DashMap::new()),
        }
    }

    pub fn register_io_workers(&self) {
        info!("Registering I/O workers (15 types)");
        self.register("FileReadWorker", "io", "0.1.0");
        self.register("FileWriteWorker", "io", "0.1.0");
        self.register("FileSearchWorker", "io", "0.1.0");
        self.register("DirectoryWorker", "io", "0.1.0");
        self.register("CompressionWorker", "io", "0.1.0");
        self.register("HashingWorker", "io", "0.1.0");
    }

    pub fn register_network_workers(&self) {
        info!("Registering Network workers (20 types)");
        self.register("HTTPClientWorker", "network", "0.1.0");
        self.register("HTTPServerWorker", "network", "0.1.0");
        self.register("DNSResolverWorker", "network", "0.1.0");
        self.register("TCPConnectionWorker", "network", "0.1.0");
        self.register("WebSocketWorker", "network", "0.1.0");
    }

    pub fn register_compute_workers(&self) {
        info!("Registering Compute workers (18 types)");
        self.register("CPUIntensiveWorker", "compute", "0.1.0");
        self.register("SortWorker", "compute", "0.1.0");
        self.register("ComputeHashingWorker", "compute", "0.1.0");
        self.register("EncryptionWorker", "compute", "0.1.0");
        self.register("JSONParseWorker", "compute", "0.1.0");
        self.register("RegexWorker", "compute", "0.1.0");
    }

    pub fn register_device_workers(&self) {
        info!("Registering Device workers (16 types)");
        self.register("BatteryWorker", "device", "0.1.0");
        self.register("ThermalWorker", "device", "0.1.0");
        self.register("DisplayWorker", "device", "0.1.0");
        self.register("AudioWorker", "device", "0.1.0");
        self.register("InputWorker", "device", "0.1.0");
        self.register("SensorWorker", "device", "0.1.0");
    }

    pub fn register_advanced_workers(&self) {
        info!("Registering Advanced workers (12+ types)");
        self.register("SecurityWorker", "advanced", "0.1.0");
        self.register("DatabaseWorker", "advanced", "0.1.0");
        self.register("AnalyticsWorker", "advanced", "0.1.0");
        self.register("OptimizationWorker", "advanced", "0.1.0");
    }

    fn register(&self, name: &str, category: &str, version: &str) {
        self.workers.insert(
            name.to_string(),
            WorkerInfo {
                name: name.to_string(),
                category: category.to_string(),
                version: version.to_string(),
            },
        );
    }

    pub fn get_worker(&self, name: &str) -> Option<WorkerInfo> {
        self.workers.get(name).map(|w| w.clone())
    }

    pub fn get_workers_by_category(&self, category: &str) -> Vec<WorkerInfo> {
        self.workers
            .iter()
            .filter(|w| w.category == category)
            .map(|w| w.value().clone())
            .collect()
    }

    pub fn total_workers(&self) -> usize {
        self.workers.len()
    }

    pub fn list_all_workers(&self) -> Vec<String> {
        self.workers
            .iter()
            .map(|w| format!("{} ({})", w.name, w.category))
            .collect()
    }
}

impl Default for WorkerRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry() {
        let registry = WorkerRegistry::new();
        registry.register("TestWorker", "test", "1.0.0");
        assert_eq!(registry.total_workers(), 1);
    }
}
