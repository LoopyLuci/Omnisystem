/// Obfuscation Engine
/// Multi-layer query obfuscation

use chacha20poly1305::{
    aead::{Aead, KeyInit},
    ChaCha20Poly1305, Nonce,
};
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct ObfuscationEngine {
    cipher: ChaCha20Poly1305,
}

impl ObfuscationEngine {
    pub fn new() -> Self {
        let key = ChaCha20Poly1305::generate_key(&mut rand::thread_rng());
        let cipher = ChaCha20Poly1305::new(&key);
        ObfuscationEngine { cipher }
    }

    pub fn encrypt(&self, data: &[u8]) -> anyhow::Result<Vec<u8>> {
        // Generate random nonce
        use rand::RngCore;
        let mut nonce_bytes = [0u8; 12];
        rand::thread_rng().fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from(nonce_bytes);

        // Encrypt
        let ciphertext = self
            .cipher
            .encrypt(&nonce, data)
            .map_err(|e| anyhow::anyhow!("Encryption failed: {}", e))?;

        // Return nonce + ciphertext
        let mut result = nonce.to_vec();
        result.extend_from_slice(&ciphertext);
        Ok(result)
    }

    pub fn decrypt(&self, data: &[u8]) -> anyhow::Result<Vec<u8>> {
        if data.len() < 12 {
            return Err(anyhow::anyhow!("Invalid ciphertext length"));
        }

        let nonce = Nonce::from_slice(&data[0..12]);
        let plaintext = self
            .cipher
            .decrypt(nonce, &data[12..])
            .map_err(|e| anyhow::anyhow!("Decryption failed: {}", e))?;

        Ok(plaintext)
    }
}

impl Default for ObfuscationEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObfuscationConfig {
    pub use_encryption: bool,
    pub use_padding: bool,
    pub use_timing_obfuscation: bool,
    pub add_decoy_traffic: bool,
}

impl Default for ObfuscationConfig {
    fn default() -> Self {
        ObfuscationConfig {
            use_encryption: true,
            use_padding: true,
            use_timing_obfuscation: true,
            add_decoy_traffic: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encryption_decryption() {
        let engine = ObfuscationEngine::new();
        let plaintext = b"test data".to_vec();
        let encrypted = engine.encrypt(&plaintext).unwrap();
        let decrypted = engine.decrypt(&encrypted).unwrap();
        assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn test_nonce_uniqueness() {
        let engine = ObfuscationEngine::new();
        let data = b"same data";
        let encrypted1 = engine.encrypt(data).unwrap();
        let encrypted2 = engine.encrypt(data).unwrap();
        // Different nonces produce different ciphertexts
        assert_ne!(encrypted1, encrypted2);
    }
}
