//! Error types

/// Errors produced by network-manager operations.
#[derive(Debug, Clone)]
pub enum Error {
    /// The CIDR subnet string could not be parsed.
    InvalidSubnet(String),
    /// A network with this name already exists.
    NetworkAlreadyExists(String),
    /// No network is tracked under this name.
    NetworkNotFound(String),
    /// `Host`-mode networks have no subnet, so IP allocation is impossible.
    NoSubnetForHostMode(String),
    /// The subnet has no more free addresses to allocate.
    SubnetExhausted(String),
    /// The container is already attached to this network.
    AlreadyAttached {
        /// Container id.
        container: String,
        /// Network name.
        network: String,
    },
    /// The container is not attached to this network.
    NotAttached {
        /// Container id.
        container: String,
        /// Network name.
        network: String,
    },
    /// Other error
    Other(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::InvalidSubnet(s) => write!(f, "invalid subnet: '{}'", s),
            Error::NetworkAlreadyExists(n) => write!(f, "network '{}' already exists", n),
            Error::NetworkNotFound(n) => write!(f, "network '{}' not found", n),
            Error::NoSubnetForHostMode(n) => write!(f, "network '{}' is host-mode and has no subnet", n),
            Error::SubnetExhausted(n) => write!(f, "subnet for network '{}' is exhausted", n),
            Error::AlreadyAttached { container, network } => {
                write!(f, "container '{}' is already attached to network '{}'", container, network)
            }
            Error::NotAttached { container, network } => {
                write!(f, "container '{}' is not attached to network '{}'", container, network)
            }
            Error::Other(msg) => write!(f, "Error: {}", msg),
        }
    }
}

impl std::error::Error for Error {}

/// Result type
pub type Result<T> = std::result::Result<T, Error>;
