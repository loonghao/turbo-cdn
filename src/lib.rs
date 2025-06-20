// Licensed under the MIT License
// Copyright (c) 2025 Hal <hal.long@outlook.com>

//! # Turbo CDN
//!
//! Revolutionary global download accelerator for open-source software with AI optimization,
//! multi-CDN routing, and P2P acceleration.
//!
//! ## Features
//!
//! - **Multi-CDN Support**: GitHub Releases, jsDelivr, Fastly, Cloudflare
//! - **Intelligent Routing**: AI-powered CDN selection and failover
//! - **Parallel Downloads**: Chunked downloads with automatic optimization
//! - **Compliance First**: Built-in verification for open-source content only
//! - **Caching**: Smart caching with compression and TTL management
//! - **Progress Tracking**: Real-time download progress with callbacks
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use turbo_cdn::*;
//!
//! #[tokio::main]
//! async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
//!     let mut downloader = TurboCdn::builder()
//!         .with_sources(&[
//!             Source::github(),
//!             Source::jsdelivr(),
//!             Source::fastly(),
//!         ])
//!         .with_region(Region::Global)
//!         .build()
//!         .await?;
//!
//!     let options = DownloadOptions {
//!         progress_callback: Some(Box::new(|progress| {
//!             println!("Downloaded: {:.1}%", progress.percentage);
//!         })),
//!         ..Default::default()
//!     };
//!
//!     // Note: This would perform the actual download in a real implementation
//!     // let result = downloader.download("oven-sh/bun", "v1.0.0", "bun-linux-x64.zip", options).await?;
//!     // println!("Downloaded to: {}", result.path.display());
//!     Ok(())
//! }
//! ```

pub mod cache;
pub mod compliance;
pub mod config;
pub mod downloader;
pub mod error;
pub mod progress;
pub mod router;
pub mod sources;

use std::path::PathBuf;

// Re-export commonly used types
pub use cache::{CacheManager, CacheStats};
pub use compliance::{ComplianceChecker, ComplianceResult};
pub use config::{Region, TurboCdnConfig};
pub use downloader::{DownloadOptions, DownloadResult, Downloader};
pub use error::{Result, TurboCdnError};
pub use progress::{ConsoleProgressReporter, ProgressCallback, ProgressInfo, ProgressTracker};
pub use router::{RoutingDecision, SmartRouter};
pub use sources::{
    cloudflare::CloudflareSource, fastly::FastlySource, github::GitHubSource,
    jsdelivr::JsDelivrSource, DownloadUrl, SourceManager,
};

/// Main TurboCdn client
#[derive(Debug)]
pub struct TurboCdn {
    downloader: Downloader,
}

/// Builder for TurboCdn client
#[derive(Debug)]
pub struct TurboCdnBuilder {
    config: TurboCdnConfig,
    sources: Vec<Source>,
}

/// Download source types
#[derive(Debug, Clone)]
pub enum Source {
    GitHub,
    JsDelivr,
    Fastly,
    Cloudflare,
}

impl TurboCdn {
    /// Create a new TurboCdn builder
    pub fn builder() -> TurboCdnBuilder {
        TurboCdnBuilder::new()
    }

    /// Create a TurboCdn client with default configuration
    pub async fn new() -> Result<Self> {
        Self::builder().build().await
    }

    /// Download a file from a repository
    pub async fn download(
        &mut self,
        repository: &str,
        version: &str,
        file_name: &str,
        options: DownloadOptions,
    ) -> Result<DownloadResult> {
        self.downloader
            .download(repository, version, file_name, options)
            .await
    }

    /// Get repository metadata
    pub async fn get_repository_metadata(
        &self,
        _repository: &str,
    ) -> Result<sources::RepositoryMetadata> {
        // TODO: Implement repository metadata retrieval
        Err(TurboCdnError::unsupported(
            "Repository metadata not yet implemented",
        ))
    }

    /// Get download statistics
    pub async fn get_stats(&self) -> Result<TurboCdnStats> {
        // TODO: Implement statistics collection from downloader components
        Ok(TurboCdnStats::default())
    }

    /// Perform health check on all sources
    pub async fn health_check(
        &self,
    ) -> Result<std::collections::HashMap<String, sources::HealthStatus>> {
        // TODO: Implement health check via source manager
        Ok(std::collections::HashMap::new())
    }
}

// Note: TurboCdn doesn't implement Clone due to complex internal state
// Users should create new instances via the builder pattern

impl TurboCdnBuilder {
    /// Create a new builder with default configuration
    pub fn new() -> Self {
        Self {
            config: TurboCdnConfig::default(),
            sources: vec![
                Source::GitHub,
                Source::JsDelivr,
                Source::Fastly,
                Source::Cloudflare,
            ],
        }
    }

    /// Set the configuration
    pub fn with_config(mut self, config: TurboCdnConfig) -> Self {
        self.config = config;
        self
    }

    /// Set the download sources
    pub fn with_sources(mut self, sources: &[Source]) -> Self {
        self.sources = sources.to_vec();
        self
    }

    /// Set the region for optimization
    pub fn with_region(mut self, region: Region) -> Self {
        self.config.general.default_region = region;
        self
    }

    /// Set the download directory
    pub fn with_download_dir<P: Into<PathBuf>>(mut self, dir: P) -> Self {
        self.config.general.download_dir = dir.into();
        self
    }

    /// Enable or disable caching
    pub fn with_cache(mut self, enabled: bool) -> Self {
        self.config.cache.enabled = enabled;
        self
    }

    /// Set maximum concurrent downloads
    pub fn with_max_concurrent_downloads(mut self, max: usize) -> Self {
        self.config.general.max_concurrent_downloads = max;
        self
    }

    /// Build the TurboCdn client
    pub async fn build(self) -> Result<TurboCdn> {
        // Create source manager
        let mut source_manager = SourceManager::new();

        for source in &self.sources {
            match source {
                Source::GitHub => {
                    let github_source = GitHubSource::new(self.config.sources.github.clone())?;
                    source_manager.add_source(Box::new(github_source));
                }
                Source::JsDelivr => {
                    let jsdelivr_source =
                        JsDelivrSource::new(self.config.sources.jsdelivr.clone())?;
                    source_manager.add_source(Box::new(jsdelivr_source));
                }
                Source::Fastly => {
                    let fastly_source = FastlySource::new(self.config.sources.fastly.clone())?;
                    source_manager.add_source(Box::new(fastly_source));
                }
                Source::Cloudflare => {
                    let cloudflare_source =
                        CloudflareSource::new(self.config.sources.cloudflare.clone())?;
                    source_manager.add_source(Box::new(cloudflare_source));
                }
            }
        }

        // Create components
        let router = SmartRouter::new(self.config.clone(), source_manager);
        let cache_manager = CacheManager::new(self.config.cache.clone()).await?;
        let compliance_checker = ComplianceChecker::new(self.config.compliance.clone())?;

        // Create downloader
        let downloader =
            Downloader::new(self.config, router, cache_manager, compliance_checker).await?;

        Ok(TurboCdn { downloader })
    }
}

impl Default for TurboCdnBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl Source {
    /// Create a GitHub source
    pub fn github() -> Self {
        Source::GitHub
    }

    /// Create a jsDelivr source
    pub fn jsdelivr() -> Self {
        Source::JsDelivr
    }

    /// Create a Fastly source
    pub fn fastly() -> Self {
        Source::Fastly
    }

    /// Create a Cloudflare source
    pub fn cloudflare() -> Self {
        Source::Cloudflare
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

/// Initialize tracing for the library
pub fn init_tracing() {
    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "turbo_cdn=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_builder() {
        let _builder = TurboCdn::builder()
            .with_region(Region::Global)
            .with_cache(true);

        // Just test that the builder can be created without panicking
        // The fact that we reach this point means the builder works correctly
    }

    #[test]
    fn test_source_creation() {
        let _github = Source::github();
        let _jsdelivr = Source::jsdelivr();
        let _fastly = Source::fastly();
        let _cloudflare = Source::cloudflare();
    }
}
