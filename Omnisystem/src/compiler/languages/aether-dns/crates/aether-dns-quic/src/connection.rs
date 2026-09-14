/// QUIC Connection Management

use std::sync::Arc;
use std::collections::HashMap;
use dashmap::DashMap;
use uuid::Uuid;

/// QUIC Connection State
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionState {
    Opening,
    Open,
    Draining,
    Closed,
}

/// QUIC Connection
#[derive(Clone)]
pub struct QuicConnection {
    pub id: String,
    pub remote_addr: String,
    pub state: ConnectionState,
    pub streams: Arc<DashMap<u64, Vec<u8>>>,
    pub created_at: u64,
}

impl QuicConnection {
    pub fn new(remote_addr: String) -> Self {
        QuicConnection {
            id: Uuid::new_v4().to_string(),
            remote_addr,
            state: ConnectionState::Opening,
            streams: Arc::new(DashMap::new()),
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        }
    }

    pub fn open(&mut self) {
        self.state = ConnectionState::Open;
    }

    pub fn close(&mut self) {
        self.state = ConnectionState::Closed;
    }

    pub fn is_open(&self) -> bool {
        self.state == ConnectionState::Open
    }

    pub fn add_stream(&self, stream_id: u64, data: Vec<u8>) {
        self.streams.insert(stream_id, data);
    }

    pub fn get_stream(&self, stream_id: u64) -> Option<Vec<u8>> {
        self.streams.get(&stream_id).map(|s| s.clone())
    }

    pub fn stream_count(&self) -> usize {
        self.streams.len()
    }
}

/// Connection Manager
pub struct ConnectionManager {
    connections: Arc<DashMap<String, QuicConnection>>,
}

impl ConnectionManager {
    pub fn new() -> Self {
        ConnectionManager {
            connections: Arc::new(DashMap::new()),
        }
    }

    pub fn create_connection(&self, remote_addr: String) -> QuicConnection {
        let conn = QuicConnection::new(remote_addr);
        self.connections.insert(conn.id.clone(), conn.clone());
        conn
    }

    pub fn get_connection(&self, conn_id: &str) -> Option<QuicConnection> {
        self.connections.get(conn_id).map(|c| c.clone())
    }

    pub fn close_connection(&self, conn_id: &str) -> bool {
        self.connections.remove(conn_id).is_some()
    }

    pub fn connection_count(&self) -> usize {
        self.connections.len()
    }

    pub fn list_connections(&self) -> Vec<QuicConnection> {
        self.connections.iter().map(|entry| entry.value().clone()).collect()
    }
}

impl Default for ConnectionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connection_creation() {
        let conn = QuicConnection::new("192.168.1.1:5000".to_string());
        assert_eq!(conn.state, ConnectionState::Opening);
        assert!(!conn.is_open());
    }

    #[test]
    fn test_connection_state_transitions() {
        let mut conn = QuicConnection::new("192.168.1.1:5000".to_string());
        conn.open();
        assert!(conn.is_open());

        conn.close();
        assert!(!conn.is_open());
        assert_eq!(conn.state, ConnectionState::Closed);
    }

    #[test]
    fn test_connection_manager() {
        let manager = ConnectionManager::new();
        let conn = manager.create_connection("192.168.1.1:5000".to_string());

        assert!(manager.get_connection(&conn.id).is_some());
        assert_eq!(manager.connection_count(), 1);

        manager.close_connection(&conn.id);
        assert_eq!(manager.connection_count(), 0);
    }
}
