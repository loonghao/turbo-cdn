// Licensed under the MIT License
// Copyright (c) 2025 Hal <hal.long@outlook.com>

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};
use tokio::fs;
use tracing::{debug, info, warn};

use crate::config::CacheConfig;
use crate::error::{Result, TurboCdnError};

/// Cache manager for downloaded files and metadata
#[derive(Debug)]
pub struct CacheManager {
    config: CacheConfig,
    metadata_store: MetadataStore,
    stats: CacheStats,
}

/// Metadata store for cache entries
#[derive(Debug)]
#[allow(dead_code)]
pub struct MetadataStore {
    cache_dir: PathBuf,
    metadata: HashMap<String, CacheEntry>,
}

/// Cache entry metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    pub key: String,
    pub file_path: PathBuf,
    pub original_url: String,
    pub file_size: u64,
    pub checksum: Option<String>,
    pub content_type: Option<String>,
    pub created_at: SystemTime,
    pub last_accessed: SystemTime,
    pub access_count: u64,
    pub compressed: bool,
    pub ttl: Duration,
}

/// Cache statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    pub total_entries: usize,
    pub total_size: u64,
    pub hit_count: u64,
    pub miss_count: u64,
    pub eviction_count: u64,
    pub last_cleanup: SystemTime,
}

impl CacheStats {
    fn new() -> Self {
        Self {
            total_entries: 0,
            total_size: 0,
            hit_count: 0,
            miss_count: 0,
            eviction_count: 0,
            last_cleanup: SystemTime::now(),
        }
    }
}

/// Cache lookup result
#[derive(Debug)]
pub enum CacheLookup {
    Hit(CacheEntry),
    Miss,
    Expired(CacheEntry),
}

impl CacheManager {
    /// Create a new cache manager
    pub async fn new(config: CacheConfig) -> Result<Self> {
        if !config.enabled {
            return Ok(Self {
                config,
                metadata_store: MetadataStore::disabled(),
                stats: CacheStats::new(),
            });
        }

        // Ensure cache directory exists
        fs::create_dir_all(&config.cache_dir).await.map_err(|e| {
            TurboCdnError::cache(format!("Failed to create cache directory: {}", e))
        })?;

        let metadata_store = MetadataStore::new(&config.cache_dir).await?;
        let cache_dir = config.cache_dir.clone();

        let mut cache_manager = Self {
            config,
            metadata_store,
            stats: CacheStats::new(),
        };

        // Perform initial cleanup
        cache_manager.cleanup().await?;

        info!("Cache manager initialized at: {}", cache_dir.display());
        Ok(cache_manager)
    }

    /// Generate cache key for a download
    pub fn generate_key(&self, repository: &str, version: &str, file_name: &str) -> String {
        use sha2::{Digest, Sha256};

        let input = format!("{}:{}:{}", repository, version, file_name);
        let mut hasher = Sha256::new();
        hasher.update(input.as_bytes());
        let hash = hasher.finalize();
        format!("{:x}", hash)
    }

    /// Look up a file in the cache
    pub async fn lookup(&mut self, key: &str) -> Result<CacheLookup> {
        if !self.config.enabled {
            return Ok(CacheLookup::Miss);
        }

        if let Some(entry) = self.metadata_store.get_entry(key) {
            // Check if file still exists
            if !entry.file_path.exists() {
                warn!(
                    "Cache entry exists but file is missing: {}",
                    entry.file_path.display()
                );
                self.metadata_store.remove_entry(key);
                return Ok(CacheLookup::Miss);
            }

            // Check if entry has expired
            if self.is_expired(&entry) {
                debug!("Cache entry expired: {}", key);
                return Ok(CacheLookup::Expired(entry.clone()));
            }

            // Update access time
            self.metadata_store.update_access(key);
            self.stats.hit_count += 1;
            debug!("Cache hit: {}", key);
            Ok(CacheLookup::Hit(entry.clone()))
        } else {
            self.stats.miss_count += 1;
            debug!("Cache miss: {}", key);
            Ok(CacheLookup::Miss)
        }
    }

    /// Store a file in the cache
    pub async fn store(
        &mut self,
        key: &str,
        data: &[u8],
        original_url: &str,
        content_type: Option<String>,
    ) -> Result<CacheEntry> {
        if !self.config.enabled {
            return Err(TurboCdnError::cache("Cache is disabled"));
        }

        // Check if we have space
        self.ensure_space_available(data.len() as u64).await?;

        // Generate file path
        let file_name = format!("{}.cache", key);
        let file_path = self.config.cache_dir.join(&file_name);

        // Compress data if enabled
        let (final_data, compressed) = if self.config.compression && data.len() > 1024 {
            match self.compress_data(data) {
                Ok(compressed_data) if compressed_data.len() < data.len() => {
                    debug!(
                        "Compressed cache entry from {} to {} bytes",
                        data.len(),
                        compressed_data.len()
                    );
                    (compressed_data, true)
                }
                _ => (data.to_vec(), false),
            }
        } else {
            (data.to_vec(), false)
        };

        // Write file
        fs::write(&file_path, &final_data)
            .await
            .map_err(|e| TurboCdnError::cache(format!("Failed to write cache file: {}", e)))?;

        // Calculate checksum
        let checksum = self.calculate_checksum(data);

        // Create cache entry
        let entry = CacheEntry {
            key: key.to_string(),
            file_path,
            original_url: original_url.to_string(),
            file_size: data.len() as u64,
            checksum: Some(checksum),
            content_type,
            created_at: SystemTime::now(),
            last_accessed: SystemTime::now(),
            access_count: 1,
            compressed,
            ttl: Duration::from_secs(self.config.ttl),
        };

        // Store metadata
        self.metadata_store.add_entry(entry.clone());

        info!("Cached file: {} ({} bytes)", key, data.len());
        Ok(entry)
    }

    /// Read a file from the cache
    pub async fn read(&self, entry: &CacheEntry) -> Result<Vec<u8>> {
        let data = fs::read(&entry.file_path)
            .await
            .map_err(|e| TurboCdnError::cache(format!("Failed to read cache file: {}", e)))?;

        if entry.compressed {
            self.decompress_data(&data)
        } else {
            Ok(data)
        }
    }

    /// Remove a file from the cache
    pub async fn remove(&mut self, key: &str) -> Result<()> {
        if let Some(entry) = self.metadata_store.get_entry(key) {
            // Remove file
            if entry.file_path.exists() {
                fs::remove_file(&entry.file_path).await.map_err(|e| {
                    TurboCdnError::cache(format!("Failed to remove cache file: {}", e))
                })?;
            }

            // Remove metadata
            self.metadata_store.remove_entry(key);
            debug!("Removed cache entry: {}", key);
        }

        Ok(())
    }

    /// Perform cache cleanup
    pub async fn cleanup(&mut self) -> Result<()> {
        if !self.config.enabled {
            return Ok(());
        }

        let mut removed_count = 0;
        let mut freed_space = 0u64;

        // Remove expired entries
        let expired_keys: Vec<String> = self
            .metadata_store
            .metadata
            .values()
            .filter(|entry| self.is_expired(entry))
            .map(|entry| entry.key.clone())
            .collect();

        for key in expired_keys {
            if let Some(entry) = self.metadata_store.get_entry(&key) {
                freed_space += entry.file_size;
            }
            self.remove(&key).await?;
            removed_count += 1;
        }

        // Check total cache size and remove LRU entries if needed
        let total_size = self.get_total_size();
        if total_size > self.config.max_size {
            let excess = total_size - self.config.max_size;
            let lru_removed = self.remove_lru_entries(excess).await?;
            removed_count += lru_removed.0;
            freed_space += lru_removed.1;
        }

        if removed_count > 0 {
            info!(
                "Cache cleanup: removed {} entries, freed {} bytes",
                removed_count, freed_space
            );
        }

        self.stats.last_cleanup = SystemTime::now();
        Ok(())
    }

    /// Get cache statistics
    pub fn get_stats(&self) -> CacheStats {
        let mut stats = self.stats.clone();
        stats.total_entries = self.metadata_store.metadata.len();
        stats.total_size = self.get_total_size();
        stats
    }

    /// Clear all cache entries
    pub async fn clear(&mut self) -> Result<()> {
        if !self.config.enabled {
            return Ok(());
        }

        let keys: Vec<String> = self.metadata_store.metadata.keys().cloned().collect();

        for key in keys {
            self.remove(&key).await?;
        }

        info!("Cache cleared");
        Ok(())
    }

    // Private helper methods

    fn is_expired(&self, entry: &CacheEntry) -> bool {
        if let Ok(elapsed) = entry.created_at.elapsed() {
            elapsed > entry.ttl
        } else {
            true // If we can't determine age, consider it expired
        }
    }

    fn get_total_size(&self) -> u64 {
        self.metadata_store
            .metadata
            .values()
            .map(|entry| entry.file_size)
            .sum()
    }

    async fn ensure_space_available(&mut self, required_space: u64) -> Result<()> {
        let current_size = self.get_total_size();
        let available_space = self.config.max_size.saturating_sub(current_size);

        if required_space > available_space {
            let space_to_free = required_space - available_space;
            self.remove_lru_entries(space_to_free).await?;
        }

        Ok(())
    }

    async fn remove_lru_entries(&mut self, space_to_free: u64) -> Result<(usize, u64)> {
        let mut entries: Vec<_> = self.metadata_store.metadata.values().cloned().collect();

        // Sort by last accessed time (oldest first)
        entries.sort_by_key(|entry| entry.last_accessed);

        let mut freed_space = 0u64;
        let mut removed_count = 0;

        for entry in entries {
            if freed_space >= space_to_free {
                break;
            }

            freed_space += entry.file_size;
            self.remove(&entry.key).await?;
            removed_count += 1;
            self.stats.eviction_count += 1;
        }

        Ok((removed_count, freed_space))
    }

    fn compress_data(&self, data: &[u8]) -> Result<Vec<u8>> {
        use flate2::write::GzEncoder;
        use flate2::Compression;
        use std::io::Write;

        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder
            .write_all(data)
            .map_err(|e| TurboCdnError::cache(format!("Compression failed: {}", e)))?;

        encoder
            .finish()
            .map_err(|e| TurboCdnError::cache(format!("Compression failed: {}", e)))
    }

    fn decompress_data(&self, data: &[u8]) -> Result<Vec<u8>> {
        use flate2::read::GzDecoder;
        use std::io::Read;

        let mut decoder = GzDecoder::new(data);
        let mut decompressed = Vec::new();
        decoder
            .read_to_end(&mut decompressed)
            .map_err(|e| TurboCdnError::cache(format!("Decompression failed: {}", e)))?;

        Ok(decompressed)
    }

    fn calculate_checksum(&self, data: &[u8]) -> String {
        use sha2::{Digest, Sha256};

        let mut hasher = Sha256::new();
        hasher.update(data);
        let hash = hasher.finalize();
        format!("{:x}", hash)
    }
}

impl MetadataStore {
    async fn new(cache_dir: &Path) -> Result<Self> {
        let metadata_file = cache_dir.join("metadata.json");
        let metadata = if metadata_file.exists() {
            let content = fs::read_to_string(&metadata_file).await.map_err(|e| {
                TurboCdnError::cache(format!("Failed to read metadata file: {}", e))
            })?;

            serde_json::from_str(&content).map_err(|e| {
                TurboCdnError::cache(format!("Failed to parse metadata file: {}", e))
            })?
        } else {
            HashMap::new()
        };

        Ok(Self {
            cache_dir: cache_dir.to_path_buf(),
            metadata,
        })
    }

    fn disabled() -> Self {
        Self {
            cache_dir: PathBuf::new(),
            metadata: HashMap::new(),
        }
    }

    fn get_entry(&self, key: &str) -> Option<CacheEntry> {
        self.metadata.get(key).cloned()
    }

    fn add_entry(&mut self, entry: CacheEntry) {
        self.metadata.insert(entry.key.clone(), entry);
        if let Err(e) = self.persist_metadata() {
            warn!("Failed to persist metadata after adding entry: {}", e);
        }
    }

    fn remove_entry(&mut self, key: &str) {
        self.metadata.remove(key);
        if let Err(e) = self.persist_metadata() {
            warn!("Failed to persist metadata after removing entry: {}", e);
        }
    }

    fn update_access(&mut self, key: &str) {
        if let Some(entry) = self.metadata.get_mut(key) {
            entry.last_accessed = SystemTime::now();
            entry.access_count += 1;
            if let Err(e) = self.persist_metadata() {
                warn!("Failed to persist metadata after updating access: {}", e);
            }
        }
    }

    fn persist_metadata(&self) -> Result<()> {
        let metadata_file = self.cache_dir.join("metadata.json");
        let content = serde_json::to_string_pretty(&self.metadata)
            .map_err(|e| TurboCdnError::cache(format!("Failed to serialize metadata: {}", e)))?;

        std::fs::write(&metadata_file, content)
            .map_err(|e| TurboCdnError::cache(format!("Failed to write metadata file: {}", e)))?;

        Ok(())
    }
}
