// Licensed under the MIT License
// Copyright (c) 2025 Hal <hal.long@outlook.com>

//! # Configuration Manager
//!
//! A robust configuration manager using figment for type-safe configuration loading
//! with support for multiple sources, environment variables, and validation.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use figment::{
    providers::{Env, Format, Toml},
    Figment,
};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use super::models::GlobalConfig;
use crate::error::{Result, TurboCdnError};

/// Configuration manager with figment-based loading
#[derive(Debug)]
pub struct ConfigManager {
    /// Current configuration
    config: Arc<RwLock<GlobalConfig>>,
    /// Configuration file paths
    config_paths: Vec<PathBuf>,
}

/// Configuration events for monitoring changes
#[derive(Debug, Clone)]
pub enum ConfigEvent {
    /// Configuration was loaded
    Loaded,
    /// Configuration was reloaded
    Reloaded,
    /// Configuration validation failed
    ValidationFailed(String),
}

impl ConfigManager {
    /// Create a new configuration manager with default settings
    pub async fn new() -> Result<Self> {
        let config_paths = Self::discover_config_files();
        let config = Self::load_config(&config_paths).await?;

        Ok(Self {
            config: Arc::new(RwLock::new(config)),
            config_paths,
        })
    }

    /// Create configuration manager from specific file
    pub async fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        let config = Self::load_config(&[path.clone()]).await?;

        Ok(Self {
            config: Arc::new(RwLock::new(config)),
            config_paths: vec![path],
        })
    }

    /// Get current configuration
    pub async fn get_config(&self) -> GlobalConfig {
        self.config.read().await.clone()
    }

    /// Get configuration as Arc for efficient sharing
    pub async fn get_config_arc(&self) -> Arc<GlobalConfig> {
        Arc::new(self.config.read().await.clone())
    }

    /// Reload configuration from sources
    pub async fn reload(&self) -> Result<()> {
        let new_config = Self::load_config(&self.config_paths).await?;
        *self.config.write().await = new_config;
        info!("Configuration reloaded successfully");
        Ok(())
    }

    /// Discover configuration files in standard locations
    fn discover_config_files() -> Vec<PathBuf> {
        let mut files = Vec::new();

        // System-wide configuration files
        let system_paths = [
            "/etc/turbo-cdn/config.toml",
            "/usr/local/etc/turbo-cdn/config.toml",
        ];

        for path in &system_paths {
            let path = PathBuf::from(path);
            if path.exists() {
                files.push(path);
            }
        }

        // User configuration files
        if let Some(home) = dirs::home_dir() {
            let user_paths = [".config/turbo-cdn/config.toml", ".turbo-cdn/config.toml"];

            for path in &user_paths {
                let path = home.join(path);
                if path.exists() {
                    files.push(path);
                }
            }
        }

        // Project-level configuration files
        let project_paths = ["turbo-cdn.toml", ".turbo-cdn.toml", "config/default.toml"];

        for filename in &project_paths {
            let path = PathBuf::from(filename);
            if path.exists() {
                files.push(path);
            }
        }

        files
    }

    /// Load configuration from multiple sources
    async fn load_config(paths: &[PathBuf]) -> Result<GlobalConfig> {
        let mut figment = Figment::new();

        // Add configuration files in order
        for path in paths {
            if path.exists() {
                debug!("Loading configuration from: {}", path.display());
                figment = figment.merge(Toml::file(path));
            } else {
                warn!("Configuration file not found: {}", path.display());
            }
        }

        // Add environment variables with TURBO_CDN prefix
        figment = figment.merge(Env::prefixed("TURBO_CDN_"));

        // Extract configuration
        let config: GlobalConfig = figment
            .extract()
            .map_err(|e| TurboCdnError::config(format!("Failed to load configuration: {}", e)))?;

        // Basic validation
        Self::validate_config(&config)?;

        info!(
            "Configuration loaded successfully from {} sources",
            paths.len()
        );
        Ok(config)
    }

    /// Basic configuration validation
    fn validate_config(config: &GlobalConfig) -> Result<()> {
        // Validate meta
        if config.meta.version.is_empty() {
            return Err(TurboCdnError::config(
                "Configuration version cannot be empty".to_string(),
            ));
        }

        if config.meta.schema_version.is_empty() {
            return Err(TurboCdnError::config(
                "Configuration schema version cannot be empty".to_string(),
            ));
        }

        // Validate general settings
        if config.general.config_check_interval.as_secs() < 30 {
            return Err(TurboCdnError::config(
                "Config check interval must be at least 30 seconds".to_string(),
            ));
        }

        // Validate regions
        if config.regions.default.is_empty() {
            return Err(TurboCdnError::config(
                "Default region cannot be empty".to_string(),
            ));
        }

        // Validate performance settings
        if config.performance.max_concurrent_downloads == 0 {
            return Err(TurboCdnError::config(
                "Max concurrent downloads must be greater than 0".to_string(),
            ));
        }

        if config.performance.retry_attempts > 10 {
            return Err(TurboCdnError::config(
                "Retry attempts should not exceed 10".to_string(),
            ));
        }

        // Validate security settings
        if config.security.allowed_protocols.is_empty() {
            return Err(TurboCdnError::config(
                "At least one protocol must be allowed".to_string(),
            ));
        }

        if config.security.user_agent.is_empty() {
            return Err(TurboCdnError::config(
                "User agent cannot be empty".to_string(),
            ));
        }

        // Validate logging settings
        let valid_levels = ["trace", "debug", "info", "warn", "error"];
        if !valid_levels.contains(&config.logging.level.as_str()) {
            return Err(TurboCdnError::config(format!(
                "Invalid log level: {}. Must be one of: {}",
                config.logging.level,
                valid_levels.join(", ")
            )));
        }

        debug!("Configuration validation passed");
        Ok(())
    }

    /// Get configuration sources information
    pub fn get_sources(&self) -> &[PathBuf] {
        &self.config_paths
    }
}

impl Default for ConfigManager {
    fn default() -> Self {
        // This is a blocking operation, but we need it for Default trait
        // In practice, use ConfigManager::new() instead
        tokio::runtime::Handle::current()
            .block_on(Self::new())
            .expect("Failed to create default ConfigManager")
    }
}

/// Configuration builder for fluent API
#[derive(Debug, Default)]
pub struct ConfigBuilder {
    config_file: Option<PathBuf>,
    enable_env_vars: bool,
}

impl ConfigBuilder {
    /// Create a new configuration builder
    pub fn new() -> Self {
        Self {
            config_file: None,
            enable_env_vars: true,
        }
    }

    /// Set configuration file path
    pub fn with_file<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.config_file = Some(path.as_ref().to_path_buf());
        self
    }

    /// Enable or disable environment variable overrides
    pub fn with_env_vars(mut self, enabled: bool) -> Self {
        self.enable_env_vars = enabled;
        self
    }

    /// Build the configuration manager
    pub async fn build(self) -> Result<ConfigManager> {
        if let Some(config_file) = self.config_file {
            ConfigManager::from_file(config_file).await
        } else {
            ConfigManager::new().await
        }
    }
}
