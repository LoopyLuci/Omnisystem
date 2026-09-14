/// DNS-over-QUIC Server (DoQ)
/// RFC 9250: DNS over QUIC

pub mod server;
pub mod handler;
pub mod config;
pub mod connection;
pub mod stream;

pub use server::DoQServer;
pub use handler::DoQHandler;
pub use config::DoQConfig;
pub use connection::{QuicConnection, ConnectionManager};
pub use stream::{QueryStream, StreamManager};
