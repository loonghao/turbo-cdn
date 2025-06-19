// Licensed under the MIT License
// Copyright (c) 2025 Hal <hal.long@outlook.com>

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use url::Url;

use crate::error::{Result, TurboCdnError};

pub mod cloudflare;
pub mod fastly;
pub mod github;
pub mod jsdelivr;

/// Trait for download sources
#[async_trait]
pub trait DownloadSource: Send + Sync {
    /// Get the name of this source
    fn name(&self) -> &str;

    /// Get the priority of this source (lower is higher priority)
    fn priority(&self) -> u8;

    /// Check if this source is enabled
    fn is_enabled(&self) -> bool;

    /// Check if this source can handle the given repository
    async fn can_handle(&self, repository: &str, version: &str, file_name: &str) -> bool;

    /// Get download URLs for a file
    async fn get_download_urls(
        &self,
        repository: &str,
        version: &str,
        file_name: &str,
    ) -> Result<Vec<DownloadUrl>>;

    /// Get metadata for a repository
    async fn get_repository_metadata(&self, repository: &str) -> Result<RepositoryMetadata>;

    /// Test the availability of this source
    async fn health_check(&self) -> Result<HealthStatus>;
}

/// Download URL with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadUrl {
    /// The actual download URL
    pub url: String,

    /// Source that provided this URL
    pub source: String,

    /// Priority of this URL (lower is higher priority)
    pub priority: u8,

    /// Expected file size in bytes (if known)
    pub size: Option<u64>,

    /// Expected checksum (if known)
    pub checksum: Option<Checksum>,

    /// Additional metadata
    pub metadata: HashMap<String, String>,

    /// Whether this URL supports range requests
    pub supports_ranges: bool,

    /// Estimated latency in milliseconds
    pub estimated_latency: Option<u64>,
}

/// Repository metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryMetadata {
    /// Repository name
    pub name: String,

    /// Repository description
    pub description: Option<String>,

    /// Repository URL
    pub url: String,

    /// License information
    pub license: Option<String>,

    /// Available versions/tags
    pub versions: Vec<String>,

    /// Available files for the latest version
    pub files: Vec<FileInfo>,

    /// Repository statistics
    pub stats: RepositoryStats,

    /// Last updated timestamp
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

/// File information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    /// File name
    pub name: String,

    /// File size in bytes
    pub size: u64,

    /// File checksum
    pub checksum: Option<Checksum>,

    /// Content type
    pub content_type: Option<String>,

    /// Download count (if available)
    pub download_count: Option<u64>,
}

/// Repository statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryStats {
    /// Number of stars
    pub stars: Option<u64>,

    /// Number of forks
    pub forks: Option<u64>,

    /// Number of watchers
    pub watchers: Option<u64>,

    /// Number of open issues
    pub open_issues: Option<u64>,

    /// Repository size in KB
    pub size: Option<u64>,
}

/// Checksum information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Checksum {
    /// Checksum algorithm
    pub algorithm: ChecksumAlgorithm,

    /// Checksum value (hex encoded)
    pub value: String,
}

/// Supported checksum algorithms
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChecksumAlgorithm {
    Sha256,
    Sha512,
    Blake3,
    Md5,
}

/// Health status of a source
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    /// Whether the source is healthy
    pub healthy: bool,

    /// Response time in milliseconds
    pub response_time: u64,

    /// Error message if unhealthy
    pub error: Option<String>,

    /// Additional status information
    pub details: HashMap<String, String>,

    /// Timestamp of the health check
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Source manager for handling multiple download sources
pub struct SourceManager {
    sources: Vec<Box<dyn DownloadSource>>,
}

impl std::fmt::Debug for SourceManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SourceManager")
            .field("sources_count", &self.sources.len())
            .finish()
    }
}

impl SourceManager {
    /// Create a new source manager
    pub fn new() -> Self {
        Self {
            sources: Vec::new(),
        }
    }

    /// Add a source to the manager
    pub fn add_source(&mut self, source: Box<dyn DownloadSource>) {
        self.sources.push(source);
        // Sort by priority (lower is higher priority)
        self.sources.sort_by_key(|s| s.priority());
    }

    /// Get all enabled sources
    pub fn enabled_sources(&self) -> Vec<&dyn DownloadSource> {
        self.sources
            .iter()
            .filter(|s| s.is_enabled())
            .map(|s| s.as_ref())
            .collect()
    }

    /// Find sources that can handle a specific repository
    pub async fn find_sources_for_repository(
        &self,
        repository: &str,
        version: &str,
        file_name: &str,
    ) -> Vec<&dyn DownloadSource> {
        let mut compatible_sources = Vec::new();

        for source in self.enabled_sources() {
            if source.can_handle(repository, version, file_name).await {
                compatible_sources.push(source);
            }
        }

        compatible_sources
    }

    /// Get download URLs from all compatible sources
    pub async fn get_all_download_urls(
        &self,
        repository: &str,
        version: &str,
        file_name: &str,
    ) -> Result<Vec<DownloadUrl>> {
        let sources = self
            .find_sources_for_repository(repository, version, file_name)
            .await;

        if sources.is_empty() {
            return Err(TurboCdnError::source_validation(
                "No compatible sources found for the requested file",
            ));
        }

        let mut all_urls = Vec::new();

        for source in sources {
            match source
                .get_download_urls(repository, version, file_name)
                .await
            {
                Ok(mut urls) => {
                    all_urls.append(&mut urls);
                }
                Err(e) => {
                    tracing::warn!("Source {} failed to provide URLs: {}", source.name(), e);
                }
            }
        }

        if all_urls.is_empty() {
            return Err(TurboCdnError::source_validation(
                "No download URLs could be obtained from any source",
            ));
        }

        // Sort URLs by priority
        all_urls.sort_by_key(|url| url.priority);

        Ok(all_urls)
    }

    /// Perform health checks on all sources
    pub async fn health_check_all(&self) -> HashMap<String, HealthStatus> {
        let mut results = HashMap::new();

        for source in &self.sources {
            let status = source
                .health_check()
                .await
                .unwrap_or_else(|e| HealthStatus {
                    healthy: false,
                    response_time: 0,
                    error: Some(e.to_string()),
                    details: HashMap::new(),
                    timestamp: chrono::Utc::now(),
                });

            results.insert(source.name().to_string(), status);
        }

        results
    }
}

impl Default for SourceManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Utility functions for URL manipulation
pub mod utils {
    use super::*;

    /// Parse a repository string into owner and name
    pub fn parse_repository(repository: &str) -> Result<(String, String)> {
        // Handle different repository formats
        let repo = repository
            .trim_start_matches("https://github.com/")
            .trim_start_matches("http://github.com/")
            .trim_end_matches(".git")
            .trim_end_matches('/');

        let parts: Vec<&str> = repo.split('/').collect();
        if parts.len() != 2 {
            return Err(TurboCdnError::source_validation(
                "Repository must be in format 'owner/name'",
            ));
        }

        Ok((parts[0].to_string(), parts[1].to_string()))
    }

    /// Validate a URL
    pub fn validate_url(url: &str) -> Result<Url> {
        Url::parse(url)
            .map_err(|e| TurboCdnError::source_validation(format!("Invalid URL '{}': {}", url, e)))
    }

    /// Extract file extension from a file name
    pub fn get_file_extension(file_name: &str) -> Option<&str> {
        std::path::Path::new(file_name)
            .extension()
            .and_then(|ext| ext.to_str())
    }

    /// Check if a file is a binary file based on its extension
    pub fn is_binary_file(file_name: &str) -> bool {
        let binary_extensions = [
            "exe", "dll", "so", "dylib", "bin", "app", "deb", "rpm", "msi", "zip", "tar", "gz",
            "bz2", "xz", "7z", "rar", "jar", "war", "ear", "dmg", "pkg", "appx",
        ];

        if let Some(ext) = get_file_extension(file_name) {
            binary_extensions.contains(&ext.to_lowercase().as_str())
        } else {
            false
        }
    }
}
