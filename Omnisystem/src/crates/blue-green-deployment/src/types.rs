use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Which physical slot ("blue" or "green") is currently serving live traffic.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Environment {
    Blue,
    Green,
}

impl Environment {
    /// The other slot.
    pub fn other(self) -> Environment {
        match self {
            Environment::Blue => Environment::Green,
            Environment::Green => Environment::Blue,
        }
    }
}

/// State of a single deployment slot.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EnvironmentState {
    pub environment: Environment,
    pub version: String,
    pub healthy: bool,
}

impl EnvironmentState {
    pub fn new(environment: Environment) -> Self {
        Self {
            environment,
            version: "unset".to_string(),
            healthy: false,
        }
    }
}

/// A record of a traffic switch (promotion or rollback).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SwitchEvent {
    pub from: Environment,
    pub to: Environment,
    pub version: String,
    pub at: DateTime<Utc>,
    pub is_rollback: bool,
}
