/// Timing Obfuscator
/// Adds jitter to prevent timing-based analysis

use crate::levels::AnonymityLevel;
use std::time::Duration;

#[derive(Clone)]
pub struct TimingObfuscator {
    min_jitter_ms: u32,
    max_jitter_ms: u32,
}

impl TimingObfuscator {
    pub fn new() -> Self {
        TimingObfuscator {
            min_jitter_ms: 1,
            max_jitter_ms: 50,
        }
    }

    pub async fn add_jitter(&self) {
        let jitter_ms = rand::random::<u32>() % (self.max_jitter_ms - self.min_jitter_ms)
            + self.min_jitter_ms;
        tokio::time::sleep(Duration::from_millis(jitter_ms as u64)).await;
    }

    pub async fn add_jitter_range(&self, min_ms: u32, max_ms: u32) {
        let jitter_ms = rand::random::<u32>() % (max_ms - min_ms) + min_ms;
        tokio::time::sleep(Duration::from_millis(jitter_ms as u64)).await;
    }

    pub fn get_random_delay_ms(&self) -> u32 {
        rand::random::<u32>() % (self.max_jitter_ms - self.min_jitter_ms) + self.min_jitter_ms
    }

    pub fn set_jitter_range(&mut self, min_ms: u32, max_ms: u32) {
        self.min_jitter_ms = min_ms;
        self.max_jitter_ms = max_ms;
    }

    /// Target jitter (ms) for a given anonymity level. Levels below the
    /// timing-obfuscation threshold (see `AnonymityLevel::requires_timing_obfuscation`)
    /// pay no jitter cost; higher levels scale linearly toward `max_jitter_ms`.
    pub fn jitter_for_level(&self, level: AnonymityLevel) -> u32 {
        if !level.requires_timing_obfuscation() {
            return 0;
        }
        let span = self.max_jitter_ms.saturating_sub(self.min_jitter_ms);
        let scaled = span * (level as u32) / (AnonymityLevel::Level5 as u32);
        self.min_jitter_ms + scaled
    }
}

impl Default for TimingObfuscator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_random_delay_in_range() {
        let obfuscator = TimingObfuscator::new();
        let delay = obfuscator.get_random_delay_ms();
        assert!(delay >= obfuscator.min_jitter_ms);
        assert!(delay <= obfuscator.max_jitter_ms);
    }

    #[tokio::test]
    async fn test_jitter_execution() {
        let obfuscator = TimingObfuscator::new();
        let start = std::time::Instant::now();
        obfuscator.add_jitter().await;
        let elapsed = start.elapsed().as_millis();
        // Should have some delay
        assert!(elapsed > 0);
    }
}
