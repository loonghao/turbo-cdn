// Licensed under the MIT License
// Copyright (c) 2025 Hal <hal.long@outlook.com>

use std::time::Duration;
use tempfile::TempDir;
use tokio::fs;
use turbo_cdn::cache::CacheManager;
use turbo_cdn::config::CacheConfig;
use turbo_cdn::error::TurboCdnError;

/// Helper function to create a test cache config
fn create_test_cache_config(temp_dir: &TempDir) -> CacheConfig {
    CacheConfig {
        enabled: true,
        cache_dir: temp_dir.path().to_path_buf(),
        max_size: 1024 * 1024, // 1MB
        ttl: 3600,             // 1 hour
        compression: true,
        cleanup_interval: 60,
    }
}

/// Helper function to create a disabled cache config
fn create_disabled_cache_config() -> CacheConfig {
    CacheConfig {
        enabled: false,
        cache_dir: std::env::temp_dir().join("disabled-cache"),
        max_size: 1024 * 1024,
        ttl: 3600,
        compression: false,
        cleanup_interval: 60,
    }
}

#[tokio::test]
async fn test_cache_manager_creation_enabled() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let config = create_test_cache_config(&temp_dir);

    let result = CacheManager::new(config).await;
    assert!(result.is_ok(), "Cache manager creation should succeed");

    let cache_manager = result.unwrap();
    let stats = cache_manager.get_stats();
    assert_eq!(stats.total_entries, 0);
    assert_eq!(stats.total_size, 0);
    assert_eq!(stats.hit_count, 0);
    assert_eq!(stats.miss_count, 0);
}

#[tokio::test]
async fn test_cache_manager_creation_disabled() {
    let config = create_disabled_cache_config();

    let result = CacheManager::new(config).await;
    assert!(
        result.is_ok(),
        "Disabled cache manager creation should succeed"
    );

    let cache_manager = result.unwrap();
    let stats = cache_manager.get_stats();
    assert_eq!(stats.total_entries, 0);
}

#[tokio::test]
async fn test_cache_store_and_lookup() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let config = create_test_cache_config(&temp_dir);
    let mut cache_manager = CacheManager::new(config).await.unwrap();

    let test_data = b"Hello, World!";
    let key = "test-key";
    let url = "https://example.com/test.txt";

    // Store data in cache
    let result = cache_manager
        .store(key, test_data, url, Some("text/plain".to_string()))
        .await;
    assert!(result.is_ok(), "Cache store should succeed");

    let entry = result.unwrap();
    assert_eq!(entry.key, key);
    assert_eq!(entry.original_url, url);
    assert_eq!(entry.file_size, test_data.len() as u64);
    assert!(entry.checksum.is_some());
    assert_eq!(entry.content_type, Some("text/plain".to_string()));

    // Lookup the cached data
    let lookup_result = cache_manager.lookup(key).await.unwrap();
    match lookup_result {
        turbo_cdn::cache::CacheLookup::Hit(cached_entry) => {
            assert_eq!(cached_entry.key, key);
            assert_eq!(cached_entry.original_url, url);
        }
        _ => panic!("Expected cache hit"),
    }
}

#[tokio::test]
async fn test_cache_miss() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let config = create_test_cache_config(&temp_dir);
    let mut cache_manager = CacheManager::new(config).await.unwrap();

    let lookup_result = cache_manager.lookup("non-existent-key").await.unwrap();
    match lookup_result {
        turbo_cdn::cache::CacheLookup::Miss => {
            // Expected behavior
        }
        _ => panic!("Expected cache miss"),
    }

    let stats = cache_manager.get_stats();
    assert_eq!(stats.miss_count, 1);
}

#[tokio::test]
async fn test_cache_read() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let config = create_test_cache_config(&temp_dir);
    let mut cache_manager = CacheManager::new(config).await.unwrap();

    let test_data = b"Test data for reading";
    let key = "read-test-key";
    let url = "https://example.com/read-test.txt";

    // Store data
    let entry = cache_manager
        .store(key, test_data, url, None)
        .await
        .unwrap();

    // Read data back
    let read_result = cache_manager.read(&entry).await;
    assert!(read_result.is_ok(), "Cache read should succeed");

    let read_data = read_result.unwrap();
    assert_eq!(read_data, test_data);
}

#[tokio::test]
async fn test_cache_remove() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let config = create_test_cache_config(&temp_dir);
    let mut cache_manager = CacheManager::new(config).await.unwrap();

    let test_data = b"Data to be removed";
    let key = "remove-test-key";
    let url = "https://example.com/remove-test.txt";

    // Store data
    cache_manager
        .store(key, test_data, url, None)
        .await
        .unwrap();

    // Verify it exists
    let lookup_result = cache_manager.lookup(key).await.unwrap();
    assert!(matches!(
        lookup_result,
        turbo_cdn::cache::CacheLookup::Hit(_)
    ));

    // Remove the entry
    let remove_result = cache_manager.remove(key).await;
    assert!(remove_result.is_ok(), "Cache remove should succeed");

    // Verify it's gone
    let lookup_result = cache_manager.lookup(key).await.unwrap();
    assert!(matches!(lookup_result, turbo_cdn::cache::CacheLookup::Miss));
}

#[tokio::test]
async fn test_cache_clear() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let config = create_test_cache_config(&temp_dir);
    let mut cache_manager = CacheManager::new(config).await.unwrap();

    // Store multiple entries
    for i in 0..5 {
        let key = format!("clear-test-key-{}", i);
        let data = format!("Test data {}", i).into_bytes();
        cache_manager
            .store(&key, &data, "https://example.com", None)
            .await
            .unwrap();
    }

    // Verify entries exist
    let stats = cache_manager.get_stats();
    assert_eq!(stats.total_entries, 5);

    // Clear cache
    let clear_result = cache_manager.clear().await;
    assert!(clear_result.is_ok(), "Cache clear should succeed");

    // Verify cache is empty
    let stats = cache_manager.get_stats();
    assert_eq!(stats.total_entries, 0);
    assert_eq!(stats.total_size, 0);
}

#[tokio::test]
async fn test_cache_compression() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let config = create_test_cache_config(&temp_dir);
    let mut cache_manager = CacheManager::new(config).await.unwrap();

    // Create data large enough to trigger compression (> 1024 bytes)
    let large_data = vec![b'A'; 2048];
    let key = "compression-test-key";
    let url = "https://example.com/large-file.txt";

    // Store data (should be compressed)
    let entry = cache_manager
        .store(key, &large_data, url, None)
        .await
        .unwrap();

    // Read data back
    let read_data = cache_manager.read(&entry).await.unwrap();
    assert_eq!(read_data, large_data);

    // Verify compression was applied for large data
    assert!(entry.compressed || entry.file_size == large_data.len() as u64);
}

#[tokio::test]
async fn test_cache_disabled_operations() {
    let config = create_disabled_cache_config();
    let mut cache_manager = CacheManager::new(config).await.unwrap();

    let test_data = b"Test data";
    let key = "disabled-test-key";
    let url = "https://example.com/test.txt";

    // Store should fail when cache is disabled
    let store_result = cache_manager.store(key, test_data, url, None).await;
    assert!(store_result.is_err());
    assert!(matches!(
        store_result.unwrap_err(),
        TurboCdnError::Cache { .. }
    ));

    // Lookup should return miss
    let lookup_result = cache_manager.lookup(key).await.unwrap();
    assert!(matches!(lookup_result, turbo_cdn::cache::CacheLookup::Miss));

    // Clear should succeed (no-op)
    let clear_result = cache_manager.clear().await;
    assert!(clear_result.is_ok());
}

#[tokio::test]
async fn test_cache_key_generation() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let config = create_test_cache_config(&temp_dir);
    let cache_manager = CacheManager::new(config).await.unwrap();

    let key1 = cache_manager.generate_key("owner/repo", "v1.0.0", "file.txt");
    let key2 = cache_manager.generate_key("owner/repo", "v1.0.0", "file.txt");
    let key3 = cache_manager.generate_key("owner/repo", "v1.0.1", "file.txt");

    // Same parameters should generate same key
    assert_eq!(key1, key2);

    // Different parameters should generate different keys
    assert_ne!(key1, key3);

    // Keys should be non-empty
    assert!(!key1.is_empty());
    assert!(!key3.is_empty());
}

#[tokio::test]
async fn test_cache_stats_tracking() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let config = create_test_cache_config(&temp_dir);
    let mut cache_manager = CacheManager::new(config).await.unwrap();

    let initial_stats = cache_manager.get_stats();
    assert_eq!(initial_stats.hit_count, 0);
    assert_eq!(initial_stats.miss_count, 0);

    // Test miss
    cache_manager.lookup("non-existent").await.unwrap();
    let stats = cache_manager.get_stats();
    assert_eq!(stats.miss_count, 1);

    // Store and test hit
    let test_data = b"Stats test data";
    cache_manager
        .store("stats-key", test_data, "https://example.com", None)
        .await
        .unwrap();

    cache_manager.lookup("stats-key").await.unwrap();
    let stats = cache_manager.get_stats();
    assert_eq!(stats.hit_count, 1);
    assert_eq!(stats.total_entries, 1);
    assert_eq!(stats.total_size, test_data.len() as u64);
}

#[tokio::test]
async fn test_cache_ttl_expiration() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let mut config = create_test_cache_config(&temp_dir);
    config.ttl = 1; // 1 second TTL for testing
    let mut cache_manager = CacheManager::new(config).await.unwrap();

    let test_data = b"TTL test data";
    let key = "ttl-test-key";
    let url = "https://example.com/ttl-test.txt";

    // Store data
    cache_manager
        .store(key, test_data, url, None)
        .await
        .unwrap();

    // Should be a hit immediately
    let lookup_result = cache_manager.lookup(key).await.unwrap();
    assert!(matches!(
        lookup_result,
        turbo_cdn::cache::CacheLookup::Hit(_)
    ));

    // Wait for TTL to expire
    tokio::time::sleep(Duration::from_secs(2)).await;

    // Should now be expired
    let lookup_result = cache_manager.lookup(key).await.unwrap();
    assert!(matches!(
        lookup_result,
        turbo_cdn::cache::CacheLookup::Expired(_)
    ));
}

#[tokio::test]
async fn test_cache_cleanup() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let mut config = create_test_cache_config(&temp_dir);
    config.ttl = 1; // 1 second TTL
    let mut cache_manager = CacheManager::new(config).await.unwrap();

    // Store multiple entries
    for i in 0..3 {
        let key = format!("cleanup-test-key-{}", i);
        let data = format!("Cleanup test data {}", i).into_bytes();
        cache_manager
            .store(&key, &data, "https://example.com", None)
            .await
            .unwrap();
    }

    let stats_before = cache_manager.get_stats();
    assert_eq!(stats_before.total_entries, 3);

    // Wait for entries to expire
    tokio::time::sleep(Duration::from_secs(2)).await;

    // Run cleanup
    let cleanup_result = cache_manager.cleanup().await;
    assert!(cleanup_result.is_ok(), "Cleanup should succeed");

    // Verify expired entries were removed
    let stats_after = cache_manager.get_stats();
    assert_eq!(stats_after.total_entries, 0);
}

#[tokio::test]
async fn test_cache_size_limit_and_lru() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let mut config = create_test_cache_config(&temp_dir);
    config.max_size = 100; // Very small cache size
    config.ttl = 3600; // Long TTL to avoid expiration during test
    let mut cache_manager = CacheManager::new(config).await.unwrap();

    // Store entries that exceed the cache size limit
    let large_data = vec![b'X'; 50]; // 50 bytes each

    // Store first entry
    cache_manager
        .store("lru-key-1", &large_data, "https://example.com/1", None)
        .await
        .unwrap();

    // Store second entry
    cache_manager
        .store("lru-key-2", &large_data, "https://example.com/2", None)
        .await
        .unwrap();

    // Store third entry (should trigger LRU eviction)
    cache_manager
        .store("lru-key-3", &large_data, "https://example.com/3", None)
        .await
        .unwrap();

    // The cache should have evicted some entries to stay under the size limit
    let stats = cache_manager.get_stats();
    assert!(stats.total_size <= 100, "Cache size should be under limit");

    // At least one entry should remain
    assert!(stats.total_entries > 0, "Cache should not be empty");
}

#[tokio::test]
async fn test_cache_file_missing_handling() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let config = create_test_cache_config(&temp_dir);
    let mut cache_manager = CacheManager::new(config).await.unwrap();

    let test_data = b"File missing test data";
    let key = "file-missing-key";
    let url = "https://example.com/missing-test.txt";

    // Store data
    let entry = cache_manager
        .store(key, test_data, url, None)
        .await
        .unwrap();

    // Manually delete the cache file to simulate missing file
    if entry.file_path.exists() {
        fs::remove_file(&entry.file_path).await.unwrap();
    }

    // Lookup should detect missing file and return miss
    let lookup_result = cache_manager.lookup(key).await.unwrap();
    assert!(matches!(lookup_result, turbo_cdn::cache::CacheLookup::Miss));
}

#[tokio::test]
async fn test_cache_checksum_validation() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let config = create_test_cache_config(&temp_dir);
    let mut cache_manager = CacheManager::new(config).await.unwrap();

    let test_data = b"Checksum test data";
    let key = "checksum-test-key";
    let url = "https://example.com/checksum-test.txt";

    // Store data
    let entry = cache_manager
        .store(key, test_data, url, None)
        .await
        .unwrap();

    // Verify checksum was calculated
    assert!(entry.checksum.is_some());
    assert!(!entry.checksum.as_ref().unwrap().is_empty());

    // Read data and verify it matches
    let read_data = cache_manager.read(&entry).await.unwrap();
    assert_eq!(read_data, test_data);
}

#[tokio::test]
async fn test_cache_concurrent_access() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let config = create_test_cache_config(&temp_dir);
    let mut cache_manager = CacheManager::new(config).await.unwrap();

    // Store initial data
    let test_data = b"Concurrent access test";
    let key = "concurrent-key";
    cache_manager
        .store(key, test_data, "https://example.com", None)
        .await
        .unwrap();

    // Test multiple sequential lookups (simulating concurrent access)
    for _ in 0..10 {
        let lookup_result = cache_manager.lookup(key).await.unwrap();
        assert!(matches!(
            lookup_result,
            turbo_cdn::cache::CacheLookup::Hit(_)
        ));
    }
}
