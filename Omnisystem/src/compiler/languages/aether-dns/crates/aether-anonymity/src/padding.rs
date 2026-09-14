/// Padding Strategy
/// Obscures message size through padding

use crate::levels::AnonymityLevel;
use rand::Rng;
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct PaddingStrategy {
    min_size: usize,
    max_size: usize,
}

impl PaddingStrategy {
    pub fn new() -> Self {
        PaddingStrategy {
            min_size: 512,
            max_size: 4096,
        }
    }

    pub fn pad(&self, data: &[u8]) -> Vec<u8> {
        let mut rng = rand::thread_rng();
        let target_size = rng.gen_range(self.min_size..=self.max_size);

        if data.len() >= target_size {
            return data.to_vec();
        }

        let mut padded = Vec::with_capacity(target_size);
        padded.extend_from_slice(data);

        // Add random padding
        let padding_size = target_size - data.len();
        let padding: Vec<u8> = (0..padding_size)
            .map(|_| rng.gen())
            .collect();

        padded.extend_from_slice(&padding);
        padded
    }

    pub fn unpad(&self, data: &[u8]) -> Vec<u8> {
        // For now, return as-is (in real implementation, would strip padding)
        data.to_vec()
    }

    pub fn set_size_range(&mut self, min: usize, max: usize) {
        self.min_size = min;
        self.max_size = max;
    }

    /// Target padded size for a given anonymity level. Levels below the
    /// padding threshold (see `AnonymityLevel::requires_padding`) pay no
    /// padding cost; higher levels scale linearly toward `max_size`.
    pub fn padding_for_level(&self, level: AnonymityLevel) -> usize {
        if !level.requires_padding() {
            return 0;
        }
        let span = self.max_size.saturating_sub(self.min_size);
        let scaled = span * (level as usize) / (AnonymityLevel::Level5 as usize);
        self.min_size + scaled
    }
}

impl Default for PaddingStrategy {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaddingConfig {
    pub min_size: usize,
    pub max_size: usize,
    pub block_size: usize,
}

impl Default for PaddingConfig {
    fn default() -> Self {
        PaddingConfig {
            min_size: 512,
            max_size: 4096,
            block_size: 256,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_padding_increases_size() {
        let strategy = PaddingStrategy::new();
        let data = b"short";
        let padded = strategy.pad(data);
        assert!(padded.len() >= strategy.min_size);
    }

    #[test]
    fn test_padding_contains_original() {
        let strategy = PaddingStrategy::new();
        let data = b"test message";
        let padded = strategy.pad(data);
        assert!(padded.starts_with(data));
    }
}
