/// DNS-over-UDP Server (Port 53)
/// Standard DNS protocol over stateless UDP

pub mod server;
pub mod handler;

pub use server::UDPDNSServer;
pub use handler::UDPQueryHandler;
