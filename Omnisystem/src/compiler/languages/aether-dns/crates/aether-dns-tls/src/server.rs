/// DoT Server Implementation (RFC 7858)

use tokio::net::{TcpListener, TcpStream};
use tokio_rustls::TlsAcceptor;
use std::sync::Arc;
use tracing::info;
use crate::config::DoTConfig;
use crate::handler::DoTHandler;
use crate::session::SessionManager;

pub struct DoTServer {
    config: DoTConfig,
    tls_acceptor: Arc<TlsAcceptor>,
    session_manager: Arc<SessionManager>,
}

impl DoTServer {
    pub fn new(config: DoTConfig, tls_acceptor: Arc<TlsAcceptor>) -> Self {
        DoTServer {
            config,
            tls_acceptor,
            session_manager: Arc::new(SessionManager::new(300)), // 5 min timeout
        }
    }

    pub async fn run(&self) -> anyhow::Result<()> {
        let addr = format!("{}:{}", self.config.listen_addr, self.config.port);
        let listener = TcpListener::bind(&addr).await?;

        info!("DoT server listening on {}", addr);

        loop {
            let (socket, peer_addr) = listener.accept().await?;
            let acceptor = Arc::clone(&self.tls_acceptor);
            let session_manager = Arc::clone(&self.session_manager);

            tokio::spawn(async move {
                if let Err(e) = Self::handle_connection(socket, acceptor, peer_addr, session_manager).await {
                    tracing::warn!("Error handling DoT connection: {}", e);
                }
            });
        }
    }

    async fn handle_connection(
        socket: TcpStream,
        acceptor: Arc<TlsAcceptor>,
        peer_addr: std::net::SocketAddr,
        session_manager: Arc<SessionManager>,
    ) -> anyhow::Result<()> {
        // Perform TLS handshake
        let tls_stream = acceptor.accept(socket).await?;

        // Create session
        let session = session_manager.create_session(peer_addr);
        tracing::debug!("New DoT session: {} from {}", session.id, peer_addr);

        // TODO: Read/write DNS messages over TLS stream
        // For now, just drop the connection
        drop(tls_stream);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_creation() {
        let config = DoTConfig::new("127.0.0.1".to_string(), 853);
        assert_eq!(config.port, 853);
    }
}
