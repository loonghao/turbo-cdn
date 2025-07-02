// Licensed under the MIT License
// Copyright (c) 2025 Hal <hal.long@outlook.com>

//! # Turbo CDN
//!
//! Intelligent download accelerator with automatic CDN optimization and concurrent chunked downloads.
//!
//! ## Features
//!
//! - **Automatic CDN Optimization**: GitHub, jsDelivr, Fastly, Cloudflare mirrors
//! - **Concurrent Downloads**: Multi-threaded chunked downloads with range requests
//! - **Smart URL Mapping**: Regex-based URL pattern matching for CDN selection
//! - **Resume Support**: Automatic resume for interrupted downloads
//! - **Geographic Awareness**: Location-based CDN selection for optimal speed
//! - **Progress Tracking**: Real-time download progress and speed monitoring
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use turbo_cdn::*;
//!
//! #[tokio::main]
//! async fn main() -> turbo_cdn::Result<()> {
//!     // Simple download with automatic CDN optimization
//!     let downloader = TurboCdn::new().await?;
//!     
//!     let result = downloader.download_from_url(
//!         "https://github.com/BurntSushi/ripgrep/releases/download/14.1.1/ripgrep-14.1.1-x86_64-pc-windows-msvc.zip"
//!     ).await?;
//!     
//!     println!("Downloaded {} bytes to: {}", result.size, result.path.display());
//!     println!("Speed: {:.2} MB/s", result.speed / 1024.0 / 1024.0);
//!     Ok(())
//! }
//! ```

pub mod adaptive_concurrency;
pub mod adaptive_speed_controller;
pub mod cdn_quality;
pub mod cli_progress;
pub mod concurrent_downloader;
pub mod config;
pub mod dns_cache;
pub mod error;
pub mod geo_detection;
pub mod http_client;
pub mod http_client_manager;
pub mod load_balancer;
pub mod logging;
pub mod mmap_writer;
pub mod progress;
pub mod server_quality_scorer;
pub mod server_tracker;
pub mod smart_chunking;
pub mod smart_downloader;
pub mod url_mapper;

// Note: Imports will be added as needed

// Re-export commonly used types
pub use concurrent_downloader::{ConcurrentDownloader, DownloadResult};
pub use config::{Region, TurboCdnConfig};
pub use error::{Result, TurboCdnError};
pub use progress::{ConsoleProgressReporter, ProgressCallback, ProgressInfo, ProgressTracker};
pub use url_mapper::UrlMapper;

// Internal imports
use std::sync::{Arc, Mutex};
use tracing::{info, warn};

/// Download options for customizing download behavior
#[derive(Default)]
pub struct DownloadOptions {
    /// Progress callback function
    pub progress_callback: Option<ProgressCallback>,
    /// Maximum number of concurrent chunks
    pub max_concurrent_chunks: Option<usize>,
    /// Chunk size for downloads
    pub chunk_size: Option<u64>,
    /// Enable resume for interrupted downloads
    pub enable_resume: bool,
    /// Custom headers to include in requests
    pub custom_headers: Option<std::collections::HashMap<String, String>>,
    /// Override timeout for this specific download
    pub timeout_override: Option<std::time::Duration>,
    /// Verify file integrity after download
    pub verify_integrity: bool,
    /// Expected file size (for progress calculation)
    pub expected_size: Option<u64>,
}

impl DownloadOptions {
    /// Create new download options with defaults
    pub fn new() -> Self {
        Self::default()
    }

    /// Set progress callback
    pub fn with_progress_callback(mut self, callback: ProgressCallback) -> Self {
        self.progress_callback = Some(callback);
        self
    }

    /// Set maximum concurrent chunks
    pub fn with_max_concurrent_chunks(mut self, chunks: usize) -> Self {
        self.max_concurrent_chunks = Some(chunks);
        self
    }

    /// Set chunk size
    pub fn with_chunk_size(mut self, size: u64) -> Self {
        self.chunk_size = Some(size);
        self
    }

    /// Enable resume support
    pub fn with_resume(mut self, enable: bool) -> Self {
        self.enable_resume = enable;
        self
    }

    /// Add custom header
    pub fn with_header<K: Into<String>, V: Into<String>>(mut self, key: K, value: V) -> Self {
        if self.custom_headers.is_none() {
            self.custom_headers = Some(std::collections::HashMap::new());
        }
        self.custom_headers
            .as_mut()
            .unwrap()
            .insert(key.into(), value.into());
        self
    }

    /// Set timeout override
    pub fn with_timeout(mut self, timeout: std::time::Duration) -> Self {
        self.timeout_override = Some(timeout);
        self
    }

    /// Enable integrity verification
    pub fn with_integrity_verification(mut self, enable: bool) -> Self {
        self.verify_integrity = enable;
        self
    }

    /// Set expected file size
    pub fn with_expected_size(mut self, size: u64) -> Self {
        self.expected_size = Some(size);
        self
    }
}

impl std::fmt::Debug for DownloadOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DownloadOptions")
            .field(
                "progress_callback",
                &self.progress_callback.as_ref().map(|_| "<callback>"),
            )
            .field("max_concurrent_chunks", &self.max_concurrent_chunks)
            .field("chunk_size", &self.chunk_size)
            .field("enable_resume", &self.enable_resume)
            .finish()
    }
}

impl Clone for DownloadOptions {
    fn clone(&self) -> Self {
        Self {
            progress_callback: None, // Cannot clone function pointers
            max_concurrent_chunks: self.max_concurrent_chunks,
            chunk_size: self.chunk_size,
            enable_resume: self.enable_resume,
            custom_headers: self.custom_headers.clone(),
            timeout_override: self.timeout_override,
            verify_integrity: self.verify_integrity,
            expected_size: self.expected_size,
        }
    }
}

/// Main TurboCdn client - simplified download accelerator
#[derive(Debug)]
pub struct TurboCdn {
    url_mapper: Arc<Mutex<UrlMapper>>,
    downloader: ConcurrentDownloader,
    #[allow(dead_code)]
    progress_tracker: Option<std::sync::Arc<ProgressTracker>>,
}

impl TurboCdn {
    /// Create a TurboCdn client with default configuration
    pub async fn new() -> Result<Self> {
        let config = TurboCdnConfig::load().unwrap_or_default();
        Self::with_config(config).await
    }

    /// Create a TurboCdn client with custom configuration
    pub async fn with_config(config: TurboCdnConfig) -> Result<Self> {
        // Auto-detect region if enabled
        let region = if config.geo_detection.auto_detect_region {
            let mut geo_detector = crate::geo_detection::GeoDetector::new(config.clone());
            match geo_detector.detect_region().await {
                Ok(detected_region) => {
                    info!("Auto-detected region: {:?}", detected_region);
                    detected_region
                }
                Err(e) => {
                    warn!("Failed to auto-detect region: {}, using default", e);
                    config.general.default_region.clone()
                }
            }
        } else {
            config.general.default_region.clone()
        };

        let url_mapper = UrlMapper::new(&config, region)?;
        let downloader = ConcurrentDownloader::with_config(&config)?;

        Ok(Self {
            url_mapper: Arc::new(Mutex::new(url_mapper)),
            downloader,
            progress_tracker: None, // Will be created per download
        })
    }

    /// Download from any supported URL with automatic CDN optimization
    ///
    /// This is the main download method that provides automatic CDN optimization
    /// and concurrent chunked downloads for maximum speed.
    ///
    /// # Arguments
    /// * `url` - The source URL to download from
    ///
    /// # Returns
    /// * `DownloadResult` containing download information
    ///
    /// # Example
    /// ```rust,no_run
    /// use turbo_cdn::*;
    ///
    /// #[tokio::main]
    /// async fn main() -> turbo_cdn::Result<()> {
    ///     let downloader = TurboCdn::new().await?;
    ///     
    ///     let result = downloader.download_from_url(
    ///         "https://github.com/BurntSushi/ripgrep/releases/download/14.1.1/ripgrep-14.1.1-x86_64-pc-windows-msvc.zip"
    ///     ).await?;
    ///     
    ///     println!("Downloaded {} bytes to: {}", result.size, result.path.display());
    ///     Ok(())
    /// }
    /// ```
    pub async fn download_from_url(&self, url: &str) -> Result<DownloadResult> {
        // Map URL to optimal CDN alternatives
        let urls = self.url_mapper.lock().unwrap().map_url(url)?;

        // Generate output filename from URL
        let filename = self.extract_filename_from_url(url)?;
        let output_path = std::env::temp_dir().join(&filename);

        // Download with concurrent downloader
        self.downloader.download(&urls, &output_path, None).await
    }

    /// Download from URL to specific path
    pub async fn download_to_path<P: AsRef<std::path::Path>>(
        &self,
        url: &str,
        output_path: P,
    ) -> Result<DownloadResult> {
        let urls = self.url_mapper.lock().unwrap().map_url(url)?;
        self.downloader.download(&urls, output_path, None).await
    }

    /// Download directly from original URL without CDN optimization
    pub async fn download_direct_from_url(&self, url: &str) -> Result<DownloadResult> {
        // Use only the original URL, no CDN mapping
        let urls = vec![url.to_string()];

        // Generate output filename from URL
        let filename = self.extract_filename_from_url(url)?;
        let output_path = std::env::temp_dir().join(&filename);

        // Download with concurrent downloader
        self.downloader.download(&urls, &output_path, None).await
    }

    /// Download directly from URL to specific path without CDN optimization
    pub async fn download_direct_to_path<P: AsRef<std::path::Path>>(
        &self,
        url: &str,
        output_path: P,
    ) -> Result<DownloadResult> {
        // Use only the original URL, no CDN mapping
        let urls = vec![url.to_string()];
        self.downloader.download(&urls, output_path, None).await
    }

    /// Smart download that automatically selects the fastest method
    pub async fn download_smart(&self, url: &str) -> Result<DownloadResult> {
        self.download_smart_with_verbose(url, false).await
    }

    /// Smart download with verbose control
    pub async fn download_smart_with_verbose(
        &self,
        url: &str,
        verbose: bool,
    ) -> Result<DownloadResult> {
        use crate::smart_downloader::SmartDownloader;

        let smart_downloader = SmartDownloader::new_with_verbose(verbose).await?;
        smart_downloader.download_smart(url).await
    }

    /// Smart download to specific path with automatic method selection
    pub async fn download_smart_to_path<P: AsRef<std::path::Path>>(
        &self,
        url: &str,
        output_path: P,
    ) -> Result<DownloadResult> {
        self.download_smart_to_path_with_verbose(url, output_path, false)
            .await
    }

    /// Smart download to specific path with verbose control
    pub async fn download_smart_to_path_with_verbose<P: AsRef<std::path::Path>>(
        &self,
        url: &str,
        output_path: P,
        verbose: bool,
    ) -> Result<DownloadResult> {
        use crate::smart_downloader::SmartDownloader;

        let smart_downloader = SmartDownloader::new_with_verbose(verbose).await?;
        smart_downloader
            .download_smart_to_path(url, output_path)
            .await
    }

    /// Download with custom options
    pub async fn download_with_options<P: AsRef<std::path::Path>>(
        &self,
        url: &str,
        output_path: P,
        options: DownloadOptions,
    ) -> Result<DownloadResult> {
        let urls = self.url_mapper.lock().unwrap().map_url(url)?;

        // Create progress tracker if callback is provided
        let progress_tracker = if options.progress_callback.is_some() {
            let expected_size = options.expected_size.unwrap_or(0);
            Some(std::sync::Arc::new(ProgressTracker::new(expected_size)))
        } else {
            None
        };

        self.downloader
            .download(&urls, output_path, progress_tracker)
            .await
    }

    /// Get optimal CDN URL without downloading
    pub async fn get_optimal_url(&self, url: &str) -> Result<String> {
        let urls = self.url_mapper.lock().unwrap().map_url(url)?;
        Ok(urls.into_iter().next().unwrap_or_else(|| url.to_string()))
    }

    /// Get all available CDN URLs for a given URL
    pub async fn get_all_cdn_urls(&self, url: &str) -> Result<Vec<String>> {
        self.url_mapper.lock().unwrap().map_url(url)
    }

    /// Check if a URL can be optimized
    pub fn can_optimize_url(&self, url: &str) -> bool {
        self.url_mapper
            .lock()
            .unwrap()
            .map_url(url)
            .map(|urls| urls.len() > 1)
            .unwrap_or(false)
    }

    /// Extract filename from URL
    fn extract_filename_from_url(&self, url: &str) -> Result<String> {
        let url_obj =
            url::Url::parse(url).map_err(|e| TurboCdnError::config(format!("Invalid URL: {e}")))?;

        let path = url_obj.path();
        let filename = path.split('/').next_back().unwrap_or("download");

        if filename.is_empty() || filename == "/" {
            Ok("download".to_string())
        } else {
            Ok(filename.to_string())
        }
    }
}

/// Statistics for TurboCdn
#[derive(Debug, Clone, Default)]
pub struct TurboCdnStats {
    /// Total downloads
    pub total_downloads: u64,

    /// Successful downloads
    pub successful_downloads: u64,

    /// Failed downloads
    pub failed_downloads: u64,

    /// Total bytes downloaded
    pub total_bytes: u64,

    /// Cache hit rate
    pub cache_hit_rate: f64,

    /// Average download speed in bytes per second
    pub average_speed: f64,
}

/// Synchronous API for blocking operations
pub mod sync_api {
    use super::*;
    use std::path::Path;

    /// Synchronous wrapper for TurboCdn
    ///
    /// Provides blocking APIs for environments that don't use async/await.
    /// Internally uses a Tokio runtime to execute async operations.
    #[derive(Debug)]
    pub struct SyncTurboCdn {
        runtime: tokio::runtime::Runtime,
        inner: TurboCdn,
    }

    impl SyncTurboCdn {
        /// Create a new synchronous TurboCdn client
        pub fn new() -> Result<Self> {
            let runtime = tokio::runtime::Runtime::new()
                .map_err(|e| TurboCdnError::internal(format!("Failed to create runtime: {e}")))?;

            let inner = runtime.block_on(TurboCdn::new())?;

            Ok(Self { runtime, inner })
        }

        /// Create a synchronous TurboCdn client with custom configuration
        pub fn with_config(config: TurboCdnConfig) -> Result<Self> {
            let runtime = tokio::runtime::Runtime::new()
                .map_err(|e| TurboCdnError::internal(format!("Failed to create runtime: {e}")))?;

            let inner = runtime.block_on(TurboCdn::with_config(config))?;

            Ok(Self { runtime, inner })
        }

        /// Download from URL (blocking)
        pub fn download_from_url(&self, url: &str) -> Result<DownloadResult> {
            self.runtime.block_on(self.inner.download_from_url(url))
        }

        /// Download to specific path (blocking)
        pub fn download_to_path<P: AsRef<Path>>(
            &self,
            url: &str,
            output_path: P,
        ) -> Result<DownloadResult> {
            self.runtime
                .block_on(self.inner.download_to_path(url, output_path))
        }

        /// Get optimal CDN URL (blocking)
        pub fn get_optimal_url(&self, url: &str) -> Result<String> {
            self.runtime.block_on(self.inner.get_optimal_url(url))
        }
    }

    /// Quick synchronous functions for simple use cases
    pub mod quick {
        use super::*;

        /// Quick download from URL with default settings (blocking)
        pub fn download_url(url: &str) -> Result<DownloadResult> {
            let client = SyncTurboCdn::new()?;
            client.download_from_url(url)
        }

        /// Quick URL optimization (blocking)
        pub fn optimize_url(url: &str) -> Result<String> {
            let client = SyncTurboCdn::new()?;
            client.get_optimal_url(url)
        }

        /// Quick download to specific path (blocking)
        pub fn download_to_path<P: AsRef<Path>>(
            url: &str,
            output_path: P,
        ) -> Result<DownloadResult> {
            let client = SyncTurboCdn::new()?;
            client.download_to_path(url, output_path)
        }
    }
}

/// Async API module for external integrations (like vx)
pub mod async_api {
    use super::*;
    use std::sync::Arc;
    use tokio::sync::Mutex;

    /// Async wrapper for TurboCdn that provides thread-safe access
    #[derive(Debug, Clone)]
    pub struct AsyncTurboCdn {
        inner: Arc<Mutex<TurboCdn>>,
    }

    impl AsyncTurboCdn {
        /// Create a new AsyncTurboCdn instance
        pub async fn new() -> Result<Self> {
            let turbo_cdn = TurboCdn::new().await?;
            Ok(Self {
                inner: Arc::new(Mutex::new(turbo_cdn)),
            })
        }

        /// Create a new AsyncTurboCdn instance with custom configuration
        pub async fn with_config(config: TurboCdnConfig) -> Result<Self> {
            let turbo_cdn = TurboCdn::with_config(config).await?;
            Ok(Self {
                inner: Arc::new(Mutex::new(turbo_cdn)),
            })
        }

        /// Download from any supported URL with automatic optimization (async version)
        pub async fn download_from_url_async(&self, url: &str) -> Result<DownloadResult> {
            let client = self.inner.lock().await;
            client.download_from_url(url).await
        }

        /// Get optimal CDN URL without downloading (async version)
        pub async fn get_optimal_url_async(&self, url: &str) -> Result<String> {
            let client = self.inner.lock().await;
            client.get_optimal_url(url).await
        }

        /// Download to specific path (async version)
        pub async fn download_to_path_async<P: AsRef<std::path::Path>>(
            &self,
            url: &str,
            output_path: P,
        ) -> Result<DownloadResult> {
            let client = self.inner.lock().await;
            client.download_to_path(url, output_path).await
        }
    }

    /// Convenience functions for quick async operations
    pub mod quick {
        use super::*;

        /// Quick download from URL with default settings
        pub async fn download_url(url: &str) -> Result<DownloadResult> {
            let client = AsyncTurboCdn::new().await?;
            client.download_from_url_async(url).await
        }

        /// Quick URL optimization
        pub async fn optimize_url(url: &str) -> Result<String> {
            let client = AsyncTurboCdn::new().await?;
            client.get_optimal_url_async(url).await
        }

        /// Quick download to specific path
        pub async fn download_url_to_path<P: AsRef<std::path::Path>>(
            url: &str,
            output_path: P,
        ) -> Result<DownloadResult> {
            let client = AsyncTurboCdn::new().await?;
            client.download_to_path_async(url, output_path).await
        }
    }
}

/// Initialize tracing for the library (deprecated - use logging module)
#[deprecated(note = "Use logging::init_api_logging() instead")]
pub fn init_tracing() {
    let _ = logging::init_api_logging();
}

/// Initialize tracing with specific level (deprecated - use logging module)
#[deprecated(note = "Use logging module functions instead")]
pub fn init_tracing_with_level(level: &str) {
    let config = logging::LoggingConfig {
        level: level.to_string(),
        ..logging::LoggingConfig::api()
    };
    let _ = logging::init_logging(config);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_turbo_cdn_new() {
        let result = TurboCdn::new().await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_turbo_cdn_stats_creation() {
        let stats = TurboCdnStats {
            total_downloads: 100,
            successful_downloads: 95,
            failed_downloads: 5,
            total_bytes: 1024 * 1024,
            cache_hit_rate: 0.8,
            average_speed: 1000.0,
        };

        assert_eq!(stats.total_downloads, 100);
        assert_eq!(stats.successful_downloads, 95);
        assert_eq!(stats.failed_downloads, 5);
        assert_eq!(stats.total_bytes, 1024 * 1024);
        assert_eq!(stats.cache_hit_rate, 0.8);
        assert_eq!(stats.average_speed, 1000.0);
    }

    #[test]
    fn test_download_result_creation() {
        use std::path::PathBuf;
        use std::time::Duration;

        let result = DownloadResult {
            path: PathBuf::from("/tmp/file.zip"),
            size: 1024,
            duration: Duration::from_secs(1),
            speed: 1024.0,
            url: "https://github.com/owner/repo/releases/download/v1.0.0/file.zip".to_string(),
            resumed: false,
        };

        assert_eq!(result.path, PathBuf::from("/tmp/file.zip"));
        assert_eq!(result.size, 1024);
        assert_eq!(result.duration, Duration::from_secs(1));
        assert_eq!(result.speed, 1024.0);
        assert!(!result.resumed);
    }
}
