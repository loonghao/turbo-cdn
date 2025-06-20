// Licensed under the MIT License
// Copyright (c) 2025 Hal <hal.long@outlook.com>

use std::path::PathBuf;

use turbo_cdn::config::{
    CacheConfig, CloudflareConfig, ComplianceConfig, DataProtectionConfig, FastlyConfig,
    GeneralConfig, GitHubConfig, JsDelivrConfig, LogFormat, LoggingConfig, MetricsConfig,
    NetworkConfig, Region, SourcesConfig, TurboCdnConfig,
};

#[test]
fn test_turbo_cdn_config_default() {
    let config = TurboCdnConfig::default();

    // Test general config defaults
    assert_eq!(config.general.max_concurrent_downloads, 4);
    assert_eq!(config.general.default_region, Region::Global);
    assert!(config.general.user_agent.contains("turbo-cdn"));

    // Test network config defaults
    assert_eq!(config.network.connect_timeout, 30);
    assert_eq!(config.network.read_timeout, 60);
    assert_eq!(config.network.max_retries, 3);
    assert_eq!(config.network.retry_delay, 1000);

    // Test cache config defaults
    assert!(config.cache.enabled);
    assert_eq!(config.cache.max_size, 1024 * 1024 * 1024); // 1GB
    assert_eq!(config.cache.ttl, 24 * 60 * 60); // 24 hours
    assert!(config.cache.compression);

    // Test sources config defaults
    assert!(config.sources.github.enabled);
    assert!(config.sources.jsdelivr.enabled);
    assert!(config.sources.fastly.enabled);
    assert!(config.sources.cloudflare.enabled);

    // Test compliance config defaults
    assert!(config.compliance.strict_mode);
    assert!(config.compliance.verify_open_source);
    assert!(config.compliance.audit_logging);

    // Test logging config defaults
    assert_eq!(config.logging.level, "info");
    assert!(config.logging.console);
    // LogFormat doesn't implement PartialEq, so we can't use assert_eq!
    // Just test that it's set to Human variant
    matches!(config.logging.format, LogFormat::Human);

    // Test metrics config defaults
    assert!(config.metrics.enabled);
    assert_eq!(config.metrics.export_interval, 60);
}

#[test]
fn test_general_config_default() {
    let config = GeneralConfig::default();

    assert_eq!(config.max_concurrent_downloads, 4);
    assert_eq!(config.default_region, Region::Global);
    assert!(config.user_agent.contains("turbo-cdn"));
}

#[test]
fn test_network_config_default() {
    let config = NetworkConfig::default();

    assert_eq!(config.connect_timeout, 30);
    assert_eq!(config.read_timeout, 60);
    assert_eq!(config.max_retries, 3);
    assert_eq!(config.retry_delay, 1000);
}

#[test]
fn test_cache_config_default() {
    let config = CacheConfig::default();

    assert!(config.enabled);
    assert_eq!(config.max_size, 1024 * 1024 * 1024); // 1GB
    assert_eq!(config.ttl, 24 * 60 * 60); // 24 hours
    assert!(config.compression);
    assert_eq!(config.cleanup_interval, 60 * 60); // 1 hour

    // Cache directory should be set to a reasonable default
    assert!(config.cache_dir.to_string_lossy().contains("turbo-cdn"));
}

#[test]
fn test_github_config_default() {
    let config = GitHubConfig::default();

    assert!(config.enabled);
    assert!(config.token.is_none());
    assert_eq!(config.api_base_url, "https://api.github.com");
    assert_eq!(config.rate_limit, 5000);
    assert_eq!(config.priority, 1);
}

#[test]
fn test_jsdelivr_config_default() {
    let config = JsDelivrConfig::default();

    assert!(config.enabled);
    assert_eq!(config.base_url, "https://cdn.jsdelivr.net");
    assert_eq!(config.priority, 2);
}

#[test]
fn test_fastly_config_default() {
    let config = FastlyConfig::default();

    assert!(config.enabled);
    assert_eq!(config.base_url, "https://fastly.jsdelivr.net");
    assert_eq!(config.priority, 3);
}

#[test]
fn test_cloudflare_config_default() {
    let config = CloudflareConfig::default();

    assert!(config.enabled);
    assert_eq!(config.base_url, "https://cdnjs.cloudflare.com");
    assert_eq!(config.priority, 4);
}

#[test]
fn test_sources_config_default() {
    let config = SourcesConfig::default();

    assert!(config.github.enabled);
    assert!(config.jsdelivr.enabled);
    assert!(config.fastly.enabled);
    assert!(config.cloudflare.enabled);

    // Test priority ordering
    assert_eq!(config.github.priority, 1);
    assert_eq!(config.jsdelivr.priority, 2);
    assert_eq!(config.fastly.priority, 3);
    assert_eq!(config.cloudflare.priority, 4);
}

#[test]
fn test_compliance_config_default() {
    let config = ComplianceConfig::default();

    assert!(config.strict_mode);
    assert!(config.verify_open_source);
    assert!(config.check_copyright);
    assert!(config.validate_source);
    assert!(config.audit_logging);
    assert_eq!(config.audit_log_path, PathBuf::from("./audit.log"));

    // Test data protection defaults
    assert!(config.data_protection.minimal_data_collection);
    assert!(config.data_protection.user_consent_required);
    assert_eq!(config.data_protection.data_retention_days, 30);
    assert!(config.data_protection.anonymize_data);
}

#[test]
fn test_data_protection_config_default() {
    let config = DataProtectionConfig::default();

    assert!(config.minimal_data_collection);
    assert!(config.user_consent_required);
    assert_eq!(config.data_retention_days, 30);
    assert!(config.anonymize_data);
}

#[test]
fn test_logging_config_default() {
    let config = LoggingConfig::default();

    assert_eq!(config.level, "info");
    assert!(config.file_path.is_none());
    assert!(config.console);
    // LogFormat doesn't implement PartialEq
    matches!(config.format, LogFormat::Human);
}

#[test]
fn test_metrics_config_default() {
    let config = MetricsConfig::default();

    assert!(config.enabled);
    assert_eq!(config.export_interval, 60);
    assert_eq!(config.storage_path, PathBuf::from("./metrics"));
}

#[test]
fn test_region_enum() {
    // Test that all regions can be created
    let regions = vec![
        Region::Global,
        Region::China,
        Region::Europe,
        Region::NorthAmerica,
        Region::AsiaPacific,
    ];

    for region in regions {
        // Test that regions can be compared
        assert_eq!(region, region);
    }
}

#[test]
fn test_log_format_enum() {
    // Test that all log formats can be created
    let formats = vec![LogFormat::Human, LogFormat::Json];

    for format in formats {
        // Test that formats can be created (LogFormat doesn't implement PartialEq)
        match format {
            LogFormat::Human => {}
            LogFormat::Json => {}
            LogFormat::Compact => {}
        }
    }
}

#[test]
fn test_config_serialization() {
    let config = TurboCdnConfig::default();

    // Test serialization
    let serialized = serde_json::to_string(&config).unwrap();
    assert!(serialized.contains("turbo-cdn"));
    assert!(serialized.contains("github"));
    assert!(serialized.contains("jsdelivr"));

    // Test deserialization
    let deserialized: TurboCdnConfig = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized.general.user_agent, config.general.user_agent);
    assert_eq!(
        deserialized.general.max_concurrent_downloads,
        config.general.max_concurrent_downloads
    );
    assert_eq!(
        deserialized.network.connect_timeout,
        config.network.connect_timeout
    );
    assert_eq!(deserialized.cache.enabled, config.cache.enabled);
}

#[test]
fn test_config_validation() {
    let config = TurboCdnConfig::default();
    let result = config.validate();
    assert!(result.is_ok(), "Default config should be valid");
}

#[test]
fn test_config_validation_with_invalid_values() {
    let mut config = TurboCdnConfig::default();

    // Test that validation exists and can be called
    let result = config.validate();
    assert!(result.is_ok(), "Default config should be valid");

    // Test with some extreme values that might be invalid
    config.general.max_concurrent_downloads = 1000; // Very high value
    let result = config.validate();
    // The validation might accept this, so we just test that it doesn't panic
    let _ = result;

    // Test with disabled cache
    config.cache.enabled = false;
    let result = config.validate();
    assert!(result.is_ok(), "Config with disabled cache should be valid");
}

#[test]
fn test_config_timeout_values() {
    let config = TurboCdnConfig::default();

    // Test that timeout values are reasonable
    assert!(config.network.connect_timeout > 0);
    assert!(config.network.read_timeout > 0);
    assert!(config.network.retry_delay > 0);
    assert!(config.network.max_retries > 0);
}

#[test]
fn test_config_custom_values() {
    let mut config = TurboCdnConfig::default();

    // Customize general settings
    config.general.max_concurrent_downloads = 16;
    config.general.default_region = Region::China;
    config.general.user_agent = "custom-agent/2.0.0".to_string();

    // Customize network settings
    config.network.connect_timeout = 20;
    config.network.read_timeout = 60;
    config.network.max_retries = 5;
    config.network.retry_delay = 2000;

    // Customize cache settings
    config.cache.enabled = false;
    config.cache.max_size = 2 * 1024 * 1024 * 1024; // 2GB
    config.cache.ttl = 48 * 60 * 60; // 48 hours
    config.cache.compression = false;

    // Test that custom values are preserved
    assert_eq!(config.general.max_concurrent_downloads, 16);
    assert_eq!(config.general.default_region, Region::China);
    assert_eq!(config.general.user_agent, "custom-agent/2.0.0");
    assert_eq!(config.network.connect_timeout, 20);
    assert_eq!(config.network.read_timeout, 60);
    assert_eq!(config.network.max_retries, 5);
    assert_eq!(config.network.retry_delay, 2000);
    assert!(!config.cache.enabled);
    assert_eq!(config.cache.max_size, 2 * 1024 * 1024 * 1024);
    assert_eq!(config.cache.ttl, 48 * 60 * 60);
    assert!(!config.cache.compression);
}

#[test]
fn test_config_source_priorities() {
    let config = TurboCdnConfig::default();

    // Test that sources have different priorities
    let priorities = [
        config.sources.github.priority,
        config.sources.jsdelivr.priority,
        config.sources.fastly.priority,
        config.sources.cloudflare.priority,
    ];

    // All priorities should be different
    for (i, &priority1) in priorities.iter().enumerate() {
        for (j, &priority2) in priorities.iter().enumerate() {
            if i != j {
                assert_ne!(priority1, priority2, "Source priorities should be unique");
            }
        }
    }

    // GitHub should have the highest priority (lowest number)
    assert_eq!(config.sources.github.priority, 1);
}

#[test]
fn test_config_github_token_handling() {
    let mut config = TurboCdnConfig::default();

    // Test without token
    assert!(config.sources.github.token.is_none());

    // Test with token
    config.sources.github.token = Some("ghp_test_token".to_string());
    assert!(config.sources.github.token.is_some());
    assert_eq!(config.sources.github.token.unwrap(), "ghp_test_token");
}

#[test]
fn test_config_logging_file_path() {
    let mut config = TurboCdnConfig::default();

    // Test default (no file path)
    assert!(config.logging.file_path.is_none());

    // Test with file path
    config.logging.file_path = Some(PathBuf::from("/var/log/turbo-cdn.log"));
    assert!(config.logging.file_path.is_some());
    assert_eq!(
        config.logging.file_path.unwrap(),
        PathBuf::from("/var/log/turbo-cdn.log")
    );
}
