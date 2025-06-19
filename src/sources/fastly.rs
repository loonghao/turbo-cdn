use async_trait::async_trait;
use std::collections::HashMap;
use tracing::debug;

use super::{
    DownloadSource, DownloadUrl, FileInfo, HealthStatus, RepositoryMetadata, RepositoryStats,
};
use crate::config::FastlyConfig;
use crate::error::{Result, TurboCdnError};

/// Fastly CDN source (via jsDelivr Fastly endpoint)
#[derive(Debug)]
pub struct FastlySource {
    config: FastlyConfig,
    client: reqwest::Client,
}

impl FastlySource {
    /// Create a new Fastly source
    pub fn new(config: FastlyConfig) -> Result<Self> {
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

    /// Build Fastly CDN URL
    fn build_cdn_url(&self, owner: &str, repo: &str, version: &str, file_path: &str) -> String {
        format!(
            "{}/gh/{}/{}@{}/{}",
            self.config.base_url, owner, repo, version, file_path
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
            .map_err(|e| TurboCdnError::Network(e))?;

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
}

#[async_trait]
impl DownloadSource for FastlySource {
    fn name(&self) -> &str {
        "fastly"
    }

    fn priority(&self) -> u8 {
        self.config.priority
    }

    fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    async fn can_handle(&self, repository: &str, _version: &str, _file_name: &str) -> bool {
        // Fastly can handle GitHub repositories via jsDelivr
        repository.contains("github.com") || self.parse_repository(repository).is_ok()
    }

    async fn get_download_urls(
        &self,
        repository: &str,
        version: &str,
        file_name: &str,
    ) -> Result<Vec<DownloadUrl>> {
        let (owner, repo) = self.parse_repository(repository)?;
        let cdn_url = self.build_cdn_url(&owner, &repo, version, file_name);

        // Check if the file exists
        if let Some(size) = self.check_file_exists(&cdn_url).await? {
            let mut metadata = HashMap::new();
            metadata.insert("cdn".to_string(), "fastly".to_string());
            metadata.insert("provider".to_string(), "jsdelivr-fastly".to_string());

            Ok(vec![DownloadUrl {
                url: cdn_url,
                source: self.name().to_string(),
                priority: self.priority(),
                size: Some(size),
                checksum: None,
                metadata,
                supports_ranges: true, // Fastly supports range requests
                estimated_latency: None,
            }])
        } else {
            Err(TurboCdnError::source_validation(format!(
                "File '{}' not found in Fastly CDN for repository {}/{} version {}",
                file_name, owner, repo, version
            )))
        }
    }

    async fn get_repository_metadata(&self, repository: &str) -> Result<RepositoryMetadata> {
        let (owner, repo) = self.parse_repository(repository)?;

        // Fastly doesn't provide repository metadata directly
        // Return minimal metadata
        Ok(RepositoryMetadata {
            name: repo.clone(),
            description: None,
            url: format!("https://github.com/{}/{}", owner, repo),
            license: None,
            versions: Vec::new(),
            files: Vec::new(),
            stats: RepositoryStats {
                stars: None,
                forks: None,
                watchers: None,
                open_issues: None,
                size: None,
            },
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
