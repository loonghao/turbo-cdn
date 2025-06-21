// Licensed under the MIT License
// Copyright (c) 2025 Hal <hal.long@outlook.com>

//! Configuration system tests

use turbo_cdn::config::{ConfigManager, GlobalConfig};

#[tokio::test]
async fn test_config_manager_creation() {
    // Create a temporary config file for testing
    let temp_dir = tempfile::tempdir().unwrap();
    let config_path = temp_dir.path().join("config.toml");

    // Write a minimal config file using our embedded default
    let default_config = include_str!("../src/config/default.toml");
    std::fs::write(&config_path, default_config).unwrap();

    let result = ConfigManager::from_file(config_path).await;
    assert!(
        result.is_ok(),
        "ConfigManager should be created successfully: {:?}",
        result.err()
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
    // Create a temporary config file for testing
    let temp_dir = tempfile::tempdir().unwrap();
    let config_path = temp_dir.path().join("config.toml");

    // Write a complete config file using our embedded default
    let default_config = include_str!("../src/config/default.toml");
    std::fs::write(&config_path, default_config).unwrap();

    let manager = ConfigManager::from_file(config_path).await.unwrap();
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
    assert_eq!(format!("{:?}", region), "Global");

    let region = Region::China;
    assert_eq!(format!("{:?}", region), "China");

    let region = Region::Custom("Test".to_string());
    assert_eq!(format!("{:?}", region), "Custom(\"Test\")");
}
