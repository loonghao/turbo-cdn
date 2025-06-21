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
pub mod domain_manager;
pub mod downloader;
pub mod error;
pub mod geo_detection;
pub mod progress;
pub mod router;
pub mod sources;

use std::path::PathBuf;
use tracing::{info, warn};

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

/// Parsed URL information from various sources
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedUrl {
    /// Repository in format "owner/repo"
    pub repository: String,
    /// Version/tag name
    pub version: String,
    /// Filename
    pub filename: String,
    /// Original URL
    pub original_url: String,
    /// Detected source type
    pub source_type: DetectedSourceType,
}

/// Detected source type from URL parsing
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DetectedSourceType {
    /// GitHub releases
    GitHub,
    /// jsDelivr CDN
    JsDelivr,
    /// Fastly CDN (jsDelivr)
    Fastly,
    /// Cloudflare CDN
    Cloudflare,
    /// npm registry
    Npm,
    /// Python Package Index (PyPI)
    PyPI,
    /// Golang module proxy
    GoProxy,
    /// Rust crates.io
    CratesIo,
    /// Maven Central Repository
    Maven,
    /// NuGet Gallery
    NuGet,
    /// Docker Hub
    DockerHub,
    /// GitLab releases
    GitLab,
    /// Bitbucket downloads
    Bitbucket,
    /// SourceForge files
    SourceForge,
    /// Other/unknown source
    Other(String),
}

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

    /// Download from any supported URL with automatic CDN optimization
    ///
    /// This method accepts URLs from various sources and automatically optimizes them:
    ///
    /// **Supported URL formats:**
    /// - GitHub: `https://github.com/owner/repo/releases/download/tag/file.zip`
    /// - GitLab: `https://gitlab.com/owner/repo/-/releases/tag/downloads/file.zip`
    /// - Bitbucket: `https://bitbucket.org/owner/repo/downloads/file.zip`
    /// - jsDelivr: `https://cdn.jsdelivr.net/gh/owner/repo@tag/file.zip`
    /// - Fastly: `https://fastly.jsdelivr.net/gh/owner/repo@tag/file.zip`
    /// - Cloudflare: `https://cdnjs.cloudflare.com/ajax/libs/library/version/file.js`
    /// - npm: `https://registry.npmjs.org/package/-/package-version.tgz`
    /// - PyPI: `https://files.pythonhosted.org/packages/source/p/package/package-version.tar.gz`
    /// - Go Proxy: `https://proxy.golang.org/module/@v/version.zip`
    /// - Crates.io: `https://crates.io/api/v1/crates/crate/version/download`
    /// - Maven: `https://repo1.maven.org/maven2/group/artifact/version/artifact-version.jar`
    /// - NuGet: `https://api.nuget.org/v3-flatcontainer/package/version/package.version.nupkg`
    /// - Docker Hub: `https://registry-1.docker.io/v2/library/image/manifests/tag`
    /// - SourceForge: `https://downloads.sourceforge.net/project/name/file.zip`
    ///
    /// **Automatic optimization:**
    /// 1. Parses the URL to extract repository, version, and filename
    /// 2. Detects user's geographic location
    /// 3. Selects the optimal CDN based on location and performance
    /// 4. Downloads using the best available source with failover
    ///
    /// # Arguments
    /// * `url` - The source URL from any supported CDN or repository
    /// * `options` - Download options (optional, uses defaults if None)
    ///
    /// # Example
    /// ```rust,no_run
    /// use turbo_cdn::*;
    ///
    /// #[tokio::main]
    /// async fn main() -> turbo_cdn::Result<()> {
    ///     let mut downloader = TurboCdn::new().await?;
    ///
    ///     // GitHub releases URL
    ///     let result = downloader.download_from_url(
    ///         "https://github.com/oven-sh/bun/releases/download/bun-v1.2.9/bun-bun-v1.2.9.zip",
    ///         None
    ///     ).await?;
    ///
    ///     // jsDelivr URL
    ///     let result = downloader.download_from_url(
    ///         "https://cdn.jsdelivr.net/gh/microsoft/vscode@1.74.0/package.json",
    ///         None
    ///     ).await?;
    ///
    ///     println!("Downloaded to: {}", result.path.display());
    ///     Ok(())
    /// }
    /// ```
    pub async fn download_from_url(
        &mut self,
        url: &str,
        options: Option<DownloadOptions>,
    ) -> Result<DownloadResult> {
        let parsed_url = self.parse_url(url)?;
        let options = options.unwrap_or_default();

        self.download(
            &parsed_url.repository,
            &parsed_url.version,
            &parsed_url.filename,
            options,
        )
        .await
    }

    /// Get the optimal CDN URL for a given source URL without downloading
    ///
    /// This method parses the input URL and returns the best CDN URL based on:
    /// 1. User's geographic location (detected automatically)
    /// 2. CDN performance metrics
    /// 3. Source availability and reliability
    ///
    /// # Arguments
    /// * `url` - The source URL from any supported CDN or repository
    ///
    /// # Returns
    /// * `String` containing the optimal CDN URL for the user's location
    ///
    /// # Example
    /// ```rust,no_run
    /// use turbo_cdn::*;
    ///
    /// #[tokio::main]
    /// async fn main() -> turbo_cdn::Result<()> {
    ///     let downloader = TurboCdn::new().await?;
    ///
    ///     let optimal_url = downloader.get_optimal_url(
    ///         "https://github.com/oven-sh/bun/releases/download/bun-v1.2.9/bun-bun-v1.2.9.zip"
    ///     ).await?;
    ///
    ///     println!("Optimal URL: {}", optimal_url);
    ///     // Might output: https://fastly.jsdelivr.net/gh/oven-sh/bun@bun-v1.2.9/bun-bun-v1.2.9.zip
    ///     Ok(())
    /// }
    /// ```
    pub async fn get_optimal_url(&self, url: &str) -> Result<String> {
        let parsed_url = self.parse_url(url)?;

        // Use the downloader's method to get optimal URL
        self.downloader
            .get_optimal_url(
                &parsed_url.repository,
                &parsed_url.version,
                &parsed_url.filename,
            )
            .await
    }

    /// Get repository metadata
    pub async fn get_repository_metadata(
        &self,
        repository: &str,
    ) -> Result<sources::RepositoryMetadata> {
        self.downloader.get_repository_metadata(repository).await
    }

    /// Parse any supported URL into components (public for testing)
    ///
    /// Supports URLs from various sources:
    /// - GitHub: `https://github.com/{owner}/{repo}/releases/download/{tag}/{filename}`
    /// - GitLab: `https://gitlab.com/{owner}/{repo}/-/releases/{tag}/downloads/{filename}`
    /// - Bitbucket: `https://bitbucket.org/{owner}/{repo}/downloads/{filename}`
    /// - jsDelivr: `https://cdn.jsdelivr.net/gh/{owner}/{repo}@{tag}/{filename}`
    /// - Fastly: `https://fastly.jsdelivr.net/gh/{owner}/{repo}@{tag}/{filename}`
    /// - Cloudflare: `https://cdnjs.cloudflare.com/ajax/libs/{library}/{version}/{filename}`
    /// - npm: `https://registry.npmjs.org/{package}/-/{package}-{version}.tgz`
    /// - PyPI: `https://files.pythonhosted.org/packages/source/{first_letter}/{package}/{package}-{version}.tar.gz`
    /// - Go Proxy: `https://proxy.golang.org/{module}/@v/{version}.zip`
    /// - Crates.io: `https://crates.io/api/v1/crates/{crate}/{version}/download`
    /// - Maven: `https://repo1.maven.org/maven2/{group_path}/{artifact}/{version}/{artifact}-{version}.jar`
    /// - NuGet: `https://api.nuget.org/v3-flatcontainer/{package}/{version}/{package}.{version}.nupkg`
    /// - Docker Hub: `https://registry-1.docker.io/v2/library/{image}/manifests/{tag}`
    /// - SourceForge: `https://downloads.sourceforge.net/project/{project}/{filename}`
    ///
    /// # Arguments
    /// * `url` - The URL to parse
    ///
    /// # Returns
    /// * `ParsedUrl` containing repository, version, filename, and detected source type
    ///
    /// # Errors
    /// * Returns error if URL format is invalid or unsupported
    pub fn parse_url(&self, url: &str) -> Result<ParsedUrl> {
        let url_obj = url::Url::parse(url)
            .map_err(|e| TurboCdnError::config(format!("Invalid URL format: {}", e)))?;

        let host = url_obj
            .host_str()
            .ok_or_else(|| TurboCdnError::config("URL must have a valid host".to_string()))?;

        match host {
            "github.com" => self.parse_github_url(&url_obj, url),
            "gitlab.com" => self.parse_gitlab_url(&url_obj, url),
            "bitbucket.org" => self.parse_bitbucket_url(&url_obj, url),
            "cdn.jsdelivr.net" => self.parse_jsdelivr_url(&url_obj, url),
            "fastly.jsdelivr.net" => self.parse_fastly_url(&url_obj, url),
            "cdnjs.cloudflare.com" => self.parse_cloudflare_url(&url_obj, url),
            "registry.npmjs.org" => self.parse_npm_url(&url_obj, url),
            "files.pythonhosted.org" => self.parse_pypi_url(&url_obj, url),
            "proxy.golang.org" => self.parse_go_proxy_url(&url_obj, url),
            "crates.io" => self.parse_crates_io_url(&url_obj, url),
            "repo1.maven.org" => self.parse_maven_url(&url_obj, url),
            "api.nuget.org" => self.parse_nuget_url(&url_obj, url),
            "registry-1.docker.io" => self.parse_docker_hub_url(&url_obj, url),
            "downloads.sourceforge.net" => self.parse_sourceforge_url(&url_obj, url),
            _ => Err(TurboCdnError::config(format!(
                "Unsupported URL host: {}. Supported hosts: github.com, gitlab.com, bitbucket.org, cdn.jsdelivr.net, fastly.jsdelivr.net, cdnjs.cloudflare.com, registry.npmjs.org, files.pythonhosted.org, proxy.golang.org, crates.io, repo1.maven.org, api.nuget.org, registry-1.docker.io, downloads.sourceforge.net",
                host
            ))),
        }
    }

    /// Parse GitHub releases URL
    fn parse_github_url(&self, url_obj: &url::Url, original_url: &str) -> Result<ParsedUrl> {
        let path = url_obj.path();
        let path_segments: Vec<&str> = path.trim_start_matches('/').split('/').collect();

        // Expected format: {owner}/{repo}/releases/download/{tag}/{filename}
        if path_segments.len() < 6 {
            return Err(TurboCdnError::config(
                "Invalid GitHub releases URL format. Expected: https://github.com/{owner}/{repo}/releases/download/{tag}/{filename}".to_string(),
            ));
        }

        if path_segments[2] != "releases" || path_segments[3] != "download" {
            return Err(TurboCdnError::config(
                "URL must be a GitHub releases download URL".to_string(),
            ));
        }

        let owner = path_segments[0];
        let repo = path_segments[1];
        let tag = path_segments[4];
        let filename = path_segments[5..].join("/");

        self.validate_components(owner, repo, tag, &filename)?;

        Ok(ParsedUrl {
            repository: format!("{}/{}", owner, repo),
            version: tag.to_string(),
            filename,
            original_url: original_url.to_string(),
            source_type: DetectedSourceType::GitHub,
        })
    }

    /// Parse GitLab releases URL
    fn parse_gitlab_url(&self, url_obj: &url::Url, original_url: &str) -> Result<ParsedUrl> {
        let path = url_obj.path();
        let path_segments: Vec<&str> = path.trim_start_matches('/').split('/').collect();

        // Expected format: {owner}/{repo}/-/releases/{tag}/downloads/{filename}
        if path_segments.len() < 7 {
            return Err(TurboCdnError::config(
                "Invalid GitLab releases URL format. Expected: https://gitlab.com/{owner}/{repo}/-/releases/{tag}/downloads/{filename}".to_string(),
            ));
        }

        if path_segments[2] != "-"
            || path_segments[3] != "releases"
            || path_segments[5] != "downloads"
        {
            return Err(TurboCdnError::config(
                "URL must be a GitLab releases download URL".to_string(),
            ));
        }

        let owner = path_segments[0];
        let repo = path_segments[1];
        let tag = path_segments[4];
        let filename = path_segments[6..].join("/");

        self.validate_components(owner, repo, tag, &filename)?;

        Ok(ParsedUrl {
            repository: format!("{}/{}", owner, repo),
            version: tag.to_string(),
            filename,
            original_url: original_url.to_string(),
            source_type: DetectedSourceType::GitLab,
        })
    }

    /// Parse Bitbucket downloads URL
    fn parse_bitbucket_url(&self, url_obj: &url::Url, original_url: &str) -> Result<ParsedUrl> {
        let path = url_obj.path();
        let path_segments: Vec<&str> = path.trim_start_matches('/').split('/').collect();

        // Expected format: {owner}/{repo}/downloads/{filename}
        if path_segments.len() < 4 {
            return Err(TurboCdnError::config(
                "Invalid Bitbucket downloads URL format. Expected: https://bitbucket.org/{owner}/{repo}/downloads/{filename}".to_string(),
            ));
        }

        if path_segments[2] != "downloads" {
            return Err(TurboCdnError::config(
                "URL must be a Bitbucket downloads URL".to_string(),
            ));
        }

        let owner = path_segments[0];
        let repo = path_segments[1];
        let filename = path_segments[3..].join("/");

        // For Bitbucket, we'll extract version from filename if possible
        let version = self
            .extract_version_from_filename(&filename)
            .unwrap_or_else(|| "latest".to_string());

        self.validate_components(owner, repo, &version, &filename)?;

        Ok(ParsedUrl {
            repository: format!("{}/{}", owner, repo),
            version,
            filename,
            original_url: original_url.to_string(),
            source_type: DetectedSourceType::Bitbucket,
        })
    }

    /// Parse jsDelivr CDN URL
    fn parse_jsdelivr_url(&self, url_obj: &url::Url, original_url: &str) -> Result<ParsedUrl> {
        let path = url_obj.path();
        let path_segments: Vec<&str> = path.trim_start_matches('/').split('/').collect();

        // Expected format: gh/{owner}/{repo}@{tag}/{filename}
        if path_segments.len() < 4 || path_segments[0] != "gh" {
            return Err(TurboCdnError::config(
                "Invalid jsDelivr URL format. Expected: https://cdn.jsdelivr.net/gh/{owner}/{repo}@{tag}/{filename}".to_string(),
            ));
        }

        let owner = path_segments[1];
        let repo_and_tag = path_segments[2];
        let filename = path_segments[3..].join("/");

        // Parse repo@tag format
        let (repo, tag) = if let Some(at_pos) = repo_and_tag.find('@') {
            let repo = &repo_and_tag[..at_pos];
            let tag = &repo_and_tag[at_pos + 1..];
            (repo, tag)
        } else {
            return Err(TurboCdnError::config(
                "Invalid jsDelivr URL: missing @tag in repository specification".to_string(),
            ));
        };

        self.validate_components(owner, repo, tag, &filename)?;

        Ok(ParsedUrl {
            repository: format!("{}/{}", owner, repo),
            version: tag.to_string(),
            filename,
            original_url: original_url.to_string(),
            source_type: DetectedSourceType::JsDelivr,
        })
    }

    /// Parse Fastly CDN URL (same format as jsDelivr)
    fn parse_fastly_url(&self, url_obj: &url::Url, original_url: &str) -> Result<ParsedUrl> {
        let path = url_obj.path();
        let path_segments: Vec<&str> = path.trim_start_matches('/').split('/').collect();

        // Expected format: gh/{owner}/{repo}@{tag}/{filename}
        if path_segments.len() < 4 || path_segments[0] != "gh" {
            return Err(TurboCdnError::config(
                "Invalid Fastly URL format. Expected: https://fastly.jsdelivr.net/gh/{owner}/{repo}@{tag}/{filename}".to_string(),
            ));
        }

        let owner = path_segments[1];
        let repo_and_tag = path_segments[2];
        let filename = path_segments[3..].join("/");

        // Parse repo@tag format
        let (repo, tag) = if let Some(at_pos) = repo_and_tag.find('@') {
            let repo = &repo_and_tag[..at_pos];
            let tag = &repo_and_tag[at_pos + 1..];
            (repo, tag)
        } else {
            return Err(TurboCdnError::config(
                "Invalid Fastly URL: missing @tag in repository specification".to_string(),
            ));
        };

        self.validate_components(owner, repo, tag, &filename)?;

        Ok(ParsedUrl {
            repository: format!("{}/{}", owner, repo),
            version: tag.to_string(),
            filename,
            original_url: original_url.to_string(),
            source_type: DetectedSourceType::Fastly,
        })
    }

    /// Parse Cloudflare CDN URL
    fn parse_cloudflare_url(&self, url_obj: &url::Url, original_url: &str) -> Result<ParsedUrl> {
        let path = url_obj.path();
        let path_segments: Vec<&str> = path.trim_start_matches('/').split('/').collect();

        // Expected format: ajax/libs/{library}/{version}/{filename}
        if path_segments.len() < 5 || path_segments[0] != "ajax" || path_segments[1] != "libs" {
            return Err(TurboCdnError::config(
                "Invalid Cloudflare URL format. Expected: https://cdnjs.cloudflare.com/ajax/libs/{library}/{version}/{filename}".to_string(),
            ));
        }

        let library = path_segments[2];
        let version = path_segments[3];
        let filename = path_segments[4..].join("/");

        if library.is_empty() || version.is_empty() || filename.is_empty() {
            return Err(TurboCdnError::config(
                "Invalid Cloudflare URL: missing required components".to_string(),
            ));
        }

        Ok(ParsedUrl {
            repository: format!("cdnjs/{}", library), // Use cdnjs as pseudo-owner
            version: version.to_string(),
            filename,
            original_url: original_url.to_string(),
            source_type: DetectedSourceType::Cloudflare,
        })
    }

    /// Parse npm registry URL
    fn parse_npm_url(&self, url_obj: &url::Url, original_url: &str) -> Result<ParsedUrl> {
        let path = url_obj.path();
        let path_segments: Vec<&str> = path.trim_start_matches('/').split('/').collect();

        // Expected format: {package}/-/{package}-{version}.tgz
        if path_segments.len() < 3 || path_segments[1] != "-" {
            return Err(TurboCdnError::config(
                "Invalid npm URL format. Expected: https://registry.npmjs.org/{package}/-/{package}-{version}.tgz".to_string(),
            ));
        }

        let package = path_segments[0];
        let filename = path_segments[2];

        // Extract version from filename (package-version.tgz)
        let version = if let Some(tgz_pos) = filename.rfind(".tgz") {
            let without_ext = &filename[..tgz_pos];
            if let Some(dash_pos) = without_ext.rfind('-') {
                &without_ext[dash_pos + 1..]
            } else {
                return Err(TurboCdnError::config(
                    "Invalid npm filename: cannot extract version".to_string(),
                ));
            }
        } else {
            return Err(TurboCdnError::config(
                "Invalid npm filename: must end with .tgz".to_string(),
            ));
        };

        if package.is_empty() || version.is_empty() {
            return Err(TurboCdnError::config(
                "Invalid npm URL: missing required components".to_string(),
            ));
        }

        Ok(ParsedUrl {
            repository: format!("npm/{}", package), // Use npm as pseudo-owner
            version: version.to_string(),
            filename: filename.to_string(),
            original_url: original_url.to_string(),
            source_type: DetectedSourceType::Npm,
        })
    }

    /// Validate URL components
    fn validate_components(
        &self,
        owner: &str,
        repo: &str,
        tag: &str,
        filename: &str,
    ) -> Result<()> {
        if owner.is_empty() || repo.is_empty() || tag.is_empty() || filename.is_empty() {
            return Err(TurboCdnError::config(
                "Invalid URL: missing required components (owner, repo, tag, or filename)"
                    .to_string(),
            ));
        }
        Ok(())
    }

    /// Extract version from filename using common patterns (public for testing)
    pub fn extract_version_from_filename(&self, filename: &str) -> Option<String> {
        // Common version patterns in filenames
        let patterns = [
            r"v?(\d+\.\d+\.\d+)",   // v1.2.3 or 1.2.3
            r"v?(\d+\.\d+)",        // v1.2 or 1.2
            r"(\d{4}-\d{2}-\d{2})", // 2023-12-01
            r"(\d{8})",             // 20231201
        ];

        for pattern in &patterns {
            if let Ok(re) = regex::Regex::new(pattern) {
                if let Some(captures) = re.captures(filename) {
                    if let Some(version) = captures.get(1) {
                        return Some(version.as_str().to_string());
                    }
                }
            }
        }

        None
    }

    /// Parse PyPI URL
    fn parse_pypi_url(&self, url_obj: &url::Url, original_url: &str) -> Result<ParsedUrl> {
        let path = url_obj.path();
        let path_segments: Vec<&str> = path.trim_start_matches('/').split('/').collect();

        // Expected format: packages/source/{first_letter}/{package}/{package}-{version}.tar.gz
        if path_segments.len() < 5 || path_segments[0] != "packages" || path_segments[1] != "source"
        {
            return Err(TurboCdnError::config(
                "Invalid PyPI URL format. Expected: https://files.pythonhosted.org/packages/source/{first_letter}/{package}/{package}-{version}.tar.gz".to_string(),
            ));
        }

        let package = path_segments[3];
        let filename = path_segments[4];

        // Extract version from filename
        let version = if let Some(tar_pos) = filename.rfind(".tar.gz") {
            let without_ext = &filename[..tar_pos];
            if let Some(dash_pos) = without_ext.rfind('-') {
                &without_ext[dash_pos + 1..]
            } else {
                return Err(TurboCdnError::config(
                    "Invalid PyPI filename: cannot extract version".to_string(),
                ));
            }
        } else {
            return Err(TurboCdnError::config(
                "Invalid PyPI filename: must end with .tar.gz".to_string(),
            ));
        };

        Ok(ParsedUrl {
            repository: format!("pypi/{}", package),
            version: version.to_string(),
            filename: filename.to_string(),
            original_url: original_url.to_string(),
            source_type: DetectedSourceType::PyPI,
        })
    }

    /// Parse Go proxy URL
    fn parse_go_proxy_url(&self, url_obj: &url::Url, original_url: &str) -> Result<ParsedUrl> {
        let path = url_obj.path();
        let path_segments: Vec<&str> = path.trim_start_matches('/').split('/').collect();

        // Expected format: {module}/@v/{version}.zip
        if path_segments.len() < 3 || !path_segments[path_segments.len() - 2].starts_with("@v") {
            return Err(TurboCdnError::config(
                "Invalid Go proxy URL format. Expected: https://proxy.golang.org/{module}/@v/{version}.zip".to_string(),
            ));
        }

        let module_parts = &path_segments[..path_segments.len() - 2];
        let module = module_parts.join("/");
        let filename = path_segments[path_segments.len() - 1];

        // Extract version from filename
        let version = if let Some(zip_pos) = filename.rfind(".zip") {
            &filename[..zip_pos]
        } else {
            return Err(TurboCdnError::config(
                "Invalid Go proxy filename: must end with .zip".to_string(),
            ));
        };

        Ok(ParsedUrl {
            repository: format!("go/{}", module),
            version: version.to_string(),
            filename: filename.to_string(),
            original_url: original_url.to_string(),
            source_type: DetectedSourceType::GoProxy,
        })
    }

    /// Parse Crates.io URL
    fn parse_crates_io_url(&self, url_obj: &url::Url, original_url: &str) -> Result<ParsedUrl> {
        let path = url_obj.path();
        let path_segments: Vec<&str> = path.trim_start_matches('/').split('/').collect();

        // Expected format: api/v1/crates/{crate}/{version}/download
        if path_segments.len() != 6
            || path_segments[0] != "api"
            || path_segments[1] != "v1"
            || path_segments[2] != "crates"
            || path_segments[5] != "download"
        {
            return Err(TurboCdnError::config(
                "Invalid Crates.io URL format. Expected: https://crates.io/api/v1/crates/{crate}/{version}/download".to_string(),
            ));
        }

        let crate_name = path_segments[3];
        let version = path_segments[4];
        let filename = format!("{}-{}.crate", crate_name, version);

        Ok(ParsedUrl {
            repository: format!("crates/{}", crate_name),
            version: version.to_string(),
            filename,
            original_url: original_url.to_string(),
            source_type: DetectedSourceType::CratesIo,
        })
    }

    /// Parse Maven Central URL
    fn parse_maven_url(&self, url_obj: &url::Url, original_url: &str) -> Result<ParsedUrl> {
        let path = url_obj.path();
        let path_segments: Vec<&str> = path.trim_start_matches('/').split('/').collect();

        // Expected format: maven2/{group_path}/{artifact}/{version}/{artifact}-{version}.jar
        if path_segments.len() < 5 || path_segments[0] != "maven2" {
            return Err(TurboCdnError::config(
                "Invalid Maven URL format. Expected: https://repo1.maven.org/maven2/{group_path}/{artifact}/{version}/{artifact}-{version}.jar".to_string(),
            ));
        }

        let artifact = path_segments[path_segments.len() - 3];
        let version = path_segments[path_segments.len() - 2];
        let filename = path_segments[path_segments.len() - 1];
        let group_path = path_segments[1..path_segments.len() - 3].join(".");

        Ok(ParsedUrl {
            repository: format!("maven/{}.{}", group_path, artifact),
            version: version.to_string(),
            filename: filename.to_string(),
            original_url: original_url.to_string(),
            source_type: DetectedSourceType::Maven,
        })
    }

    /// Parse NuGet URL
    fn parse_nuget_url(&self, url_obj: &url::Url, original_url: &str) -> Result<ParsedUrl> {
        let path = url_obj.path();
        let path_segments: Vec<&str> = path.trim_start_matches('/').split('/').collect();

        // Expected format: v3-flatcontainer/{package}/{version}/{package}.{version}.nupkg
        if path_segments.len() != 4 || path_segments[0] != "v3-flatcontainer" {
            return Err(TurboCdnError::config(
                "Invalid NuGet URL format. Expected: https://api.nuget.org/v3-flatcontainer/{package}/{version}/{package}.{version}.nupkg".to_string(),
            ));
        }

        let package = path_segments[1];
        let version = path_segments[2];
        let filename = path_segments[3];

        Ok(ParsedUrl {
            repository: format!("nuget/{}", package),
            version: version.to_string(),
            filename: filename.to_string(),
            original_url: original_url.to_string(),
            source_type: DetectedSourceType::NuGet,
        })
    }

    /// Parse Docker Hub URL
    fn parse_docker_hub_url(&self, url_obj: &url::Url, original_url: &str) -> Result<ParsedUrl> {
        let path = url_obj.path();
        let path_segments: Vec<&str> = path.trim_start_matches('/').split('/').collect();

        // Expected format: v2/library/{image}/manifests/{tag}
        if path_segments.len() != 5
            || path_segments[0] != "v2"
            || path_segments[1] != "library"
            || path_segments[3] != "manifests"
        {
            return Err(TurboCdnError::config(
                "Invalid Docker Hub URL format. Expected: https://registry-1.docker.io/v2/library/{image}/manifests/{tag}".to_string(),
            ));
        }

        let image = path_segments[2];
        let tag = path_segments[4];
        let filename = format!("{}-{}.tar", image, tag);

        Ok(ParsedUrl {
            repository: format!("docker/{}", image),
            version: tag.to_string(),
            filename,
            original_url: original_url.to_string(),
            source_type: DetectedSourceType::DockerHub,
        })
    }

    /// Parse SourceForge URL
    fn parse_sourceforge_url(&self, url_obj: &url::Url, original_url: &str) -> Result<ParsedUrl> {
        let path = url_obj.path();
        let path_segments: Vec<&str> = path.trim_start_matches('/').split('/').collect();

        // Expected format: project/{project}/{filename}
        if path_segments.len() < 3 || path_segments[0] != "project" {
            return Err(TurboCdnError::config(
                "Invalid SourceForge URL format. Expected: https://downloads.sourceforge.net/project/{project}/{filename}".to_string(),
            ));
        }

        let project = path_segments[1];
        let filename = path_segments[2..].join("/");

        // Try to extract version from filename
        let version = self
            .extract_version_from_filename(&filename)
            .unwrap_or_else(|| "latest".to_string());

        Ok(ParsedUrl {
            repository: format!("sourceforge/{}", project),
            version,
            filename,
            original_url: original_url.to_string(),
            source_type: DetectedSourceType::SourceForge,
        })
    }

    /// Get download statistics
    pub async fn get_stats(&self) -> Result<TurboCdnStats> {
        self.downloader.get_stats().await
    }

    /// Perform health check on all sources
    pub async fn health_check(
        &self,
    ) -> Result<std::collections::HashMap<String, sources::HealthStatus>> {
        self.downloader.health_check().await
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
        self.config.regions.default = region.to_string();
        self
    }

    /// Set the download directory
    pub fn with_download_dir<P: Into<PathBuf>>(self, _dir: P) -> Self {
        // Note: Download directory is now handled by the cache configuration
        // This method is kept for API compatibility but doesn't modify anything
        self
    }

    /// Enable or disable caching
    pub fn with_cache(mut self, enabled: bool) -> Self {
        self.config.performance.cache.enabled = enabled;
        self
    }

    /// Set maximum concurrent downloads
    pub fn with_max_concurrent_downloads(mut self, max: usize) -> Self {
        self.config.performance.max_concurrent_downloads = max;
        self
    }

    /// Build the TurboCdn client
    pub async fn build(mut self) -> Result<TurboCdn> {
        // Auto-detect geographic region if enabled
        if self.config.regions.auto_detect {
            match self.auto_detect_region().await {
                Ok(detected_region) => {
                    info!("Auto-detected region: {:?}", detected_region);
                    self.config.regions.default = detected_region.to_string();
                }
                Err(e) => {
                    warn!("Failed to auto-detect region: {}, using default", e);
                }
            }
        }

        // Create source manager
        let mut source_manager = SourceManager::new();

        for source in &self.sources {
            match source {
                Source::GitHub => {
                    if let Some(github_config) = self.config.mirrors.configs.get("github") {
                        let github_source = GitHubSource::new(github_config.clone())?;
                        source_manager.add_source(Box::new(github_source));
                    }
                }
                Source::JsDelivr => {
                    if let Some(jsdelivr_config) = self.config.mirrors.configs.get("jsdelivr") {
                        let jsdelivr_source = JsDelivrSource::new(jsdelivr_config.clone())?;
                        source_manager.add_source(Box::new(jsdelivr_source));
                    }
                }
                Source::Fastly => {
                    if let Some(fastly_config) = self.config.mirrors.configs.get("fastly") {
                        let fastly_source = FastlySource::new(fastly_config.clone())?;
                        source_manager.add_source(Box::new(fastly_source));
                    }
                }
                Source::Cloudflare => {
                    if let Some(cloudflare_config) = self.config.mirrors.configs.get("cloudflare") {
                        let cloudflare_source = CloudflareSource::new(cloudflare_config.clone())?;
                        source_manager.add_source(Box::new(cloudflare_source));
                    }
                }
            }
        }

        // Create components with performance data loading
        let router = SmartRouter::new_with_data(self.config.clone(), source_manager).await?;
        let cache_manager = CacheManager::new(self.config.performance.cache.clone()).await?;
        let compliance_checker = ComplianceChecker::new(self.config.security.clone())?;

        // Create downloader
        let downloader =
            Downloader::new(self.config, router, cache_manager, compliance_checker).await?;

        Ok(TurboCdn { downloader })
    }

    /// Auto-detect user's geographic region
    async fn auto_detect_region(&self) -> Result<Region> {
        use crate::geo_detection::GeoDetector;

        let mut detector = GeoDetector::new();
        detector.detect_region().await
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
            let turbo_cdn = TurboCdn::builder().with_config(config).build().await?;
            Ok(Self {
                inner: Arc::new(Mutex::new(turbo_cdn)),
            })
        }

        /// Create a new AsyncTurboCdn instance with builder
        pub async fn with_builder(builder: TurboCdnBuilder) -> Result<Self> {
            let turbo_cdn = builder.build().await?;
            Ok(Self {
                inner: Arc::new(Mutex::new(turbo_cdn)),
            })
        }

        /// Download from any supported URL with automatic optimization (async version)
        pub async fn download_from_url_async(
            &self,
            url: &str,
            options: Option<DownloadOptions>,
        ) -> Result<DownloadResult> {
            let mut client = self.inner.lock().await;
            client.download_from_url(url, options).await
        }

        /// Get optimal CDN URL without downloading (async version)
        pub async fn get_optimal_url_async(&self, url: &str) -> Result<String> {
            let client = self.inner.lock().await;
            client.get_optimal_url(url).await
        }

        /// Parse URL into components (async version)
        pub async fn parse_url_async(&self, url: &str) -> Result<ParsedUrl> {
            let client = self.inner.lock().await;
            client.parse_url(url)
        }

        /// Download a file by repository, version, and filename (async version)
        pub async fn download_async(
            &self,
            repository: &str,
            version: &str,
            file_name: &str,
            options: Option<DownloadOptions>,
        ) -> Result<DownloadResult> {
            let mut client = self.inner.lock().await;
            client
                .download(repository, version, file_name, options.unwrap_or_default())
                .await
        }

        /// Get repository metadata (async version)
        pub async fn get_repository_metadata_async(
            &self,
            repository: &str,
        ) -> Result<sources::RepositoryMetadata> {
            let client = self.inner.lock().await;
            client.get_repository_metadata(repository).await
        }

        /// Extract version from filename (async version)
        pub async fn extract_version_from_filename_async(&self, filename: &str) -> Option<String> {
            let client = self.inner.lock().await;
            client.extract_version_from_filename(filename)
        }

        /// Get download statistics (async version)
        pub async fn get_stats_async(&self) -> Result<TurboCdnStats> {
            let client = self.inner.lock().await;
            client.get_stats().await
        }

        /// Perform health check on all sources (async version)
        pub async fn health_check_async(
            &self,
        ) -> Result<std::collections::HashMap<String, sources::HealthStatus>> {
            let client = self.inner.lock().await;
            client.health_check().await
        }
    }

    /// Async builder for AsyncTurboCdn
    #[derive(Debug)]
    pub struct AsyncTurboCdnBuilder {
        builder: TurboCdnBuilder,
    }

    impl AsyncTurboCdnBuilder {
        /// Create a new async builder
        pub fn new() -> Self {
            Self {
                builder: TurboCdnBuilder::new(),
            }
        }

        /// Set the configuration
        pub fn with_config(mut self, config: TurboCdnConfig) -> Self {
            self.builder = self.builder.with_config(config);
            self
        }

        /// Set the download sources
        pub fn with_sources(mut self, sources: &[Source]) -> Self {
            self.builder = self.builder.with_sources(sources);
            self
        }

        /// Set the region for optimization
        pub fn with_region(mut self, region: Region) -> Self {
            self.builder = self.builder.with_region(region);
            self
        }

        /// Set the download directory
        pub fn with_download_dir<P: Into<PathBuf>>(mut self, dir: P) -> Self {
            self.builder = self.builder.with_download_dir(dir);
            self
        }

        /// Enable or disable caching
        pub fn with_cache(mut self, enabled: bool) -> Self {
            self.builder = self.builder.with_cache(enabled);
            self
        }

        /// Set maximum concurrent downloads
        pub fn with_max_concurrent_downloads(mut self, max: usize) -> Self {
            self.builder = self.builder.with_max_concurrent_downloads(max);
            self
        }

        /// Build the AsyncTurboCdn client
        pub async fn build(self) -> Result<AsyncTurboCdn> {
            AsyncTurboCdn::with_builder(self.builder).await
        }
    }

    impl Default for AsyncTurboCdnBuilder {
        fn default() -> Self {
            Self::new()
        }
    }

    /// Convenience functions for quick async operations
    pub mod quick {
        use super::*;

        /// Quick download from URL with default settings
        pub async fn download_url(url: &str) -> Result<DownloadResult> {
            let client = AsyncTurboCdn::new().await?;
            client.download_from_url_async(url, None).await
        }

        /// Quick URL optimization
        pub async fn optimize_url(url: &str) -> Result<String> {
            let client = AsyncTurboCdn::new().await?;
            client.get_optimal_url_async(url).await
        }

        /// Quick URL parsing
        pub async fn parse_url(url: &str) -> Result<ParsedUrl> {
            let client = AsyncTurboCdn::new().await?;
            client.parse_url_async(url).await
        }

        /// Quick repository download
        pub async fn download_repository(
            repository: &str,
            version: &str,
            file_name: &str,
        ) -> Result<DownloadResult> {
            let client = AsyncTurboCdn::new().await?;
            client
                .download_async(repository, version, file_name, None)
                .await
        }
    }
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
    fn test_parsed_url_creation() {
        let parsed = ParsedUrl {
            repository: "owner/repo".to_string(),
            version: "v1.0.0".to_string(),
            filename: "file.zip".to_string(),
            original_url: "https://github.com/owner/repo/releases/download/v1.0.0/file.zip"
                .to_string(),
            source_type: DetectedSourceType::GitHub,
        };

        assert_eq!(parsed.repository, "owner/repo");
        assert_eq!(parsed.version, "v1.0.0");
        assert_eq!(parsed.filename, "file.zip");
        assert_eq!(parsed.source_type, DetectedSourceType::GitHub);
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
            source: "github".to_string(),
            url: "https://github.com/owner/repo/releases/download/v1.0.0/file.zip".to_string(),
            from_cache: false,
            checksum: None,
        };

        assert_eq!(result.path, PathBuf::from("/tmp/file.zip"));
        assert_eq!(result.size, 1024);
        assert_eq!(result.duration, Duration::from_secs(1));
        assert_eq!(result.speed, 1024.0);
        assert_eq!(result.source, "github");
        assert!(!result.from_cache);
        assert!(result.checksum.is_none());
    }

    #[test]
    fn test_builder_configuration() {
        let builder = TurboCdn::builder()
            .with_cache(false)
            .with_max_concurrent_downloads(10);

        assert!(!builder.config.performance.cache.enabled);
        assert_eq!(builder.config.performance.max_concurrent_downloads, 10);
        assert_eq!(builder.sources.len(), 4); // Default sources
    }
}
