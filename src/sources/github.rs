// Licensed under the MIT License
// Copyright (c) 2025 Hal <hal.long@outlook.com>

use async_trait::async_trait;
use serde::Deserialize;
use std::collections::HashMap;
use tracing::{debug, warn};

use super::{
    DownloadSource, DownloadUrl, FileInfo, HealthStatus, RepositoryMetadata, RepositoryStats,
};
use crate::config::GitHubConfig;
use crate::error::{Result, TurboCdnError};

/// GitHub download source
#[derive(Debug)]
pub struct GitHubSource {
    config: GitHubConfig,
    client: reqwest::Client,
}

/// GitHub API response for repository information
#[derive(Debug, Deserialize)]
struct GitHubRepository {
    name: String,
    description: Option<String>,
    html_url: String,
    license: Option<GitHubLicense>,
    stargazers_count: u64,
    forks_count: u64,
    watchers_count: u64,
    open_issues_count: u64,
    size: u64,
    updated_at: String,
}

/// GitHub license information
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct GitHubLicense {
    name: String,
    spdx_id: Option<String>,
}

/// GitHub API response for releases
#[derive(Debug, Clone, Deserialize)]
struct GitHubRelease {
    tag_name: String,
    name: Option<String>,
    published_at: String,
    assets: Vec<GitHubAsset>,
}

/// GitHub release asset
#[derive(Debug, Clone, Deserialize)]
struct GitHubAsset {
    name: String,
    size: u64,
    download_count: u64,
    browser_download_url: String,
    content_type: String,
}

/// GitHub API response for tags
#[derive(Debug, Deserialize)]
struct GitHubTag {
    name: String,
}

impl GitHubSource {
    /// Create a new GitHub source
    pub fn new(config: GitHubConfig) -> Result<Self> {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            reqwest::header::USER_AGENT,
            reqwest::header::HeaderValue::from_static("turbo-cdn"),
        );

        if let Some(token) = &config.token {
            let auth_value = format!("token {}", token);
            headers.insert(
                reqwest::header::AUTHORIZATION,
                reqwest::header::HeaderValue::from_str(&auth_value)
                    .map_err(|e| TurboCdnError::config(format!("Invalid GitHub token: {}", e)))?,
            );
        }

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| TurboCdnError::config(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self { config, client })
    }

    /// Parse repository string into owner and name
    fn parse_repository(&self, repository: &str) -> Result<(String, String)> {
        super::utils::parse_repository(repository)
    }

    /// Get repository information from GitHub API
    async fn get_repository_info(&self, owner: &str, repo: &str) -> Result<GitHubRepository> {
        let url = format!("{}/repos/{}/{}", self.config.api_base_url, owner, repo);

        debug!("Fetching repository info from: {}", url);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(TurboCdnError::Network)?;

        if !response.status().is_success() {
            return Err(TurboCdnError::source_validation(format!(
                "GitHub API returned status {}: {}",
                response.status(),
                response.text().await.unwrap_or_default()
            )));
        }

        let repo_info: GitHubRepository = response.json().await.map_err(|e| {
            TurboCdnError::source_validation(format!("Failed to parse GitHub API response: {}", e))
        })?;

        Ok(repo_info)
    }

    /// Get releases for a repository
    async fn get_releases(&self, owner: &str, repo: &str) -> Result<Vec<GitHubRelease>> {
        let url = format!(
            "{}/repos/{}/{}/releases",
            self.config.api_base_url, owner, repo
        );

        debug!("Fetching releases from: {}", url);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(TurboCdnError::Network)?;

        if !response.status().is_success() {
            return Err(TurboCdnError::source_validation(format!(
                "GitHub API returned status {}: {}",
                response.status(),
                response.text().await.unwrap_or_default()
            )));
        }

        let releases: Vec<GitHubRelease> = response.json().await.map_err(|e| {
            TurboCdnError::source_validation(format!("Failed to parse GitHub releases: {}", e))
        })?;

        Ok(releases)
    }

    /// Get tags for a repository
    async fn get_tags(&self, owner: &str, repo: &str) -> Result<Vec<GitHubTag>> {
        let url = format!("{}/repos/{}/{}/tags", self.config.api_base_url, owner, repo);

        debug!("Fetching tags from: {}", url);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(TurboCdnError::Network)?;

        if !response.status().is_success() {
            return Err(TurboCdnError::source_validation(format!(
                "GitHub API returned status {}: {}",
                response.status(),
                response.text().await.unwrap_or_default()
            )));
        }

        let tags: Vec<GitHubTag> = response.json().await.map_err(|e| {
            TurboCdnError::source_validation(format!("Failed to parse GitHub tags: {}", e))
        })?;

        Ok(tags)
    }

    /// Find a release by version
    async fn find_release(&self, owner: &str, repo: &str, version: &str) -> Result<GitHubRelease> {
        let releases = self.get_releases(owner, repo).await?;

        // Try to find exact match first
        for release in &releases {
            if release.tag_name == version {
                return Ok(release.clone());
            }
        }

        // Try to find match with 'v' prefix
        let version_with_v = if version.starts_with('v') {
            version.to_string()
        } else {
            format!("v{}", version)
        };

        for release in &releases {
            if release.tag_name == version_with_v {
                return Ok(release.clone());
            }
        }

        // If no exact match, try partial match
        for release in &releases {
            if release.tag_name.contains(version) {
                warn!(
                    "Using partial match for version '{}': found '{}'",
                    version, release.tag_name
                );
                return Ok(release.clone());
            }
        }

        Err(TurboCdnError::source_validation(format!(
            "Version '{}' not found in repository {}/{}",
            version, owner, repo
        )))
    }
}

#[async_trait]
impl DownloadSource for GitHubSource {
    fn name(&self) -> &str {
        "github"
    }

    fn priority(&self) -> u8 {
        self.config.priority.min(255) as u8
    }

    fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    async fn can_handle(&self, repository: &str, _version: &str, _file_name: &str) -> bool {
        // GitHub source can handle any repository that looks like a GitHub repository
        repository.contains("github.com") || self.parse_repository(repository).is_ok()
    }

    async fn get_download_urls(
        &self,
        repository: &str,
        version: &str,
        file_name: &str,
    ) -> Result<Vec<DownloadUrl>> {
        let (owner, repo) = self.parse_repository(repository)?;
        let release = self.find_release(&owner, &repo, version).await?;

        let mut urls = Vec::new();

        // Find the requested file in release assets
        for asset in &release.assets {
            if asset.name == file_name {
                let mut metadata = HashMap::new();
                metadata.insert(
                    "release_name".to_string(),
                    release.name.clone().unwrap_or_default(),
                );
                metadata.insert("published_at".to_string(), release.published_at.clone());
                metadata.insert(
                    "download_count".to_string(),
                    asset.download_count.to_string(),
                );

                // Add original GitHub URL
                urls.push(DownloadUrl {
                    url: asset.browser_download_url.clone(),
                    source: self.name().to_string(),
                    priority: self.priority(),
                    size: Some(asset.size),
                    checksum: None, // GitHub doesn't provide checksums in API
                    metadata: metadata.clone(),
                    supports_ranges: true, // GitHub supports range requests
                    estimated_latency: None,
                });

                // Add GitHub releases mirror URLs for better download speed
                let mirror_urls = vec![
                    format!("https://ghfast.top/{}", asset.browser_download_url),
                    format!("https://mirror.ghproxy.com/{}", asset.browser_download_url),
                    format!("https://ghproxy.net/{}", asset.browser_download_url),
                ];

                for (index, mirror_url) in mirror_urls.iter().enumerate() {
                    let mut mirror_metadata = metadata.clone();
                    mirror_metadata.insert("mirror_type".to_string(), "github_releases".to_string());
                    mirror_metadata.insert("mirror_index".to_string(), index.to_string());

                    urls.push(DownloadUrl {
                        url: mirror_url.clone(),
                        source: format!("{}_mirror_{}", self.name(), index + 1),
                        priority: self.priority() + 10 + (index as u8), // Lower priority than original
                        size: Some(asset.size),
                        checksum: None,
                        metadata: mirror_metadata,
                        supports_ranges: true,
                        estimated_latency: Some(100), // Assume mirrors might be faster
                    });
                }
                break;
            }
        }

        if urls.is_empty() {
            return Err(TurboCdnError::source_validation(format!(
                "File '{}' not found in release '{}' of repository {}/{}",
                file_name, version, owner, repo
            )));
        }

        Ok(urls)
    }

    async fn get_repository_metadata(&self, repository: &str) -> Result<RepositoryMetadata> {
        let (owner, repo) = self.parse_repository(repository)?;
        let repo_info = self.get_repository_info(&owner, &repo).await?;

        // Get releases and tags
        let releases = self.get_releases(&owner, &repo).await.unwrap_or_default();
        let tags = self.get_tags(&owner, &repo).await.unwrap_or_default();

        // Combine versions from releases and tags
        let mut versions = Vec::new();
        for release in &releases {
            versions.push(release.tag_name.clone());
        }
        for tag in &tags {
            if !versions.contains(&tag.name) {
                versions.push(tag.name.clone());
            }
        }

        // Get files from the latest release
        let files = if let Some(latest_release) = releases.first() {
            latest_release
                .assets
                .iter()
                .map(|asset| FileInfo {
                    name: asset.name.clone(),
                    size: asset.size,
                    checksum: None,
                    content_type: Some(asset.content_type.clone()),
                    download_count: Some(asset.download_count),
                })
                .collect()
        } else {
            Vec::new()
        };

        let stats = RepositoryStats {
            stars: Some(repo_info.stargazers_count),
            forks: Some(repo_info.forks_count),
            watchers: Some(repo_info.watchers_count),
            open_issues: Some(repo_info.open_issues_count),
            size: Some(repo_info.size),
        };

        let last_updated = chrono::DateTime::parse_from_rfc3339(&repo_info.updated_at)
            .map(|dt| dt.with_timezone(&chrono::Utc))
            .unwrap_or_else(|_| chrono::Utc::now());

        Ok(RepositoryMetadata {
            name: repo_info.name,
            description: repo_info.description,
            url: repo_info.html_url,
            license: repo_info.license.map(|l| l.name),
            versions,
            files,
            stats,
            last_updated,
        })
    }

    async fn health_check(&self) -> Result<HealthStatus> {
        let start_time = std::time::Instant::now();
        let url = format!("{}/rate_limit", self.config.api_base_url);

        let response = self.client.get(&url).send().await;

        let response_time = start_time.elapsed().as_millis() as u64;

        match response {
            Ok(resp) if resp.status().is_success() => {
                let mut details = HashMap::new();
                details.insert("status_code".to_string(), resp.status().to_string());

                Ok(HealthStatus {
                    healthy: true,
                    response_time,
                    error: None,
                    details,
                    timestamp: chrono::Utc::now(),
                })
            }
            Ok(resp) => {
                let mut details = HashMap::new();
                details.insert("status_code".to_string(), resp.status().to_string());

                Ok(HealthStatus {
                    healthy: false,
                    response_time,
                    error: Some(format!("HTTP {}", resp.status())),
                    details,
                    timestamp: chrono::Utc::now(),
                })
            }
            Err(e) => Ok(HealthStatus {
                healthy: false,
                response_time,
                error: Some(e.to_string()),
                details: HashMap::new(),
                timestamp: chrono::Utc::now(),
            }),
        }
    }
}
