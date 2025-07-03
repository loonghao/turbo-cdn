// Licensed under the MIT License
// Copyright (c) 2025 Hal <hal.long@outlook.com>

//! # Configuration System
//!
//! Robust, type-safe configuration management based on figment.

use figment::{
    providers::{Env, Format, Toml},
    Figment,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Main configuration for TurboCdn
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TurboCdnConfig {
    /// General settings
    pub general: GeneralConfig,
    /// Performance settings
    pub performance: PerformanceConfig,
    /// Security settings
    pub security: SecurityConfig,
    /// Geographic detection settings
    pub geo_detection: GeoDetectionConfig,
    /// Testing configuration
    pub testing: TestingConfig,
    /// URL mapping rules
    pub url_mapping_rules: Vec<UrlMappingRuleConfig>,
}

/// URL mapping rule configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UrlMappingRuleConfig {
    /// Rule name for identification
    pub name: String,
    /// Regex pattern to match URLs
    pub pattern: String,
    /// Replacement URL templates (in priority order)
    pub replacements: Vec<String>,
    /// Applicable regions for this rule
    pub regions: Vec<Region>,
    /// Priority (lower = higher priority)
    pub priority: u32,
    /// Whether this rule is enabled
    pub enabled: bool,
}

/// General configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    /// Enable debug mode
    pub debug: bool,
    /// Default region
    pub default_region: Region,
    /// User agent string
    pub user_agent: String,
    /// Enable URL mapping cache
    pub enable_url_cache: bool,
    /// URL cache TTL in seconds
    pub url_cache_ttl: u64,
    /// Maximum cache entries
    pub max_cache_entries: usize,
}

/// Performance configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// Maximum concurrent downloads
    pub max_concurrent_downloads: usize,
    /// Chunk size for downloads
    pub chunk_size: u64,
    /// Connection timeout in seconds
    pub timeout: u64,
    /// Maximum retry attempts
    pub retry_attempts: usize,
    /// Enable adaptive chunking
    pub adaptive_chunking: bool,
    /// HTTP connection pool settings
    pub pool_max_idle_per_host: usize,
    /// Pool idle timeout in seconds
    pub pool_idle_timeout: u64,
    /// TCP keepalive timeout in seconds
    pub tcp_keepalive: u64,
    /// Enable HTTP/2 prior knowledge
    pub http2_prior_knowledge: bool,
    /// Minimum chunk size in bytes
    pub min_chunk_size: u64,
    /// Maximum chunk size in bytes
    pub max_chunk_size: u64,
    /// Speed threshold for adaptive chunking in bytes per second
    pub speed_threshold_bytes_per_sec: u64,
    /// Enable adaptive concurrency control
    pub adaptive_concurrency: Option<bool>,
    /// Minimum concurrent downloads
    pub min_concurrent_downloads: Option<u32>,
    /// Maximum concurrent downloads limit
    pub max_concurrent_downloads_limit: Option<u32>,
    /// Network congestion threshold (0.0 to 1.0)
    pub network_congestion_threshold: Option<f64>,
    /// Enable DNS caching
    pub dns_cache_enabled: Option<bool>,
    /// DNS cache TTL in seconds
    pub dns_cache_ttl_seconds: Option<u64>,
    /// Maximum DNS cache entries
    pub dns_cache_max_entries: Option<usize>,
    /// Enable smart chunking
    pub smart_chunking_enabled: Option<bool>,
    /// Chunk performance history size
    pub chunk_performance_history_size: Option<usize>,
}

/// Security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Verify SSL certificates
    pub verify_ssl: bool,
    /// Allowed protocols
    pub allowed_protocols: Vec<String>,
}

/// Geographic detection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeoDetectionConfig {
    /// IP detection APIs
    pub ip_apis: Vec<String>,
    /// IP detection timeout in seconds
    pub ip_detection_timeout: u64,
    /// Enable automatic region detection
    pub auto_detect_region: bool,
}

/// Testing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestingConfig {
    /// Test URLs for connectivity testing
    pub test_urls: Vec<String>,
    /// Speed test file sizes
    pub speed_test_sizes: Vec<u64>,
}

#[allow(clippy::derivable_impls)]
impl Default for TurboCdnConfig {
    fn default() -> Self {
        Self {
            general: GeneralConfig::default(),
            performance: PerformanceConfig::default(),
            security: SecurityConfig::default(),
            geo_detection: GeoDetectionConfig::default(),
            testing: TestingConfig::default(),
            url_mapping_rules: Vec::new(), // Will be loaded from config file
        }
    }
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            debug: false,
            default_region: Region::Global,
            user_agent: format!("turbo-cdn/{}", env!("CARGO_PKG_VERSION")),
            enable_url_cache: true,
            url_cache_ttl: 3600, // 1 hour
            max_cache_entries: 1000,
        }
    }
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            max_concurrent_downloads: 32, // 增加并发数以实现turbo速度
            chunk_size: 1024 * 1024,      // 1MB chunks for better concurrency
            timeout: 30,
            retry_attempts: 3,
            adaptive_chunking: true,
            pool_max_idle_per_host: 50, // 增加连接池大小
            pool_idle_timeout: 90,
            tcp_keepalive: 60,
            http2_prior_knowledge: true,
            min_chunk_size: 128 * 1024, // 128KB for more granular chunks
            max_chunk_size: 5 * 1024 * 1024, // 5MB
            speed_threshold_bytes_per_sec: 1024 * 1024, // 1MB/s
            adaptive_concurrency: Some(true),
            min_concurrent_downloads: Some(8), // 更高的最小并发数
            max_concurrent_downloads_limit: Some(64), // 更高的最大并发数
            network_congestion_threshold: Some(0.3), // 更激进的阈值
            dns_cache_enabled: Some(true),
            dns_cache_ttl_seconds: Some(300),
            dns_cache_max_entries: Some(1000),
            smart_chunking_enabled: Some(true),
            chunk_performance_history_size: Some(100),
        }
    }
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            verify_ssl: true,
            allowed_protocols: vec!["https".to_string(), "http".to_string()],
        }
    }
}

impl Default for GeoDetectionConfig {
    fn default() -> Self {
        Self {
            ip_apis: vec![
                "https://ipapi.co/json/".to_string(),
                "https://ip-api.com/json/".to_string(),
                "https://ipinfo.io/json".to_string(),
                "https://api.ipify.org?format=json".to_string(),
            ],
            ip_detection_timeout: 5,
            auto_detect_region: true,
        }
    }
}

impl Default for TestingConfig {
    fn default() -> Self {
        Self {
            test_urls: vec![
                "https://github.com/BurntSushi/ripgrep/releases/download/14.1.1/ripgrep-14.1.1-x86_64-pc-windows-msvc.zip".to_string(),
                "https://cdn.jsdelivr.net/npm/jquery@3.6.0/dist/jquery.min.js".to_string(),
            ],
            speed_test_sizes: vec![1048576, 10485760, 104857600], // 1MB, 10MB, 100MB
        }
    }
}

impl TurboCdnConfig {
    /// Load configuration from multiple sources
    #[allow(clippy::result_large_err)]
    pub fn load() -> Result<Self, figment::Error> {
        let config_content = include_str!("default.toml");

        Figment::new()
            .merge(Toml::string(config_content))
            .merge(Env::prefixed("TURBO_CDN_").split("__"))
            .extract()
    }

    /// Create with custom config file
    #[allow(clippy::result_large_err)]
    pub fn load_from_file<P: Into<PathBuf>>(path: P) -> Result<Self, figment::Error> {
        let config_content = include_str!("default.toml");

        Figment::new()
            .merge(Toml::string(config_content))
            .merge(Toml::file(path.into()))
            .merge(Env::prefixed("TURBO_CDN_").split("__"))
            .extract()
    }
}

/// Region enum for compatibility
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum Region {
    China,
    Asia,
    #[default]
    Global,
    AsiaPacific,
    Europe,
    NorthAmerica,
    Custom(String),
}

impl std::fmt::Display for Region {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Region::China => write!(f, "China"),
            Region::Asia => write!(f, "Asia"),
            Region::Global => write!(f, "Global"),
            Region::AsiaPacific => write!(f, "AsiaPacific"),
            Region::Europe => write!(f, "Europe"),
            Region::NorthAmerica => write!(f, "NorthAmerica"),
            Region::Custom(name) => write!(f, "{name}"),
        }
    }
}

impl std::str::FromStr for Region {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "China" => Ok(Region::China),
            "Asia" => Ok(Region::Asia),
            "Global" => Ok(Region::Global),
            "AsiaPacific" => Ok(Region::AsiaPacific),
            "Europe" => Ok(Region::Europe),
            "NorthAmerica" => Ok(Region::NorthAmerica),
            other => Ok(Region::Custom(other.to_string())),
        }
    }
}
