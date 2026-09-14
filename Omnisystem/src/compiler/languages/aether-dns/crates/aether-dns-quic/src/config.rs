/// DoQ Server Configuration (RFC 9250)

#[derive(Debug, Clone)]
pub struct DoQConfig {
    pub listen_addr: String,
    pub port: u16,
    pub cert_path: String,
    pub key_path: String,
    pub max_connections: usize,
    pub idle_timeout_ms: u64,
    pub stream_receive_window: u64,
    pub connection_receive_window: u64,
}

impl DoQConfig {
    pub fn new(listen_addr: String, port: u16) -> Self {
        DoQConfig {
            listen_addr,
            port,
            cert_path: "/etc/certs/cert.pem".to_string(),
            key_path: "/etc/certs/key.pem".to_string(),
            max_connections: 10000,
            idle_timeout_ms: 30000,
            stream_receive_window: 1024 * 1024, // 1MB
            connection_receive_window: 10 * 1024 * 1024, // 10MB
        }
    }

    pub fn with_cert(mut self, cert_path: String, key_path: String) -> Self {
        self.cert_path = cert_path;
        self.key_path = key_path;
        self
    }
}

impl Default for DoQConfig {
    fn default() -> Self {
        Self::new("127.0.0.1".to_string(), 443)
    }
}
