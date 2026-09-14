/// TLS Session Management

use std::collections::HashMap;
use std::sync::Arc;
use dashmap::DashMap;
use uuid::Uuid;
use std::net::SocketAddr;
use std::time::Instant;

#[derive(Clone)]
pub struct TlsSession {
    pub id: String,
    pub client_addr: SocketAddr,
    pub established_at: Instant,
    pub last_activity: Instant,
    pub query_count: u64,
}

pub struct SessionManager {
    sessions: Arc<DashMap<String, TlsSession>>,
    max_idle_time_secs: u64,
}

impl SessionManager {
    pub fn new(max_idle_time_secs: u64) -> Self {
        SessionManager {
            sessions: Arc::new(DashMap::new()),
            max_idle_time_secs,
        }
    }

    pub fn create_session(&self, client_addr: SocketAddr) -> TlsSession {
        let session = TlsSession {
            id: Uuid::new_v4().to_string(),
            client_addr,
            established_at: Instant::now(),
            last_activity: Instant::now(),
            query_count: 0,
        };

        self.sessions.insert(session.id.clone(), session.clone());
        session
    }

    pub fn get_session(&self, session_id: &str) -> Option<TlsSession> {
        self.sessions.get(session_id).map(|s| s.clone())
    }

    pub fn record_query(&self, session_id: &str) {
        if let Some(mut session) = self.sessions.get_mut(session_id) {
            session.last_activity = Instant::now();
            session.query_count += 1;
        }
    }

    pub fn cleanup_idle_sessions(&self) {
        let now = Instant::now();
        let mut to_remove = Vec::new();

        for entry in self.sessions.iter() {
            let session = entry.value();
            if now.duration_since(session.last_activity).as_secs() > self.max_idle_time_secs {
                to_remove.push(session.id.clone());
            }
        }

        for id in to_remove {
            self.sessions.remove(&id);
        }
    }

    pub fn session_count(&self) -> usize {
        self.sessions.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_session_creation() {
        let manager = SessionManager::new(3600);
        let addr = "127.0.0.1:5000".parse().unwrap();
        let session = manager.create_session(addr);
        assert!(!session.id.is_empty());
    }

    #[test]
    fn test_session_retrieval() {
        let manager = SessionManager::new(3600);
        let addr = "127.0.0.1:5000".parse().unwrap();
        let session = manager.create_session(addr);
        let retrieved = manager.get_session(&session.id);
        assert!(retrieved.is_some());
    }

    #[test]
    fn test_query_recording() {
        let manager = SessionManager::new(3600);
        let addr = "127.0.0.1:5000".parse().unwrap();
        let session = manager.create_session(addr);
        manager.record_query(&session.id);
        let updated = manager.get_session(&session.id).unwrap();
        assert_eq!(updated.query_count, 1);
    }
}
