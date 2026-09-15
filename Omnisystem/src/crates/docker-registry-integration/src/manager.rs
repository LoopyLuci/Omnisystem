use crate::error::{Error, Result};
use crate::types::{AuthState, Direction, Transfer, TransferState};
use std::collections::HashMap;
use std::sync::RwLock;

/// A registry's response to an authentication attempt. This crate never
/// makes real network calls or handles real credentials -- callers (and
/// tests) supply this directly, standing in for what a real client would
/// receive back from the registry's token endpoint.
#[derive(Debug, Clone)]
pub enum AuthOutcome {
    /// Authentication succeeded; a bearer token was granted with the given
    /// (fake) identifier and time-to-live in logical ticks.
    Granted {
        /// Opaque fake token identifier.
        token: String,
        /// How many ticks from the request tick until the token expires.
        ttl_ticks: u64,
    },
    /// Authentication was rejected (bad fake credentials, account locked, etc).
    Denied,
}

/// Models the registry push/pull protocol's state machine: authenticate
/// (against a caller-supplied mocked outcome, never a real registry),
/// then start/advance/complete transfers, refusing any transfer operation
/// while unauthenticated or expired.
pub struct Manager {
    auth: RwLock<AuthState>,
    transfers: RwLock<HashMap<String, Transfer>>,
}

impl Manager {
    /// Create a manager with no authentication yet attempted.
    pub fn new() -> Self {
        Self { auth: RwLock::new(AuthState::Unauthenticated), transfers: RwLock::new(HashMap::new()) }
    }

    /// Apply a (mocked) authentication outcome at the given logical tick.
    pub fn authenticate(&self, outcome: AuthOutcome, tick: u64) {
        let mut auth = self.auth.write().unwrap();
        *auth = match outcome {
            AuthOutcome::Granted { token, ttl_ticks } => {
                AuthState::Authenticated { token, expires_at_tick: tick.saturating_add(ttl_ticks) }
            }
            AuthOutcome::Denied => AuthState::Rejected,
        };
    }

    /// Whether the client is currently authenticated (token exists and
    /// hasn't expired as of `tick`).
    pub fn is_authenticated(&self, tick: u64) -> bool {
        matches!(&*self.auth.read().unwrap(), AuthState::Authenticated { expires_at_tick, .. } if *expires_at_tick > tick)
    }

    fn require_auth(&self, tick: u64) -> Result<()> {
        if self.is_authenticated(tick) {
            Ok(())
        } else {
            Err(Error::NotAuthenticated)
        }
    }

    /// Begin a push or pull transfer for `reference`, in the `Requested`
    /// state. Requires a currently-valid authentication.
    pub fn start_transfer(&self, id: &str, reference: &str, direction: Direction, tick: u64) -> Result<()> {
        self.require_auth(tick)?;
        self.transfers.write().unwrap().insert(
            id.to_string(),
            Transfer { reference: reference.to_string(), direction, state: TransferState::Requested },
        );
        Ok(())
    }

    /// Advance a transfer: `Requested` -> `InProgress` -> `Completed`.
    /// Requires a currently-valid authentication (a transfer can't
    /// complete if the token expired mid-flight).
    pub fn advance_transfer(&self, id: &str, tick: u64) -> Result<TransferState> {
        self.require_auth(tick)?;
        let mut transfers = self.transfers.write().unwrap();
        let transfer = transfers.get_mut(id).ok_or_else(|| Error::UnknownTransfer(id.to_string()))?;

        let next = match transfer.state {
            TransferState::Requested => TransferState::InProgress,
            TransferState::InProgress => TransferState::Completed,
            from @ (TransferState::Completed | TransferState::Failed) => {
                return Err(Error::InvalidTransferTransition { id: id.to_string(), from });
            }
        };
        transfer.state = next;
        Ok(next)
    }

    /// Mark an in-flight transfer as failed (e.g. connection drop, registry
    /// error). Valid from `Requested` or `InProgress`.
    pub fn fail_transfer(&self, id: &str) -> Result<()> {
        let mut transfers = self.transfers.write().unwrap();
        let transfer = transfers.get_mut(id).ok_or_else(|| Error::UnknownTransfer(id.to_string()))?;
        match transfer.state {
            TransferState::Requested | TransferState::InProgress => {
                transfer.state = TransferState::Failed;
                Ok(())
            }
            from => Err(Error::InvalidTransferTransition { id: id.to_string(), from }),
        }
    }

    /// Current state of a tracked transfer.
    pub fn transfer_state(&self, id: &str) -> Result<TransferState> {
        self.transfers.read().unwrap().get(id).map(|t| t.state).ok_or_else(|| Error::UnknownTransfer(id.to_string()))
    }
}

impl Default for Manager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn granted(ttl: u64) -> AuthOutcome {
        AuthOutcome::Granted { token: "fake-token-123".to_string(), ttl_ticks: ttl }
    }

    #[test]
    fn cannot_transfer_before_authenticating() {
        let m = Manager::new();
        let err = m.start_transfer("t1", "myorg/app:1.0", Direction::Push, 0).unwrap_err();
        assert!(matches!(err, Error::NotAuthenticated));
    }

    #[test]
    fn authenticated_client_can_push_through_completion() {
        let m = Manager::new();
        m.authenticate(granted(100), 0);
        assert!(m.is_authenticated(0));

        m.start_transfer("t1", "myorg/app:1.0", Direction::Push, 0).unwrap();
        assert_eq!(m.advance_transfer("t1", 1).unwrap(), TransferState::InProgress);
        assert_eq!(m.advance_transfer("t1", 2).unwrap(), TransferState::Completed);
    }

    #[test]
    fn denied_auth_blocks_transfers() {
        let m = Manager::new();
        m.authenticate(AuthOutcome::Denied, 0);
        assert!(!m.is_authenticated(0));
        assert!(m.start_transfer("t1", "myorg/app:1.0", Direction::Pull, 0).is_err());
    }

    #[test]
    fn expired_token_blocks_further_transfer_progress() {
        let m = Manager::new();
        m.authenticate(granted(5), 0);
        m.start_transfer("t1", "myorg/app:1.0", Direction::Pull, 0).unwrap();
        // Token expires at tick 5; advancing at tick 10 should be refused.
        let err = m.advance_transfer("t1", 10).unwrap_err();
        assert!(matches!(err, Error::NotAuthenticated));
    }

    #[test]
    fn completed_transfer_cannot_advance_further() {
        let m = Manager::new();
        m.authenticate(granted(100), 0);
        m.start_transfer("t1", "img", Direction::Push, 0).unwrap();
        m.advance_transfer("t1", 1).unwrap();
        m.advance_transfer("t1", 2).unwrap();
        assert!(matches!(
            m.advance_transfer("t1", 3).unwrap_err(),
            Error::InvalidTransferTransition { from: TransferState::Completed, .. }
        ));
    }

    #[test]
    fn failing_a_transfer_mid_flight_is_terminal() {
        let m = Manager::new();
        m.authenticate(granted(100), 0);
        m.start_transfer("t1", "img", Direction::Push, 0).unwrap();
        m.advance_transfer("t1", 1).unwrap();
        m.fail_transfer("t1").unwrap();
        assert_eq!(m.transfer_state("t1").unwrap(), TransferState::Failed);
        assert!(m.fail_transfer("t1").is_err());
    }

    #[test]
    fn unknown_transfer_operations_error() {
        let m = Manager::new();
        m.authenticate(granted(100), 0);
        assert!(matches!(m.advance_transfer("ghost", 0).unwrap_err(), Error::UnknownTransfer(_)));
    }
}
