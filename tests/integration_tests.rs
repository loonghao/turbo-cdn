// Licensed under the MIT License
// Copyright (c) 2025 Hal <hal.long@outlook.com>

//! Integration tests for turbo-cdn
//!
//! These tests verify the integration between different components.

use std::time::Duration;
use turbo_cdn::*;

/// Test full workflow: create client, get optimal URL, check stats
#[tokio::test]
async fn test_full_workflow() {
    // Create client with builder
    let cdn = TurboCdn::builder()
        .with_auto_detect_region(false)
        .with_region(Region::China)
        .with_max_concurrent_downloads(8)
        .with_timeout(30)
        .build()
        .await
        .expect("Failed to create TurboCdn");

    // Get optimal URL for a GitHub release
    let url = "https://github.com/BurntSushi/ripgrep/releases/download/14.1.1/ripgrep-14.1.1-x86_64-pc-windows-msvc.zip";
    let optimal = cdn
        .get_optimal_url(url)
        .await
        .expect("Failed to get optimal URL");

    // Should return a valid URL
    assert!(!optimal.is_empty());

    // Check stats
    let stats = cdn.get_stats().await;
    assert!(stats.uptime > Duration::ZERO);

    // Check performance summary
    let summary = cdn.get_performance_summary();
    assert_eq!(summary.total_servers, 0); // No downloads yet
}

/// Test URL mapping for different regions
#[tokio::test]
async fn test_url_mapping_regions() {
    let url = "https://github.com/user/repo/releases/download/v1.0.0/file.zip";

    // Test China region
    let cdn_china = TurboCdn::builder()
        .with_auto_detect_region(false)
        .with_region(Region::China)
        .build()
        .await
        .expect("Failed to create TurboCdn for China");

    let urls_china = cdn_china
        .get_all_cdn_urls(url)
        .await
        .expect("Failed to get CDN URLs");

    // China should have CDN mirrors
    assert!(urls_china.len() > 1);

    // Test Global region
    let cdn_global = TurboCdn::builder()
        .with_auto_detect_region(false)
        .with_region(Region::Global)
        .build()
        .await
        .expect("Failed to create TurboCdn for Global");

    let urls_global = cdn_global
        .get_all_cdn_urls(url)
        .await
        .expect("Failed to get CDN URLs");

    // Global should also have CDN mirrors
    assert!(!urls_global.is_empty());
}

/// Test sync API
#[test]
fn test_sync_api() {
    use turbo_cdn::sync_api::SyncTurboCdn;

    let cdn = SyncTurboCdn::new().expect("Failed to create SyncTurboCdn");

    let url = "https://github.com/user/repo/releases/download/v1.0.0/file.zip";
    let optimal = cdn.get_optimal_url(url).expect("Failed to get optimal URL");

    assert!(!optimal.is_empty());
}

/// Test async API module
#[tokio::test]
async fn test_async_api_module() {
    use turbo_cdn::async_api::AsyncTurboCdn;

    let cdn = AsyncTurboCdn::new()
        .await
        .expect("Failed to create AsyncTurboCdn");

    let url = "https://github.com/user/repo/releases/download/v1.0.0/file.zip";
    let optimal = cdn
        .get_optimal_url_async(url)
        .await
        .expect("Failed to get optimal URL");

    assert!(!optimal.is_empty());
}

/// Test quick async API
#[tokio::test]
async fn test_quick_async_api() {
    use turbo_cdn::async_api::quick;

    let url = "https://github.com/user/repo/releases/download/v1.0.0/file.zip";
    let optimal = quick::optimize_url(url)
        .await
        .expect("Failed to optimize URL");

    assert!(!optimal.is_empty());
}

/// Test quick sync API
#[test]
fn test_quick_sync_api() {
    use turbo_cdn::sync_api::quick;

    let url = "https://github.com/user/repo/releases/download/v1.0.0/file.zip";
    let optimal = quick::optimize_url(url).expect("Failed to optimize URL");

    assert!(!optimal.is_empty());
}

/// Test concurrent access to TurboCdn
#[tokio::test]
async fn test_concurrent_access() {
    use std::sync::Arc;

    let cdn = Arc::new(
        TurboCdn::builder()
            .with_auto_detect_region(false)
            .with_region(Region::Global)
            .build()
            .await
            .expect("Failed to create TurboCdn"),
    );

    let mut handles = vec![];

    for i in 0..10 {
        let cdn_clone = Arc::clone(&cdn);
        let handle = tokio::spawn(async move {
            let url = format!(
                "https://github.com/user/repo{}/releases/download/v1.0.0/file.zip",
                i
            );
            cdn_clone.get_optimal_url(&url).await
        });
        handles.push(handle);
    }

    for handle in handles {
        let result = handle.await.expect("Task panicked");
        assert!(result.is_ok());
    }
}

/// Test server tracker integration
#[test]
fn test_server_tracker_integration() {
    use turbo_cdn::server_tracker::ServerTracker;

    let mut tracker = ServerTracker::new();

    // Simulate multiple downloads
    for i in 0..5 {
        let url = format!("http://server{}.example.com", i);
        let speed = (i + 1) as f64 * 1024.0 * 1024.0; // 1-5 MB/s
        tracker.record_success(&url, speed, Duration::from_millis(100));
    }

    // Select best servers
    let urls: Vec<String> = (0..5)
        .map(|i| format!("http://server{}.example.com", i))
        .collect();

    let selected = tracker.select_best_servers(&urls, 3);

    assert_eq!(selected.len(), 3);
    // Fastest server (server4) should be first
    assert!(selected[0].contains("server4"));
}

/// Test URL mapper with different URL patterns
#[test]
fn test_url_mapper_patterns() {
    let config = TurboCdnConfig::load().unwrap_or_default();
    let mapper = UrlMapper::new(&config, Region::China).expect("Failed to create UrlMapper");

    // Test GitHub releases
    let github_url = "https://github.com/user/repo/releases/download/v1.0.0/file.zip";
    let mapped = mapper.map_url(github_url).expect("Failed to map URL");
    assert!(mapped.len() > 1);

    // Test PyPI
    let pypi_url = "https://pypi.org/simple/package/";
    let mapped = mapper.map_url(pypi_url).expect("Failed to map URL");
    // PyPI mapping depends on config rules
    assert!(!mapped.is_empty());

    // Test npm
    let npm_url = "https://registry.npmjs.org/package";
    let mapped = mapper.map_url(npm_url).expect("Failed to map URL");
    assert!(!mapped.is_empty());
}

/// Test download options with all configurations
#[test]
fn test_download_options_complete() {
    let options = DownloadOptions::new()
        .with_max_concurrent_chunks(16)
        .with_chunk_size(4 * 1024 * 1024)
        .with_resume(true)
        .with_header("Accept", "application/octet-stream")
        .with_header("Cache-Control", "no-cache")
        .with_timeout(Duration::from_secs(120))
        .with_integrity_verification(true)
        .with_expected_size(100 * 1024 * 1024);

    assert_eq!(options.max_concurrent_chunks, Some(16));
    assert_eq!(options.chunk_size, Some(4 * 1024 * 1024));
    assert!(options.enable_resume);
    assert!(options.verify_integrity);
    assert_eq!(options.expected_size, Some(100 * 1024 * 1024));
    assert_eq!(options.timeout_override, Some(Duration::from_secs(120)));

    let headers = options.custom_headers.as_ref().unwrap();
    assert_eq!(headers.len(), 2);
}

/// Test error types
#[test]
fn test_error_types() {
    use turbo_cdn::TurboCdnError;

    // Test different error constructors
    let network_err = TurboCdnError::network("Connection refused");
    assert!(network_err.to_string().contains("Network"));

    let config_err = TurboCdnError::config("Invalid setting");
    assert!(config_err.to_string().contains("Configuration"));

    let io_err = TurboCdnError::io("File not found");
    assert!(io_err.to_string().contains("IO"));

    let internal_err = TurboCdnError::internal("Internal error");
    assert!(internal_err.to_string().contains("Internal"));
}

/// Test configuration loading
#[test]
fn test_config_loading() {
    // Test default config
    let config = TurboCdnConfig::default();
    assert!(config.performance.max_concurrent_downloads > 0);
    assert!(config.performance.chunk_size > 0);

    // Test loading from embedded config
    let loaded = TurboCdnConfig::load();
    assert!(loaded.is_ok());

    let loaded = loaded.unwrap();
    assert!(!loaded.url_mapping_rules.is_empty());
}

/// Test cache statistics
#[test]
fn test_cache_statistics() {
    let config = TurboCdnConfig::default();
    let mapper = UrlMapper::new(&config, Region::Global).expect("Failed to create UrlMapper");

    // Get initial cache stats
    let (total, expired) = mapper.cache_stats();
    assert_eq!(total, 0);
    assert_eq!(expired, 0);

    // Map some URLs to populate cache
    let _ = mapper.map_url("https://github.com/user/repo/releases/download/v1.0.0/file.zip");

    let (total, _) = mapper.cache_stats();
    assert!(total > 0);
}

/// Test rule count
#[test]
fn test_rule_count() {
    let config = TurboCdnConfig::load().unwrap_or_default();
    let mapper = UrlMapper::new(&config, Region::Global).expect("Failed to create UrlMapper");

    // Should have rules loaded
    let count = mapper.rule_count();
    assert!(count > 0);
}
