/// QUIC Stream Management for DNS

use serde::{Deserialize, Serialize};

/// Query Stream State
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StreamState {
    Idle,
    Open,
    HalfClosed,
    Closed,
    Reset,
}

/// DNS Query Stream
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryStream {
    pub stream_id: u64,
    pub connection_id: String,
    pub state: StreamState,
    pub request_data: Vec<u8>,
    pub response_data: Option<Vec<u8>>,
    pub created_at: u64,
    pub bytes_sent: u64,
    pub bytes_received: u64,
}

impl QueryStream {
    pub fn new(stream_id: u64, connection_id: String) -> Self {
        QueryStream {
            stream_id,
            connection_id,
            state: StreamState::Idle,
            request_data: Vec::new(),
            response_data: None,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            bytes_sent: 0,
            bytes_received: 0,
        }
    }

    pub fn write_request(&mut self, data: Vec<u8>) -> anyhow::Result<()> {
        if self.state != StreamState::Idle && self.state != StreamState::Open {
            return Err(anyhow::anyhow!("Stream not open for writing"));
        }

        self.request_data = data.clone();
        self.bytes_received += data.len() as u64;
        self.state = StreamState::Open;
        Ok(())
    }

    pub fn write_response(&mut self, data: Vec<u8>) -> anyhow::Result<()> {
        if self.state == StreamState::Closed {
            return Err(anyhow::anyhow!("Stream closed"));
        }

        self.response_data = Some(data.clone());
        self.bytes_sent += data.len() as u64;
        self.state = StreamState::HalfClosed;
        Ok(())
    }

    pub fn close(&mut self) {
        self.state = StreamState::Closed;
    }

    pub fn reset(&mut self) {
        self.state = StreamState::Reset;
    }

    pub fn is_active(&self) -> bool {
        self.state != StreamState::Closed && self.state != StreamState::Reset
    }

    pub fn latency_ms(&self) -> u64 {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        now - self.created_at
    }
}

/// Stream Manager
pub struct StreamManager {
    streams: std::sync::Arc<dashmap::DashMap<u64, QueryStream>>,
}

impl StreamManager {
    pub fn new() -> Self {
        StreamManager {
            streams: std::sync::Arc::new(dashmap::DashMap::new()),
        }
    }

    pub fn create_stream(&self, stream_id: u64, connection_id: String) -> QueryStream {
        let stream = QueryStream::new(stream_id, connection_id);
        self.streams.insert(stream_id, stream.clone());
        stream
    }

    pub fn get_stream(&self, stream_id: u64) -> Option<QueryStream> {
        self.streams.get(&stream_id).map(|s| s.clone())
    }

    pub fn write_request(&self, stream_id: u64, data: Vec<u8>) -> anyhow::Result<()> {
        match self.streams.get_mut(&stream_id) {
            Some(mut stream) => stream.write_request(data),
            None => Err(anyhow::anyhow!("Stream not found: {}", stream_id)),
        }
    }

    pub fn write_response(&self, stream_id: u64, data: Vec<u8>) -> anyhow::Result<()> {
        match self.streams.get_mut(&stream_id) {
            Some(mut stream) => stream.write_response(data),
            None => Err(anyhow::anyhow!("Stream not found: {}", stream_id)),
        }
    }

    pub fn close_stream(&self, stream_id: u64) -> bool {
        if let Some(mut stream) = self.streams.get_mut(&stream_id) {
            stream.close();
            true
        } else {
            false
        }
    }

    pub fn stream_count(&self) -> usize {
        self.streams.len()
    }

    pub fn active_streams(&self) -> usize {
        self.streams.iter().filter(|s| s.value().is_active()).count()
    }

    pub fn stats(&self) -> StreamStats {
        let mut total_bytes_sent = 0u64;
        let mut total_bytes_received = 0u64;
        let mut total_latency_ms = 0u64;
        let count = self.streams.len() as u64;

        for entry in self.streams.iter() {
            total_bytes_sent += entry.value().bytes_sent;
            total_bytes_received += entry.value().bytes_received;
            total_latency_ms += entry.value().latency_ms();
        }

        StreamStats {
            active_streams: self.active_streams(),
            total_streams: self.stream_count(),
            total_bytes_sent,
            total_bytes_received,
            avg_latency_ms: if count > 0 { total_latency_ms / count } else { 0 },
        }
    }
}

impl Default for StreamManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for StreamManager {
    fn clone(&self) -> Self {
        StreamManager {
            streams: std::sync::Arc::clone(&self.streams),
        }
    }
}

/// Stream Statistics
#[derive(Debug, Clone)]
pub struct StreamStats {
    pub active_streams: usize,
    pub total_streams: usize,
    pub total_bytes_sent: u64,
    pub total_bytes_received: u64,
    pub avg_latency_ms: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stream_creation() {
        let stream = QueryStream::new(1, "conn1".to_string());
        assert_eq!(stream.stream_id, 1);
        assert_eq!(stream.state, StreamState::Idle);
    }

    #[test]
    fn test_stream_lifecycle() {
        let mut stream = QueryStream::new(1, "conn1".to_string());

        stream.write_request(vec![1, 2, 3]).unwrap();
        assert_eq!(stream.state, StreamState::Open);

        stream.write_response(vec![4, 5, 6]).unwrap();
        assert_eq!(stream.state, StreamState::HalfClosed);

        stream.close();
        assert_eq!(stream.state, StreamState::Closed);
        assert!(!stream.is_active());
    }

    #[test]
    fn test_stream_manager() {
        let manager = StreamManager::new();
        let stream = manager.create_stream(1, "conn1".to_string());

        manager.write_request(stream.stream_id, vec![1, 2, 3]).unwrap();
        manager.write_response(stream.stream_id, vec![4, 5, 6]).unwrap();

        let stats = manager.stats();
        assert_eq!(stats.total_streams, 1);
        assert_eq!(stats.total_bytes_sent, 3);
        assert_eq!(stats.total_bytes_received, 3);
    }
}
