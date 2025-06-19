// Licensed under the MIT License
// Copyright (c) 2025 Hal <hal.long@outlook.com>

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use tracing::{debug, info};

use crate::config::{Region, TurboCdnConfig};
use crate::error::{Result, TurboCdnError};
use crate::sources::{DownloadUrl, SourceManager};

/// Intelligent router for selecting optimal download sources
#[derive(Debug)]
#[allow(dead_code)]
pub struct SmartRouter {
    config: TurboCdnConfig,
    source_manager: SourceManager,
    performance_tracker: PerformanceTracker,
    region_optimizer: RegionOptimizer,
}

/// Performance tracking for sources and URLs
#[derive(Debug)]
pub struct PerformanceTracker {
    source_metrics: HashMap<String, SourceMetrics>,
    url_metrics: HashMap<String, UrlMetrics>,
}

/// Region-based optimization
#[derive(Debug)]
pub struct RegionOptimizer {
    current_region: Region,
    region_preferences: HashMap<Region, Vec<String>>,
}

/// Metrics for a download source
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceMetrics {
    pub source_name: String,
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub average_response_time: f64,
    pub average_download_speed: f64, // bytes per second
    pub last_success: Option<chrono::DateTime<chrono::Utc>>,
    pub last_failure: Option<chrono::DateTime<chrono::Utc>>,
    pub reliability_score: f64, // 0.0 to 1.0
}

/// Metrics for a specific URL
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UrlMetrics {
    pub url: String,
    pub source: String,
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub average_response_time: f64,
    pub average_download_speed: f64,
    pub last_used: chrono::DateTime<chrono::Utc>,
    pub consecutive_failures: u32,
}

/// Routing decision with reasoning
#[derive(Debug, Clone)]
pub struct RoutingDecision {
    pub selected_urls: Vec<DownloadUrl>,
    pub reasoning: Vec<String>,
    pub fallback_urls: Vec<DownloadUrl>,
    pub estimated_download_time: Option<Duration>,
}

/// Download performance result
#[derive(Debug, Clone)]
pub struct DownloadPerformance {
    pub url: String,
    pub source: String,
    pub success: bool,
    pub response_time: Duration,
    pub download_speed: f64, // bytes per second
    pub bytes_downloaded: u64,
    pub error: Option<String>,
}

impl SmartRouter {
    /// Create a new smart router
    pub fn new(config: TurboCdnConfig, source_manager: SourceManager) -> Self {
        let performance_tracker = PerformanceTracker::new();
        let region_optimizer = RegionOptimizer::new(config.general.default_region);

        Self {
            config,
            source_manager,
            performance_tracker,
            region_optimizer,
        }
    }

    /// Route a download request to optimal sources
    pub async fn route_download(
        &mut self,
        repository: &str,
        version: &str,
        file_name: &str,
    ) -> Result<RoutingDecision> {
        let mut reasoning = Vec::new();

        // Get all available download URLs
        let all_urls = self
            .source_manager
            .get_all_download_urls(repository, version, file_name)
            .await?;

        if all_urls.is_empty() {
            return Err(TurboCdnError::routing("No download URLs available"));
        }

        reasoning.push(format!("Found {} potential download URLs", all_urls.len()));

        // Score and rank URLs
        let mut scored_urls: Vec<(DownloadUrl, f64)> = all_urls
            .into_iter()
            .map(|url| {
                let score = self.calculate_url_score(&url);
                (url, score)
            })
            .collect();

        // Sort by score (higher is better)
        scored_urls.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Apply region optimization
        scored_urls = self.region_optimizer.optimize_for_region(scored_urls);
        reasoning.push(format!(
            "Applied region optimization for {:?}",
            self.region_optimizer.current_region
        ));

        // Select primary URLs (top performers)
        let primary_count = std::cmp::min(3, scored_urls.len());
        let selected_urls: Vec<DownloadUrl> = scored_urls
            .iter()
            .take(primary_count)
            .map(|(url, _)| url.clone())
            .collect();

        // Select fallback URLs
        let fallback_urls: Vec<DownloadUrl> = scored_urls
            .iter()
            .skip(primary_count)
            .map(|(url, _)| url.clone())
            .collect();

        reasoning.push(format!(
            "Selected {} primary URLs and {} fallback URLs",
            selected_urls.len(),
            fallback_urls.len()
        ));

        // Estimate download time
        let estimated_download_time = self.estimate_download_time(&selected_urls);

        Ok(RoutingDecision {
            selected_urls,
            reasoning,
            fallback_urls,
            estimated_download_time,
        })
    }

    /// Calculate score for a download URL
    fn calculate_url_score(&self, url: &DownloadUrl) -> f64 {
        let mut score = 100.0; // Base score

        // Factor in source priority (lower priority number = higher score)
        score -= url.priority as f64 * 5.0;

        // Factor in source performance metrics
        if let Some(source_metrics) = self.performance_tracker.source_metrics.get(&url.source) {
            score += source_metrics.reliability_score * 50.0;

            // Bonus for recent successful downloads
            if let Some(last_success) = source_metrics.last_success {
                let hours_since_success = chrono::Utc::now()
                    .signed_duration_since(last_success)
                    .num_hours();

                if hours_since_success < 24 {
                    score += 10.0;
                }
            }

            // Penalty for recent failures
            if let Some(last_failure) = source_metrics.last_failure {
                let hours_since_failure = chrono::Utc::now()
                    .signed_duration_since(last_failure)
                    .num_hours();

                if hours_since_failure < 1 {
                    score -= 30.0;
                } else if hours_since_failure < 6 {
                    score -= 15.0;
                }
            }
        }

        // Factor in URL-specific metrics
        if let Some(url_metrics) = self.performance_tracker.url_metrics.get(&url.url) {
            // Penalty for consecutive failures
            score -= url_metrics.consecutive_failures as f64 * 10.0;

            // Bonus for good download speed
            if url_metrics.average_download_speed > 1_000_000.0 {
                // > 1 MB/s
                score += 15.0;
            }
        }

        // Factor in estimated latency
        if let Some(latency) = url.estimated_latency {
            score -= latency as f64 / 10.0; // Penalty for high latency
        }

        // Bonus for range support
        if url.supports_ranges {
            score += 10.0;
        }

        score.max(0.0) // Ensure non-negative score
    }

    /// Estimate download time based on selected URLs
    fn estimate_download_time(&self, urls: &[DownloadUrl]) -> Option<Duration> {
        if urls.is_empty() {
            return None;
        }

        // Use the best performing URL for estimation
        let best_url = &urls[0];

        if let Some(size) = best_url.size {
            if let Some(url_metrics) = self.performance_tracker.url_metrics.get(&best_url.url) {
                if url_metrics.average_download_speed > 0.0 {
                    let estimated_seconds = size as f64 / url_metrics.average_download_speed;
                    return Some(Duration::from_secs_f64(estimated_seconds));
                }
            }

            // Fallback to source metrics
            if let Some(source_metrics) = self
                .performance_tracker
                .source_metrics
                .get(&best_url.source)
            {
                if source_metrics.average_download_speed > 0.0 {
                    let estimated_seconds = size as f64 / source_metrics.average_download_speed;
                    return Some(Duration::from_secs_f64(estimated_seconds));
                }
            }
        }

        None
    }

    /// Record download performance for learning
    pub fn record_performance(&mut self, performance: DownloadPerformance) {
        self.performance_tracker.record_performance(performance);
    }

    /// Get performance statistics
    pub fn get_performance_stats(&self) -> &PerformanceTracker {
        &self.performance_tracker
    }

    /// Update region preference
    pub fn set_region(&mut self, region: Region) {
        self.region_optimizer.current_region = region;
        info!("Updated router region to {:?}", region);
    }
}

impl PerformanceTracker {
    fn new() -> Self {
        Self {
            source_metrics: HashMap::new(),
            url_metrics: HashMap::new(),
        }
    }

    fn record_performance(&mut self, performance: DownloadPerformance) {
        // Update source metrics
        let source_metrics = self
            .source_metrics
            .entry(performance.source.clone())
            .or_insert_with(|| SourceMetrics::new(performance.source.clone()));

        source_metrics.update(&performance);

        // Update URL metrics
        let url_metrics = self
            .url_metrics
            .entry(performance.url.clone())
            .or_insert_with(|| {
                UrlMetrics::new(performance.url.clone(), performance.source.clone())
            });

        url_metrics.update(&performance);

        debug!(
            "Recorded performance for {}: success={}, speed={:.2} MB/s",
            performance.url,
            performance.success,
            performance.download_speed / 1_000_000.0
        );
    }
}

impl SourceMetrics {
    fn new(source_name: String) -> Self {
        Self {
            source_name,
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            average_response_time: 0.0,
            average_download_speed: 0.0,
            last_success: None,
            last_failure: None,
            reliability_score: 1.0,
        }
    }

    fn update(&mut self, performance: &DownloadPerformance) {
        self.total_requests += 1;

        if performance.success {
            self.successful_requests += 1;
            self.last_success = Some(chrono::Utc::now());

            // Update averages
            let response_time_ms = performance.response_time.as_millis() as f64;
            self.average_response_time = self.update_average(
                self.average_response_time,
                response_time_ms,
                self.successful_requests,
            );

            self.average_download_speed = self.update_average(
                self.average_download_speed,
                performance.download_speed,
                self.successful_requests,
            );
        } else {
            self.failed_requests += 1;
            self.last_failure = Some(chrono::Utc::now());
        }

        // Update reliability score
        self.reliability_score = self.successful_requests as f64 / self.total_requests as f64;
    }

    fn update_average(&self, current_avg: f64, new_value: f64, count: u64) -> f64 {
        if count == 1 {
            new_value
        } else {
            (current_avg * (count - 1) as f64 + new_value) / count as f64
        }
    }
}

impl UrlMetrics {
    fn new(url: String, source: String) -> Self {
        Self {
            url,
            source,
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            average_response_time: 0.0,
            average_download_speed: 0.0,
            last_used: chrono::Utc::now(),
            consecutive_failures: 0,
        }
    }

    fn update(&mut self, performance: &DownloadPerformance) {
        self.total_requests += 1;
        self.last_used = chrono::Utc::now();

        if performance.success {
            self.successful_requests += 1;
            self.consecutive_failures = 0;

            // Update averages
            let response_time_ms = performance.response_time.as_millis() as f64;
            self.average_response_time = self.update_average(
                self.average_response_time,
                response_time_ms,
                self.successful_requests,
            );

            self.average_download_speed = self.update_average(
                self.average_download_speed,
                performance.download_speed,
                self.successful_requests,
            );
        } else {
            self.failed_requests += 1;
            self.consecutive_failures += 1;
        }
    }

    fn update_average(&self, current_avg: f64, new_value: f64, count: u64) -> f64 {
        if count == 1 {
            new_value
        } else {
            (current_avg * (count - 1) as f64 + new_value) / count as f64
        }
    }
}

impl RegionOptimizer {
    fn new(region: Region) -> Self {
        let mut region_preferences = HashMap::new();

        // Define region-specific source preferences
        region_preferences.insert(
            Region::China,
            vec![
                "fastly".to_string(),
                "jsdelivr".to_string(),
                "github".to_string(),
                "cloudflare".to_string(),
            ],
        );

        region_preferences.insert(
            Region::AsiaPacific,
            vec![
                "cloudflare".to_string(),
                "fastly".to_string(),
                "jsdelivr".to_string(),
                "github".to_string(),
            ],
        );

        region_preferences.insert(
            Region::Europe,
            vec![
                "jsdelivr".to_string(),
                "cloudflare".to_string(),
                "fastly".to_string(),
                "github".to_string(),
            ],
        );

        region_preferences.insert(
            Region::NorthAmerica,
            vec![
                "github".to_string(),
                "cloudflare".to_string(),
                "fastly".to_string(),
                "jsdelivr".to_string(),
            ],
        );

        Self {
            current_region: region,
            region_preferences,
        }
    }

    fn optimize_for_region(
        &self,
        mut scored_urls: Vec<(DownloadUrl, f64)>,
    ) -> Vec<(DownloadUrl, f64)> {
        if let Some(preferences) = self.region_preferences.get(&self.current_region) {
            // Boost scores for preferred sources in this region
            for (url, score) in &mut scored_urls {
                if let Some(position) = preferences.iter().position(|p| p == &url.source) {
                    let boost = (preferences.len() - position) as f64 * 5.0;
                    *score += boost;
                }
            }

            // Re-sort after applying regional preferences
            scored_urls.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        }

        scored_urls
    }
}
