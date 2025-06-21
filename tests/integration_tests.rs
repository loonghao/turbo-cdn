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

#[test]
fn test_config_creation() {
    let config = TurboCdnConfig::default();
    // Test that config can be created successfully
    assert!(config.general.enabled);
    assert_eq!(config.general.default_region, "Global");
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

    // Create a simple timeout error for testing (which is retryable)
    let network_error = TurboCdnError::timeout("timeout error");
    assert!(network_error.is_retryable());
}

#[tokio::test]
async fn test_cache_manager() {
    let config = config::CacheConfig {
        enabled: true,
        directory: Some(
            std::env::temp_dir()
                .join("turbo-cdn-test")
                .to_string_lossy()
                .to_string(),
        ),
        max_size: "1MB".to_string(),
        ttl: std::time::Duration::from_secs(3600),
        cleanup_interval: std::time::Duration::from_secs(60),
    };

    let result = CacheManager::new(config).await;
    assert!(
        result.is_ok(),
        "Cache manager should initialize successfully"
    );
}

#[tokio::test]
async fn test_compliance_checker() {
    let config = config::ComplianceConfig {
        verify_ssl: true,
        verify_checksums: true,
        allowed_protocols: vec!["https".to_string(), "http".to_string()],
        user_agent: "turbo-cdn/1.0".to_string(),
        custom_headers: std::collections::HashMap::new(),
        audit_logging: true,
        audit_log_path: "~/.turbo-cdn/audit.log".to_string(),
        validate_source: false,
        verify_open_source: false,
        strict_mode: false,
        data_protection: Default::default(),
    };
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
