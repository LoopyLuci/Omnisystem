/// DoH Request Handler (RFC 8484)

use actix_web::{web, HttpRequest, HttpResponse};
use aether_dns_core::protocol::DNSMessage;
use aether_dns_core::serialization::{DNSDeserializer, DNSSerializer};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use tracing::{debug, warn};
use serde_json::json;

pub struct DoHHandler;

impl DoHHandler {
    /// Handle POST request (RFC 8484 POST method)
    /// Body is raw DNS wire format
    pub async fn handle_post(
        body: web::Bytes,
        _req: HttpRequest,
    ) -> HttpResponse {
        debug!("DoH POST request: {} bytes", body.len());

        // Parse DNS message from wire format
        match DNSDeserializer::deserialize(&body) {
            Ok(mut dns_msg) => {
                // Process query: set response flag
                dns_msg.header.flags.qr = true;
                dns_msg.header.flags.ra = true;

                // Serialize response to wire format
                match DNSSerializer::serialize(&dns_msg) {
                    Ok(response_bytes) => {
                        HttpResponse::Ok()
                            .content_type("application/dns-message")
                            .body(response_bytes)
                    }
                    Err(e) => {
                        warn!("Failed to serialize DoH response: {}", e);
                        HttpResponse::InternalServerError()
                            .json(json!({"error": "Serialization failed"}))
                    }
                }
            }
            Err(e) => {
                warn!("Failed to parse DoH request: {}", e);
                HttpResponse::BadRequest()
                    .content_type("application/json")
                    .json(json!({
                        "error": "Invalid DNS message format",
                        "details": e.to_string()
                    }))
            }
        }
    }

    /// Handle GET request (RFC 8484 GET method)
    /// Query is base64url encoded DNS wire format
    pub async fn handle_get(
        query: web::Query<std::collections::HashMap<String, String>>,
        _req: HttpRequest,
    ) -> HttpResponse {
        debug!("DoH GET request");

        if let Some(dns_param) = query.get("dns") {
            // Decode base64url encoded DNS message
            match URL_SAFE_NO_PAD.decode(dns_param) {
                Ok(decoded) => {
                    // Parse DNS message
                    match DNSDeserializer::deserialize(&decoded) {
                        Ok(mut dns_msg) => {
                            // Process query: set response flag
                            dns_msg.header.flags.qr = true;
                            dns_msg.header.flags.ra = true;

                            // Serialize response
                            match DNSSerializer::serialize(&dns_msg) {
                                Ok(response_bytes) => {
                                    let encoded = URL_SAFE_NO_PAD.encode(&response_bytes);
                                    HttpResponse::Ok()
                                        .content_type("application/dns-message")
                                        .body(encoded)
                                }
                                Err(e) => {
                                    warn!("Failed to serialize: {}", e);
                                    HttpResponse::InternalServerError()
                                        .json(json!({"error": "Serialization failed"}))
                                }
                            }
                        }
                        Err(e) => {
                            warn!("Failed to parse DNS message: {}", e);
                            HttpResponse::BadRequest()
                                .json(json!({
                                    "error": "Invalid DNS message format"
                                }))
                        }
                    }
                }
                Err(e) => {
                    warn!("Failed to decode base64: {}", e);
                    HttpResponse::BadRequest()
                        .json(json!({
                            "error": "Invalid base64url encoding"
                        }))
                }
            }
        } else {
            HttpResponse::BadRequest()
                .json(json!({
                    "error": "Missing 'dns' parameter"
                }))
        }
    }

    /// Handle HEAD request (RFC 8484 HEAD method)
    pub async fn handle_head(_req: HttpRequest) -> HttpResponse {
        HttpResponse::Ok()
            .content_type("application/dns-message")
            .finish()
    }
}
