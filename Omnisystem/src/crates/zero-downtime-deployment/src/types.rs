use serde::{Deserialize, Serialize};

/// Lifecycle state of one service instance being replaced.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InstanceState {
    pub name: String,
    pub ready: bool,
    pub draining: bool,
    pub in_flight_requests: u32,
}

impl InstanceState {
    pub fn new(name: String) -> Self {
        Self {
            name,
            ready: false,
            draining: false,
            in_flight_requests: 0,
        }
    }
}
