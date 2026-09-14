/// DoQ Server Implementation (RFC 9250)

use std::sync::Arc;
use tracing::info;
use crate::config::DoQConfig;

pub struct DoQServer {
    config: DoQConfig,
}

impl DoQServer {
    pub fn new(config: DoQConfig) -> Self {
        DoQServer { config }
    }

    pub async fn run(&self) -> anyhow::Result<()> {
        let addr = format!("{}:{}", self.config.listen_addr, self.config.port);

        info!("DoQ server listening on {}:udp", addr);

        // TODO: Implement Quinn QUIC server
        // For now, just a stub
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_creation() {
        let config = DoQConfig::new("127.0.0.1".to_string(), 443);
        assert_eq!(config.port, 443);
    }

    #[test]
    fn test_server_creation() {
        let config = DoQConfig::default();
        let _server = DoQServer::new(config);
    }
}
