/// Orchestrator
/// Central management and coordination

use crate::omnisystem_module::{AetherModule, ModuleMetadata, ModuleState};
use crate::transfer_daemon::TransferDaemonIntegration;
use crate::config::AetherConfig;
use dashmap::DashMap;
use std::sync::Arc;

pub struct AetherOrchestrator {
    module: Arc<AetherModule>,
    config: Arc<AetherConfig>,
    transfer_daemon: Option<Arc<TransferDaemonIntegration>>,
    submodules: Arc<DashMap<String, ModuleState>>,
}

impl AetherOrchestrator {
    pub fn new(config: AetherConfig) -> anyhow::Result<Self> {
        config.validate()?;

        let metadata = ModuleMetadata {
            name: "AETHER DNS".to_string(),
            version: "0.1.0".to_string(),
            description: "Private, Anonymous, Threat-Resistant DNS System".to_string(),
            author: "AETHER Team".to_string(),
            capabilities: vec![
                "dns-resolution".to_string(),
                "anonymity-layers".to_string(),
                "threat-detection".to_string(),
                "analytics".to_string(),
                "relay-network".to_string(),
            ],
        };

        let module = Arc::new(AetherModule::new(metadata));

        let transfer_daemon = if config.transfer_daemon.enabled {
            Some(Arc::new(TransferDaemonIntegration::new(
                &config.transfer_daemon.endpoint,
                &config.transfer_daemon.api_key,
            )))
        } else {
            None
        };

        Ok(AetherOrchestrator {
            module,
            config: Arc::new(config),
            transfer_daemon,
            submodules: Arc::new(DashMap::new()),
        })
    }

    pub async fn initialize(&self) -> anyhow::Result<()> {
        tracing::info!("Initializing AETHER DNS system");

        // Register submodules
        self.submodules
            .insert("dns-core".to_string(), ModuleState::Registered);
        self.submodules
            .insert("anonymity".to_string(), ModuleState::Registered);
        self.submodules
            .insert("threat-detection".to_string(), ModuleState::Registered);
        self.submodules
            .insert("relay-network".to_string(), ModuleState::Registered);
        self.submodules
            .insert("analytics".to_string(), ModuleState::Registered);

        tracing::info!("AETHER DNS initialized with {} submodules", self.submodules.len());
        Ok(())
    }

    pub async fn start(&self) -> anyhow::Result<()> {
        tracing::info!("Starting AETHER DNS");

        // Transition submodules to Running
        for mut entry in self.submodules.iter_mut() {
            *entry.value_mut() = ModuleState::Running;
        }

        tracing::info!("AETHER DNS started successfully");
        Ok(())
    }

    pub async fn stop(&self) -> anyhow::Result<()> {
        tracing::info!("Stopping AETHER DNS");

        // Transition submodules to Stopped
        for mut entry in self.submodules.iter_mut() {
            *entry.value_mut() = ModuleState::Stopped;
        }

        tracing::info!("AETHER DNS stopped");
        Ok(())
    }

    pub fn get_config(&self) -> Arc<AetherConfig> {
        Arc::clone(&self.config)
    }

    pub fn get_module(&self) -> Arc<AetherModule> {
        Arc::clone(&self.module)
    }

    pub fn get_transfer_daemon(&self) -> Option<Arc<TransferDaemonIntegration>> {
        self.transfer_daemon.as_ref().map(Arc::clone)
    }

    pub fn submodule_state(&self, name: &str) -> Option<ModuleState> {
        self.submodules.get(name).map(|entry| *entry.value())
    }

    pub fn all_submodules_running(&self) -> bool {
        self.submodules.iter().all(|entry| *entry.value() == ModuleState::Running)
    }

    pub fn system_status(&self) -> String {
        format!(
            "AETHER DNS Status: {} submodules, {} running",
            self.submodules.len(),
            self.submodules
                .iter()
                .filter(|e| *e.value() == ModuleState::Running)
                .count()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_orchestrator_creation() {
        let config = AetherConfig::default();
        let orchestrator = AetherOrchestrator::new(config).unwrap();
        assert_eq!(orchestrator.submodules.len(), 0);
    }

    #[tokio::test]
    async fn test_orchestrator_initialize() {
        let config = AetherConfig::default();
        let orchestrator = AetherOrchestrator::new(config).unwrap();
        assert!(orchestrator.initialize().await.is_ok());
        assert_eq!(orchestrator.submodules.len(), 5);
    }

    #[tokio::test]
    async fn test_orchestrator_start_stop() {
        let config = AetherConfig::default();
        let orchestrator = AetherOrchestrator::new(config).unwrap();
        orchestrator.initialize().await.unwrap();
        assert!(orchestrator.start().await.is_ok());
        assert!(orchestrator.all_submodules_running());
        assert!(orchestrator.stop().await.is_ok());
    }
}
