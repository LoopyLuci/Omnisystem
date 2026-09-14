/// SocketWorker - Low-level socket operations

use omnisystem_workers_core::prelude::*;
use async_trait::async_trait;
use std::time::Duration;

pub struct SocketWorker;

pub enum SocketOperation {
    Create { protocol: String },
    Bind { address: String, port: u16 },
    Listen { backlog: u32 },
    Connect { address: String, port: u16 },
    Send { data: Vec<u8> },
    Receive { size: usize },
    Close,
}

pub enum SocketResult {
    Created(String),
    Bound(String),
    Listening,
    Connected,
    Sent(usize),
    Data(Vec<u8>),
    Closed,
}

#[async_trait]
impl Worker for SocketWorker {
    type Input = SocketOperation;
    type Output = SocketResult;

    async fn execute(&self, op: Self::Input) -> WorkerResult<Self::Output> {
        match op {
            SocketOperation::Create { protocol } => {
                Ok(SocketResult::Created(format!("socket://{}", protocol)))
            },
            SocketOperation::Bind { address, port } => {
                Ok(SocketResult::Bound(format!("{}:{}", address, port)))
            },
            SocketOperation::Listen { backlog } => {
                Ok(SocketResult::Listening)
            },
            SocketOperation::Connect { address, port } => {
                Ok(SocketResult::Connected)
            },
            SocketOperation::Send { data } => {
                Ok(SocketResult::Sent(data.len()))
            },
            SocketOperation::Receive { size } => {
                Ok(SocketResult::Data(vec![0u8; size]))
            },
            SocketOperation::Close => {
                Ok(SocketResult::Closed)
            }
        }
    }

    fn name(&self) -> &str {
        "SocketWorker"
    }

    fn timeout(&self) -> Duration {
        Duration::from_secs(30)
    }

    fn priority(&self) -> Priority {
        Priority::High
    }
}
