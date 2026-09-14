/// DoQ Query Handler (RFC 9250)

use aether_dns_core::protocol::DNSMessage;
use aether_dns_core::serialization::{DNSDeserializer, DNSSerializer};
use tracing::{debug, warn};

pub struct DoQHandler;

impl DoQHandler {
    /// Handle DoQ stream (RFC 9250)
    /// Message format: 2-byte length + DNS message
    pub async fn handle_stream(
        data: &[u8],
    ) -> anyhow::Result<Vec<u8>> {
        // Parse message with 2-byte length prefix
        if data.len() < 2 {
            return Err(anyhow::anyhow!("Message too short"));
        }

        let msg_len = u16::from_be_bytes([data[0], data[1]]) as usize;
        if data.len() < 2 + msg_len {
            return Err(anyhow::anyhow!("Incomplete message"));
        }

        let dns_data = &data[2..2 + msg_len];

        // Parse DNS message
        match DNSDeserializer::deserialize(dns_data) {
            Ok(mut dns_msg) => {
                debug!("Processed DoQ query ({} bytes)", msg_len);

                // Process query: set response flag
                dns_msg.header.flags.qr = true;
                dns_msg.header.flags.ra = true;

                // Serialize response
                match DNSSerializer::serialize(&dns_msg) {
                    Ok(response) => {
                        // Wrap with length prefix
                        Ok(Self::wrap_message(&response))
                    }
                    Err(e) => {
                        warn!("Failed to serialize response: {}", e);
                        Err(anyhow::anyhow!("Serialization failed: {}", e))
                    }
                }
            }
            Err(e) => {
                warn!("Failed to parse DoQ message: {}", e);
                Err(anyhow::anyhow!("Invalid DNS message: {}", e))
            }
        }
    }

    /// Wrap DNS message for DoQ (RFC 9250)
    pub fn wrap_message(msg: &[u8]) -> Vec<u8> {
        let len = msg.len() as u16;
        let mut wrapped = Vec::with_capacity(msg.len() + 2);
        wrapped.extend_from_slice(&len.to_be_bytes());
        wrapped.extend_from_slice(msg);
        wrapped
    }
}
