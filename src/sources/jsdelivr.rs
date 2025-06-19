// Licensed under the MIT License
// Copyright (c) 2025 Hal <hal.long@outlook.com>

use async_trait::async_trait;
use serde::Deserialize;
use std::collections::HashMap;
use tracing::debug;

use super::{
    DownloadSource, DownloadUrl, FileInfo, HealthStatus, RepositoryMetadata, RepositoryStats,
};
use crate::config::JsDelivrConfig;
use crate::error::{Result, TurboCdnError};

/// jsDelivr CDN source
#[derive(Debug)]
pub struct JsDelivrSource {
    config: JsDelivrConfig,
    client: reqwest::Client,
}

/// jsDelivr API response for package information
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct JsDelivrPackage {
    #[serde(rename = "type")]
    package_type: String,
    name: String,
    description: Option<String>,
    versions: Vec<JsDelivrVersion>,
}

/// jsDelivr version information
#[derive(Debug, Deserialize)]
struct JsDelivrVersion {
    version: String,
    files: Vec<JsDelivrFile>,
}

/// jsDelivr file information
#[derive(Debug, Deserialize)]
struct JsDelivrFile {
    name: String,
    hash: String,
    size: u64,
}

impl JsDelivrSource {
    /// Create a new jsDelivr source
    pub fn new(config: JsDelivrConfig) -> Result<Self> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| TurboCdnError::config(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self { config, client })
    }

    /// Parse repository string into owner and name
    fn parse_repository(&self, repository: &str) -> Result<(String, String)> {
        super::utils::parse_repository(repository)
    }

    /// Get package information from jsDelivr API
    async fn get_package_info(&self, owner: &str, repo: &str) -> Result<JsDelivrPackage> {
        let url = format!("https://data.jsdelivr.com/v1/package/gh/{}/{}", owner, repo);

        debug!("Fetching package info from jsDelivr: {}", url);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(TurboCdnError::Network)?;

        if !response.status().is_success() {
            return Err(TurboCdnError::source_validation(format!(
                "jsDelivr API returned status {}: {}",
                response.status(),
                response.text().await.unwrap_or_default()
            )));
        }

        let package_info: JsDelivrPackage = response.json().await.map_err(|e| {
            TurboCdnError::source_validation(format!(
                "Failed to parse jsDelivr API response: {}",
                e
            ))
        })?;

        Ok(package_info)
    }

    /// Get files for a specific version
    async fn get_version_files(
        &self,
        owner: &str,
        repo: &str,
        version: &str,
    ) -> Result<Vec<JsDelivrFile>> {
        let url = format!(
            "https://data.jsdelivr.com/v1/package/gh/{}/{}@{}/flat",
            owner, repo, version
        );

        debug!("Fetching version files from jsDelivr: {}", url);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(TurboCdnError::Network)?;

        if !response.status().is_success() {
            return Err(TurboCdnError::source_validation(format!(
                "jsDelivr API returned status {}: {}",
                response.status(),
                response.text().await.unwrap_or_default()
            )));
        }

        #[derive(Deserialize)]
        struct FilesResponse {
            files: Vec<JsDelivrFile>,
        }

        let files_response: FilesResponse = response.json().await.map_err(|e| {
            TurboCdnError::source_validation(format!(
                "Failed to parse jsDelivr files response: {}",
                e
            ))
        })?;

        Ok(files_response.files)
    }

    /// Build jsDelivr CDN URL
    fn build_cdn_url(&self, owner: &str, repo: &str, version: &str, file_path: &str) -> String {
        format!(
            "{}/gh/{}/{}@{}/{}",
            self.config.base_url, owner, repo, version, file_path
        )
    }
}

#[async_trait]
impl DownloadSource for JsDelivrSource {
    fn name(&self) -> &str {
        "jsdelivr"
    }

    fn priority(&self) -> u8 {
        self.config.priority
    }

    fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    async fn can_handle(&self, repository: &str, _version: &str, _file_name: &str) -> bool {
        // jsDelivr can handle GitHub repositories
        repository.contains("github.com") || self.parse_repository(repository).is_ok()
    }

    async fn get_download_urls(
        &self,
        repository: &str,
        version: &str,
        file_name: &str,
    ) -> Result<Vec<DownloadUrl>> {
        let (owner, repo) = self.parse_repository(repository)?;

        // Try to get files for the specific version
        let files = self.get_version_files(&owner, &repo, version).await?;

        let mut urls = Vec::new();

        // Look for the requested file
        for file in &files {
            if file.name == file_name || file.name.ends_with(&format!("/{}", file_name)) {
                let cdn_url = self.build_cdn_url(&owner, &repo, version, &file.name);

                let mut metadata = HashMap::new();
                metadata.insert("hash".to_string(), file.hash.clone());
                metadata.insert("cdn".to_string(), "jsdelivr".to_string());

                urls.push(DownloadUrl {
                    url: cdn_url,
                    source: self.name().to_string(),
                    priority: self.priority(),
                    size: Some(file.size),
                    checksum: None, // jsDelivr provides hashes but not in standard format
                    metadata,
                    supports_ranges: true, // jsDelivr supports range requests
                    estimated_latency: None,
                });
            }
        }

        // If no exact match found, try building URL directly
        if urls.is_empty() {
            let cdn_url = self.build_cdn_url(&owner, &repo, version, file_name);

            // Test if the URL exists
            let head_response = self
                .client
                .head(&cdn_url)
                .send()
                .await
                .map_err(TurboCdnError::Network)?;

            if head_response.status().is_success() {
                let size = head_response
                    .headers()
                    .get(reqwest::header::CONTENT_LENGTH)
                    .and_then(|v| v.to_str().ok())
                    .and_then(|s| s.parse().ok());

                let mut metadata = HashMap::new();
                metadata.insert("cdn".to_string(), "jsdelivr".to_string());
                metadata.insert("direct_url".to_string(), "true".to_string());

                urls.push(DownloadUrl {
                    url: cdn_url,
                    source: self.name().to_string(),
                    priority: self.priority(),
                    size,
                    checksum: None,
                    metadata,
                    supports_ranges: true,
                    estimated_latency: None,
                });
            }
        }

        if urls.is_empty() {
            return Err(TurboCdnError::source_validation(format!(
                "File '{}' not found in jsDelivr for repository {}/{} version {}",
                file_name, owner, repo, version
            )));
        }

        Ok(urls)
    }

    async fn get_repository_metadata(&self, repository: &str) -> Result<RepositoryMetadata> {
        let (owner, repo) = self.parse_repository(repository)?;
        let package_info = self.get_package_info(&owner, &repo).await?;

        let versions: Vec<String> = package_info
            .versions
            .iter()
            .map(|v| v.version.clone())
            .collect();

        // Get files from the latest version
        let files = if let Some(latest_version) = package_info.versions.first() {
            latest_version
                .files
                .iter()
                .map(|file| FileInfo {
                    name: file.name.clone(),
                    size: file.size,
                    checksum: None,
                    content_type: None,
                    download_count: None,
                })
                .collect()
        } else {
            Vec::new()
        };

        let stats = RepositoryStats {
            stars: None,
            forks: None,
            watchers: None,
            open_issues: None,
            size: None,
        };

        Ok(RepositoryMetadata {
            name: package_info.name,
            description: package_info.description,
            url: format!("https://github.com/{}/{}", owner, repo),
            license: None,
            versions,
            files,
            stats,
            last_updated: chrono::Utc::now(),
        })
    }

    async fn health_check(&self) -> Result<HealthStatus> {
        let start_time = std::time::Instant::now();
        let url = format!(
            "{}/gh/jquery/jquery@3.6.0/dist/jquery.min.js",
            self.config.base_url
        );

        let response = self.client.head(&url).send().await;

        let response_time = start_time.elapsed().as_millis() as u64;

        match response {
            Ok(resp) if resp.status().is_success() => {
                let mut details = HashMap::new();
                details.insert("status_code".to_string(), resp.status().to_string());
                details.insert("test_url".to_string(), url);

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
                details.insert("test_url".to_string(), url);

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
