/// HTTPClientWorker - HTTP/HTTPS requests

use omnisystem_workers_core::prelude::*;
use async_trait::async_trait;
use std::time::Duration;

pub struct HTTPClientWorker;

pub struct HTTPRequest {
    pub url: String,
    pub method: String,
    pub headers: std::collections::HashMap<String, String>,
    pub body: Option<Vec<u8>>,
}

#[async_trait]
impl Worker for HTTPClientWorker {
    type Input = HTTPRequest;
    type Output = String;

    async fn execute(&self, request: Self::Input) -> WorkerResult<Self::Output> {
        let client = reqwest::Client::new();
        let method = match request.method.as_str() {
            "GET" => reqwest::Method::GET,
            "POST" => reqwest::Method::POST,
            "PUT" => reqwest::Method::PUT,
            "DELETE" => reqwest::Method::DELETE,
            _ => reqwest::Method::GET,
        };

        let mut req = client.request(method, &request.url);

        for (key, value) in request.headers {
            req = req.header(key, value);
        }

        if let Some(body) = request.body {
            req = req.body(body);
        }

        match req.send().await {
            Ok(response) => {
                match response.text().await {
                    Ok(text) => Ok(text),
                    Err(e) => Err(WorkerError::ExecutionFailed(e.to_string())),
                }
            }
            Err(e) => Err(WorkerError::ExecutionFailed(e.to_string())),
        }
    }

    fn name(&self) -> &str {
        "HTTPClientWorker"
    }

    fn timeout(&self) -> Duration {
        Duration::from_secs(30)
    }
}
