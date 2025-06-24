// Licensed under the MIT License
// Copyright (c) 2025 Hal <hal.long@outlook.com>

//! Adaptive Concurrency Control System
//!
//! This module implements intelligent concurrency control that adapts to network conditions,
//! detects congestion, and applies adaptive backoff algorithms for optimal performance.

use crate::config::TurboCdnConfig;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Network congestion metrics
#[derive(Debug, Clone)]
pub struct CongestionMetrics {
    /// Current round-trip time in milliseconds
    pub rtt_ms: f64,
    /// Packet loss rate (0.0 to 1.0)
    pub loss_rate: f64,
    /// Bandwidth utilization (0.0 to 1.0)
    pub bandwidth_utilization: f64,
    /// Error rate in recent requests (0.0 to 1.0)
    pub error_rate: f64,
    /// Timestamp of last measurement
    pub last_updated: Instant,
}

impl Default for CongestionMetrics {
    fn default() -> Self {
        Self {
            rtt_ms: 50.0,
            loss_rate: 0.0,
            bandwidth_utilization: 0.5,
            error_rate: 0.0,
            last_updated: Instant::now(),
        }
    }
}

/// Adaptive concurrency controller
#[derive(Debug)]
pub struct AdaptiveConcurrencyController {
    /// Current concurrency level
    current_concurrency: AtomicU32,
    /// Minimum allowed concurrency
    min_concurrency: u32,
    /// Maximum allowed concurrency
    max_concurrency: u32,
    /// Network congestion threshold (0.0 to 1.0)
    congestion_threshold: f64,
    /// Current congestion metrics
    metrics: Arc<RwLock<CongestionMetrics>>,
    /// Total requests made
    total_requests: AtomicU64,
    /// Total successful requests
    successful_requests: AtomicU64,
    /// Total failed requests
    failed_requests: AtomicU64,
    /// Last adjustment time
    last_adjustment: Arc<RwLock<Instant>>,
    /// Adjustment interval
    adjustment_interval: Duration,
}

impl AdaptiveConcurrencyController {
    /// Create a new adaptive concurrency controller
    pub fn new(config: &TurboCdnConfig) -> Self {
        let initial_concurrency = (config.performance.max_concurrent_downloads as u32).min(
            config
                .performance
                .max_concurrent_downloads_limit
                .unwrap_or(32),
        );

        Self {
            current_concurrency: AtomicU32::new(initial_concurrency),
            min_concurrency: config.performance.min_concurrent_downloads.unwrap_or(4),
            max_concurrency: config
                .performance
                .max_concurrent_downloads_limit
                .unwrap_or(32),
            congestion_threshold: config
                .performance
                .network_congestion_threshold
                .unwrap_or(0.5),
            metrics: Arc::new(RwLock::new(CongestionMetrics::default())),
            total_requests: AtomicU64::new(0),
            successful_requests: AtomicU64::new(0),
            failed_requests: AtomicU64::new(0),
            last_adjustment: Arc::new(RwLock::new(Instant::now())),
            adjustment_interval: Duration::from_secs(5),
        }
    }

    /// Get current concurrency level
    pub fn current_concurrency(&self) -> u32 {
        self.current_concurrency.load(Ordering::Relaxed)
    }

    /// Record a successful request with timing information
    pub async fn record_success(&self, duration: Duration, bytes_transferred: u64) {
        self.total_requests.fetch_add(1, Ordering::Relaxed);
        self.successful_requests.fetch_add(1, Ordering::Relaxed);

        // Update metrics
        let mut metrics = self.metrics.write().await;
        let rtt_ms = duration.as_millis() as f64;

        // Exponential moving average for RTT
        metrics.rtt_ms = metrics.rtt_ms * 0.8 + rtt_ms * 0.2;

        // Calculate bandwidth utilization (simplified)
        let bandwidth_mbps =
            (bytes_transferred as f64 * 8.0) / (duration.as_secs_f64() * 1_000_000.0);
        metrics.bandwidth_utilization = (bandwidth_mbps / 100.0).min(1.0); // Assume 100Mbps baseline

        metrics.last_updated = Instant::now();
        drop(metrics);

        // Check if adjustment is needed
        self.maybe_adjust_concurrency().await;
    }

    /// Record a failed request
    pub async fn record_failure(&self, error_type: &str) {
        self.total_requests.fetch_add(1, Ordering::Relaxed);
        self.failed_requests.fetch_add(1, Ordering::Relaxed);

        // Update error rate
        let error_rate = {
            let mut metrics = self.metrics.write().await;
            let total = self.total_requests.load(Ordering::Relaxed) as f64;
            let failed = self.failed_requests.load(Ordering::Relaxed) as f64;
            metrics.error_rate = failed / total.max(1.0);
            metrics.last_updated = Instant::now();
            metrics.error_rate
        };

        warn!(
            "Request failed: {} (error rate: {:.2}%)",
            error_type,
            error_rate * 100.0
        );

        // Immediate adjustment for high error rates
        if error_rate > 0.1 {
            self.decrease_concurrency("High error rate").await;
        }
    }

    /// Check if concurrency adjustment is needed
    async fn maybe_adjust_concurrency(&self) {
        let last_adjustment = *self.last_adjustment.read().await;
        if last_adjustment.elapsed() < self.adjustment_interval {
            return;
        }

        let metrics = self.metrics.read().await;
        let congestion_score = self.calculate_congestion_score(&metrics);
        drop(metrics);

        if congestion_score > self.congestion_threshold {
            self.decrease_concurrency("Network congestion detected")
                .await;
        } else if congestion_score < self.congestion_threshold * 0.5 {
            self.increase_concurrency("Network conditions improved")
                .await;
        }

        *self.last_adjustment.write().await = Instant::now();
    }

    /// Calculate network congestion score (0.0 to 1.0)
    fn calculate_congestion_score(&self, metrics: &CongestionMetrics) -> f64 {
        // Weighted combination of different metrics
        let rtt_score = (metrics.rtt_ms / 500.0).min(1.0); // 500ms = max RTT
        let error_score = metrics.error_rate;
        let bandwidth_score = 1.0 - metrics.bandwidth_utilization;

        // Weighted average: RTT 40%, Error 40%, Bandwidth 20%
        (rtt_score * 0.4) + (error_score * 0.4) + (bandwidth_score * 0.2)
    }

    /// Increase concurrency level
    async fn increase_concurrency(&self, reason: &str) {
        let current = self.current_concurrency.load(Ordering::Relaxed);
        if current < self.max_concurrency {
            let new_concurrency = (current + 1).min(self.max_concurrency);
            self.current_concurrency
                .store(new_concurrency, Ordering::Relaxed);
            info!(
                "Increased concurrency: {} -> {} ({})",
                current, new_concurrency, reason
            );
        }
    }

    /// Decrease concurrency level
    async fn decrease_concurrency(&self, reason: &str) {
        let current = self.current_concurrency.load(Ordering::Relaxed);
        if current > self.min_concurrency {
            let new_concurrency = (current - 1).max(self.min_concurrency);
            self.current_concurrency
                .store(new_concurrency, Ordering::Relaxed);
            warn!(
                "Decreased concurrency: {} -> {} ({})",
                current, new_concurrency, reason
            );
        }
    }

    /// Get performance statistics
    pub async fn get_stats(&self) -> ConcurrencyStats {
        let metrics = self.metrics.read().await;
        let total = self.total_requests.load(Ordering::Relaxed);
        let successful = self.successful_requests.load(Ordering::Relaxed);
        let failed = self.failed_requests.load(Ordering::Relaxed);

        ConcurrencyStats {
            current_concurrency: self.current_concurrency(),
            min_concurrency: self.min_concurrency,
            max_concurrency: self.max_concurrency,
            total_requests: total,
            successful_requests: successful,
            failed_requests: failed,
            success_rate: if total > 0 {
                successful as f64 / total as f64
            } else {
                0.0
            },
            current_rtt_ms: metrics.rtt_ms,
            error_rate: metrics.error_rate,
            bandwidth_utilization: metrics.bandwidth_utilization,
            congestion_score: self.calculate_congestion_score(&metrics),
        }
    }

    /// Reset statistics
    pub fn reset_stats(&self) {
        self.total_requests.store(0, Ordering::Relaxed);
        self.successful_requests.store(0, Ordering::Relaxed);
        self.failed_requests.store(0, Ordering::Relaxed);
        debug!("Concurrency controller statistics reset");
    }

    /// Apply adaptive backoff for failed requests
    pub async fn adaptive_backoff(&self, attempt: u32) -> Duration {
        let metrics = self.metrics.read().await;
        let base_delay = Duration::from_millis(100);

        // Exponential backoff with jitter
        let exponential_delay = base_delay * 2_u32.pow(attempt.min(6));

        // Adjust based on network conditions
        let congestion_multiplier = 1.0 + metrics.error_rate * 2.0;
        let adjusted_delay = Duration::from_millis(
            (exponential_delay.as_millis() as f64 * congestion_multiplier) as u64,
        );

        // Add jitter (±25%)
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        attempt.hash(&mut hasher);
        let hash = hasher.finish();
        let jitter = (hash as f64 / u64::MAX as f64) * 0.5 - 0.25; // -0.25 to +0.25
        let final_delay =
            Duration::from_millis(((adjusted_delay.as_millis() as f64) * (1.0 + jitter)) as u64);

        debug!(
            "Adaptive backoff: attempt {}, delay: {:?}",
            attempt, final_delay
        );
        final_delay.min(Duration::from_secs(30)) // Cap at 30 seconds
    }
}

/// Concurrency performance statistics
#[derive(Debug, Clone)]
pub struct ConcurrencyStats {
    pub current_concurrency: u32,
    pub min_concurrency: u32,
    pub max_concurrency: u32,
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub success_rate: f64,
    pub current_rtt_ms: f64,
    pub error_rate: f64,
    pub bandwidth_utilization: f64,
    pub congestion_score: f64,
}

impl std::fmt::Display for ConcurrencyStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, 
            "Concurrency: {}/{}-{} | Requests: {}/{} ({:.1}% success) | RTT: {:.1}ms | Congestion: {:.1}%",
            self.current_concurrency,
            self.min_concurrency,
            self.max_concurrency,
            self.successful_requests,
            self.total_requests,
            self.success_rate * 100.0,
            self.current_rtt_ms,
            self.congestion_score * 100.0
        )
    }
}
