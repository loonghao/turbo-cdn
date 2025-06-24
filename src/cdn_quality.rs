// Licensed under the MIT License
// Copyright (c) 2025 Hal <hal.long@outlook.com>

//! CDN Quality Assessment and Real-time Performance Monitoring
//!
//! This module provides comprehensive CDN performance evaluation including
//! latency testing, bandwidth measurement, availability checks, and dynamic scoring.

use crate::config::TurboCdnConfig;
use crate::error::Result;
use crate::http_client::{create_best_client, HttpClient};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// CDN performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CdnMetrics {
    /// Average latency in milliseconds
    pub latency_ms: f64,
    /// Bandwidth in bytes per second
    pub bandwidth_bps: f64,
    /// Success rate (0.0 to 1.0)
    pub success_rate: f64,
    /// Availability score (0.0 to 1.0)
    pub availability: f64,
    /// Overall quality score (0.0 to 100.0)
    pub quality_score: f64,
    /// Last updated timestamp
    pub last_updated: chrono::DateTime<chrono::Utc>,
    /// Number of tests performed
    pub test_count: u32,
}

impl Default for CdnMetrics {
    fn default() -> Self {
        Self {
            latency_ms: 0.0,
            bandwidth_bps: 0.0,
            success_rate: 1.0,
            availability: 1.0,
            quality_score: 50.0, // Neutral starting score
            last_updated: chrono::Utc::now(),
            test_count: 0,
        }
    }
}

/// CDN quality assessor with real-time monitoring
pub struct CdnQualityAssessor {
    metrics: Arc<RwLock<HashMap<String, CdnMetrics>>>,
    http_client: Box<dyn HttpClient>,
    config: TurboCdnConfig,
    test_urls: Vec<String>,
}

impl std::fmt::Debug for CdnQualityAssessor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CdnQualityAssessor")
            .field("metrics", &"<metrics>")
            .field("http_client", &"<http_client>")
            .field("config", &"<config>")
            .field("test_urls", &self.test_urls)
            .finish()
    }
}

impl CdnQualityAssessor {
    /// Create a new CDN quality assessor
    pub fn new(config: TurboCdnConfig) -> Result<Self> {
        let timeout = Duration::from_secs(config.performance.timeout);
        let http_client = create_best_client(timeout)?;

        Ok(Self {
            metrics: Arc::new(RwLock::new(HashMap::new())),
            http_client,
            test_urls: config.testing.test_urls.clone(),
            config,
        })
    }

    /// Assess quality of a specific CDN URL
    pub async fn assess_cdn_quality(&self, cdn_url: &str) -> Result<CdnMetrics> {
        info!("Assessing CDN quality for: {}", cdn_url);

        let start_time = Instant::now();

        // Perform multiple tests
        let latency = self.test_latency(cdn_url).await?;
        let bandwidth = self.test_bandwidth(cdn_url).await?;
        let availability = self.test_availability(cdn_url).await?;

        // Calculate quality score
        let quality_score = self.calculate_quality_score(latency, bandwidth, availability);

        let metrics = CdnMetrics {
            latency_ms: latency,
            bandwidth_bps: bandwidth,
            success_rate: if availability > 0.0 { 1.0 } else { 0.0 },
            availability,
            quality_score,
            last_updated: chrono::Utc::now(),
            test_count: 1,
        };

        // Update stored metrics
        self.update_metrics(cdn_url, &metrics).await;

        debug!(
            "CDN assessment completed in {:.2}s: {} (score: {:.1})",
            start_time.elapsed().as_secs_f64(),
            cdn_url,
            quality_score
        );

        Ok(metrics)
    }

    /// Test latency to a CDN
    async fn test_latency(&self, url: &str) -> Result<f64> {
        let start = Instant::now();

        match self.http_client.head(url).await {
            Ok(response) => {
                let latency = start.elapsed().as_millis() as f64;
                if response.status == 200 || response.status == 404 {
                    // 404 is acceptable for HEAD requests to some CDNs
                    Ok(latency)
                } else {
                    Ok(latency * 2.0) // Penalize non-200 responses
                }
            }
            Err(_) => Ok(5000.0), // 5 second penalty for failed requests
        }
    }

    /// Test bandwidth to a CDN
    async fn test_bandwidth(&self, base_url: &str) -> Result<f64> {
        // Use a small test file for bandwidth measurement
        let _test_size = 1024 * 100; // 100KB test

        // Try to construct a test URL
        let test_url = if base_url.contains("github.com") {
            // Use a known small file from GitHub
            "https://raw.githubusercontent.com/octocat/Hello-World/master/README"
        } else if base_url.contains("jsdelivr.net") {
            // Use a small library file
            "https://cdn.jsdelivr.net/npm/lodash@4.17.21/package.json"
        } else {
            // Fallback to HEAD request timing
            return self.estimate_bandwidth_from_latency(base_url).await;
        };

        let start = Instant::now();

        match self.http_client.get(test_url).await {
            Ok(response) => {
                let duration = start.elapsed();
                let size = response.body.len() as f64;

                if duration.as_millis() > 0 && size > 0.0 {
                    let bandwidth = size / duration.as_secs_f64();
                    Ok(bandwidth)
                } else {
                    Ok(1024.0 * 1024.0) // 1MB/s default
                }
            }
            Err(_) => Ok(512.0 * 1024.0), // 512KB/s penalty for failed requests
        }
    }

    /// Estimate bandwidth from latency (fallback method)
    async fn estimate_bandwidth_from_latency(&self, url: &str) -> Result<f64> {
        let latency = self.test_latency(url).await?;

        // Rough estimation: lower latency = higher potential bandwidth
        let estimated_bandwidth = if latency < 50.0 {
            10.0 * 1024.0 * 1024.0 // 10MB/s for very low latency
        } else if latency < 100.0 {
            5.0 * 1024.0 * 1024.0 // 5MB/s for low latency
        } else if latency < 200.0 {
            2.0 * 1024.0 * 1024.0 // 2MB/s for medium latency
        } else {
            1.0 * 1024.0 * 1024.0 // 1MB/s for high latency
        };

        Ok(estimated_bandwidth)
    }

    /// Test availability of a CDN
    async fn test_availability(&self, url: &str) -> Result<f64> {
        match self.http_client.head(url).await {
            Ok(response) => {
                if response.status >= 200 && response.status < 500 {
                    Ok(1.0) // Available
                } else {
                    Ok(0.5) // Partially available (server error)
                }
            }
            Err(_) => Ok(0.0), // Not available
        }
    }

    /// Calculate overall quality score
    fn calculate_quality_score(&self, latency: f64, bandwidth: f64, availability: f64) -> f64 {
        // Normalize metrics to 0-100 scale
        let latency_score = ((500.0 - latency.min(500.0)) / 500.0 * 100.0).max(0.0);
        let bandwidth_score = ((bandwidth / (10.0 * 1024.0 * 1024.0)).min(1.0) * 100.0).max(0.0);
        let availability_score = availability * 100.0;

        // Weighted average: latency 40%, bandwidth 40%, availability 20%
        let quality_score =
            (latency_score * 0.4) + (bandwidth_score * 0.4) + (availability_score * 0.2);

        quality_score.clamp(0.0, 100.0)
    }

    /// Update metrics for a CDN
    async fn update_metrics(&self, cdn_url: &str, new_metrics: &CdnMetrics) {
        let mut metrics_map = self.metrics.write().await;

        if let Some(existing) = metrics_map.get_mut(cdn_url) {
            // Update existing metrics with exponential moving average
            let alpha = 0.3; // Weight for new measurement
            existing.latency_ms =
                existing.latency_ms * (1.0 - alpha) + new_metrics.latency_ms * alpha;
            existing.bandwidth_bps =
                existing.bandwidth_bps * (1.0 - alpha) + new_metrics.bandwidth_bps * alpha;
            existing.availability =
                existing.availability * (1.0 - alpha) + new_metrics.availability * alpha;
            existing.quality_score = self.calculate_quality_score(
                existing.latency_ms,
                existing.bandwidth_bps,
                existing.availability,
            );
            existing.last_updated = chrono::Utc::now();
            existing.test_count += 1;
        } else {
            // Insert new metrics
            metrics_map.insert(cdn_url.to_string(), new_metrics.clone());
        }
    }

    /// Get metrics for a specific CDN
    pub async fn get_metrics(&self, cdn_url: &str) -> Option<CdnMetrics> {
        let metrics_map = self.metrics.read().await;
        metrics_map.get(cdn_url).cloned()
    }

    /// Get all CDN metrics sorted by quality score
    pub async fn get_all_metrics_sorted(&self) -> Vec<(String, CdnMetrics)> {
        let metrics_map = self.metrics.read().await;
        let mut sorted_metrics: Vec<_> = metrics_map
            .iter()
            .map(|(url, metrics)| (url.clone(), metrics.clone()))
            .collect();

        sorted_metrics.sort_by(|a, b| {
            b.1.quality_score
                .partial_cmp(&a.1.quality_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        sorted_metrics
    }

    /// Sort URLs by their quality scores
    pub async fn sort_urls_by_quality(&self, urls: Vec<String>) -> Vec<String> {
        let metrics_map = self.metrics.read().await;
        let mut url_scores: Vec<_> = urls
            .into_iter()
            .map(|url| {
                let score = metrics_map
                    .get(&url)
                    .map(|m| m.quality_score)
                    .unwrap_or(50.0); // Default neutral score
                (url, score)
            })
            .collect();

        url_scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        url_scores.into_iter().map(|(url, _)| url).collect()
    }

    /// Perform background quality assessment for all test URLs
    pub async fn background_assessment(&self) -> Result<()> {
        info!("Starting background CDN quality assessment");

        for test_url in &self.test_urls {
            if let Err(e) = self.assess_cdn_quality(test_url).await {
                warn!("Failed to assess CDN quality for {}: {}", test_url, e);
            }

            // Small delay between tests to avoid overwhelming servers
            tokio::time::sleep(Duration::from_millis(100)).await;
        }

        info!("Background CDN quality assessment completed");
        Ok(())
    }

    /// Get quality assessment summary
    pub async fn get_assessment_summary(&self) -> HashMap<String, f64> {
        let metrics_map = self.metrics.read().await;
        metrics_map
            .iter()
            .map(|(url, metrics)| (url.clone(), metrics.quality_score))
            .collect()
    }
}
