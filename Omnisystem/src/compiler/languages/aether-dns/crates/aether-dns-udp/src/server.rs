use tokio::net::UdpSocket;
use std::sync::Arc;
use tracing::{info, warn, debug};
use aether_dns_core::protocol::DNSMessage;
use aether_dns_core::serialization::{DNSDeserializer, DNSSerializer};
use crate::handler::UDPQueryHandler;

pub struct ServerConfig {
    pub listen_addr: String,
    pub port: u16,
    pub max_packet_size: usize,
    pub timeout_ms: u64,
}

pub struct UDPDNSServer {
    config: ServerConfig,
    handler: Arc<UDPQueryHandler>,
}

impl UDPDNSServer {
    pub fn new(config: ServerConfig, handler: Arc<UDPQueryHandler>) -> Self {
        UDPDNSServer { config, handler }
    }

    pub async fn run(&self) -> anyhow::Result<()> {
        let addr = format!("{}:{}", self.config.listen_addr, self.config.port);
        let socket = Arc::new(UdpSocket::bind(&addr).await?);
        info!("UDP DNS server listening on {}", addr);

        let mut buf = vec![0u8; self.config.max_packet_size];

        loop {
            let (n, peer_addr) = socket.recv_from(&mut buf).await?;
            let data = buf[..n].to_vec();
            let socket_clone = Arc::clone(&socket);
            let handler = Arc::clone(&self.handler);

            tokio::spawn(async move {
                if let Err(e) = Self::handle_query(&socket_clone, peer_addr, &data, &handler).await {
                    warn!("Error handling query from {}: {}", peer_addr, e);
                }
            });
        }
    }

    async fn handle_query(
        socket: &UdpSocket,
        peer_addr: std::net::SocketAddr,
        data: &[u8],
        handler: &Arc<UDPQueryHandler>,
    ) -> anyhow::Result<()> {
        // Parse query
        let query_msg = DNSDeserializer::deserialize(data)?;

        debug!("Received query from {}: {} questions", peer_addr, query_msg.questions.len());

        // Process query
        let response = handler.handle_query(&query_msg, peer_addr.ip().to_string()).await?;

        // Send response
        let response_bytes = DNSSerializer::serialize(&response)?;
        socket.send_to(&response_bytes, peer_addr).await?;
        debug!("Sent response to {}", peer_addr);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_creation() {
        let config = ServerConfig {
            listen_addr: "127.0.0.1".to_string(),
            port: 5353,
            max_packet_size: 512,
            timeout_ms: 5000,
        };
        assert_eq!(config.port, 5353);
        assert_eq!(config.listen_addr, "127.0.0.1");
    }
}
