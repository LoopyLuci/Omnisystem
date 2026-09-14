/// DoT Server Configuration (RFC 7858)

#[derive(Debug, Clone)]
pub struct DoTConfig {
    pub listen_addr: String,
    pub port: u16,
    pub cert_path: String,
    pub key_path: String,
    pub max_connections: usize,
    pub timeout_ms: u64,
    pub tls_version: TlsVersion,
    pub cipher_suites: Vec<String>,
}

#[derive(Debug, Clone, Copy)]
pub enum TlsVersion {
    TLS12,
    TLS13,
}

impl DoTConfig {
    pub fn new(listen_addr: String, port: u16) -> Self {
        DoTConfig {
            listen_addr,
            port,
            cert_path: "/etc/certs/cert.pem".to_string(),
            key_path: "/etc/certs/key.pem".to_string(),
            max_connections: 10000,
            timeout_ms: 30000,
            tls_version: TlsVersion::TLS13,
            cipher_suites: vec![],
        }
    }

    pub fn with_cert(mut self, cert_path: String, key_path: String) -> Self {
        self.cert_path = cert_path;
        self.key_path = key_path;
        self
    }

    pub fn with_timeout(mut self, timeout_ms: u64) -> Self {
        self.timeout_ms = timeout_ms;
        self
    }
}

impl Default for DoTConfig {
    fn default() -> Self {
        Self::new("127.0.0.1".to_string(), 853)
    }
}
