/// Anonymity Levels (0-5)

use serde::{Deserialize, Serialize};

/// 6 Anonymity Levels - Increasing Privacy
/// Level 0: Direct (fastest) → Level 5: Maximum (slowest but most private)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum AnonymityLevel {
    /// No anonymity - direct connection
    Level0 = 0,
    /// Single relay hop
    Level1 = 1,
    /// Double relay hop
    Level2 = 2,
    /// Triple relay hop (recommended)
    Level3 = 3,
    /// Onion routing (Tor-like)
    Level4 = 4,
    /// Maximum privacy (multi-relay + padding + timing obfuscation)
    Level5 = 5,
}

impl AnonymityLevel {
    pub fn from_u8(n: u8) -> Self {
        match n {
            0 => AnonymityLevel::Level0,
            1 => AnonymityLevel::Level1,
            2 => AnonymityLevel::Level2,
            3 => AnonymityLevel::Level3,
            4 => AnonymityLevel::Level4,
            5 => AnonymityLevel::Level5,
            _ => AnonymityLevel::Level0,
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            AnonymityLevel::Level0 => "Direct - No anonymity (fastest)",
            AnonymityLevel::Level1 => "Single relay hop",
            AnonymityLevel::Level2 => "Double relay hop",
            AnonymityLevel::Level3 => "Triple relay hop (recommended)",
            AnonymityLevel::Level4 => "Onion routing (Tor-like)",
            AnonymityLevel::Level5 => "Maximum privacy (multi-relay + obfuscation)",
        }
    }

    pub fn estimated_latency_ms(&self) -> u32 {
        match self {
            AnonymityLevel::Level0 => 5,
            AnonymityLevel::Level1 => 50,
            AnonymityLevel::Level2 => 100,
            AnonymityLevel::Level3 => 150,
            AnonymityLevel::Level4 => 250,
            AnonymityLevel::Level5 => 500,
        }
    }

    pub fn requires_padding(&self) -> bool {
        *self >= AnonymityLevel::Level2
    }

    pub fn requires_timing_obfuscation(&self) -> bool {
        *self >= AnonymityLevel::Level3
    }

    pub fn requires_decoy_traffic(&self) -> bool {
        *self >= AnonymityLevel::Level4
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_level_conversion() {
        assert_eq!(AnonymityLevel::from_u8(0), AnonymityLevel::Level0);
        assert_eq!(AnonymityLevel::from_u8(3), AnonymityLevel::Level3);
        assert_eq!(AnonymityLevel::from_u8(5), AnonymityLevel::Level5);
    }

    #[test]
    fn test_level_properties() {
        let level3 = AnonymityLevel::Level3;
        assert!(level3.requires_padding());
        assert!(level3.requires_timing_obfuscation());
        assert!(!level3.requires_decoy_traffic());
    }

    #[test]
    fn test_level_descriptions() {
        assert!(!AnonymityLevel::Level0.description().is_empty());
        assert!(!AnonymityLevel::Level5.description().is_empty());
    }
}
