// Licensed under the MIT License
// Copyright (c) 2025 Hal <hal.long@outlook.com>

//! DNS Cache System
//!
//! This module implements a high-performance DNS cache to reduce DNS query latency
//! and improve overall download performance.

use dashmap::DashMap;
use std::net::IpAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// DNS cache entry
#[derive(Debug, Clone)]
pub struct DnsCacheEntry {
    /// Resolved IP addresses
    pub addresses: Vec<IpAddr>,
    /// Time when entry was created
    pub created_at: Instant,
    /// Time-to-live for this entry
    pub ttl: Duration,
    /// Number of times this entry has been used
    pub hit_count: u64,
    /// Last time this entry was accessed
    pub last_accessed: Instant,
}

impl DnsCacheEntry {
    /// Check if this cache entry is still valid
    pub fn is_valid(&self) -> bool {
        self.created_at.elapsed() < self.ttl
    }

    /// Check if this entry is expired
    pub fn is_expired(&self) -> bool {
        !self.is_valid()
    }

    /// Update access statistics
    pub fn mark_accessed(&mut self) {
        self.hit_count += 1;
        self.last_accessed = Instant::now();
    }
}

/// DNS cache statistics
#[derive(Debug, Clone)]
pub struct DnsCacheStats {
    /// Total number of cache entries
    pub total_entries: usize,
    /// Number of cache hits
    pub cache_hits: u64,
    /// Number of cache misses
    pub cache_misses: u64,
    /// Cache hit ratio
    pub hit_ratio: f64,
    /// Number of expired entries cleaned up
    pub expired_cleaned: u64,
    /// Total DNS resolution time saved (estimated)
    pub time_saved_ms: u64,
}

/// High-performance DNS cache
#[derive(Debug)]
pub struct DnsCache {
    /// Cache storage using DashMap for concurrent access
    cache: DashMap<String, DnsCacheEntry>,
    /// Cache statistics
    stats: Arc<RwLock<DnsCacheStats>>,
    /// Default TTL for cache entries
    default_ttl: Duration,
    /// Maximum number of cache entries
    max_entries: usize,
    /// Cleanup interval
    cleanup_interval: Duration,
    /// Last cleanup time
    last_cleanup: Arc<RwLock<Instant>>,
}

impl DnsCache {
    /// Create a new DNS cache
    pub fn new(default_ttl: Duration, max_entries: usize) -> Self {
        Self {
            cache: DashMap::new(),
            stats: Arc::new(RwLock::new(DnsCacheStats {
                total_entries: 0,
                cache_hits: 0,
                cache_misses: 0,
                hit_ratio: 0.0,
                expired_cleaned: 0,
                time_saved_ms: 0,
            })),
            default_ttl,
            max_entries,
            cleanup_interval: Duration::from_secs(300), // 5 minutes
            last_cleanup: Arc::new(RwLock::new(Instant::now())),
        }
    }

    /// Create a DNS cache with default settings
    pub fn with_defaults() -> Self {
        Self::new(
            Duration::from_secs(300), // 5 minutes TTL
            1000,                     // Max 1000 entries
        )
    }

    /// Resolve hostname with caching
    pub async fn resolve(&self, hostname: &str) -> Option<Vec<IpAddr>> {
        // Check cache first
        if let Some(mut entry) = self.cache.get_mut(hostname) {
            if entry.is_valid() {
                entry.mark_accessed();

                // Update statistics
                let mut stats = self.stats.write().await;
                stats.cache_hits += 1;
                stats.time_saved_ms += 50; // Estimate 50ms saved per cache hit
                self.update_hit_ratio(&mut stats);
                drop(stats);

                debug!(
                    "DNS cache hit for {}: {} addresses",
                    hostname,
                    entry.addresses.len()
                );
                return Some(entry.addresses.clone());
            } else {
                // Entry expired, remove it
                drop(entry);
                self.cache.remove(hostname);
                debug!("Removed expired DNS cache entry for {}", hostname);
            }
        }

        // Cache miss - perform actual DNS resolution
        match self.perform_dns_resolution(hostname).await {
            Ok(addresses) => {
                if !addresses.is_empty() {
                    self.insert(hostname, addresses.clone(), None).await;

                    // Update statistics
                    let mut stats = self.stats.write().await;
                    stats.cache_misses += 1;
                    self.update_hit_ratio(&mut stats);
                    drop(stats);

                    debug!(
                        "DNS resolved and cached for {}: {} addresses",
                        hostname,
                        addresses.len()
                    );
                    Some(addresses)
                } else {
                    warn!("DNS resolution returned no addresses for {}", hostname);
                    None
                }
            }
            Err(e) => {
                warn!("DNS resolution failed for {}: {}", hostname, e);

                // Update statistics
                let mut stats = self.stats.write().await;
                stats.cache_misses += 1;
                self.update_hit_ratio(&mut stats);
                drop(stats);

                None
            }
        }
    }

    /// Insert entry into cache
    pub async fn insert(&self, hostname: &str, addresses: Vec<IpAddr>, ttl: Option<Duration>) {
        // Check if cleanup is needed
        self.maybe_cleanup().await;

        // Enforce max entries limit
        if self.cache.len() >= self.max_entries {
            self.evict_oldest_entries().await;
        }

        let entry = DnsCacheEntry {
            addresses,
            created_at: Instant::now(),
            ttl: ttl.unwrap_or(self.default_ttl),
            hit_count: 0,
            last_accessed: Instant::now(),
        };

        self.cache.insert(hostname.to_string(), entry);

        // Update statistics
        let mut stats = self.stats.write().await;
        stats.total_entries = self.cache.len();
        drop(stats);

        debug!(
            "Cached DNS entry for {} (TTL: {:?})",
            hostname, self.default_ttl
        );
    }

    /// Perform actual DNS resolution
    async fn perform_dns_resolution(
        &self,
        hostname: &str,
    ) -> Result<Vec<IpAddr>, Box<dyn std::error::Error + Send + Sync>> {
        use tokio::net::lookup_host;

        let start = Instant::now();
        let addrs: Vec<IpAddr> = lookup_host((hostname, 80))
            .await?
            .map(|addr| addr.ip())
            .collect();

        let duration = start.elapsed();
        debug!("DNS resolution for {} took {:?}", hostname, duration);

        Ok(addrs)
    }

    /// Update cache hit ratio
    fn update_hit_ratio(&self, stats: &mut DnsCacheStats) {
        let total = stats.cache_hits + stats.cache_misses;
        stats.hit_ratio = if total > 0 {
            stats.cache_hits as f64 / total as f64
        } else {
            0.0
        };
    }

    /// Maybe perform cache cleanup
    async fn maybe_cleanup(&self) {
        let last_cleanup = *self.last_cleanup.read().await;
        if last_cleanup.elapsed() >= self.cleanup_interval {
            self.cleanup_expired().await;
            *self.last_cleanup.write().await = Instant::now();
        }
    }

    /// Clean up expired cache entries
    pub async fn cleanup_expired(&self) {
        let mut expired_count = 0;
        let mut to_remove = Vec::new();

        // Collect expired entries
        for entry in self.cache.iter() {
            if entry.value().is_expired() {
                to_remove.push(entry.key().clone());
            }
        }

        // Remove expired entries
        for key in to_remove {
            self.cache.remove(&key);
            expired_count += 1;
        }

        if expired_count > 0 {
            // Update statistics
            let mut stats = self.stats.write().await;
            stats.expired_cleaned += expired_count;
            stats.total_entries = self.cache.len();
            drop(stats);

            info!("Cleaned up {} expired DNS cache entries", expired_count);
        }
    }

    /// Evict oldest entries when cache is full
    async fn evict_oldest_entries(&self) {
        let target_size = self.max_entries * 3 / 4; // Remove 25% of entries
        let mut entries_to_remove = Vec::new();

        // Collect entries with their last access time
        let mut entries: Vec<_> = self
            .cache
            .iter()
            .map(|entry| (entry.key().clone(), entry.value().last_accessed))
            .collect();

        // Sort by last accessed time (oldest first)
        entries.sort_by_key(|(_, last_accessed)| *last_accessed);

        // Mark oldest entries for removal
        let remove_count = self.cache.len().saturating_sub(target_size);
        for (key, _) in entries.into_iter().take(remove_count) {
            entries_to_remove.push(key);
        }

        // Remove entries
        for key in entries_to_remove {
            self.cache.remove(&key);
        }

        // Update statistics
        let mut stats = self.stats.write().await;
        stats.total_entries = self.cache.len();
        drop(stats);

        info!("Evicted {} oldest DNS cache entries", remove_count);
    }

    /// Get cache statistics
    pub async fn get_stats(&self) -> DnsCacheStats {
        let mut stats = self.stats.read().await.clone();
        stats.total_entries = self.cache.len();
        stats
    }

    /// Clear all cache entries
    pub async fn clear(&self) {
        self.cache.clear();

        let mut stats = self.stats.write().await;
        stats.total_entries = 0;
        drop(stats);

        info!("DNS cache cleared");
    }

    /// Get cache entry for hostname (for debugging)
    pub fn get_entry(&self, hostname: &str) -> Option<DnsCacheEntry> {
        self.cache.get(hostname).map(|entry| entry.clone())
    }

    /// Check if hostname is cached
    pub fn contains(&self, hostname: &str) -> bool {
        self.cache.contains_key(hostname)
    }

    /// Get number of cached entries
    pub fn len(&self) -> usize {
        self.cache.len()
    }

    /// Check if cache is empty
    pub fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }
}

impl std::fmt::Display for DnsCacheStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,
            "DNS Cache: {} entries | Hit ratio: {:.1}% ({}/{}) | Time saved: {}ms | Expired cleaned: {}",
            self.total_entries,
            self.hit_ratio * 100.0,
            self.cache_hits,
            self.cache_hits + self.cache_misses,
            self.time_saved_ms,
            self.expired_cleaned
        )
    }
}
