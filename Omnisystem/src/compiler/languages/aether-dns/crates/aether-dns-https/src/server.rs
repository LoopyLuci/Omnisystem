/// DoH Server Implementation (RFC 8484)

use actix_web::{web, App, HttpServer, middleware::Logger};
use actix_cors::Cors;
use std::sync::Arc;
use tracing::info;
use crate::handler::DoHHandler;
use crate::config::DoHConfig;

pub struct DoHServer {
    config: DoHConfig,
}

impl DoHServer {
    pub fn new(config: DoHConfig) -> Self {
        DoHServer { config }
    }

    pub async fn run(&self) -> anyhow::Result<()> {
        let config = self.config.clone();
        let addr = format!("{}:{}", config.listen_addr, config.port);

        info!("Starting DoH server on https://{}/dns-query", addr);

        HttpServer::new(move || {
            let cors = Cors::default()
                .allow_any_origin()
                .allow_any_method()
                .allow_any_header()
                .max_age(3600);

            App::new()
                .wrap(cors)
                .wrap(Logger::default())
                .service(
                    web::scope("/dns-query")
                        .route("", web::post().to(DoHHandler::handle_post))
                        .route("", web::get().to(DoHHandler::handle_get))
                        .route("", web::head().to(DoHHandler::handle_head))
                )
                .service(
                    web::scope("/api/dns")
                        .route("", web::post().to(DoHHandler::handle_post))
                        .route("", web::get().to(DoHHandler::handle_get))
                )
        })
        .bind(&addr)?
        .run()
        .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_creation() {
        let config = DoHConfig::new("127.0.0.1".to_string(), 443);
        assert_eq!(config.port, 443);
    }

    #[test]
    fn test_server_creation() {
        let config = DoHConfig::default();
        let _server = DoHServer::new(config);
    }
}
