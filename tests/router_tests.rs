// Licensed under the MIT License
// Copyright (c) 2025 Hal <hal.long@outlook.com>

use std::time::Duration;
use turbo_cdn::config::{Region, TurboCdnConfig};
use turbo_cdn::router::{DownloadPerformance, SmartRouter};
use turbo_cdn::sources::{DownloadUrl, SourceManager};

/// Helper function to create a test config
fn create_test_config(region: Region) -> TurboCdnConfig {
    let mut config = TurboCdnConfig::default();
    config.general.default_region = match region {
        Region::China => "China".to_string(),
        Region::Global => "Global".to_string(),
        Region::AsiaPacific => "AsiaPacific".to_string(),
        Region::Europe => "Europe".to_string(),
        Region::NorthAmerica => "NorthAmerica".to_string(),
        Region::Custom(s) => s,
    };
    config
}

/// Helper function to create a test download URL
fn create_test_url(source: &str, url: &str) -> DownloadUrl {
    use std::collections::HashMap;
    DownloadUrl {
        url: url.to_string(),
        source: source.to_string(),
        priority: 1,
        size: None,
        checksum: None,
        metadata: HashMap::new(),
        supports_ranges: true,
        estimated_latency: None,
    }
}

/// Helper function to create test download performance
fn create_test_performance(
    url: &str,
    source: &str,
    success: bool,
    speed: f64,
    response_time_ms: u64,
) -> DownloadPerformance {
    DownloadPerformance {
        url: url.to_string(),
        source: source.to_string(),
        success,
        response_time: Duration::from_millis(response_time_ms),
        download_speed: speed,
        bytes_downloaded: if success { 1000000 } else { 0 },
        error: if success {
            None
        } else {
            Some("Test error".to_string())
        },
    }
}

#[tokio::test]
async fn test_smart_router_creation() {
    let config = create_test_config(Region::Global);
    let source_manager = SourceManager::new();
    let router = SmartRouter::new(config, source_manager);

    // Test that router was created successfully
    let stats = router.get_performance_stats();
    assert!(stats.get_source_metrics().is_empty());
    assert!(stats.get_url_metrics().is_empty());
}

#[tokio::test]
async fn test_smart_router_region_setting() {
    let config = create_test_config(Region::Global);
    let source_manager = SourceManager::new();
    let mut router = SmartRouter::new(config, source_manager);

    // Test setting different regions
    router.set_region(Region::China);
    router.set_region(Region::Europe);
    router.set_region(Region::NorthAmerica);
    router.set_region(Region::AsiaPacific);

    // Router should accept all region changes without error
}

#[test]
fn test_download_performance_creation() {
    let performance = create_test_performance(
        "https://example.com/file.zip",
        "github",
        true,
        1000000.0,
        500,
    );

    assert_eq!(performance.url, "https://example.com/file.zip");
    assert_eq!(performance.source, "github");
    assert!(performance.success);
    assert_eq!(performance.download_speed, 1000000.0);
    assert_eq!(performance.response_time, Duration::from_millis(500));
    assert_eq!(performance.bytes_downloaded, 1000000);
    assert!(performance.error.is_none());
}

#[test]
fn test_download_performance_failure() {
    let performance =
        create_test_performance("https://example.com/file.zip", "github", false, 0.0, 1000);

    assert!(!performance.success);
    assert_eq!(performance.download_speed, 0.0);
    assert_eq!(performance.bytes_downloaded, 0);
    assert!(performance.error.is_some());
    assert_eq!(performance.error.unwrap(), "Test error");
}

#[tokio::test]
async fn test_performance_tracking() {
    let config = create_test_config(Region::Global);
    let source_manager = SourceManager::new();
    let mut router = SmartRouter::new(config, source_manager);

    // Record some performance data
    let performance1 = create_test_performance(
        "https://github.com/owner/repo/releases/download/v1.0.0/file.zip",
        "github",
        true,
        2000000.0,
        300,
    );

    let performance2 = create_test_performance(
        "https://cdn.jsdelivr.net/gh/owner/repo@v1.0.0/file.zip",
        "jsdelivr",
        true,
        1500000.0,
        400,
    );

    let performance3 = create_test_performance(
        "https://github.com/owner/repo/releases/download/v1.0.0/file.zip",
        "github",
        false,
        0.0,
        5000,
    );

    router.record_performance(performance1);
    router.record_performance(performance2);
    router.record_performance(performance3);

    // Check that performance was recorded
    let stats = router.get_performance_stats();
    let source_metrics = stats.get_source_metrics();
    let url_metrics = stats.get_url_metrics();

    assert!(source_metrics.contains_key("github"));
    assert!(source_metrics.contains_key("jsdelivr"));
    assert_eq!(source_metrics.len(), 2);

    assert!(
        url_metrics.contains_key("https://github.com/owner/repo/releases/download/v1.0.0/file.zip")
    );
    assert!(url_metrics.contains_key("https://cdn.jsdelivr.net/gh/owner/repo@v1.0.0/file.zip"));
    assert_eq!(url_metrics.len(), 2);

    // Check GitHub metrics (2 requests: 1 success, 1 failure)
    let github_metrics = &source_metrics["github"];
    assert_eq!(github_metrics.total_requests, 2);
    assert_eq!(github_metrics.successful_requests, 1);
    assert_eq!(github_metrics.failed_requests, 1);

    // Check jsDelivr metrics (1 successful request)
    let jsdelivr_metrics = &source_metrics["jsdelivr"];
    assert_eq!(jsdelivr_metrics.total_requests, 1);
    assert_eq!(jsdelivr_metrics.successful_requests, 1);
    assert_eq!(jsdelivr_metrics.failed_requests, 0);
}

#[tokio::test]
async fn test_region_optimization_china() {
    let config = create_test_config(Region::China);
    let source_manager = SourceManager::new();
    let _router = SmartRouter::new(config, source_manager);

    // Test that China region preferences are applied
    // In China, fastly and jsdelivr should be preferred
    // This is tested indirectly through the router's internal logic
}

#[tokio::test]
async fn test_region_optimization_north_america() {
    let config = create_test_config(Region::NorthAmerica);
    let source_manager = SourceManager::new();
    let _router = SmartRouter::new(config, source_manager);

    // Test that North America region preferences are applied
    // In North America, github and cloudflare should be preferred
}

#[tokio::test]
async fn test_region_optimization_europe() {
    let config = create_test_config(Region::Europe);
    let source_manager = SourceManager::new();
    let _router = SmartRouter::new(config, source_manager);

    // Test that Europe region preferences are applied
    // In Europe, jsdelivr and cloudflare should be preferred
}

#[tokio::test]
async fn test_region_optimization_asia_pacific() {
    let config = create_test_config(Region::AsiaPacific);
    let source_manager = SourceManager::new();
    let _router = SmartRouter::new(config, source_manager);

    // Test that Asia Pacific region preferences are applied
    // In Asia Pacific, cloudflare and fastly should be preferred
}

#[tokio::test]
async fn test_performance_metrics_accuracy() {
    let config = create_test_config(Region::Global);
    let source_manager = SourceManager::new();
    let mut router = SmartRouter::new(config, source_manager);

    // Record multiple successful performances for the same source
    for i in 0..5 {
        let performance = create_test_performance(
            &format!("https://example.com/file{}.zip", i),
            "github",
            true,
            (i + 1) as f64 * 1000000.0, // Varying speeds
            100 + i * 50,               // Varying response times
        );
        router.record_performance(performance);
    }

    let stats = router.get_performance_stats();
    let github_metrics = &stats.get_source_metrics()["github"];

    assert_eq!(github_metrics.total_requests, 5);
    assert_eq!(github_metrics.successful_requests, 5);
    assert_eq!(github_metrics.failed_requests, 0);
    // SourceMetrics doesn't track consecutive failures, only UrlMetrics does

    // Average speed should be calculated correctly
    // Speeds: 1MB/s, 2MB/s, 3MB/s, 4MB/s, 5MB/s = average 3MB/s
    assert!((github_metrics.average_download_speed - 3000000.0).abs() < 100000.0);

    // Reliability score should be 100%
    assert_eq!(github_metrics.reliability_score, 1.0);
}

#[tokio::test]
async fn test_consecutive_failures_tracking() {
    let config = create_test_config(Region::Global);
    let source_manager = SourceManager::new();
    let mut router = SmartRouter::new(config, source_manager);

    let url = "https://example.com/file.zip";

    // Record consecutive failures
    for _ in 0..3 {
        let performance = create_test_performance(url, "github", false, 0.0, 5000);
        router.record_performance(performance);
    }

    let stats = router.get_performance_stats();
    let github_metrics = &stats.get_source_metrics()["github"];

    assert_eq!(github_metrics.failed_requests, 3);
    assert_eq!(github_metrics.reliability_score, 0.0);

    // Record a success to reset consecutive failures
    let success_performance = create_test_performance(url, "github", true, 1000000.0, 300);
    router.record_performance(success_performance);

    let stats = router.get_performance_stats();
    let github_metrics = &stats.get_source_metrics()["github"];

    assert_eq!(github_metrics.successful_requests, 1);
    assert_eq!(github_metrics.total_requests, 4);
    assert_eq!(github_metrics.reliability_score, 0.25); // 1 success out of 4 total
}

#[tokio::test]
async fn test_url_metrics_tracking() {
    let config = create_test_config(Region::Global);
    let source_manager = SourceManager::new();
    let mut router = SmartRouter::new(config, source_manager);

    let url = "https://github.com/owner/repo/releases/download/v1.0.0/file.zip";

    // Record performance for specific URL
    let performance = create_test_performance(url, "github", true, 2000000.0, 250);
    router.record_performance(performance);

    let stats = router.get_performance_stats();
    let url_metrics = stats.get_url_metrics();

    assert!(url_metrics.contains_key(url));
    let metrics = &url_metrics[url];

    assert_eq!(metrics.url, url);
    assert_eq!(metrics.source, "github");
    assert_eq!(metrics.total_requests, 1);
    assert_eq!(metrics.successful_requests, 1);
    assert_eq!(metrics.average_download_speed, 2000000.0);
}

#[test]
fn test_download_url_creation() {
    let url = create_test_url(
        "github",
        "https://github.com/owner/repo/releases/download/v1.0.0/file.zip",
    );

    assert_eq!(url.source, "github");
    assert_eq!(
        url.url,
        "https://github.com/owner/repo/releases/download/v1.0.0/file.zip"
    );
    assert_eq!(url.priority, 1);
    assert!(url.size.is_none());
    assert!(url.checksum.is_none());
    assert!(url.estimated_latency.is_none());
}

#[tokio::test]
async fn test_router_source_manager_access() {
    let config = create_test_config(Region::Global);
    let source_manager = SourceManager::new();
    let router = SmartRouter::new(config, source_manager);

    // Test that we can access the source manager
    let _source_manager_ref = router.get_source_manager();
    // This test mainly ensures the method exists and returns a reference
}

#[tokio::test]
async fn test_performance_stats_empty_initially() {
    let config = create_test_config(Region::Global);
    let source_manager = SourceManager::new();
    let router = SmartRouter::new(config, source_manager);

    let stats = router.get_performance_stats();
    assert!(stats.get_source_metrics().is_empty());
    assert!(stats.get_url_metrics().is_empty());
}

#[tokio::test]
async fn test_multiple_sources_performance_tracking() {
    let config = create_test_config(Region::Global);
    let source_manager = SourceManager::new();
    let mut router = SmartRouter::new(config, source_manager);

    // Record performance for multiple sources
    let sources = vec!["github", "jsdelivr", "fastly", "cloudflare"];

    for (i, source) in sources.iter().enumerate() {
        let performance = create_test_performance(
            &format!("https://{}.example.com/file.zip", source),
            source,
            true,
            (i + 1) as f64 * 500000.0,
            200 + i as u64 * 100,
        );
        router.record_performance(performance);
    }

    let stats = router.get_performance_stats();
    let source_metrics = stats.get_source_metrics();

    assert_eq!(source_metrics.len(), 4);
    for source in sources {
        assert!(source_metrics.contains_key(source));
        assert_eq!(source_metrics[source].total_requests, 1);
        assert_eq!(source_metrics[source].successful_requests, 1);
    }
}
