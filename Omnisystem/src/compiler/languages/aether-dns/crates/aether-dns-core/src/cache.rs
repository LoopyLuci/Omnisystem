/// Efficient DNS Caching Layer
/// LRU cache with TTL management

use crate::protocol::{DNSRecord, RecordType};
use dashmap::DashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

#[derive(Clone)]
pub struct CacheEntry {
    pub records: Vec<DNSRecord>,
    pub inserted_at: Instant,
    pub ttl: u32,
    pub hit_count: Arc<AtomicU64>,
}

impl CacheEntry {
    pub fn is_expired(&self) -> bool {
        self.inserted_at.elapsed() > Duration::from_secs(self.ttl as u64)
    }

    pub fn ttl_remaining(&self) -> u32 {
        let elapsed = self.inserted_at.elapsed().as_secs();
        if elapsed >= self.ttl as u64 {
            0
        } else {
            (self.ttl as u64 - elapsed) as u32
        }
    }
}

pub struct CacheStats {
    pub hits: Arc<AtomicU64>,
    pub misses: Arc<AtomicU64>,
    pub evictions: Arc<AtomicU64>,
}

impl CacheStats {
    pub fn new() -> Self {
        CacheStats {
            hits: Arc::new(AtomicU64::new(0)),
            misses: Arc::new(AtomicU64::new(0)),
            evictions: Arc::new(AtomicU64::new(0)),
        }
    }

    pub fn hit_rate(&self) -> f64 {
        let hits = self.hits.load(Ordering::Relaxed);
        let total = hits + self.misses.load(Ordering::Relaxed);
        if total == 0 {
            0.0
        } else {
            hits as f64 / total as f64
        }
    }
}

pub struct DNSCache {
    entries: Arc<DashMap<String, CacheEntry>>,
    size_limit: usize,
    current_size: Arc<AtomicU64>,
    stats: Arc<CacheStats>,
}

impl DNSCache {
    pub fn new(size_limit: usize) -> Self {
        DNSCache {
            entries: Arc::new(DashMap::new()),
            size_limit,
            current_size: Arc::new(AtomicU64::new(0)),
            stats: Arc::new(CacheStats::new()),
        }
    }

    pub async fn get(&self, key: &str) -> Option<Vec<DNSRecord>> {
        if let Some(entry) = self.entries.get(key) {
            if !entry.is_expired() {
                entry.hit_count.fetch_add(1, Ordering::Relaxed);
                self.stats.hits.fetch_add(1, Ordering::Relaxed);
                return Some(entry.records.clone());
            } else {
                // Expired entry - remove it
                drop(entry);
                self.entries.remove(key);
            }
        }

        self.stats.misses.fetch_add(1, Ordering::Relaxed);
        None
    }

    pub async fn set(&self, key: String, records: Vec<DNSRecord>, ttl: u32) {
        // Calculate entry size (approximate)
        let entry_size = std::mem::size_of_val(&records) + records.iter().map(|r| r.rdata.len()).sum::<usize>();

        // Check if we need to evict
        let current = self.current_size.load(Ordering::Relaxed) as usize;
        if current + entry_size > self.size_limit {
            self.evict_lru().await;
        }

        let entry = CacheEntry {
            records,
            inserted_at: Instant::now(),
            ttl,
            hit_count: Arc::new(AtomicU64::new(0)),
        };

        self.entries.insert(key, entry);
        self.current_size.fetch_add(entry_size as u64, Ordering::Relaxed);
    }

    async fn evict_lru(&self) {
        // Evict 10% of entries with lowest hit count
        let count = self.entries.len();
        let to_evict = std::cmp::max(1, count / 10);

        let mut entries: Vec<_> = self.entries.iter()
            .map(|r| (r.key().clone(), r.hit_count.load(Ordering::Relaxed)))
            .collect();

        entries.sort_by_key(|e| e.1);

        for (key, _) in entries.iter().take(to_evict) {
            if let Some((_, entry)) = self.entries.remove(key) {
                let size = std::mem::size_of_val(&entry.records) + entry.records.iter().map(|r| r.rdata.len()).sum::<usize>();
                self.current_size.fetch_sub(size as u64, Ordering::Relaxed);
                self.stats.evictions.fetch_add(1, Ordering::Relaxed);
            }
        }
    }

    pub async fn clear(&self) {
        self.entries.clear();
        self.current_size.store(0, Ordering::Relaxed);
    }

    pub fn size(&self) -> usize {
        self.current_size.load(Ordering::Relaxed) as usize
    }

    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }

    pub fn hit_rate(&self) -> f64 {
        self.stats.hit_rate()
    }

    pub fn stats(&self) -> (u64, u64, u64) {
        (
            self.stats.hits.load(Ordering::Relaxed),
            self.stats.misses.load(Ordering::Relaxed),
            self.stats.evictions.load(Ordering::Relaxed),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cache_hit() {
        let cache = DNSCache::new(1024 * 1024);
        let records = vec![];
        let key = "example.com".to_string();

        cache.set(key.clone(), records.clone(), 300).await;
        let result = cache.get(&key).await;
        assert!(result.is_some());
    }

    #[tokio::test]
    async fn test_cache_miss() {
        let cache = DNSCache::new(1024 * 1024);
        let result = cache.get("nonexistent.com").await;
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_cache_expiry() {
        let cache = DNSCache::new(1024 * 1024);
        let records = vec![];
        let key = "example.com".to_string();

        cache.set(key.clone(), records, 1).await;  // 1 second TTL
        let result = cache.get(&key).await;
        assert!(result.is_some());

        // Wait for expiry
        tokio::time::sleep(Duration::from_secs(2)).await;
        let result = cache.get(&key).await;
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_cache_hit_rate() {
        let cache = DNSCache::new(1024 * 1024);
        let records = vec![];
        let key = "example.com".to_string();

        cache.set(key.clone(), records, 300).await;
        cache.get(&key).await;
        cache.get(&key).await;
        cache.get("missing.com").await;

        let rate = cache.hit_rate();
        assert!(rate > 0.5);  // 2 hits out of 3
    }
}
