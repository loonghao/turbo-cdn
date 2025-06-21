// Licensed under the MIT License
// Copyright (c) 2025 Hal <hal.long@outlook.com>

//! # Configuration Models
//!
//! Type-safe configuration data structures with zero hardcoding.
//! All default values are loaded from external configuration files.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

/// Root configuration structure
///
/// This is the main configuration container that holds all subsystem configurations.
/// No default values are hardcoded - all defaults come from configuration files.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalConfig {
    /// Configuration metadata
    pub meta: MetaConfig,

    /// General application settings
    pub general: GeneralConfig,

    /// Region and geographic settings
    pub regions: RegionConfig,

    /// Mirror and CDN configurations
    pub mirrors: MirrorConfigs,

    /// Domain validation settings
    pub domains: DomainConfig,

    /// Performance and optimization settings
    pub performance: PerformanceConfig,

    /// Security configuration
    pub security: SecurityConfig,

    /// Logging configuration
    pub logging: LoggingConfig,

    /// Monitoring and metrics
    pub monitoring: MonitoringConfig,

    /// Experimental features
    #[serde(default)]
    pub experimental: ExperimentalConfig,
}

/// Configuration metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaConfig {
    /// Configuration version
    pub version: String,

    /// Schema version for compatibility checking
    pub schema_version: String,

    /// Last updated timestamp (ISO 8601)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_updated: Option<String>,

    /// Enable automatic configuration updates
    #[serde(default)]
    pub auto_update: bool,

    /// Configuration description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// General application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    /// Enable configuration system
    pub enabled: bool,

    /// Configuration check interval
    #[serde(with = "humantime_serde")]
    pub config_check_interval: Duration,

    /// Configuration cache TTL
    #[serde(with = "humantime_serde")]
    pub config_cache_ttl: Duration,

    /// Debug mode
    #[serde(default)]
    pub debug_mode: bool,

    /// Application name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app_name: Option<String>,

    /// User agent string template
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_agent_template: Option<String>,

    /// User agent string (for compatibility)
    #[serde(default = "default_user_agent")]
    pub user_agent: String,

    /// Maximum concurrent downloads
    #[serde(default = "default_max_concurrent")]
    pub max_concurrent_downloads: usize,

    /// Download directory
    #[serde(default = "default_download_dir")]
    pub download_dir: std::path::PathBuf,

    /// Default region
    #[serde(default = "default_region")]
    pub default_region: String,
}

/// Region and geographic configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegionConfig {
    /// Auto-detect user region
    pub auto_detect: bool,

    /// Default region when detection fails
    pub default: String,

    /// Enable China-specific optimizations
    #[serde(default)]
    pub china_optimization: bool,

    /// Region detection timeout
    #[serde(with = "humantime_serde")]
    pub detection_timeout: Duration,

    /// Network test timeout
    #[serde(with = "humantime_serde")]
    pub network_test_timeout: Duration,

    /// Region-specific configurations
    #[serde(default)]
    pub regions: HashMap<String, RegionSpecificConfig>,
}

/// Region-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegionSpecificConfig {
    /// Enable this region
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Priority boost for this region (can be negative)
    #[serde(default)]
    pub priority_boost: i32,

    /// Fallback timeout for this region
    #[serde(with = "humantime_serde")]
    pub fallback_timeout: Duration,

    /// Region-specific mirror preferences
    #[serde(default)]
    pub preferred_mirrors: Vec<String>,

    /// Custom DNS servers for this region
    #[serde(default)]
    pub dns_servers: Vec<String>,
}

/// Mirror configurations container
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MirrorConfigs {
    /// Global mirror settings
    pub enabled: bool,

    /// Health check interval
    #[serde(with = "humantime_serde")]
    pub health_check_interval: Duration,

    /// Enable sync status checking
    #[serde(default)]
    pub sync_check_enabled: bool,

    /// Maximum concurrent health checks
    pub max_concurrent_health_checks: usize,

    /// Default mirror timeout
    #[serde(with = "humantime_serde")]
    pub default_timeout: Duration,

    /// Mirror configurations by type
    #[serde(default)]
    pub configs: HashMap<String, MirrorConfig>,
}

/// Individual mirror configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MirrorConfig {
    /// Enable this mirror type
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Human-readable description
    pub description: String,

    /// Mirror sources in priority order
    pub sources: Vec<MirrorSource>,

    /// URL transformation patterns
    #[serde(default)]
    pub url_patterns: Vec<UrlPattern>,

    /// Health check configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub health_check: Option<HealthCheckConfig>,

    /// Warning message for potentially risky mirrors
    #[serde(skip_serializing_if = "Option::is_none")]
    pub warning: Option<String>,

    /// Mirror-specific timeout
    #[serde(skip_serializing_if = "Option::is_none", with = "humantime_serde")]
    pub timeout: Option<Duration>,

    /// Base URL for API access (for compatibility)
    #[serde(default = "default_api_base_url")]
    pub api_base_url: String,

    /// Base URL for downloads (for compatibility)
    #[serde(default = "default_base_url")]
    pub base_url: String,

    /// Mirror priority (for compatibility)
    #[serde(default = "default_priority")]
    pub priority: u32,

    /// Authentication token (for compatibility)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
}

/// Mirror source definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MirrorSource {
    /// Mirror base URL
    pub url: String,

    /// Applicable regions
    pub regions: Vec<String>,

    /// Trust level (0-100)
    #[serde(default = "default_trust_level")]
    pub trust_level: u8,

    /// Tags for categorization and filtering
    #[serde(default)]
    pub tags: Vec<String>,

    /// Weight for load balancing (higher = more preferred)
    #[serde(default = "default_weight")]
    pub weight: u32,

    /// Maximum concurrent connections to this mirror
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_connections: Option<usize>,
}

/// URL transformation pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UrlPattern {
    /// Source pattern (supports regex)
    pub from: String,

    /// Target replacement pattern
    pub to: String,

    /// Applicable regions
    pub regions: Vec<String>,

    /// Pattern priority (higher = checked first)
    #[serde(default = "default_priority")]
    pub priority: u32,

    /// Whether this pattern uses regex
    #[serde(default)]
    pub is_regex: bool,
}

/// Health check configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckConfig {
    /// Health check endpoint path
    pub path: String,

    /// Expected HTTP status code
    #[serde(default = "default_http_status")]
    pub expected_status: u16,

    /// Request timeout
    #[serde(with = "humantime_serde")]
    pub timeout: Duration,

    /// Check interval
    #[serde(skip_serializing_if = "Option::is_none", with = "humantime_serde")]
    pub interval: Option<Duration>,

    /// Number of retries before marking as unhealthy
    #[serde(default = "default_retries")]
    pub retries: u32,
}

/// Domain validation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainConfig {
    /// Enable domain validation
    #[serde(default = "default_true")]
    pub validation_enabled: bool,

    /// Strict mode (whitelist only)
    #[serde(default)]
    pub strict_mode: bool,

    /// Allow subdomains of trusted domains
    #[serde(default = "default_true")]
    pub allow_subdomains: bool,

    /// Minimum trust level required
    #[serde(default = "default_min_trust")]
    pub min_trust_level: u8,

    /// Domain cache TTL
    #[serde(with = "humantime_serde")]
    pub cache_ttl: Duration,

    /// Trusted domains
    #[serde(default)]
    pub trusted: HashMap<String, TrustedDomain>,

    /// Domain categories and their trust levels
    #[serde(default)]
    pub categories: HashMap<String, DomainCategory>,
}

/// Trusted domain definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustedDomain {
    /// Domain name
    pub domain: String,

    /// Trust level (0-100)
    pub trust_level: u8,

    /// Domain category
    pub category: String,

    /// Tags for filtering
    #[serde(default)]
    pub tags: Vec<String>,

    /// Additional notes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

/// Domain category configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainCategory {
    /// Category name
    pub name: String,

    /// Category description
    pub description: String,

    /// Default trust level for this category
    pub default_trust_level: u8,

    /// Whether domains in this category are allowed by default
    #[serde(default = "default_true")]
    pub allowed_by_default: bool,
}

/// Performance configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// Maximum concurrent downloads
    pub max_concurrent_downloads: usize,

    /// Download chunk size
    pub chunk_size: String,

    /// Request timeout
    #[serde(with = "humantime_serde")]
    pub timeout: Duration,

    /// Number of retry attempts
    pub retry_attempts: usize,

    /// Delay between retries
    #[serde(with = "humantime_serde")]
    pub retry_delay: Duration,

    /// Connection pool size
    pub connection_pool_size: usize,

    /// Keep-alive timeout
    #[serde(with = "humantime_serde")]
    pub keep_alive_timeout: Duration,

    /// Cache configuration
    pub cache: CacheConfig,

    /// Bandwidth limits
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bandwidth: Option<BandwidthConfig>,
}

/// Cache configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// Enable caching
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Maximum cache size
    pub max_size: String,

    /// Cache TTL
    #[serde(with = "humantime_serde")]
    pub ttl: Duration,

    /// Cache cleanup interval
    #[serde(with = "humantime_serde")]
    pub cleanup_interval: Duration,

    /// Cache directory
    #[serde(skip_serializing_if = "Option::is_none")]
    pub directory: Option<String>,
}

/// Bandwidth configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BandwidthConfig {
    /// Maximum download speed (bytes per second)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_download_speed: Option<u64>,

    /// Maximum upload speed (bytes per second)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_upload_speed: Option<u64>,

    /// Enable adaptive bandwidth
    #[serde(default)]
    pub adaptive: bool,
}

/// Security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Verify SSL certificates
    #[serde(default = "default_true")]
    pub verify_ssl: bool,

    /// Verify file checksums
    #[serde(default = "default_true")]
    pub verify_checksums: bool,

    /// Allowed protocols
    pub allowed_protocols: Vec<String>,

    /// User agent string
    pub user_agent: String,

    /// Custom headers
    #[serde(default)]
    pub custom_headers: HashMap<String, String>,

    /// Enable audit logging
    #[serde(default)]
    pub audit_logging: bool,

    /// Audit log file path
    #[serde(default = "default_audit_log_path")]
    pub audit_log_path: String,

    /// Validate source authenticity
    #[serde(default)]
    pub validate_source: bool,

    /// Verify open source licenses
    #[serde(default)]
    pub verify_open_source: bool,

    /// Enable strict security mode
    #[serde(default)]
    pub strict_mode: bool,

    /// Data protection settings
    #[serde(default)]
    pub data_protection: DataProtectionConfig,
}

/// Data protection configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DataProtectionConfig {
    /// Require user consent for data collection
    #[serde(default)]
    pub user_consent_required: bool,

    /// Enable data anonymization
    #[serde(default)]
    pub anonymize_data: bool,

    /// Data retention period in days
    #[serde(default = "default_retention_days")]
    pub retention_days: u32,
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Log level
    pub level: String,

    /// Log format
    pub format: String,

    /// Log output destination
    pub output: String,

    /// Enable audit logging
    #[serde(default)]
    pub audit_enabled: bool,

    /// Audit log file path
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audit_file: Option<String>,

    /// Log rotation configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rotation: Option<LogRotationConfig>,
}

/// Log rotation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogRotationConfig {
    /// Maximum log file size
    pub max_size: String,

    /// Number of log files to keep
    pub max_files: usize,

    /// Rotation interval
    #[serde(with = "humantime_serde")]
    pub interval: Duration,
}

/// Monitoring configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    /// Enable monitoring
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Metrics collection interval
    #[serde(with = "humantime_serde")]
    pub metrics_interval: Duration,

    /// Health check interval
    #[serde(with = "humantime_serde")]
    pub health_check_interval: Duration,

    /// Metrics export configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub export: Option<MetricsExportConfig>,
}

/// Metrics export configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsExportConfig {
    /// Export format (prometheus, json, etc.)
    pub format: String,

    /// Export endpoint
    pub endpoint: String,

    /// Export interval
    #[serde(with = "humantime_serde")]
    pub interval: Duration,
}

/// Experimental features configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ExperimentalConfig {
    /// Enable machine learning optimization
    #[serde(default)]
    pub ml_optimization: bool,

    /// Enable predictive caching
    #[serde(default)]
    pub predictive_caching: bool,

    /// Enable P2P sharing
    #[serde(default)]
    pub p2p_sharing: bool,

    /// Enable advanced analytics
    #[serde(default)]
    pub advanced_analytics: bool,
}

// Default value functions to avoid hardcoding
fn default_true() -> bool {
    true
}
fn default_trust_level() -> u8 {
    80
}
fn default_weight() -> u32 {
    100
}
fn default_priority() -> u32 {
    100
}
fn default_http_status() -> u16 {
    200
}
fn default_retries() -> u32 {
    3
}
fn default_min_trust() -> u8 {
    50
}
fn default_user_agent() -> String {
    "turbo-cdn/1.0".to_string()
}
fn default_max_concurrent() -> usize {
    8
}
fn default_download_dir() -> std::path::PathBuf {
    std::path::PathBuf::from("./downloads")
}
fn default_region() -> String {
    "Global".to_string()
}
fn default_audit_log_path() -> String {
    "~/.turbo-cdn/audit.log".to_string()
}
fn default_retention_days() -> u32 {
    30
}
fn default_api_base_url() -> String {
    "https://api.github.com".to_string()
}
fn default_base_url() -> String {
    "https://github.com".to_string()
}

impl Default for GlobalConfig {
    fn default() -> Self {
        use std::collections::HashMap;
        use std::time::Duration;

        Self {
            meta: MetaConfig {
                version: "1.0".to_string(),
                schema_version: "2025.1".to_string(),
                last_updated: None,
                auto_update: true,
                description: Some("Default configuration".to_string()),
            },
            general: GeneralConfig {
                enabled: true,
                config_check_interval: Duration::from_secs(3600),
                config_cache_ttl: Duration::from_secs(300),
                debug_mode: false,
                app_name: Some("turbo-cdn".to_string()),
                user_agent_template: Some("turbo-cdn/{version}".to_string()),
                user_agent: default_user_agent(),
                max_concurrent_downloads: default_max_concurrent(),
                download_dir: default_download_dir(),
                default_region: default_region(),
            },
            regions: RegionConfig {
                auto_detect: true,
                default: "Global".to_string(),
                china_optimization: true,
                detection_timeout: Duration::from_secs(5),
                network_test_timeout: Duration::from_secs(3),
                regions: HashMap::new(),
            },
            mirrors: MirrorConfigs {
                enabled: true,
                health_check_interval: Duration::from_secs(600),
                sync_check_enabled: true,
                max_concurrent_health_checks: 5,
                default_timeout: Duration::from_secs(30),
                configs: HashMap::new(),
            },
            domains: DomainConfig {
                validation_enabled: true,
                strict_mode: false,
                allow_subdomains: true,
                min_trust_level: 50,
                cache_ttl: Duration::from_secs(3600),
                trusted: HashMap::new(),
                categories: HashMap::new(),
            },
            performance: PerformanceConfig {
                max_concurrent_downloads: 8,
                chunk_size: "1MB".to_string(),
                timeout: Duration::from_secs(30),
                retry_attempts: 3,
                retry_delay: Duration::from_secs(1),
                connection_pool_size: 20,
                keep_alive_timeout: Duration::from_secs(30),
                cache: CacheConfig {
                    enabled: true,
                    max_size: "10GB".to_string(),
                    ttl: Duration::from_secs(86400),
                    cleanup_interval: Duration::from_secs(3600),
                    directory: None,
                },
                bandwidth: None,
            },
            security: SecurityConfig {
                verify_ssl: true,
                verify_checksums: true,
                allowed_protocols: vec!["https".to_string(), "http".to_string()],
                user_agent: "turbo-cdn/1.0".to_string(),
                custom_headers: HashMap::new(),
                audit_logging: true,
                audit_log_path: default_audit_log_path(),
                validate_source: false,
                verify_open_source: false,
                strict_mode: false,
                data_protection: DataProtectionConfig::default(),
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                format: "json".to_string(),
                output: "stdout".to_string(),
                audit_enabled: true,
                audit_file: Some("~/.turbo-cdn/audit.log".to_string()),
                rotation: None,
            },
            monitoring: MonitoringConfig {
                enabled: true,
                metrics_interval: Duration::from_secs(60),
                health_check_interval: Duration::from_secs(300),
                export: None,
            },
            experimental: ExperimentalConfig::default(),
        }
    }
}

impl GlobalConfig {
    /// Get connection timeout from performance settings
    pub fn connect_timeout(&self) -> Duration {
        self.performance.timeout
    }
}
