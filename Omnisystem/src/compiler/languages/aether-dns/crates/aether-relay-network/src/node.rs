/// Relay Node Definition

use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use std::time::Instant;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelayNodeInfo {
    pub id: String,
    pub pubkey: Vec<u8>,
    pub endpoints: Vec<String>, // Multiple endpoints for load balancing
    pub latency_ms: u32,
    pub bandwidth_mbps: u32,
    pub reliability: f64,        // 0.0-1.0
    pub privacy_rating: f64,     // 0.0-5.0
    pub location: GeoLocation,
    pub supports_ipv6: bool,
    pub supports_quic: bool,
    pub supports_tls: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeoLocation {
    pub country: String,
    pub city: String,
    pub asn: u32,
    pub latitude: f64,
    pub longitude: f64,
}

impl GeoLocation {
    pub fn distance_to(&self, other: &GeoLocation) -> f64 {
        // Haversine distance formula
        let lat1 = self.latitude.to_radians();
        let lat2 = other.latitude.to_radians();
        let dlat = (other.latitude - self.latitude).to_radians();
        let dlon = (other.longitude - self.longitude).to_radians();

        let a = (dlat / 2.0).sin().powi(2)
            + lat1.cos() * lat2.cos() * (dlon / 2.0).sin().powi(2);
        let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());
        let earth_radius_km = 6371.0;

        earth_radius_km * c
    }
}

#[derive(Clone)]
pub struct RelayNode {
    pub info: RelayNodeInfo,
    pub last_heartbeat: Instant,
    pub query_count: u64,
}

impl RelayNode {
    pub fn new(info: RelayNodeInfo) -> Self {
        RelayNode {
            info,
            last_heartbeat: Instant::now(),
            query_count: 0,
        }
    }

    pub fn update_heartbeat(&mut self) {
        self.last_heartbeat = Instant::now();
    }

    pub fn is_healthy(&self, timeout_secs: u64) -> bool {
        self.last_heartbeat.elapsed().as_secs() < timeout_secs && self.info.reliability > 0.95
    }

    pub fn record_query(&mut self) {
        self.query_count += 1;
        self.update_heartbeat();
    }

    pub fn score(&self) -> f64 {
        // Composite score based on reliability, privacy, and latency
        let reliability_weight = 0.4;
        let privacy_weight = 0.4;
        let latency_weight = 0.2;

        let latency_score = 1.0 - (self.info.latency_ms as f64 / 500.0).min(1.0);

        reliability_weight * self.info.reliability
            + privacy_weight * (self.info.privacy_rating / 5.0)
            + latency_weight * latency_score
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_geo_location_distance() {
        let loc1 = GeoLocation {
            country: "US".to_string(),
            city: "New York".to_string(),
            asn: 1234,
            latitude: 40.7128,
            longitude: -74.0060,
        };

        let loc2 = GeoLocation {
            country: "US".to_string(),
            city: "Los Angeles".to_string(),
            asn: 5678,
            latitude: 34.0522,
            longitude: -118.2437,
        };

        let distance = loc1.distance_to(&loc2);
        assert!(distance > 3000.0); // ~4000km
        assert!(distance < 5000.0);
    }

    #[test]
    fn test_relay_node_health() {
        let info = RelayNodeInfo {
            id: "node1".to_string(),
            pubkey: vec![0; 32],
            endpoints: vec!["127.0.0.1:9000".to_string()],
            latency_ms: 50,
            bandwidth_mbps: 1000,
            reliability: 0.99,
            privacy_rating: 4.8,
            location: GeoLocation {
                country: "US".to_string(),
                city: "SF".to_string(),
                asn: 1234,
                latitude: 37.7749,
                longitude: -122.4194,
            },
            supports_ipv6: true,
            supports_quic: true,
            supports_tls: true,
        };

        let node = RelayNode::new(info);
        assert!(node.is_healthy(300));
    }

    #[test]
    fn test_node_scoring() {
        let info = RelayNodeInfo {
            id: "node1".to_string(),
            pubkey: vec![0; 32],
            endpoints: vec!["127.0.0.1:9000".to_string()],
            latency_ms: 50,
            bandwidth_mbps: 1000,
            reliability: 0.99,
            privacy_rating: 4.8,
            location: GeoLocation {
                country: "US".to_string(),
                city: "SF".to_string(),
                asn: 1234,
                latitude: 37.7749,
                longitude: -122.4194,
            },
            supports_ipv6: true,
            supports_quic: true,
            supports_tls: true,
        };

        let node = RelayNode::new(info);
        let score = node.score();
        assert!(score > 0.0 && score <= 1.0);
    }
}
