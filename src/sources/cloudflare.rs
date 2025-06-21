// Licensed under the MIT License
// Copyright (c) 2025 Hal <hal.long@outlook.com>

use async_trait::async_trait;
use serde::Deserialize;
use std::collections::HashMap;
use tracing::debug;

use super::{
    DownloadSource, DownloadUrl, FileInfo, HealthStatus, RepositoryMetadata, RepositoryStats,
};
use crate::config::CloudflareConfig;
use crate::error::{Result, TurboCdnError};

/// Cloudflare CDN source (via cdnjs)
#[derive(Debug)]
pub struct CloudflareSource {
    config: CloudflareConfig,
    client: reqwest::Client,
}

/// CDNJS API response for library search
#[derive(Debug, Deserialize)]
struct CdnjsSearchResponse {
    results: Vec<CdnjsLibrary>,
}

/// CDNJS library information
#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
struct CdnjsLibrary {
    name: String,
    description: Option<String>,
    version: String,
    homepage: Option<String>,
    repository: Option<CdnjsRepository>,
    #[serde(rename = "autoupdate")]
    auto_update: Option<CdnjsAutoUpdate>,
}

/// CDNJS repository information
#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
struct CdnjsRepository {
    #[serde(rename = "type")]
    repo_type: String,
    url: String,
}

/// CDNJS auto-update information
#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
struct CdnjsAutoUpdate {
    source: String,
    target: String,
}

/// CDNJS library details
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct CdnjsLibraryDetails {
    name: String,
    description: Option<String>,
    version: String,
    homepage: Option<String>,
    repository: Option<CdnjsRepository>,
    versions: Vec<String>,
    files: Vec<String>,
}

impl CloudflareSource {
    /// Create a new Cloudflare source
    pub fn new(config: CloudflareConfig) -> Result<Self> {
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

    /// Search for a library in CDNJS
    async fn search_library(&self, query: &str) -> Result<Vec<CdnjsLibrary>> {
        let url = format!("https://api.cdnjs.com/libraries?search={}&limit=10", query);

        debug!("Searching CDNJS for: {}", query);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(TurboCdnError::Network)?;

        if !response.status().is_success() {
            return Err(TurboCdnError::source_validation(format!(
                "CDNJS API returned status {}: {}",
                response.status(),
                response.text().await.unwrap_or_default()
            )));
        }

        let search_response: CdnjsSearchResponse = response.json().await.map_err(|e| {
            TurboCdnError::source_validation(format!(
                "Failed to parse CDNJS search response: {}",
                e
            ))
        })?;

        Ok(search_response.results)
    }

    /// Get library details from CDNJS
    async fn get_library_details(&self, library_name: &str) -> Result<CdnjsLibraryDetails> {
        let url = format!("https://api.cdnjs.com/libraries/{}", library_name);

        debug!("Fetching CDNJS library details: {}", url);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(TurboCdnError::Network)?;

        if !response.status().is_success() {
            return Err(TurboCdnError::source_validation(format!(
                "CDNJS API returned status {}: {}",
                response.status(),
                response.text().await.unwrap_or_default()
            )));
        }

        let library_details: CdnjsLibraryDetails = response.json().await.map_err(|e| {
            TurboCdnError::source_validation(format!(
                "Failed to parse CDNJS library details: {}",
                e
            ))
        })?;

        Ok(library_details)
    }

    /// Build CDNJS URL
    fn build_cdn_url(&self, library_name: &str, version: &str, file_path: &str) -> String {
        format!(
            "{}/ajax/libs/{}/{}/{}",
            self.config.base_url, library_name, version, file_path
        )
    }

    /// Check if a file exists at the given URL
    async fn check_file_exists(&self, url: &str) -> Result<Option<u64>> {
        debug!("Checking file existence at: {}", url);

        let response = self
            .client
            .head(url)
            .send()
            .await
            .map_err(TurboCdnError::Network)?;

        if response.status().is_success() {
            let size = response
                .headers()
                .get(reqwest::header::CONTENT_LENGTH)
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.parse().ok());
            Ok(size)
        } else {
            Ok(None)
        }
    }

    /// Find matching library for repository
    async fn find_library_for_repository(&self, repository: &str) -> Result<Option<CdnjsLibrary>> {
        let (_, repo_name) = self.parse_repository(repository)?;

        // Search for libraries that might match this repository
        let search_results = self.search_library(&repo_name).await?;

        // Try to find exact match or close match
        for library in &search_results {
            if let Some(repo_info) = &library.repository {
                if repo_info.url.contains(repository) || repository.contains(&library.name) {
                    return Ok(Some(library.clone()));
                }
            }

            // Also check if the library name matches the repository name
            if library.name.to_lowercase() == repo_name.to_lowercase() {
                return Ok(Some(library.clone()));
            }
        }

        Ok(None)
    }
}

#[async_trait]
impl DownloadSource for CloudflareSource {
    fn name(&self) -> &str {
        "cloudflare"
    }

    fn priority(&self) -> u8 {
        self.config.priority.min(255) as u8
    }

    fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    async fn can_handle(&self, repository: &str, _version: &str, _file_name: &str) -> bool {
        // Cloudflare (CDNJS) can handle popular JavaScript libraries
        // We'll check if we can find a matching library
        if let Ok(library) = self.find_library_for_repository(repository).await {
            library.is_some()
        } else {
            false
        }
    }

    async fn get_download_urls(
        &self,
        repository: &str,
        version: &str,
        file_name: &str,
    ) -> Result<Vec<DownloadUrl>> {
        let library = self
            .find_library_for_repository(repository)
            .await?
            .ok_or_else(|| {
                TurboCdnError::source_validation(format!(
                    "No matching library found in CDNJS for repository: {}",
                    repository
                ))
            })?;

        let library_details = self.get_library_details(&library.name).await?;

        // Check if the requested version exists
        if !library_details.versions.contains(&version.to_string()) {
            return Err(TurboCdnError::source_validation(format!(
                "Version '{}' not found for library '{}' in CDNJS",
                version, library.name
            )));
        }

        let cdn_url = self.build_cdn_url(&library.name, version, file_name);

        // Check if the file exists
        if let Some(size) = self.check_file_exists(&cdn_url).await? {
            let mut metadata = HashMap::new();
            metadata.insert("cdn".to_string(), "cloudflare".to_string());
            metadata.insert("library_name".to_string(), library.name.clone());
            metadata.insert("provider".to_string(), "cdnjs".to_string());

            Ok(vec![DownloadUrl {
                url: cdn_url,
                source: self.name().to_string(),
                priority: self.priority(),
                size: Some(size),
                checksum: None,
                metadata,
                supports_ranges: true, // Cloudflare supports range requests
                estimated_latency: None,
            }])
        } else {
            Err(TurboCdnError::source_validation(format!(
                "File '{}' not found in CDNJS for library '{}' version {}",
                file_name, library.name, version
            )))
        }
    }

    async fn get_repository_metadata(&self, repository: &str) -> Result<RepositoryMetadata> {
        let library = self
            .find_library_for_repository(repository)
            .await?
            .ok_or_else(|| {
                TurboCdnError::source_validation(format!(
                    "No matching library found in CDNJS for repository: {}",
                    repository
                ))
            })?;

        let library_details = self.get_library_details(&library.name).await?;

        let files = library_details
            .files
            .iter()
            .map(|file_name| FileInfo {
                name: file_name.clone(),
                size: 0, // CDNJS doesn't provide file sizes in the API
                checksum: None,
                content_type: None,
                download_count: None,
            })
            .collect();

        let stats = RepositoryStats {
            stars: None,
            forks: None,
            watchers: None,
            open_issues: None,
            size: None,
        };

        Ok(RepositoryMetadata {
            name: library_details.name,
            description: library_details.description,
            url: library_details
                .homepage
                .unwrap_or_else(|| repository.to_string()),
            license: None,
            versions: library_details.versions,
            files,
            stats,
            last_updated: chrono::Utc::now(),
        })
    }

    async fn health_check(&self) -> Result<HealthStatus> {
        let start_time = std::time::Instant::now();
        let url = format!(
            "{}/ajax/libs/jquery/3.6.0/jquery.min.js",
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
