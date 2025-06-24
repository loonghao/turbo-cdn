// Licensed under the MIT License
// Copyright (c) 2025 Hal <hal.long@outlook.com>

//! # Mirror Manager
//!
//! Manages GitHub releases mirrors based on configuration and user region.

use crate::config::{ReleasesMirrorConfig, ReleasesMirrorSource, Region};
use crate::error::{Result as TurboCdnResult, TurboCdnError};
use crate::sources::DownloadUrl;
use std::collections::HashMap;
use tracing::{debug, info, warn};

/// Mirror manager for GitHub releases
#[derive(Debug, Clone)]
pub struct MirrorManager {
    config: ReleasesMirrorConfig,
    user_region: Region,
    health_status: HashMap<String, bool>,
}

impl MirrorManager {
    /// Create a new mirror manager
    pub fn new(config: ReleasesMirrorConfig, user_region: Region) -> Self {
        Self {
            config,
            user_region,
            health_status: HashMap::new(),
        }
    }

    /// Generate mirror URLs for a GitHub releases URL
    pub async fn generate_mirror_urls(
        &self,
        original_url: &str,
        original_download_url: &DownloadUrl,
    ) -> TurboCdnResult<Vec<DownloadUrl>> {
        if !self.config.enabled {
            debug!("GitHub releases mirrors are disabled");
            return Ok(vec![]);
        }

        let mut mirror_urls = Vec::new();
        let user_region_str = self.user_region.to_string();

        // Filter and sort mirrors by region and priority
        let mut applicable_mirrors: Vec<&ReleasesMirrorSource> = self
            .config
            .sources
            .iter()
            .filter(|mirror| {
                mirror.enabled
                    && (mirror.regions.contains(&user_region_str)
                        || mirror.regions.contains(&"Global".to_string()))
            })
            .collect();

        // Sort by priority (lower number = higher priority)
        applicable_mirrors.sort_by_key(|mirror| mirror.priority);

        info!(
            "Found {} applicable mirrors for region: {}",
            applicable_mirrors.len(),
            user_region_str
        );

        for (_index, mirror) in applicable_mirrors.iter().enumerate() {
            // Check health status if health checks are enabled
            if self.config.health_check_enabled {
                if let Some(is_healthy) = self.health_status.get(&mirror.name) {
                    if !is_healthy {
                        warn!("Skipping unhealthy mirror: {}", mirror.name);
                        continue;
                    }
                }
            }

            // Generate mirror URL
            let mirror_url = mirror.url_template.replace("{original_url}", original_url);

            // Create metadata
            let mut metadata = original_download_url.metadata.clone();
            metadata.insert("mirror_type".to_string(), "github_releases".to_string());
            metadata.insert("mirror_name".to_string(), mirror.name.clone());
            metadata.insert("mirror_region".to_string(), user_region_str.clone());
            metadata.insert("mirror_priority".to_string(), mirror.priority.to_string());

            // Calculate priority (original has priority 0, mirrors have higher numbers)
            let mirror_priority = original_download_url.priority + mirror.priority as u8;

            let download_url = DownloadUrl {
                url: mirror_url,
                source: format!("{}_mirror_{}", original_download_url.source, mirror.name),
                priority: mirror_priority,
                size: original_download_url.size,
                checksum: original_download_url.checksum.clone(),
                metadata,
                supports_ranges: true, // Assume mirrors support range requests
                estimated_latency: mirror.estimated_latency_ms,
            };

            debug!(
                "Generated mirror URL: {} (priority: {}, mirror: {})",
                download_url.url, download_url.priority, mirror.name
            );

            mirror_urls.push(download_url);
        }

        info!("Generated {} mirror URLs", mirror_urls.len());
        Ok(mirror_urls)
    }

    /// Perform health checks on all configured mirrors
    pub async fn health_check(&mut self) -> TurboCdnResult<()> {
        if !self.config.health_check_enabled {
            debug!("Health checks are disabled");
            return Ok(());
        }

        info!("Starting health checks for {} mirrors", self.config.sources.len());

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_millis(
                self.config.health_check_timeout_ms,
            ))
            .build()
            .map_err(|e| TurboCdnError::network(format!("Failed to create HTTP client: {}", e)))?;

        for mirror in &self.config.sources {
            if !mirror.enabled {
                continue;
            }

            let default_health_url = mirror.url_template.replace("{original_url}", "");
            let health_url = mirror
                .health_check_url
                .as_ref()
                .unwrap_or(&default_health_url);

            debug!("Health checking mirror: {} at {}", mirror.name, health_url);

            let is_healthy = match client.head(health_url).send().await {
                Ok(response) => {
                    let is_ok = response.status().is_success();
                    if is_ok {
                        debug!("Mirror {} is healthy", mirror.name);
                    } else {
                        warn!(
                            "Mirror {} returned status: {}",
                            mirror.name,
                            response.status()
                        );
                    }
                    is_ok
                }
                Err(e) => {
                    warn!("Mirror {} health check failed: {}", mirror.name, e);
                    false
                }
            };

            self.health_status.insert(mirror.name.clone(), is_healthy);
        }

        let healthy_count = self.health_status.values().filter(|&&h| h).count();
        info!(
            "Health check completed: {}/{} mirrors are healthy",
            healthy_count,
            self.health_status.len()
        );

        Ok(())
    }

    /// Get health status of all mirrors
    pub fn get_health_status(&self) -> &HashMap<String, bool> {
        &self.health_status
    }

    /// Check if auto-selection by region is enabled
    pub fn is_auto_select_enabled(&self) -> bool {
        self.config.auto_select_by_region
    }

    /// Get the user's region
    pub fn get_user_region(&self) -> &Region {
        &self.user_region
    }

    /// Update user region
    pub fn set_user_region(&mut self, region: Region) {
        info!("Updating user region from {} to {}", self.user_region, region);
        self.user_region = region;
    }

    /// Get enabled mirrors for the current region
    pub fn get_enabled_mirrors(&self) -> Vec<&ReleasesMirrorSource> {
        let user_region_str = self.user_region.to_string();
        
        self.config
            .sources
            .iter()
            .filter(|mirror| {
                mirror.enabled
                    && (mirror.regions.contains(&user_region_str)
                        || mirror.regions.contains(&"Global".to_string()))
            })
            .collect()
    }

    /// Get mirror statistics
    pub fn get_stats(&self) -> MirrorStats {
        let total_mirrors = self.config.sources.len();
        let enabled_mirrors = self.config.sources.iter().filter(|m| m.enabled).count();
        let healthy_mirrors = self.health_status.values().filter(|&&h| h).count();
        let applicable_mirrors = self.get_enabled_mirrors().len();

        MirrorStats {
            total_mirrors,
            enabled_mirrors,
            healthy_mirrors,
            applicable_mirrors,
            health_check_enabled: self.config.health_check_enabled,
            auto_select_enabled: self.config.auto_select_by_region,
            user_region: self.user_region.clone(),
        }
    }
}

/// Mirror statistics
#[derive(Debug, Clone)]
pub struct MirrorStats {
    pub total_mirrors: usize,
    pub enabled_mirrors: usize,
    pub healthy_mirrors: usize,
    pub applicable_mirrors: usize,
    pub health_check_enabled: bool,
    pub auto_select_enabled: bool,
    pub user_region: Region,
}

impl std::fmt::Display for MirrorStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Mirror Stats: {}/{} enabled, {}/{} healthy, {} applicable for region {}",
            self.enabled_mirrors,
            self.total_mirrors,
            self.healthy_mirrors,
            self.enabled_mirrors,
            self.applicable_mirrors,
            self.user_region
        )
    }
}
