// Licensed under the MIT License
// Copyright (c) 2025 Hal <hal.long@outlook.com>

use std::path::PathBuf;
use std::time::Duration;
use tempfile::TempDir;
use turbo_cdn::cache::CacheManager;
use turbo_cdn::compliance::ComplianceChecker;
use turbo_cdn::config::{CacheConfig, ComplianceConfig, TurboCdnConfig};
use turbo_cdn::downloader::{DownloadOptions, Downloader};
use turbo_cdn::router::SmartRouter;
use turbo_cdn::sources::SourceManager;

/// Helper function to create a test config
fn create_test_config() -> TurboCdnConfig {
    TurboCdnConfig::default()
}

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

/// Helper function to create test download options
#[allow(dead_code)]
fn create_test_download_options(output_dir: Option<PathBuf>) -> DownloadOptions {
    DownloadOptions {
        max_concurrent_chunks: 4,
        chunk_size: 1024 * 512, // 512KB
        max_retries: 2,
        retry_delay: Duration::from_millis(100),
        timeout: Duration::from_secs(10),
        use_cache: true,
        verify_checksum: false, // Disable for testing
        output_dir,
        headers: reqwest::header::HeaderMap::new(),
        progress_callback: None,
    }
}

#[tokio::test]
async fn test_downloader_creation() {
    let config = create_test_config();
    let source_manager = SourceManager::new();
    let router = SmartRouter::new(config.clone(), source_manager);

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let cache_config = create_test_cache_config(&temp_dir);
    let cache_manager = CacheManager::new(cache_config).await.unwrap();

    let compliance_config = ComplianceConfig::default();
    let compliance_checker = ComplianceChecker::new(compliance_config).unwrap();

    let result = Downloader::new(config, router, cache_manager, compliance_checker).await;
    assert!(result.is_ok(), "Downloader creation should succeed");
}

#[test]
fn test_download_options_default() {
    let options = DownloadOptions::default();

    assert_eq!(options.max_concurrent_chunks, 8);
    assert_eq!(options.chunk_size, 1024 * 1024); // 1MB
    assert_eq!(options.max_retries, 3);
    assert_eq!(options.retry_delay, Duration::from_millis(1000));
    assert_eq!(options.timeout, Duration::from_secs(30));
    assert!(options.use_cache);
    assert!(options.verify_checksum);
    assert!(options.output_dir.is_none());
    assert!(options.headers.is_empty());
    assert!(options.progress_callback.is_none());
}

#[test]
fn test_download_options_builder() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    let options = DownloadOptions::builder()
        .max_concurrent_chunks(16)
        .chunk_size(2 * 1024 * 1024) // 2MB
        .max_retries(5)
        .retry_delay(Duration::from_millis(500))
        .timeout(Duration::from_secs(60))
        .use_cache(false)
        .verify_checksum(false)
        .output_dir(temp_dir.path().to_path_buf())
        .build();

    assert_eq!(options.max_concurrent_chunks, 16);
    assert_eq!(options.chunk_size, 2 * 1024 * 1024);
    assert_eq!(options.max_retries, 5);
    assert_eq!(options.retry_delay, Duration::from_millis(500));
    assert_eq!(options.timeout, Duration::from_secs(60));
    assert!(!options.use_cache);
    assert!(!options.verify_checksum);
    assert_eq!(options.output_dir, Some(temp_dir.path().to_path_buf()));
}

#[test]
fn test_download_options_builder_basic_configuration() {
    // Test basic builder functionality
    let options = DownloadOptions::builder()
        .max_concurrent_chunks(4)
        .chunk_size(512 * 1024)
        .build();

    assert_eq!(options.max_concurrent_chunks, 4);
    assert_eq!(options.chunk_size, 512 * 1024);
    assert!(options.headers.is_empty());
    assert!(options.progress_callback.is_none());
}

#[test]
fn test_download_options_builder_fluent_interface() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    // Test that builder methods can be chained
    let options = DownloadOptions::builder()
        .max_concurrent_chunks(4)
        .chunk_size(512 * 1024)
        .max_retries(2)
        .retry_delay(Duration::from_millis(200))
        .timeout(Duration::from_secs(15))
        .use_cache(true)
        .verify_checksum(true)
        .output_dir(temp_dir.path().to_path_buf())
        .build();

    assert_eq!(options.max_concurrent_chunks, 4);
    assert_eq!(options.chunk_size, 512 * 1024);
    assert_eq!(options.max_retries, 2);
    assert_eq!(options.retry_delay, Duration::from_millis(200));
    assert_eq!(options.timeout, Duration::from_secs(15));
    assert!(options.use_cache);
    assert!(options.verify_checksum);
    assert_eq!(options.output_dir, Some(temp_dir.path().to_path_buf()));
}

#[tokio::test]
async fn test_downloader_with_invalid_user_agent() {
    let mut config = create_test_config();
    config.general.user_agent = "invalid\r\nuser-agent".to_string(); // Invalid header value

    let source_manager = SourceManager::new();
    let router = SmartRouter::new(config.clone(), source_manager);

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let cache_config = create_test_cache_config(&temp_dir);
    let cache_manager = CacheManager::new(cache_config).await.unwrap();

    let compliance_config = ComplianceConfig::default();
    let compliance_checker = ComplianceChecker::new(compliance_config).unwrap();

    let result = Downloader::new(config, router, cache_manager, compliance_checker).await;
    assert!(
        result.is_err(),
        "Downloader creation should fail with invalid user agent"
    );

    if let Err(error) = result {
        assert_eq!(error.category(), "config");
        assert!(error.to_string().contains("Invalid user agent"));
    }
}

#[tokio::test]
async fn test_downloader_semaphore_limit() {
    let mut config = create_test_config();
    config.general.max_concurrent_downloads = 2; // Limit to 2 concurrent downloads

    let source_manager = SourceManager::new();
    let router = SmartRouter::new(config.clone(), source_manager);

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let cache_config = create_test_cache_config(&temp_dir);
    let cache_manager = CacheManager::new(cache_config).await.unwrap();

    let compliance_config = ComplianceConfig::default();
    let compliance_checker = ComplianceChecker::new(compliance_config).unwrap();

    let downloader = Downloader::new(config, router, cache_manager, compliance_checker)
        .await
        .unwrap();

    // The downloader should be created successfully with the semaphore limit
    // We can't easily test the actual semaphore behavior without real downloads
    // but we can verify the downloader was created with the correct configuration
    drop(downloader); // Just ensure it was created successfully
}

#[test]
fn test_download_options_validation() {
    // Test that download options can be created with various valid configurations
    let valid_configs = vec![
        (1, 1024),              // Minimum values
        (32, 10 * 1024 * 1024), // Large values
        (8, 1024 * 1024),       // Default-like values
    ];

    for (chunks, chunk_size) in valid_configs {
        let options = DownloadOptions::builder()
            .max_concurrent_chunks(chunks)
            .chunk_size(chunk_size)
            .build();

        assert_eq!(options.max_concurrent_chunks, chunks);
        assert_eq!(options.chunk_size, chunk_size);
    }
}

#[test]
fn test_download_options_timeout_configurations() {
    let timeout_configs = vec![
        Duration::from_secs(1),   // Very short
        Duration::from_secs(30),  // Default
        Duration::from_secs(300), // Long timeout
    ];

    for timeout in timeout_configs {
        let options = DownloadOptions::builder().timeout(timeout).build();

        assert_eq!(options.timeout, timeout);
    }
}

#[test]
fn test_download_options_retry_configurations() {
    let retry_configs = vec![
        (0, Duration::from_millis(0)),     // No retries
        (3, Duration::from_millis(1000)),  // Default
        (10, Duration::from_millis(5000)), // Many retries with long delay
    ];

    for (retries, delay) in retry_configs {
        let options = DownloadOptions::builder()
            .max_retries(retries)
            .retry_delay(delay)
            .build();

        assert_eq!(options.max_retries, retries);
        assert_eq!(options.retry_delay, delay);
    }
}

#[test]
fn test_download_options_cache_configurations() {
    // Test cache enabled
    let options_cache_enabled = DownloadOptions::builder().use_cache(true).build();
    assert!(options_cache_enabled.use_cache);

    // Test cache disabled
    let options_cache_disabled = DownloadOptions::builder().use_cache(false).build();
    assert!(!options_cache_disabled.use_cache);
}

#[test]
fn test_download_options_checksum_configurations() {
    // Test checksum verification enabled
    let options_checksum_enabled = DownloadOptions::builder().verify_checksum(true).build();
    assert!(options_checksum_enabled.verify_checksum);

    // Test checksum verification disabled
    let options_checksum_disabled = DownloadOptions::builder().verify_checksum(false).build();
    assert!(!options_checksum_disabled.verify_checksum);
}

#[test]
fn test_download_options_output_dir_configurations() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    // Test with output directory
    let options_with_dir = DownloadOptions::builder()
        .output_dir(temp_dir.path().to_path_buf())
        .build();
    assert_eq!(
        options_with_dir.output_dir,
        Some(temp_dir.path().to_path_buf())
    );

    // Test without output directory (default)
    let options_without_dir = DownloadOptions::builder().build();
    assert!(options_without_dir.output_dir.is_none());
}

#[test]
fn test_download_options_headers_default() {
    // Test that default options have empty headers
    let options = DownloadOptions::default();
    assert!(options.headers.is_empty());

    // Test that builder creates options with empty headers by default
    let options = DownloadOptions::builder().build();
    assert!(options.headers.is_empty());
}

#[test]
fn test_download_options_builder_reset() {
    // Test that each builder creates independent options
    let builder = DownloadOptions::builder();

    let options1 = builder.max_concurrent_chunks(4).build();
    let options2 = DownloadOptions::builder().max_concurrent_chunks(8).build();

    assert_eq!(options1.max_concurrent_chunks, 4);
    assert_eq!(options2.max_concurrent_chunks, 8);
}
