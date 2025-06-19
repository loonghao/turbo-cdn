use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;

use crate::error::{Result, TurboCdnError};

/// Global configuration for turbo-cdn
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TurboCdnConfig {
    /// General settings
    pub general: GeneralConfig,

    /// Network settings
    pub network: NetworkConfig,

    /// Cache settings
    pub cache: CacheConfig,

    /// Source configurations
    pub sources: SourcesConfig,

    /// Compliance settings
    pub compliance: ComplianceConfig,

    /// Logging settings
    pub logging: LoggingConfig,

    /// Metrics settings
    pub metrics: MetricsConfig,
}

/// General configuration settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    /// User agent string for HTTP requests
    pub user_agent: String,

    /// Default download directory
    pub download_dir: PathBuf,

    /// Maximum number of concurrent downloads
    pub max_concurrent_downloads: usize,

    /// Default region for CDN selection
    pub default_region: Region,

    /// Enable debug mode
    pub debug: bool,
}

/// Network configuration settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// Connection timeout in seconds
    pub connect_timeout: u64,

    /// Read timeout in seconds
    pub read_timeout: u64,

    /// Maximum number of redirects to follow
    pub max_redirects: usize,

    /// Maximum number of retry attempts
    pub max_retries: usize,

    /// Retry delay in milliseconds
    pub retry_delay: u64,

    /// Maximum number of concurrent chunks per download
    pub max_concurrent_chunks: usize,

    /// Chunk size in bytes
    pub chunk_size: usize,

    /// Enable HTTP/2
    pub http2: bool,

    /// Enable HTTP/3 (QUIC)
    pub http3: bool,

    /// Proxy settings
    pub proxy: Option<ProxyConfig>,
}

/// Cache configuration settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// Enable caching
    pub enabled: bool,

    /// Cache directory
    pub cache_dir: PathBuf,

    /// Maximum cache size in bytes
    pub max_size: u64,

    /// Cache TTL in seconds
    pub ttl: u64,

    /// Enable compression for cached files
    pub compression: bool,

    /// Cache cleanup interval in seconds
    pub cleanup_interval: u64,
}

/// Source configurations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourcesConfig {
    /// GitHub configuration
    pub github: GitHubConfig,

    /// jsDelivr configuration
    pub jsdelivr: JsDelivrConfig,

    /// Fastly configuration
    pub fastly: FastlyConfig,

    /// Cloudflare configuration
    pub cloudflare: CloudflareConfig,

    /// Custom sources
    pub custom: Vec<CustomSourceConfig>,
}

/// GitHub source configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubConfig {
    /// Enable GitHub source
    pub enabled: bool,

    /// GitHub API token (optional)
    pub token: Option<String>,

    /// API base URL
    pub api_base_url: String,

    /// Rate limit per hour
    pub rate_limit: usize,

    /// Priority (lower is higher priority)
    pub priority: u8,
}

/// jsDelivr source configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsDelivrConfig {
    /// Enable jsDelivr source
    pub enabled: bool,

    /// Base URL
    pub base_url: String,

    /// Priority (lower is higher priority)
    pub priority: u8,
}

/// Fastly source configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FastlyConfig {
    /// Enable Fastly source
    pub enabled: bool,

    /// Base URL
    pub base_url: String,

    /// Priority (lower is higher priority)
    pub priority: u8,
}

/// Cloudflare source configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudflareConfig {
    /// Enable Cloudflare source
    pub enabled: bool,

    /// Base URL
    pub base_url: String,

    /// Priority (lower is higher priority)
    pub priority: u8,
}

/// Custom source configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomSourceConfig {
    /// Source name
    pub name: String,

    /// Enable this source
    pub enabled: bool,

    /// Base URL
    pub base_url: String,

    /// Priority (lower is higher priority)
    pub priority: u8,

    /// Custom headers
    pub headers: HashMap<String, String>,
}

/// Proxy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyConfig {
    /// Proxy URL
    pub url: String,

    /// Username for proxy authentication
    pub username: Option<String>,

    /// Password for proxy authentication
    pub password: Option<String>,
}

/// Compliance configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceConfig {
    /// Enable strict compliance checking
    pub strict_mode: bool,

    /// Verify open source licenses
    pub verify_open_source: bool,

    /// Check copyright status
    pub check_copyright: bool,

    /// Validate source legitimacy
    pub validate_source: bool,

    /// Enable audit logging
    pub audit_logging: bool,

    /// Audit log file path
    pub audit_log_path: PathBuf,

    /// Data protection settings
    pub data_protection: DataProtectionConfig,
}

/// Data protection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataProtectionConfig {
    /// Minimize data collection
    pub minimal_data_collection: bool,

    /// Require user consent
    pub user_consent_required: bool,

    /// Data retention period in days
    pub data_retention_days: u32,

    /// Enable data anonymization
    pub anonymize_data: bool,
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Log level
    pub level: String,

    /// Log file path
    pub file_path: Option<PathBuf>,

    /// Enable console logging
    pub console: bool,

    /// Log format
    pub format: LogFormat,
}

/// Metrics configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsConfig {
    /// Enable metrics collection
    pub enabled: bool,

    /// Metrics export interval in seconds
    pub export_interval: u64,

    /// Metrics storage path
    pub storage_path: PathBuf,
}

/// Geographic regions for CDN optimization
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Region {
    /// North America
    NorthAmerica,
    /// Europe
    Europe,
    /// Asia Pacific
    AsiaPacific,
    /// China
    China,
    /// Global (auto-detect)
    Global,
}

/// Log format options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogFormat {
    /// Human-readable format
    Human,
    /// JSON format
    Json,
    /// Compact format
    Compact,
}

impl Default for TurboCdnConfig {
    fn default() -> Self {
        Self {
            general: GeneralConfig::default(),
            network: NetworkConfig::default(),
            cache: CacheConfig::default(),
            sources: SourcesConfig::default(),
            compliance: ComplianceConfig::default(),
            logging: LoggingConfig::default(),
            metrics: MetricsConfig::default(),
        }
    }
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            user_agent: format!("turbo-cdn/{}", env!("CARGO_PKG_VERSION")),
            download_dir: dirs::download_dir().unwrap_or_else(|| PathBuf::from("./downloads")),
            max_concurrent_downloads: 4,
            default_region: Region::Global,
            debug: false,
        }
    }
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            connect_timeout: 30,
            read_timeout: 60,
            max_redirects: 10,
            max_retries: 3,
            retry_delay: 1000,
            max_concurrent_chunks: 8,
            chunk_size: 1024 * 1024, // 1MB
            http2: true,
            http3: false,
            proxy: None,
        }
    }
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            cache_dir: dirs::cache_dir()
                .unwrap_or_else(|| PathBuf::from("./cache"))
                .join("turbo-cdn"),
            max_size: 1024 * 1024 * 1024, // 1GB
            ttl: 24 * 60 * 60,            // 24 hours
            compression: true,
            cleanup_interval: 60 * 60, // 1 hour
        }
    }
}

impl Default for SourcesConfig {
    fn default() -> Self {
        Self {
            github: GitHubConfig::default(),
            jsdelivr: JsDelivrConfig::default(),
            fastly: FastlyConfig::default(),
            cloudflare: CloudflareConfig::default(),
            custom: Vec::new(),
        }
    }
}

impl Default for GitHubConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            token: None,
            api_base_url: "https://api.github.com".to_string(),
            rate_limit: 5000,
            priority: 1,
        }
    }
}

impl Default for JsDelivrConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            base_url: "https://cdn.jsdelivr.net".to_string(),
            priority: 2,
        }
    }
}

impl Default for FastlyConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            base_url: "https://fastly.jsdelivr.net".to_string(),
            priority: 3,
        }
    }
}

impl Default for CloudflareConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            base_url: "https://cdnjs.cloudflare.com".to_string(),
            priority: 4,
        }
    }
}

impl Default for ComplianceConfig {
    fn default() -> Self {
        Self {
            strict_mode: true,
            verify_open_source: true,
            check_copyright: true,
            validate_source: true,
            audit_logging: true,
            audit_log_path: PathBuf::from("./audit.log"),
            data_protection: DataProtectionConfig::default(),
        }
    }
}

impl Default for DataProtectionConfig {
    fn default() -> Self {
        Self {
            minimal_data_collection: true,
            user_consent_required: true,
            data_retention_days: 30,
            anonymize_data: true,
        }
    }
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            file_path: None,
            console: true,
            format: LogFormat::Human,
        }
    }
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            export_interval: 60,
            storage_path: PathBuf::from("./metrics"),
        }
    }
}

impl TurboCdnConfig {
    /// Load configuration from file
    pub fn from_file<P: AsRef<std::path::Path>>(path: P) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| TurboCdnError::config(format!("Failed to read config file: {}", e)))?;

        let config: Self = toml::from_str(&content)
            .map_err(|e| TurboCdnError::config(format!("Failed to parse config file: {}", e)))?;

        config.validate()?;
        Ok(config)
    }

    /// Save configuration to file
    pub fn to_file<P: AsRef<std::path::Path>>(&self, path: P) -> Result<()> {
        let content = toml::to_string_pretty(self)
            .map_err(|e| TurboCdnError::config(format!("Failed to serialize config: {}", e)))?;

        std::fs::write(path, content)
            .map_err(|e| TurboCdnError::config(format!("Failed to write config file: {}", e)))?;

        Ok(())
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<()> {
        // Validate timeouts
        if self.network.connect_timeout == 0 {
            return Err(TurboCdnError::config(
                "Connect timeout must be greater than 0",
            ));
        }

        if self.network.read_timeout == 0 {
            return Err(TurboCdnError::config("Read timeout must be greater than 0"));
        }

        // Validate chunk settings
        if self.network.max_concurrent_chunks == 0 {
            return Err(TurboCdnError::config(
                "Max concurrent chunks must be greater than 0",
            ));
        }

        if self.network.chunk_size == 0 {
            return Err(TurboCdnError::config("Chunk size must be greater than 0"));
        }

        // Validate cache settings
        if self.cache.enabled && self.cache.max_size == 0 {
            return Err(TurboCdnError::config(
                "Cache max size must be greater than 0 when caching is enabled",
            ));
        }

        // Validate at least one source is enabled
        let sources_enabled = self.sources.github.enabled
            || self.sources.jsdelivr.enabled
            || self.sources.fastly.enabled
            || self.sources.cloudflare.enabled
            || self.sources.custom.iter().any(|s| s.enabled);

        if !sources_enabled {
            return Err(TurboCdnError::config(
                "At least one download source must be enabled",
            ));
        }

        Ok(())
    }

    /// Get connect timeout as Duration
    pub fn connect_timeout(&self) -> Duration {
        Duration::from_secs(self.network.connect_timeout)
    }

    /// Get read timeout as Duration
    pub fn read_timeout(&self) -> Duration {
        Duration::from_secs(self.network.read_timeout)
    }

    /// Get retry delay as Duration
    pub fn retry_delay(&self) -> Duration {
        Duration::from_millis(self.network.retry_delay)
    }
}
