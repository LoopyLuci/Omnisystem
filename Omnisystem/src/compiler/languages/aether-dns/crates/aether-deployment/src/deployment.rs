/// Deployment Configuration
/// Multi-environment deployment support

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Environment {
    Development,
    Staging,
    Production,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentConfig {
    pub environment: Environment,
    pub instance_count: u32,
    pub region: String,
    pub enable_monitoring: bool,
    pub enable_auto_scaling: bool,
    pub log_level: String,
    pub backup_enabled: bool,
    pub backup_interval_hours: u32,
}

impl DeploymentConfig {
    pub fn development() -> Self {
        DeploymentConfig {
            environment: Environment::Development,
            instance_count: 1,
            region: "local".to_string(),
            enable_monitoring: false,
            enable_auto_scaling: false,
            log_level: "debug".to_string(),
            backup_enabled: false,
            backup_interval_hours: 0,
        }
    }

    pub fn staging() -> Self {
        DeploymentConfig {
            environment: Environment::Staging,
            instance_count: 3,
            region: "us-east-1".to_string(),
            enable_monitoring: true,
            enable_auto_scaling: true,
            log_level: "info".to_string(),
            backup_enabled: true,
            backup_interval_hours: 6,
        }
    }

    pub fn production() -> Self {
        DeploymentConfig {
            environment: Environment::Production,
            instance_count: 10,
            region: "us-east-1,us-west-2,eu-west-1".to_string(),
            enable_monitoring: true,
            enable_auto_scaling: true,
            log_level: "warn".to_string(),
            backup_enabled: true,
            backup_interval_hours: 1,
        }
    }

    pub fn validate(&self) -> anyhow::Result<()> {
        if self.instance_count == 0 {
            return Err(anyhow::anyhow!("Instance count must be > 0"));
        }

        if self.backup_enabled && self.backup_interval_hours == 0 {
            return Err(anyhow::anyhow!("Backup enabled but interval is 0"));
        }

        Ok(())
    }

    pub fn is_production(&self) -> bool {
        self.environment == Environment::Production
    }

    pub fn is_development(&self) -> bool {
        self.environment == Environment::Development
    }

    pub fn max_qps(&self) -> u32 {
        match self.environment {
            Environment::Development => 1000,
            Environment::Staging => 100_000,
            Environment::Production => 1_000_000,
        }
    }

    pub fn sla_uptime_percent(&self) -> f64 {
        match self.environment {
            Environment::Development => 99.0,
            Environment::Staging => 99.5,
            Environment::Production => 99.99,
        }
    }

    pub fn to_json(&self) -> anyhow::Result<String> {
        Ok(serde_json::to_string_pretty(self)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dev_config() {
        let config = DeploymentConfig::development();
        assert!(config.is_development());
        assert_eq!(config.instance_count, 1);
    }

    #[test]
    fn test_prod_config() {
        let config = DeploymentConfig::production();
        assert!(config.is_production());
        assert_eq!(config.instance_count, 10);
        assert!(config.enable_monitoring);
    }

    #[test]
    fn test_config_validation() {
        let mut config = DeploymentConfig::development();
        assert!(config.validate().is_ok());

        config.instance_count = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_sla_uptime() {
        let dev = DeploymentConfig::development();
        let prod = DeploymentConfig::production();

        assert!(prod.sla_uptime_percent() > dev.sla_uptime_percent());
    }
}
