// Licensed under the MIT License
// Copyright (c) 2025 Hal <hal.long@outlook.com>

//! GitHub Releases version listing API with CDN fallback
//!
//! This module provides APIs for fetching GitHub releases version lists,
//! with automatic fallback to jsDelivr's data API when the GitHub API
//! is rate-limited or unavailable.
//!
//! # Features
//!
//! - **GitHub API**: Direct access to GitHub releases (requires token for higher rate limits)
//! - **jsDelivr Fallback**: Automatic fallback to jsDelivr data API (no rate limits)
//! - **Version Filtering**: Filter pre-releases, drafts, and specific patterns
//!
//! # Example
//!
//! ```rust,no_run
//! use turbo_cdn::github_releases::GitHubReleasesFetcher;
//!
//! #[tokio::main]
//! async fn main() -> turbo_cdn::Result<()> {
//!     let fetcher = GitHubReleasesFetcher::new();
//!     
//!     // Fetch versions with automatic fallback
//!     let versions = fetcher.fetch_versions("BurntSushi", "ripgrep").await?;
//!     for version in &versions {
//!         println!("{}", version);
//!     }
//!     
//!     // Fetch detailed release info
//!     let releases = fetcher.list_releases("BurntSushi", "ripgrep").await?;
//!     for release in &releases {
//!         println!("{} (prerelease: {})", release.tag_name, release.prerelease);
//!     }
//!     
//!     Ok(())
//! }
//! ```

use crate::error::{Result, TurboCdnError};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{debug, info, warn};

/// Default timeout for API requests
const API_TIMEOUT: Duration = Duration::from_secs(30);

/// GitHub API base URL
const GITHUB_API_BASE: &str = "https://api.github.com";

/// jsDelivr data API base URL (fallback, no rate limits)
const JSDELIVR_DATA_API_BASE: &str = "https://data.jsdelivr.com/v1";

/// Maximum number of releases to fetch per page from GitHub API
const GITHUB_PER_PAGE: u32 = 100;

/// Release information from GitHub
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseInfo {
    /// Release tag name (e.g., "v1.0.0")
    pub tag_name: String,
    /// Release name/title
    pub name: Option<String>,
    /// Whether this is a pre-release
    pub prerelease: bool,
    /// Whether this is a draft
    pub draft: bool,
    /// Published date (ISO 8601)
    pub published_at: Option<String>,
    /// List of asset names available in this release
    pub assets: Vec<AssetInfo>,
}

/// Asset information from a GitHub release
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetInfo {
    /// Asset file name
    pub name: String,
    /// Asset size in bytes
    pub size: u64,
    /// Download URL
    pub browser_download_url: String,
    /// Content type
    pub content_type: Option<String>,
    /// Download count
    pub download_count: u64,
}

/// Version information from jsDelivr (simplified)
#[derive(Debug, Clone, Serialize, Deserialize)]
struct JsDelivrPackageResponse {
    /// Package type
    #[serde(rename = "type")]
    package_type: Option<String>,
    /// Package name
    name: Option<String>,
    /// Available versions
    versions: Vec<JsDelivrVersion>,
}

/// jsDelivr version entry
#[derive(Debug, Clone, Serialize, Deserialize)]
struct JsDelivrVersion {
    /// Version string
    version: String,
}

/// Options for filtering releases
#[derive(Debug, Clone)]
pub struct FetchOptions {
    /// Include pre-releases
    pub include_prereleases: bool,
    /// Include drafts (requires authentication)
    pub include_drafts: bool,
    /// Maximum number of versions to return
    pub max_versions: Option<usize>,
    /// GitHub personal access token for higher rate limits
    pub github_token: Option<String>,
    /// Request timeout
    pub timeout: Duration,
}

impl Default for FetchOptions {
    fn default() -> Self {
        Self {
            include_prereleases: false,
            include_drafts: false,
            max_versions: None,
            github_token: None,
            timeout: API_TIMEOUT,
        }
    }
}

impl FetchOptions {
    /// Create new fetch options
    pub fn new() -> Self {
        Self::default()
    }

    /// Include pre-releases in results
    pub fn with_prereleases(mut self, include: bool) -> Self {
        self.include_prereleases = include;
        self
    }

    /// Include drafts in results
    pub fn with_drafts(mut self, include: bool) -> Self {
        self.include_drafts = include;
        self
    }

    /// Set maximum number of versions to return
    pub fn with_max_versions(mut self, max: usize) -> Self {
        self.max_versions = Some(max);
        self
    }

    /// Set GitHub token for authentication
    pub fn with_github_token<S: Into<String>>(mut self, token: S) -> Self {
        self.github_token = Some(token.into());
        self
    }

    /// Set request timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }
}

/// Source of version data
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataSource {
    /// Data fetched from GitHub API
    GitHub,
    /// Data fetched from jsDelivr CDN (fallback)
    JsDelivr,
}

impl std::fmt::Display for DataSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DataSource::GitHub => write!(f, "GitHub API"),
            DataSource::JsDelivr => write!(f, "jsDelivr CDN"),
        }
    }
}

/// Result of a version fetch operation, including data source info
#[derive(Debug, Clone)]
pub struct VersionsResult {
    /// List of version strings
    pub versions: Vec<String>,
    /// Where the data came from
    pub source: DataSource,
}

/// Result of a releases fetch operation, including data source info
#[derive(Debug, Clone)]
pub struct ReleasesResult {
    /// List of release info
    pub releases: Vec<ReleaseInfo>,
    /// Where the data came from
    pub source: DataSource,
}

/// Fetcher for GitHub releases with CDN fallback
///
/// Provides methods to fetch version lists and release information
/// from GitHub, with automatic fallback to jsDelivr when GitHub API
/// is unavailable or rate-limited.
#[derive(Debug, Clone)]
pub struct GitHubReleasesFetcher {
    options: FetchOptions,
}

impl GitHubReleasesFetcher {
    /// Create a new fetcher with default options
    pub fn new() -> Self {
        // Try to get GitHub token from environment
        let token = std::env::var("GITHUB_TOKEN")
            .or_else(|_| std::env::var("GH_TOKEN"))
            .ok();

        let options = FetchOptions {
            github_token: token,
            ..FetchOptions::default()
        };

        Self { options }
    }

    /// Create a new fetcher with custom options
    pub fn with_options(options: FetchOptions) -> Self {
        Self { options }
    }

    /// Fetch version strings for a GitHub repository
    ///
    /// Tries GitHub API first, falls back to jsDelivr if GitHub is unavailable.
    ///
    /// # Arguments
    /// * `owner` - Repository owner (e.g., "BurntSushi")
    /// * `repo` - Repository name (e.g., "ripgrep")
    ///
    /// # Returns
    /// * `VersionsResult` containing version strings and data source info
    pub async fn fetch_versions_with_source(
        &self,
        owner: &str,
        repo: &str,
    ) -> Result<VersionsResult> {
        // Try GitHub API first
        match self.fetch_versions_from_github(owner, repo).await {
            Ok(versions) => {
                info!(
                    "Fetched {} versions from GitHub API for {}/{}",
                    versions.len(),
                    owner,
                    repo
                );
                Ok(VersionsResult {
                    versions,
                    source: DataSource::GitHub,
                })
            }
            Err(github_err) => {
                warn!(
                    "GitHub API failed for {}/{}: {}, falling back to jsDelivr",
                    owner, repo, github_err
                );
                // Fallback to jsDelivr
                match self.fetch_versions_from_jsdelivr(owner, repo).await {
                    Ok(versions) => {
                        info!(
                            "Fetched {} versions from jsDelivr for {}/{}",
                            versions.len(),
                            owner,
                            repo
                        );
                        Ok(VersionsResult {
                            versions,
                            source: DataSource::JsDelivr,
                        })
                    }
                    Err(jsdelivr_err) => {
                        // Both sources failed
                        Err(TurboCdnError::download(format!(
                            "Failed to fetch versions for {}/{}: GitHub error: {}; jsDelivr error: {}",
                            owner, repo, github_err, jsdelivr_err
                        )))
                    }
                }
            }
        }
    }

    /// Fetch version strings (simple API without source info)
    ///
    /// # Arguments
    /// * `owner` - Repository owner
    /// * `repo` - Repository name
    ///
    /// # Returns
    /// * `Vec<String>` of version tags
    pub async fn fetch_versions(&self, owner: &str, repo: &str) -> Result<Vec<String>> {
        let result = self.fetch_versions_with_source(owner, repo).await?;
        Ok(result.versions)
    }

    /// List detailed release information for a GitHub repository
    ///
    /// Tries GitHub API first, falls back to jsDelivr if GitHub is unavailable.
    /// Note: jsDelivr fallback returns limited information (only tag names).
    ///
    /// # Arguments
    /// * `owner` - Repository owner
    /// * `repo` - Repository name
    ///
    /// # Returns
    /// * `ReleasesResult` containing release info and data source info
    pub async fn list_releases_with_source(
        &self,
        owner: &str,
        repo: &str,
    ) -> Result<ReleasesResult> {
        // Try GitHub API first
        match self.list_releases_from_github(owner, repo).await {
            Ok(releases) => {
                info!(
                    "Fetched {} releases from GitHub API for {}/{}",
                    releases.len(),
                    owner,
                    repo
                );
                Ok(ReleasesResult {
                    releases,
                    source: DataSource::GitHub,
                })
            }
            Err(github_err) => {
                warn!(
                    "GitHub API failed for {}/{}: {}, falling back to jsDelivr",
                    owner, repo, github_err
                );
                // Fallback to jsDelivr (limited info)
                match self.fetch_versions_from_jsdelivr(owner, repo).await {
                    Ok(versions) => {
                        let releases: Vec<ReleaseInfo> = versions
                            .into_iter()
                            .map(|tag| ReleaseInfo {
                                tag_name: tag,
                                name: None,
                                prerelease: false,
                                draft: false,
                                published_at: None,
                                assets: Vec::new(),
                            })
                            .collect();
                        info!(
                            "Fetched {} versions from jsDelivr for {}/{} (limited info)",
                            releases.len(),
                            owner,
                            repo
                        );
                        Ok(ReleasesResult {
                            releases,
                            source: DataSource::JsDelivr,
                        })
                    }
                    Err(jsdelivr_err) => {
                        Err(TurboCdnError::download(format!(
                            "Failed to fetch releases for {}/{}: GitHub error: {}; jsDelivr error: {}",
                            owner, repo, github_err, jsdelivr_err
                        )))
                    }
                }
            }
        }
    }

    /// List detailed release information (simple API without source info)
    pub async fn list_releases(&self, owner: &str, repo: &str) -> Result<Vec<ReleaseInfo>> {
        let result = self.list_releases_with_source(owner, repo).await?;
        Ok(result.releases)
    }

    /// Fetch the latest release version
    pub async fn fetch_latest_version(&self, owner: &str, repo: &str) -> Result<String> {
        let versions = self.fetch_versions(owner, repo).await?;
        versions.into_iter().next().ok_or_else(|| {
            TurboCdnError::download(format!("No releases found for {}/{}", owner, repo))
        })
    }

    /// Fetch versions from GitHub API
    async fn fetch_versions_from_github(
        &self,
        owner: &str,
        repo: &str,
    ) -> Result<Vec<String>> {
        let releases = self.list_releases_from_github(owner, repo).await?;
        Ok(releases.into_iter().map(|r| r.tag_name).collect())
    }

    /// Fetch detailed releases from GitHub API
    async fn list_releases_from_github(
        &self,
        owner: &str,
        repo: &str,
    ) -> Result<Vec<ReleaseInfo>> {
        crate::init_rustls_provider();

        let url = format!(
            "{}/repos/{}/{}/releases?per_page={}",
            GITHUB_API_BASE, owner, repo, GITHUB_PER_PAGE
        );

        debug!("Fetching releases from GitHub: {}", url);

        let mut builder = reqwest::Client::builder()
            .timeout(self.options.timeout)
            .build()
            .map_err(|e| TurboCdnError::network(format!("Failed to create HTTP client: {e}")))?
            .get(&url)
            .header("User-Agent", "turbo-cdn")
            .header("Accept", "application/vnd.github.v3+json");

        // Add authentication if token is available
        if let Some(ref token) = self.options.github_token {
            builder = builder.header("Authorization", format!("Bearer {token}"));
        }

        let response = builder
            .send()
            .await
            .map_err(|e| TurboCdnError::network(format!("GitHub API request failed: {e}")))?;

        let status = response.status();

        // Check for rate limiting
        if status.as_u16() == 403 || status.as_u16() == 429 {
            let rate_limit_remaining = response
                .headers()
                .get("x-ratelimit-remaining")
                .and_then(|v| v.to_str().ok())
                .unwrap_or("unknown");
            return Err(TurboCdnError::rate_limit(format!(
                "GitHub API rate limited (remaining: {rate_limit_remaining})"
            )));
        }

        if !status.is_success() {
            return Err(TurboCdnError::from_status_code(
                status.as_u16(),
                format!("{GITHUB_API_BASE}/repos/{owner}/{repo}/releases"),
            ));
        }

        let github_releases: Vec<GitHubApiRelease> = response
            .json()
            .await
            .map_err(|e| TurboCdnError::internal(format!("Failed to parse GitHub response: {e}")))?;

        // Convert and filter
        let releases: Vec<ReleaseInfo> = github_releases
            .into_iter()
            .filter(|r| {
                (self.options.include_drafts || !r.draft)
                    && (self.options.include_prereleases || !r.prerelease)
            })
            .map(|r| ReleaseInfo {
                tag_name: r.tag_name,
                name: r.name,
                prerelease: r.prerelease,
                draft: r.draft,
                published_at: r.published_at,
                assets: r
                    .assets
                    .into_iter()
                    .map(|a| AssetInfo {
                        name: a.name,
                        size: a.size,
                        browser_download_url: a.browser_download_url,
                        content_type: a.content_type,
                        download_count: a.download_count,
                    })
                    .collect(),
            })
            .collect();

        // Apply max_versions limit
        let releases = if let Some(max) = self.options.max_versions {
            releases.into_iter().take(max).collect()
        } else {
            releases
        };

        Ok(releases)
    }

    /// Fetch versions from jsDelivr data API (fallback, no rate limits)
    async fn fetch_versions_from_jsdelivr(
        &self,
        owner: &str,
        repo: &str,
    ) -> Result<Vec<String>> {
        crate::init_rustls_provider();

        let url = format!(
            "{}/package/gh/{}/{}",
            JSDELIVR_DATA_API_BASE, owner, repo
        );

        debug!("Fetching versions from jsDelivr: {}", url);

        let response = reqwest::Client::builder()
            .timeout(self.options.timeout)
            .build()
            .map_err(|e| TurboCdnError::network(format!("Failed to create HTTP client: {e}")))?
            .get(&url)
            .header("User-Agent", "turbo-cdn")
            .send()
            .await
            .map_err(|e| TurboCdnError::network(format!("jsDelivr API request failed: {e}")))?;

        let status = response.status();

        if !status.is_success() {
            return Err(TurboCdnError::from_status_code(
                status.as_u16(),
                url,
            ));
        }

        let package_info: JsDelivrPackageResponse = response
            .json()
            .await
            .map_err(|e| TurboCdnError::internal(format!("Failed to parse jsDelivr response: {e}")))?;

        let mut versions: Vec<String> = package_info
            .versions
            .into_iter()
            .map(|v| v.version)
            .collect();

        // Filter pre-releases if needed (best-effort based on semver conventions)
        if !self.options.include_prereleases {
            versions.retain(|v| {
                !v.contains("-alpha")
                    && !v.contains("-beta")
                    && !v.contains("-rc")
                    && !v.contains("-dev")
                    && !v.contains("-pre")
            });
        }

        // Apply max_versions limit
        if let Some(max) = self.options.max_versions {
            versions.truncate(max);
        }

        Ok(versions)
    }
}

impl Default for GitHubReleasesFetcher {
    fn default() -> Self {
        Self::new()
    }
}

/// GitHub API release response (internal deserialization structure)
#[derive(Debug, Deserialize)]
struct GitHubApiRelease {
    tag_name: String,
    name: Option<String>,
    prerelease: bool,
    draft: bool,
    published_at: Option<String>,
    assets: Vec<GitHubApiAsset>,
}

/// GitHub API asset response (internal deserialization structure)
#[derive(Debug, Deserialize)]
struct GitHubApiAsset {
    name: String,
    size: u64,
    browser_download_url: String,
    content_type: Option<String>,
    download_count: u64,
}

// ============================================================================
// Convenience functions
// ============================================================================

/// Fetch version strings for a GitHub repository (convenience function)
///
/// Uses default options with automatic GitHub token detection from environment.
///
/// # Example
/// ```rust,no_run
/// use turbo_cdn::github_releases;
///
/// #[tokio::main]
/// async fn main() -> turbo_cdn::Result<()> {
///     let versions = github_releases::fetch_versions("BurntSushi", "ripgrep").await?;
///     println!("Latest version: {}", versions[0]);
///     Ok(())
/// }
/// ```
pub async fn fetch_versions(owner: &str, repo: &str) -> Result<Vec<String>> {
    GitHubReleasesFetcher::new().fetch_versions(owner, repo).await
}

/// List detailed release info for a GitHub repository (convenience function)
pub async fn list_releases(owner: &str, repo: &str) -> Result<Vec<ReleaseInfo>> {
    GitHubReleasesFetcher::new().list_releases(owner, repo).await
}

/// Fetch the latest release version (convenience function)
pub async fn fetch_latest_version(owner: &str, repo: &str) -> Result<String> {
    GitHubReleasesFetcher::new()
        .fetch_latest_version(owner, repo)
        .await
}

/// Fetch versions with detailed source information (convenience function)
pub async fn fetch_versions_with_source(
    owner: &str,
    repo: &str,
) -> Result<VersionsResult> {
    GitHubReleasesFetcher::new()
        .fetch_versions_with_source(owner, repo)
        .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fetch_options_default() {
        let options = FetchOptions::default();
        assert!(!options.include_prereleases);
        assert!(!options.include_drafts);
        assert!(options.max_versions.is_none());
        assert_eq!(options.timeout, API_TIMEOUT);
    }

    #[test]
    fn test_fetch_options_builder() {
        let options = FetchOptions::new()
            .with_prereleases(true)
            .with_drafts(true)
            .with_max_versions(10)
            .with_github_token("test-token")
            .with_timeout(Duration::from_secs(60));

        assert!(options.include_prereleases);
        assert!(options.include_drafts);
        assert_eq!(options.max_versions, Some(10));
        assert_eq!(options.github_token, Some("test-token".to_string()));
        assert_eq!(options.timeout, Duration::from_secs(60));
    }

    #[test]
    fn test_data_source_display() {
        assert_eq!(DataSource::GitHub.to_string(), "GitHub API");
        assert_eq!(DataSource::JsDelivr.to_string(), "jsDelivr CDN");
    }

    #[test]
    fn test_release_info_serialization() {
        let release = ReleaseInfo {
            tag_name: "v1.0.0".to_string(),
            name: Some("Release 1.0.0".to_string()),
            prerelease: false,
            draft: false,
            published_at: Some("2024-01-01T00:00:00Z".to_string()),
            assets: vec![AssetInfo {
                name: "app-linux-x64.tar.gz".to_string(),
                size: 1024,
                browser_download_url: "https://github.com/owner/repo/releases/download/v1.0.0/app-linux-x64.tar.gz".to_string(),
                content_type: Some("application/gzip".to_string()),
                download_count: 100,
            }],
        };

        let json = serde_json::to_string(&release).unwrap();
        assert!(json.contains("v1.0.0"));
        assert!(json.contains("app-linux-x64.tar.gz"));
    }

    #[test]
    fn test_fetcher_default() {
        let fetcher = GitHubReleasesFetcher::new();
        assert!(!fetcher.options.include_prereleases);
    }

    #[test]
    fn test_fetcher_with_options() {
        let options = FetchOptions::new()
            .with_prereleases(true)
            .with_max_versions(5);
        let fetcher = GitHubReleasesFetcher::with_options(options);
        assert!(fetcher.options.include_prereleases);
        assert_eq!(fetcher.options.max_versions, Some(5));
    }
}
