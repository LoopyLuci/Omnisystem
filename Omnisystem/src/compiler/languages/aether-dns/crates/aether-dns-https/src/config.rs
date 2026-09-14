/// DoH Server Configuration

#[derive(Debug, Clone)]
pub struct DoHConfig {
    pub listen_addr: String,
    pub port: u16,
    pub cert_path: String,
    pub key_path: String,
    pub max_body_size: usize,
    pub timeout_ms: u64,
    pub max_requests_per_second: u32,
}

impl DoHConfig {
    pub fn new(listen_addr: String, port: u16) -> Self {
        DoHConfig {
            listen_addr,
            port,
            cert_path: "/etc/certs/cert.pem".to_string(),
            key_path: "/etc/certs/key.pem".to_string(),
            max_body_size: 4096,
            timeout_ms: 5000,
            max_requests_per_second: 1000,
        }
    }

    pub fn with_cert(mut self, cert_path: String, key_path: String) -> Self {
        self.cert_path = cert_path;
        self.key_path = key_path;
        self
    }
}

impl Default for DoHConfig {
    fn default() -> Self {
        Self::new("127.0.0.1".to_string(), 443)
    }
}
