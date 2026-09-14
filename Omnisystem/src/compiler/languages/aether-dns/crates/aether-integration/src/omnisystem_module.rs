/// Omnisystem Module Integration
/// AETHER as a first-class Omnisystem module

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModuleState {
    Registered,
    Loaded,
    Ready,
    Running,
    Shutting,
    Stopped,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleMetadata {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub capabilities: Vec<String>,
}

pub struct AetherModule {
    metadata: ModuleMetadata,
    state: ModuleState,
    initialization_time_ms: u64,
}

impl AetherModule {
    pub fn new(metadata: ModuleMetadata) -> Self {
        AetherModule {
            metadata,
            state: ModuleState::Registered,
            initialization_time_ms: 0,
        }
    }

    pub fn load(&mut self) -> anyhow::Result<()> {
        if self.state != ModuleState::Registered {
            return Err(anyhow::anyhow!("Module not in Registered state"));
        }
        self.state = ModuleState::Loaded;
        Ok(())
    }

    pub fn initialize(&mut self) -> anyhow::Result<()> {
        if self.state != ModuleState::Loaded {
            return Err(anyhow::anyhow!("Module not in Loaded state"));
        }
        self.state = ModuleState::Ready;
        Ok(())
    }

    pub async fn run(&mut self) -> anyhow::Result<()> {
        if self.state != ModuleState::Ready {
            return Err(anyhow::anyhow!("Module not in Ready state"));
        }
        self.state = ModuleState::Running;
        Ok(())
    }

    pub fn shutdown(&mut self) -> anyhow::Result<()> {
        if self.state == ModuleState::Stopped {
            return Err(anyhow::anyhow!("Module already stopped"));
        }
        self.state = ModuleState::Shutting;
        Ok(())
    }

    pub fn finalize(&mut self) -> anyhow::Result<()> {
        if self.state != ModuleState::Shutting {
            return Err(anyhow::anyhow!("Module not shutting down"));
        }
        self.state = ModuleState::Stopped;
        Ok(())
    }

    pub fn state(&self) -> ModuleState {
        self.state
    }

    pub fn metadata(&self) -> &ModuleMetadata {
        &self.metadata
    }

    pub fn capabilities(&self) -> &[String] {
        &self.metadata.capabilities
    }

    pub fn has_capability(&self, capability: &str) -> bool {
        self.metadata.capabilities.iter().any(|c| c == capability)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aether_module_creation() {
        let metadata = ModuleMetadata {
            name: "AETHER DNS".to_string(),
            version: "0.1.0".to_string(),
            description: "Private DNS system".to_string(),
            author: "AETHER Team".to_string(),
            capabilities: vec![
                "dns-resolution".to_string(),
                "privacy".to_string(),
                "threat-detection".to_string(),
            ],
        };
        let module = AetherModule::new(metadata);
        assert_eq!(module.state(), ModuleState::Registered);
    }

    #[test]
    fn test_module_lifecycle() {
        let metadata = ModuleMetadata {
            name: "AETHER DNS".to_string(),
            version: "0.1.0".to_string(),
            description: "Private DNS system".to_string(),
            author: "AETHER Team".to_string(),
            capabilities: vec![],
        };
        let mut module = AetherModule::new(metadata);

        assert!(module.load().is_ok());
        assert_eq!(module.state(), ModuleState::Loaded);

        assert!(module.initialize().is_ok());
        assert_eq!(module.state(), ModuleState::Ready);
    }

    #[test]
    fn test_capabilities() {
        let metadata = ModuleMetadata {
            name: "AETHER DNS".to_string(),
            version: "0.1.0".to_string(),
            description: "Private DNS system".to_string(),
            author: "AETHER Team".to_string(),
            capabilities: vec!["dns-resolution".to_string()],
        };
        let module = AetherModule::new(metadata);
        assert!(module.has_capability("dns-resolution"));
        assert!(!module.has_capability("unknown"));
    }
}
