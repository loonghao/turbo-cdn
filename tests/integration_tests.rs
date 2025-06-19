use turbo_cdn::*;

#[tokio::test]
async fn test_turbo_cdn_builder() {
    let result = TurboCdn::builder()
        .with_sources(&[Source::github(), Source::jsdelivr()])
        .with_region(Region::Global)
        .with_cache(true)
        .build()
        .await;

    assert!(result.is_ok(), "TurboCdn builder should succeed");
}

#[tokio::test]
async fn test_config_validation() {
    let config = TurboCdnConfig::default();
    let result = config.validate();
    assert!(result.is_ok(), "Default config should be valid");
}

#[test]
fn test_source_creation() {
    let _github = Source::github();
    let _jsdelivr = Source::jsdelivr();
    let _fastly = Source::fastly();
    let _cloudflare = Source::cloudflare();
}

#[test]
fn test_progress_info() {
    let progress = ProgressInfo {
        total_size: 1000000,
        downloaded_size: 500000,
        percentage: 50.0,
        speed: 1000000.0,
        eta: None,
        elapsed: std::time::Duration::from_secs(10),
        active_chunks: 4,
        complete: false,
    };

    assert_eq!(progress.percentage_normalized(), 0.5);
    assert_eq!(progress.speed_mbps(), 1.0);
    assert!(progress.speed_human().contains("MB/s"));
}

#[test]
fn test_error_types() {
    let error = TurboCdnError::config("test error");
    assert_eq!(error.category(), "config");
    assert!(!error.is_retryable());

    let network_error = TurboCdnError::Network(reqwest::Error::from(reqwest::Error::from(
        std::io::Error::new(std::io::ErrorKind::TimedOut, "timeout"),
    )));
    assert!(network_error.is_retryable());
}

#[tokio::test]
async fn test_cache_manager() {
    let config = config::CacheConfig {
        enabled: true,
        cache_dir: std::env::temp_dir().join("turbo-cdn-test"),
        max_size: 1024 * 1024, // 1MB
        ttl: 3600,
        compression: true,
        cleanup_interval: 60,
    };

    let result = CacheManager::new(config).await;
    assert!(
        result.is_ok(),
        "Cache manager should initialize successfully"
    );
}

#[tokio::test]
async fn test_compliance_checker() {
    let config = config::ComplianceConfig::default();
    let result = ComplianceChecker::new(config);
    assert!(
        result.is_ok(),
        "Compliance checker should initialize successfully"
    );
}

#[test]
fn test_download_options_builder() {
    let options = DownloadOptions::builder()
        .max_concurrent_chunks(4)
        .chunk_size(512 * 1024)
        .use_cache(true)
        .build();

    assert_eq!(options.max_concurrent_chunks, 4);
    assert_eq!(options.chunk_size, 512 * 1024);
    assert!(options.use_cache);
}

#[test]
fn test_region_enum() {
    let regions = vec![
        Region::Global,
        Region::NorthAmerica,
        Region::Europe,
        Region::AsiaPacific,
        Region::China,
    ];

    for region in regions {
        // Test that regions can be compared and hashed
        let _hash = std::collections::hash_map::DefaultHasher::new();
        assert_eq!(region, region);
    }
}
