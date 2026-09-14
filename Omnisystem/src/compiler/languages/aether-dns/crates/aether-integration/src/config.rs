/// Configuration Management
/// AETHER DNS configuration and lifecycle

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AetherConfig {
    pub dns: DnsConfig,
    pub anonymity: AnonymityConfig,
    pub threat_detection: ThreatDetectionConfig,
    pub transfer_daemon: TransferDaemonConfig,
    pub logging: LoggingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsConfig {
    pub listen_addresses: Vec<String>,
    pub upstream_resolvers: Vec<String>,
    pub cache_size_mb: usize,
    pub max_query_size_bytes: usize,
    pub timeout_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnonymityConfig {
    pub default_level: u8,
    pub relay_count_min: usize,
    pub relay_count_max: usize,
    pub padding_enabled: bool,
    pub timing_obfuscation_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreatDetectionConfig {
    pub enabled: bool,
    pub dga_detection: bool,
    pub phishing_detection: bool,
    pub rate_limit_per_second: u32,
    pub rate_limit_per_minute: u32,
    pub block_threshold: f64,
    pub alert_threshold: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferDaemonConfig {
    pub enabled: bool,
    pub endpoint: String,
    pub api_key: String,
    pub send_analytics: bool,
    pub send_alerts: bool,
    pub analytics_interval_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub format: String,
    pub file_path: Option<String>,
    pub console_output: bool,
}

impl AetherConfig {
    pub fn default() -> Self {
        AetherConfig {
            dns: DnsConfig {
                listen_addresses: vec!["127.0.0.1:53".to_string()],
                upstream_resolvers: vec![
                    "8.8.8.8:53".to_string(),
                    "1.1.1.1:53".to_string(),
                ],
                cache_size_mb: 100,
                max_query_size_bytes: 512,
                timeout_ms: 5000,
            },
            anonymity: AnonymityConfig {
                default_level: 3,
                relay_count_min: 1,
                relay_count_max: 5,
                padding_enabled: true,
                timing_obfuscation_enabled: true,
            },
            threat_detection: ThreatDetectionConfig {
                enabled: true,
                dga_detection: true,
                phishing_detection: true,
                rate_limit_per_second: 500,
                rate_limit_per_minute: 6000,
                block_threshold: 0.75,
                alert_threshold: 0.55,
            },
            transfer_daemon: TransferDaemonConfig {
                enabled: false,
                endpoint: "localhost:9000".to_string(),
                api_key: "".to_string(),
                send_analytics: true,
                send_alerts: true,
                analytics_interval_secs: 300,
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                format: "json".to_string(),
                file_path: None,
                console_output: true,
            },
        }
    }

    pub fn from_json(json: &str) -> anyhow::Result<Self> {
        let config = serde_json::from_str(json)?;
        Ok(config)
    }

    pub fn to_json(&self) -> anyhow::Result<String> {
        Ok(serde_json::to_string_pretty(self)?)
    }

    pub fn validate(&self) -> anyhow::Result<()> {
        if self.dns.cache_size_mb == 0 {
            return Err(anyhow::anyhow!("Cache size must be > 0"));
        }

        if self.threat_detection.block_threshold > 1.0
            || self.threat_detection.block_threshold < 0.0
        {
            return Err(anyhow::anyhow!("Block threshold must be 0.0-1.0"));
        }

        if self.anonymity.default_level > 5 {
            return Err(anyhow::anyhow!("Anonymity level must be 0-5"));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AetherConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_serialization() {
        let config = AetherConfig::default();
        let json = config.to_json().unwrap();
        let parsed = AetherConfig::from_json(&json).unwrap();
        assert_eq!(parsed.dns.cache_size_mb, config.dns.cache_size_mb);
    }

    #[test]
    fn test_config_validation() {
        let mut config = AetherConfig::default();
        config.threat_detection.block_threshold = 1.5;
        assert!(config.validate().is_err());
    }
}
