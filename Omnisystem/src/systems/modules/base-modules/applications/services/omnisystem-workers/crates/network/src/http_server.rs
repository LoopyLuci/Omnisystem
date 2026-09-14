/// HTTPServerWorker - HTTP server request handling

use omnisystem_workers_core::prelude::*;
use async_trait::async_trait;
use std::time::Duration;

pub struct HTTPServerWorker;

pub struct HTTPResponse {
    pub status: u16,
    pub body: String,
}

#[async_trait]
impl Worker for HTTPServerWorker {
    type Input = String;
    type Output = HTTPResponse;

    async fn execute(&self, _request: Self::Input) -> WorkerResult<Self::Output> {
        Ok(HTTPResponse {
            status: 200,
            body: "OK".to_string(),
        })
    }

    fn name(&self) -> &str {
        "HTTPServerWorker"
    }

    fn timeout(&self) -> Duration {
        Duration::from_secs(10)
    }
}
