//! Error types

/// Errors produced by registry-integration operations.
#[derive(Debug, Clone)]
pub enum Error {
    /// The client isn't authenticated (or its token expired) for this operation.
    NotAuthenticated,
    /// No transfer is tracked under this id.
    UnknownTransfer(String),
    /// The requested state transition isn't valid for a transfer's current state.
    InvalidTransferTransition {
        /// The transfer id.
        id: String,
        /// The state the transfer was in.
        from: crate::types::TransferState,
    },
    /// Other error
    Other(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::NotAuthenticated => write!(f, "not authenticated with the registry"),
            Error::UnknownTransfer(id) => write!(f, "no transfer tracked with id '{}'", id),
            Error::InvalidTransferTransition { id, from } => {
                write!(f, "transfer '{}' cannot advance from state {:?}", id, from)
            }
            Error::Other(msg) => write!(f, "Error: {}", msg),
        }
    }
}

impl std::error::Error for Error {}

/// Result type
pub type Result<T> = std::result::Result<T, Error>;
