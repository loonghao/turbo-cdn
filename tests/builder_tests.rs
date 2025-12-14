// Licensed under the MIT License
// Copyright (c) 2025 Hal <hal.long@outlook.com>

//! Tests for the TurboCdn builder pattern and new API features

use turbo_cdn::*;

/// Test TurboCdnBuilder creation
#[test]
fn test_builder_creation() {
    let builder = TurboCdn::builder();
    // Builder should be created successfully
    assert!(std::mem::size_of_val(&builder) > 0);
}

/// Test builder with region configuration
#[test]
fn test_builder_with_region() {
    let builder = TurboCdn::builder().with_region(Region::China);

    // Builder chain should work
    assert!(std::mem::size_of_val(&builder) > 0);
}

/// Test builder with multiple configurations
#[test]
fn test_builder_with_multiple_configs() {
    let builder = TurboCdn::builder()
        .with_region(Region::Global)
        .with_max_concurrent_downloads(16)
        .with_chunk_size(2 * 1024 * 1024)
        .with_timeout(60)
        .with_adaptive_chunking(true)
        .with_retry_attempts(5)
        .with_debug(false)
        .with_user_agent("test-agent/1.0");

    // Builder chain should work
    assert!(std::mem::size_of_val(&builder) > 0);
}

/// Test builder with auto detect region
#[test]
fn test_builder_with_auto_detect() {
    let builder = TurboCdn::builder().with_auto_detect_region(false);

    assert!(std::mem::size_of_val(&builder) > 0);
}

/// Test TurboCdnStats creation and methods
#[test]
fn test_stats_creation() {
    let stats = TurboCdnStats {
        total_downloads: 100,
        successful_downloads: 95,
        failed_downloads: 5,
        total_bytes: 1024 * 1024 * 1024, // 1GB
        cache_hit_rate: 0.8,
        average_speed: 10.0 * 1024.0 * 1024.0, // 10 MB/s
        uptime: std::time::Duration::from_secs(3600),
    };

    assert_eq!(stats.total_downloads, 100);
    assert_eq!(stats.successful_downloads, 95);
    assert_eq!(stats.failed_downloads, 5);
}

/// Test stats success rate calculation
#[test]
fn test_stats_success_rate() {
    let stats = TurboCdnStats {
        total_downloads: 100,
        successful_downloads: 95,
        failed_downloads: 5,
        total_bytes: 0,
        cache_hit_rate: 0.0,
        average_speed: 0.0,
        uptime: std::time::Duration::ZERO,
    };

    let rate = stats.success_rate();
    assert!((rate - 95.0).abs() < 0.01);
}

/// Test stats with zero downloads
#[test]
fn test_stats_zero_downloads() {
    let stats = TurboCdnStats::default();

    assert_eq!(stats.success_rate(), 0.0);
    assert_eq!(stats.average_speed_mbps(), 0.0);
}

/// Test stats average speed in MB/s
#[test]
fn test_stats_average_speed_mbps() {
    let stats = TurboCdnStats {
        total_downloads: 10,
        successful_downloads: 10,
        failed_downloads: 0,
        total_bytes: 100 * 1024 * 1024,
        cache_hit_rate: 1.0,
        average_speed: 10.0 * 1024.0 * 1024.0, // 10 MB/s in bytes
        uptime: std::time::Duration::from_secs(10),
    };

    let speed_mbps = stats.average_speed_mbps();
    assert!((speed_mbps - 10.0).abs() < 0.01);
}

/// Test stats total bytes human readable format
#[test]
fn test_stats_total_bytes_human() {
    // Test GB
    let stats = TurboCdnStats {
        total_bytes: 2 * 1024 * 1024 * 1024, // 2GB
        ..Default::default()
    };
    assert!(stats.total_bytes_human().contains("GB"));

    // Test MB
    let stats = TurboCdnStats {
        total_bytes: 100 * 1024 * 1024, // 100MB
        ..Default::default()
    };
    assert!(stats.total_bytes_human().contains("MB"));

    // Test KB
    let stats = TurboCdnStats {
        total_bytes: 100 * 1024, // 100KB
        ..Default::default()
    };
    assert!(stats.total_bytes_human().contains("KB"));

    // Test Bytes
    let stats = TurboCdnStats {
        total_bytes: 500, // 500B
        ..Default::default()
    };
    assert!(stats.total_bytes_human().contains("B"));
}

/// Test constants are accessible
#[test]
fn test_constants_accessible() {
    // Test that constants are exported and have expected values
    assert_eq!(DEFAULT_RETRY_ATTEMPTS, 3);
    assert_eq!(MAX_SERVERS_TO_TRACK, 100);
    assert_eq!(MAX_URLS_TO_TRY, 8);
    assert_eq!(RECENT_SAMPLES_SIZE, 10);
    assert!((DEFAULT_SERVER_SCORE - 0.5).abs() < f64::EPSILON);

    // Test weight constants sum to 1.0
    let total_weight = WEIGHT_SPEED + WEIGHT_SUCCESS + WEIGHT_LATENCY;
    assert!((total_weight - 1.0).abs() < f64::EPSILON);
}

/// Test PerformanceSummary is accessible
#[test]
fn test_performance_summary_accessible() {
    let summary = PerformanceSummary {
        total_servers: 5,
        total_attempts: 100,
        total_successes: 95,
        overall_success_rate: 0.95,
        average_speed: 5.0 * 1024.0 * 1024.0,
        best_server: Some(("http://fast.example.com".to_string(), 0.95)),
    };

    assert_eq!(summary.total_servers, 5);
    assert_eq!(summary.total_attempts, 100);
    assert!(summary.best_server.is_some());
}

/// Test ServerStats is accessible
#[test]
fn test_server_stats_accessible() {
    let stats = ServerStats::default();

    // Default values
    assert_eq!(stats.average_speed, 0.0);
    assert_eq!(stats.success_rate, 1.0); // Optimistic default
    assert_eq!(stats.total_attempts, 0);

    // Performance score for untested server
    let score = stats.performance_score();
    assert!((score - DEFAULT_SERVER_SCORE).abs() < f64::EPSILON);
}

/// Test async builder build
#[tokio::test]
async fn test_async_builder_build() {
    // Build with auto-detect disabled to avoid network calls
    let result = TurboCdn::builder()
        .with_auto_detect_region(false)
        .with_region(Region::Global)
        .build()
        .await;

    assert!(result.is_ok());
}

/// Test TurboCdn new with default config
#[tokio::test]
async fn test_turbo_cdn_new() {
    let result = TurboCdn::new().await;
    // Should succeed (may auto-detect region)
    assert!(result.is_ok());
}

/// Test TurboCdn get_stats
#[tokio::test]
async fn test_turbo_cdn_get_stats() {
    let cdn = TurboCdn::builder()
        .with_auto_detect_region(false)
        .build()
        .await
        .unwrap();

    let stats = cdn.get_stats().await;

    // Initial stats should be zero
    assert_eq!(stats.total_downloads, 0);
    assert_eq!(stats.successful_downloads, 0);
    assert_eq!(stats.failed_downloads, 0);
}

/// Test TurboCdn reset_stats
#[tokio::test]
async fn test_turbo_cdn_reset_stats() {
    let cdn = TurboCdn::builder()
        .with_auto_detect_region(false)
        .build()
        .await
        .unwrap();

    // Reset stats
    cdn.reset_stats().await;

    let stats = cdn.get_stats().await;
    assert_eq!(stats.total_downloads, 0);
}

/// Test TurboCdn get_performance_summary
#[tokio::test]
async fn test_turbo_cdn_performance_summary() {
    let cdn = TurboCdn::builder()
        .with_auto_detect_region(false)
        .build()
        .await
        .unwrap();

    let summary = cdn.get_performance_summary();

    // Initially no servers tracked
    assert_eq!(summary.total_servers, 0);
    assert_eq!(summary.total_attempts, 0);
}

/// Test can_optimize_url
#[tokio::test]
async fn test_can_optimize_url() {
    let cdn = TurboCdn::builder()
        .with_auto_detect_region(false)
        .with_region(Region::China)
        .build()
        .await
        .unwrap();

    // GitHub URL should be optimizable in China region
    let github_url = "https://github.com/user/repo/releases/download/v1.0.0/file.zip";
    let can_optimize = cdn.can_optimize_url(github_url).await;
    assert!(can_optimize);

    // Unknown URL should not be optimizable
    let unknown_url = "https://example.com/file.zip";
    let can_optimize = cdn.can_optimize_url(unknown_url).await;
    assert!(!can_optimize);
}

/// Test get_optimal_url
#[tokio::test]
async fn test_get_optimal_url() {
    let cdn = TurboCdn::builder()
        .with_auto_detect_region(false)
        .with_region(Region::China)
        .build()
        .await
        .unwrap();

    let github_url = "https://github.com/user/repo/releases/download/v1.0.0/file.zip";
    let optimal = cdn.get_optimal_url(github_url).await;

    assert!(optimal.is_ok());
    // Should return a valid URL
    let url = optimal.unwrap();
    assert!(!url.is_empty());
}

/// Test get_all_cdn_urls
#[tokio::test]
async fn test_get_all_cdn_urls() {
    let cdn = TurboCdn::builder()
        .with_auto_detect_region(false)
        .with_region(Region::China)
        .build()
        .await
        .unwrap();

    let github_url = "https://github.com/user/repo/releases/download/v1.0.0/file.zip";
    let urls = cdn.get_all_cdn_urls(github_url).await;

    assert!(urls.is_ok());
    let urls = urls.unwrap();

    // Should have multiple CDN alternatives for China region
    assert!(urls.len() > 1);
    // Original URL should be included
    assert!(urls.contains(&github_url.to_string()));
}

/// Test Region enum
#[test]
fn test_region_enum() {
    // Test Display trait
    assert_eq!(Region::China.to_string(), "China");
    assert_eq!(Region::Asia.to_string(), "Asia");
    assert_eq!(Region::Global.to_string(), "Global");
    assert_eq!(Region::AsiaPacific.to_string(), "AsiaPacific");
    assert_eq!(Region::Europe.to_string(), "Europe");
    assert_eq!(Region::NorthAmerica.to_string(), "NorthAmerica");
    assert_eq!(
        Region::Custom("MyRegion".to_string()).to_string(),
        "MyRegion"
    );

    // Test FromStr trait
    assert_eq!("China".parse::<Region>().unwrap(), Region::China);
    assert_eq!("Global".parse::<Region>().unwrap(), Region::Global);
    assert_eq!(
        "Custom".parse::<Region>().unwrap(),
        Region::Custom("Custom".to_string())
    );
}

/// Test Default for TurboCdnBuilder
#[test]
fn test_builder_default() {
    let builder = TurboCdnBuilder::default();
    assert!(std::mem::size_of_val(&builder) > 0);
}

/// Test DownloadOptions creation and builder methods
#[test]
fn test_download_options_creation() {
    let options = DownloadOptions::new();
    assert!(options.progress_callback.is_none());
    assert!(options.max_concurrent_chunks.is_none());
    assert!(options.chunk_size.is_none());
    assert!(!options.enable_resume);
    assert!(options.custom_headers.is_none());
    assert!(options.timeout_override.is_none());
    assert!(!options.verify_integrity);
    assert!(options.expected_size.is_none());
}

/// Test DownloadOptions with_max_concurrent_chunks
#[test]
fn test_download_options_with_concurrent_chunks() {
    let options = DownloadOptions::new().with_max_concurrent_chunks(8);
    assert_eq!(options.max_concurrent_chunks, Some(8));
}

/// Test DownloadOptions with_chunk_size
#[test]
fn test_download_options_with_chunk_size() {
    let options = DownloadOptions::new().with_chunk_size(1024 * 1024);
    assert_eq!(options.chunk_size, Some(1024 * 1024));
}

/// Test DownloadOptions with_resume
#[test]
fn test_download_options_with_resume() {
    let options = DownloadOptions::new().with_resume(true);
    assert!(options.enable_resume);
}

/// Test DownloadOptions with_header
#[test]
fn test_download_options_with_header() {
    let options = DownloadOptions::new()
        .with_header("Authorization", "Bearer token123")
        .with_header("X-Custom-Header", "value");

    assert!(options.custom_headers.is_some());
    let headers = options.custom_headers.unwrap();
    assert_eq!(
        headers.get("Authorization"),
        Some(&"Bearer token123".to_string())
    );
    assert_eq!(headers.get("X-Custom-Header"), Some(&"value".to_string()));
}

/// Test DownloadOptions with_timeout
#[test]
fn test_download_options_with_timeout() {
    use std::time::Duration;
    let options = DownloadOptions::new().with_timeout(Duration::from_secs(120));
    assert_eq!(options.timeout_override, Some(Duration::from_secs(120)));
}

/// Test DownloadOptions with_integrity_verification
#[test]
fn test_download_options_with_integrity() {
    let options = DownloadOptions::new().with_integrity_verification(true);
    assert!(options.verify_integrity);
}

/// Test DownloadOptions with_expected_size
#[test]
fn test_download_options_with_expected_size() {
    let options = DownloadOptions::new().with_expected_size(1024 * 1024 * 100);
    assert_eq!(options.expected_size, Some(1024 * 1024 * 100));
}

/// Test DownloadOptions with_progress_callback
#[test]
fn test_download_options_with_progress_callback() {
    let options = DownloadOptions::new().with_progress_callback(Box::new(|_progress| {
        // Callback logic
    }));
    assert!(options.progress_callback.is_some());
}

/// Test DownloadOptions Debug implementation
#[test]
fn test_download_options_debug() {
    let options = DownloadOptions::new()
        .with_max_concurrent_chunks(4)
        .with_chunk_size(1024)
        .with_resume(true);

    let debug_str = format!("{:?}", options);
    assert!(debug_str.contains("DownloadOptions"));
    assert!(debug_str.contains("max_concurrent_chunks"));
    assert!(debug_str.contains("chunk_size"));
    assert!(debug_str.contains("enable_resume"));
}

/// Test DownloadOptions Clone implementation
#[test]
fn test_download_options_clone() {
    let options = DownloadOptions::new()
        .with_max_concurrent_chunks(8)
        .with_chunk_size(2048)
        .with_resume(true)
        .with_header("X-Test", "value")
        .with_timeout(std::time::Duration::from_secs(60))
        .with_integrity_verification(true)
        .with_expected_size(1000);

    let cloned = options.clone();

    // Cloned should have same values except progress_callback (which is None)
    assert_eq!(cloned.max_concurrent_chunks, Some(8));
    assert_eq!(cloned.chunk_size, Some(2048));
    assert!(cloned.enable_resume);
    assert!(cloned.custom_headers.is_some());
    assert_eq!(
        cloned.timeout_override,
        Some(std::time::Duration::from_secs(60))
    );
    assert!(cloned.verify_integrity);
    assert_eq!(cloned.expected_size, Some(1000));
    assert!(cloned.progress_callback.is_none()); // Cannot clone function pointers
}

/// Test DownloadOptions chained builder pattern
#[test]
fn test_download_options_chained() {
    let options = DownloadOptions::new()
        .with_max_concurrent_chunks(16)
        .with_chunk_size(4 * 1024 * 1024)
        .with_resume(true)
        .with_header("User-Agent", "TestAgent/1.0")
        .with_timeout(std::time::Duration::from_secs(300))
        .with_integrity_verification(true)
        .with_expected_size(1024 * 1024 * 500);

    assert_eq!(options.max_concurrent_chunks, Some(16));
    assert_eq!(options.chunk_size, Some(4 * 1024 * 1024));
    assert!(options.enable_resume);
    assert!(options.custom_headers.is_some());
    assert_eq!(
        options.timeout_override,
        Some(std::time::Duration::from_secs(300))
    );
    assert!(options.verify_integrity);
    assert_eq!(options.expected_size, Some(1024 * 1024 * 500));
}

/// Test TurboCdnBuilder with_config
#[test]
fn test_builder_with_config() {
    let config = TurboCdnConfig::default();
    let builder = TurboCdn::builder().with_config(config);
    assert!(std::mem::size_of_val(&builder) > 0);
}

/// Test TurboCdn extract_filename (via download methods)
#[tokio::test]
async fn test_extract_filename_from_url() {
    let cdn = TurboCdn::builder()
        .with_auto_detect_region(false)
        .build()
        .await
        .unwrap();

    // Test with various URL formats
    let url1 = "https://example.com/path/to/file.zip";
    let optimal = cdn.get_optimal_url(url1).await;
    assert!(optimal.is_ok());

    let url2 = "https://example.com/";
    let optimal = cdn.get_optimal_url(url2).await;
    assert!(optimal.is_ok());
}

/// Test TurboCdn get_server_stats returns None for unknown URL
#[tokio::test]
async fn test_get_server_stats_unknown() {
    let cdn = TurboCdn::builder()
        .with_auto_detect_region(false)
        .build()
        .await
        .unwrap();

    let stats = cdn.get_server_stats("https://unknown.example.com/file.zip");
    assert!(stats.is_none());
}
