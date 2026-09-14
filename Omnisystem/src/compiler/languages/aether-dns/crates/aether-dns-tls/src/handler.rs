/// DoT Query Handler (RFC 7858)

use aether_dns_core::protocol::DNSMessage;
use aether_dns_core::serialization::{DNSDeserializer, DNSSerializer};
use std::io::{Cursor, Read};
use tracing::{debug, warn};

pub struct DoTHandler;

impl DoTHandler {
    /// Handle incoming TLS stream
    /// RFC 7858: 2-byte message length prefix + DNS message
    pub async fn handle_connection(
        data: &[u8],
    ) -> anyhow::Result<Vec<u8>> {
        let mut cursor = Cursor::new(data);
        let mut responses = Vec::new();

        while cursor.position() < data.len() as u64 {
            // Read 2-byte message length (network byte order)
            if cursor.position() + 2 > data.len() as u64 {
                break;
            }

            let mut len_bytes = [0u8; 2];
            cursor.read_exact(&mut len_bytes)?;
            let msg_len = u16::from_be_bytes(len_bytes) as usize;

            // Read DNS message
            if cursor.position() + msg_len as u64 > data.len() as u64 {
                warn!("Incomplete DNS message");
                break;
            }

            let mut msg_bytes = vec![0u8; msg_len];
            cursor.read_exact(&mut msg_bytes)?;

            // Parse and process query
            match DNSDeserializer::deserialize(&msg_bytes) {
                Ok(mut dns_msg) => {
                    debug!("Processed DoT query ({} bytes)", msg_len);

                    // Process query: set response flag
                    dns_msg.header.flags.qr = true;
                    dns_msg.header.flags.ra = true;

                    // Serialize response
                    match DNSSerializer::serialize(&dns_msg) {
                        Ok(response) => {
                            // Wrap with length prefix
                            let wrapped = Self::wrap_message(&response);
                            responses.extend_from_slice(&wrapped);
                        }
                        Err(e) => {
                            warn!("Failed to serialize response: {}", e);
                        }
                    }
                }
                Err(e) => {
                    warn!("Failed to parse DNS message: {}", e);
                }
            }
        }

        Ok(responses)
    }

    /// Wrap DNS message with length prefix (RFC 7858)
    pub fn wrap_message(msg: &[u8]) -> Vec<u8> {
        let len = msg.len() as u16;
        let mut wrapped = Vec::with_capacity(msg.len() + 2);
        wrapped.extend_from_slice(&len.to_be_bytes());
        wrapped.extend_from_slice(msg);
        wrapped
    }

    /// Unwrap message with length prefix
    pub fn unwrap_message(data: &[u8]) -> anyhow::Result<(usize, &[u8])> {
        if data.len() < 2 {
            return Err(anyhow::anyhow!("Message too short"));
        }

        let len = u16::from_be_bytes([data[0], data[1]]) as usize;
        if data.len() < 2 + len {
            return Err(anyhow::anyhow!("Incomplete message"));
        }

        Ok((len, &data[2..2 + len]))
    }
}
