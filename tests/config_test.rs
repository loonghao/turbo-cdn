// Licensed under the MIT License
// Copyright (c) 2025 Hal <hal.long@outlook.com>

//! Configuration system tests

use turbo_cdn::config::{ConfigManager, GlobalConfig};

#[tokio::test]
async fn test_config_manager_creation() {
    let result = ConfigManager::new().await;
    assert!(
        result.is_ok(),
        "ConfigManager should be created successfully"
    );
}

#[tokio::test]
async fn test_config_default() {
    let config = GlobalConfig::default();
    assert_eq!(config.meta.version, "1.0");
    assert_eq!(config.meta.schema_version, "2025.1");
    assert!(config.general.enabled);
    assert!(config.performance.cache.enabled);
}

#[tokio::test]
async fn test_config_manager_get_config() {
    let manager = ConfigManager::new().await.unwrap();
    let config = manager.get_config().await;

    // Basic validation
    assert!(!config.meta.version.is_empty());
    assert!(!config.meta.schema_version.is_empty());
    assert!(!config.regions.default.is_empty());
}

#[test]
fn test_region_enum() {
    use turbo_cdn::config::Region;

    let region = Region::Global;
    assert_eq!(region.to_string(), "Global");

    let region = Region::China;
    assert_eq!(region.to_string(), "China");

    let region = Region::Custom("Test".to_string());
    assert_eq!(region.to_string(), "Test");
}
