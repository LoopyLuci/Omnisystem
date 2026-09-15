//! Registry tracking secret metadata and access grants, and computing
//! rotation-due status across the fleet.

use crate::error::{Error, Result};
use crate::types::{AccessGrant, SecretMetadata};
use std::collections::BTreeMap;

/// In-memory secret metadata + access-grant registry.
#[derive(Debug, Default)]
pub struct SecretRegistry {
    secrets: BTreeMap<String, SecretMetadata>,
    grants: Vec<AccessGrant>,
}

impl SecretRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Register (or replace) a secret's metadata.
    pub fn register(&mut self, metadata: SecretMetadata) {
        self.secrets.insert(metadata.name.clone(), metadata);
    }

    /// Look up a secret's metadata.
    pub fn get(&self, name: &str) -> Result<&SecretMetadata> {
        self.secrets.get(name).ok_or_else(|| Error::UnknownSecret(name.to_string()))
    }

    /// Record that a secret was rotated: bumps the version, resets age to
    /// zero.
    pub fn rotate(&mut self, name: &str) -> Result<&SecretMetadata> {
        let meta = self.secrets.get_mut(name).ok_or_else(|| Error::UnknownSecret(name.to_string()))?;
        meta.version += 1;
        meta.age_days = 0;
        Ok(meta)
    }

    /// All secrets currently overdue for rotation, sorted by name.
    pub fn overdue_secrets(&self) -> Vec<&SecretMetadata> {
        self.secrets.values().filter(|m| m.rotation_due()).collect()
    }

    /// Grant access to a secret for a principal.
    pub fn grant_access(&mut self, grant: AccessGrant) -> Result<()> {
        if !self.secrets.contains_key(&grant.secret_name) {
            return Err(Error::UnknownSecret(grant.secret_name.clone()));
        }
        // Replace any existing grant for the same (secret, principal) pair.
        self.grants.retain(|g| !(g.secret_name == grant.secret_name && g.principal == grant.principal));
        self.grants.push(grant);
        Ok(())
    }

    /// Revoke a principal's access grant on a secret.
    pub fn revoke_access(&mut self, secret_name: &str, principal: &str) -> Result<()> {
        let before = self.grants.len();
        self.grants.retain(|g| !(g.secret_name == secret_name && g.principal == principal));
        if self.grants.len() == before {
            return Err(Error::UnknownGrant { secret: secret_name.to_string(), principal: principal.to_string() });
        }
        Ok(())
    }

    /// All principals with any access grant on a secret.
    pub fn grants_for(&self, secret_name: &str) -> Vec<&AccessGrant> {
        self.grants.iter().filter(|g| g.secret_name == secret_name).collect()
    }

    /// Whether a principal can read a given secret (has a grant with
    /// `can_read`).
    pub fn can_read(&self, secret_name: &str, principal: &str) -> bool {
        self.grants.iter().any(|g| g.secret_name == secret_name && g.principal == principal && g.can_read)
    }
}
