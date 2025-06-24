// Licensed under the MIT License
// Copyright (c) 2025 Hal <hal.long@outlook.com>

//! Server quality scoring and intelligent failover system
//!
//! This module implements advanced server quality assessment and
//! intelligent failover mechanisms for optimal download performance.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Server quality metrics
#[derive(Debug, Clone)]
pub struct ServerMetrics {
    /// Server URL
    pub url: String,
    /// Average response time over recent requests
    pub avg_response_time: Duration,
    /// Success rate (0.0 to 1.0)
    pub success_rate: f64,
    /// Average download speed in bytes per second
    pub avg_speed: f64,
    /// Connection establishment time
    pub connection_time: Duration,
    /// Time to first byte
    pub ttfb: Duration,
    /// Number of total requests
    pub total_requests: u64,
    /// Number of failed requests
    pub failed_requests: u64,
    /// Last successful request time
    pub last_success: Option<Instant>,
    /// Last failure time
    pub last_failure: Option<Instant>,
    /// Consecutive failures count
    pub consecutive_failures: u32,
    /// Server reliability score (0.0 to 1.0)
    pub reliability_score: f64,
    /// Geographic region hint
    pub region: Option<String>,
    /// Server type (CDN, mirror, original)
    pub server_type: ServerType,
    /// Quality score (0.0 to 100.0)
    pub quality_score: f64,
    /// Last updated timestamp
    pub last_updated: Instant,
}

/// Server type classification
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServerType {
    /// Original source server
    Original,
    /// CDN edge server
    Cdn,
    /// Mirror server
    Mirror,
    /// Proxy server
    Proxy,
    /// Unknown type
    Unknown,
}

impl ServerType {
    /// Get base score multiplier for server type
    pub fn score_multiplier(&self) -> f64 {
        match self {
            ServerType::Cdn => 1.2,      // CDNs are generally faster
            ServerType::Mirror => 1.1,   // Mirrors are usually reliable
            ServerType::Original => 1.0, // Baseline
            ServerType::Proxy => 0.9,    // Proxies may add latency
            ServerType::Unknown => 0.8,  // Unknown servers get penalty
        }
    }
}

impl Default for ServerMetrics {
    fn default() -> Self {
        Self {
            url: String::new(),
            avg_response_time: Duration::from_millis(100),
            success_rate: 1.0,
            avg_speed: 1024.0 * 1024.0, // 1 MB/s default
            connection_time: Duration::from_millis(50),
            ttfb: Duration::from_millis(100),
            total_requests: 0,
            failed_requests: 0,
            last_success: None,
            last_failure: None,
            consecutive_failures: 0,
            reliability_score: 1.0,
            region: None,
            server_type: ServerType::Unknown,
            quality_score: 50.0,
            last_updated: Instant::now(),
        }
    }
}

/// Server quality scorer with intelligent failover
pub struct ServerQualityScorer {
    /// Server metrics storage
    metrics: Arc<RwLock<HashMap<String, ServerMetrics>>>,
    /// Scoring configuration
    config: ScoringConfig,
    /// Failover thresholds
    failover_config: FailoverConfig,
}

/// Scoring configuration
#[derive(Debug, Clone)]
pub struct ScoringConfig {
    /// Weight for response time in scoring (0.0 to 1.0)
    pub response_time_weight: f64,
    /// Weight for success rate in scoring (0.0 to 1.0)
    pub success_rate_weight: f64,
    /// Weight for download speed in scoring (0.0 to 1.0)
    pub speed_weight: f64,
    /// Weight for reliability in scoring (0.0 to 1.0)
    pub reliability_weight: f64,
    /// Maximum response time considered acceptable (ms)
    pub max_acceptable_response_time: Duration,
    /// Minimum success rate considered acceptable
    pub min_acceptable_success_rate: f64,
    /// Minimum speed considered acceptable (bytes/sec)
    pub min_acceptable_speed: f64,
}

impl Default for ScoringConfig {
    fn default() -> Self {
        Self {
            response_time_weight: 0.3,
            success_rate_weight: 0.3,
            speed_weight: 0.25,
            reliability_weight: 0.15,
            max_acceptable_response_time: Duration::from_millis(2000),
            min_acceptable_success_rate: 0.8,
            min_acceptable_speed: 100.0 * 1024.0, // 100 KB/s
        }
    }
}

/// Failover configuration
#[derive(Debug, Clone)]
pub struct FailoverConfig {
    /// Maximum consecutive failures before marking server as down
    pub max_consecutive_failures: u32,
    /// Time to wait before retrying a failed server
    pub retry_delay: Duration,
    /// Minimum quality score to consider server healthy
    pub min_healthy_score: f64,
    /// Circuit breaker threshold
    pub circuit_breaker_threshold: f64,
}

impl Default for FailoverConfig {
    fn default() -> Self {
        Self {
            max_consecutive_failures: 3,
            retry_delay: Duration::from_secs(30),
            min_healthy_score: 20.0,
            circuit_breaker_threshold: 10.0,
        }
    }
}

impl ServerQualityScorer {
    /// Create a new server quality scorer
    pub fn new() -> Self {
        Self::with_config(ScoringConfig::default(), FailoverConfig::default())
    }

    /// Create a new server quality scorer with custom configuration
    pub fn with_config(scoring_config: ScoringConfig, failover_config: FailoverConfig) -> Self {
        Self {
            metrics: Arc::new(RwLock::new(HashMap::new())),
            config: scoring_config,
            failover_config,
        }
    }

    /// Record a successful request
    pub async fn record_success(
        &self,
        url: &str,
        response_time: Duration,
        speed: f64,
        connection_time: Duration,
        ttfb: Duration,
    ) {
        let mut metrics = self.metrics.write().await;
        let server_metrics = metrics.entry(url.to_string()).or_default();

        // Update metrics with exponential moving average
        let alpha = 0.3; // Smoothing factor
        
        server_metrics.avg_response_time = Duration::from_millis(
            ((1.0 - alpha) * server_metrics.avg_response_time.as_millis() as f64 + 
             alpha * response_time.as_millis() as f64) as u64
        );

        server_metrics.avg_speed = (1.0 - alpha) * server_metrics.avg_speed + alpha * speed;
        
        server_metrics.connection_time = Duration::from_millis(
            ((1.0 - alpha) * server_metrics.connection_time.as_millis() as f64 + 
             alpha * connection_time.as_millis() as f64) as u64
        );

        server_metrics.ttfb = Duration::from_millis(
            ((1.0 - alpha) * server_metrics.ttfb.as_millis() as f64 + 
             alpha * ttfb.as_millis() as f64) as u64
        );

        server_metrics.total_requests += 1;
        server_metrics.last_success = Some(Instant::now());
        server_metrics.consecutive_failures = 0;
        server_metrics.last_updated = Instant::now();

        // Update success rate
        server_metrics.success_rate = 
            server_metrics.total_requests as f64 / 
            (server_metrics.total_requests + server_metrics.failed_requests) as f64;

        // Update reliability score
        self.update_reliability_score(server_metrics);

        // Recalculate quality score
        server_metrics.quality_score = self.calculate_quality_score(server_metrics);

        debug!("Updated server metrics for {}: score={:.1}, speed={:.1}KB/s, response_time={:?}", 
               url, server_metrics.quality_score, server_metrics.avg_speed / 1024.0, 
               server_metrics.avg_response_time);
    }

    /// Record a failed request
    pub async fn record_failure(&self, url: &str, error_type: FailureType) {
        let mut metrics = self.metrics.write().await;
        let server_metrics = metrics.entry(url.to_string()).or_default();

        server_metrics.failed_requests += 1;
        server_metrics.consecutive_failures += 1;
        server_metrics.last_failure = Some(Instant::now());
        server_metrics.last_updated = Instant::now();

        // Update success rate
        server_metrics.success_rate = 
            server_metrics.total_requests as f64 / 
            (server_metrics.total_requests + server_metrics.failed_requests) as f64;

        // Apply failure penalty based on error type
        let penalty = match error_type {
            FailureType::Timeout => 0.1,
            FailureType::ConnectionRefused => 0.2,
            FailureType::DnsFailure => 0.15,
            FailureType::HttpError => 0.05,
            FailureType::NetworkError => 0.1,
        };

        server_metrics.reliability_score = (server_metrics.reliability_score - penalty).max(0.0);

        // Recalculate quality score
        server_metrics.quality_score = self.calculate_quality_score(server_metrics);

        warn!("Recorded failure for {}: consecutive={}, score={:.1}, error={:?}", 
              url, server_metrics.consecutive_failures, server_metrics.quality_score, error_type);
    }

    /// Calculate quality score for a server
    fn calculate_quality_score(&self, metrics: &ServerMetrics) -> f64 {
        // Response time score (0-100, lower is better)
        let response_time_score = {
            let max_time = self.config.max_acceptable_response_time.as_millis() as f64;
            let actual_time = metrics.avg_response_time.as_millis() as f64;
            ((max_time - actual_time) / max_time * 100.0).max(0.0).min(100.0)
        };

        // Success rate score (0-100)
        let success_rate_score = metrics.success_rate * 100.0;

        // Speed score (0-100)
        let speed_score = {
            let min_speed = self.config.min_acceptable_speed;
            let actual_speed = metrics.avg_speed;
            ((actual_speed / min_speed).log2() * 20.0 + 50.0).max(0.0).min(100.0)
        };

        // Reliability score (0-100)
        let reliability_score = metrics.reliability_score * 100.0;

        // Weighted average
        let weighted_score = 
            response_time_score * self.config.response_time_weight +
            success_rate_score * self.config.success_rate_weight +
            speed_score * self.config.speed_weight +
            reliability_score * self.config.reliability_weight;

        // Apply server type multiplier
        let final_score = weighted_score * metrics.server_type.score_multiplier();

        // Apply consecutive failure penalty
        let failure_penalty = metrics.consecutive_failures as f64 * 5.0;
        
        (final_score - failure_penalty).max(0.0).min(100.0)
    }

    /// Update reliability score based on recent performance
    fn update_reliability_score(&self, metrics: &mut ServerMetrics) {
        let now = Instant::now();
        
        // Boost reliability for recent successes
        if let Some(last_success) = metrics.last_success {
            let time_since_success = now.duration_since(last_success);
            if time_since_success < Duration::from_secs(300) { // 5 minutes
                metrics.reliability_score = (metrics.reliability_score + 0.01).min(1.0);
            }
        }

        // Penalize for recent failures
        if let Some(last_failure) = metrics.last_failure {
            let time_since_failure = now.duration_since(last_failure);
            if time_since_failure < Duration::from_secs(60) { // 1 minute
                metrics.reliability_score = (metrics.reliability_score - 0.02).max(0.0);
            }
        }
    }

    /// Get ranked list of servers by quality score
    pub async fn get_ranked_servers(&self, urls: &[String]) -> Vec<(String, f64)> {
        let metrics = self.metrics.read().await;
        let mut ranked: Vec<(String, f64)> = urls
            .iter()
            .map(|url| {
                let score = metrics.get(url)
                    .map(|m| m.quality_score)
                    .unwrap_or(50.0); // Default score for unknown servers
                (url.clone(), score)
            })
            .collect();

        // Sort by score (highest first)
        ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        debug!("Server rankings: {:?}", ranked);
        ranked
    }

    /// Check if server should be failed over
    pub async fn should_failover(&self, url: &str) -> bool {
        let metrics = self.metrics.read().await;
        
        if let Some(server_metrics) = metrics.get(url) {
            // Check consecutive failures
            if server_metrics.consecutive_failures >= self.failover_config.max_consecutive_failures {
                return true;
            }

            // Check quality score
            if server_metrics.quality_score < self.failover_config.circuit_breaker_threshold {
                return true;
            }

            // Check if server is in retry delay period
            if let Some(last_failure) = server_metrics.last_failure {
                if last_failure.elapsed() < self.failover_config.retry_delay {
                    return true;
                }
            }
        }

        false
    }

    /// Get server metrics
    pub async fn get_metrics(&self, url: &str) -> Option<ServerMetrics> {
        let metrics = self.metrics.read().await;
        metrics.get(url).cloned()
    }

    /// Get all server metrics
    pub async fn get_all_metrics(&self) -> HashMap<String, ServerMetrics> {
        self.metrics.read().await.clone()
    }

    /// Clear metrics for a server
    pub async fn clear_server_metrics(&self, url: &str) {
        let mut metrics = self.metrics.write().await;
        metrics.remove(url);
        info!("Cleared metrics for server: {}", url);
    }

    /// Clear all metrics
    pub async fn clear_all_metrics(&self) {
        let mut metrics = self.metrics.write().await;
        metrics.clear();
        info!("Cleared all server metrics");
    }
}

/// Failure type classification
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FailureType {
    /// Request timeout
    Timeout,
    /// Connection refused
    ConnectionRefused,
    /// DNS resolution failure
    DnsFailure,
    /// HTTP error (4xx, 5xx)
    HttpError,
    /// Network error
    NetworkError,
}

impl Default for ServerQualityScorer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_server_scoring() {
        let scorer = ServerQualityScorer::new();
        let url = "https://example.com";

        // Record a successful request
        scorer.record_success(
            url,
            Duration::from_millis(100),
            1024.0 * 1024.0, // 1 MB/s
            Duration::from_millis(50),
            Duration::from_millis(80),
        ).await;

        let metrics = scorer.get_metrics(url).await.unwrap();
        assert!(metrics.quality_score > 50.0);
        assert_eq!(metrics.consecutive_failures, 0);
    }

    #[tokio::test]
    async fn test_failure_handling() {
        let scorer = ServerQualityScorer::new();
        let url = "https://example.com";

        // Record multiple failures
        for _ in 0..3 {
            scorer.record_failure(url, FailureType::Timeout).await;
        }

        assert!(scorer.should_failover(url).await);
    }

    #[tokio::test]
    async fn test_server_ranking() {
        let scorer = ServerQualityScorer::new();
        let urls = vec![
            "https://fast.example.com".to_string(),
            "https://slow.example.com".to_string(),
        ];

        // Make fast server better
        scorer.record_success(
            &urls[0],
            Duration::from_millis(50),
            10.0 * 1024.0 * 1024.0, // 10 MB/s
            Duration::from_millis(20),
            Duration::from_millis(30),
        ).await;

        // Make slow server worse
        scorer.record_success(
            &urls[1],
            Duration::from_millis(500),
            100.0 * 1024.0, // 100 KB/s
            Duration::from_millis(200),
            Duration::from_millis(300),
        ).await;

        let ranked = scorer.get_ranked_servers(&urls).await;
        assert_eq!(ranked[0].0, urls[0]); // Fast server should be first
        assert!(ranked[0].1 > ranked[1].1); // Fast server should have higher score
    }
}
