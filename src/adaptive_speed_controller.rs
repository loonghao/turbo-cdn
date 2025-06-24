// Licensed under the MIT License
// Copyright (c) 2025 Hal <hal.long@outlook.com>

//! Adaptive speed controller with dynamic optimization
//!
//! This module implements real-time speed detection and adaptive
//! parameter adjustment for optimal download performance.

use std::collections::VecDeque;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, info};

/// Speed measurement sample
#[derive(Debug, Clone)]
pub struct SpeedSample {
    /// Timestamp of measurement
    pub timestamp: Instant,
    /// Download speed in bytes per second
    pub speed: f64,
    /// Bytes downloaded in this sample
    pub bytes: u64,
    /// Duration of this sample
    pub duration: Duration,
    /// Concurrent connections used
    pub concurrent_connections: u32,
    /// Chunk size used
    pub chunk_size: u64,
    /// Server URL
    pub server_url: String,
}

/// Network condition assessment
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkCondition {
    /// Excellent network (>50 MB/s)
    Excellent,
    /// Good network (10-50 MB/s)
    Good,
    /// Fair network (1-10 MB/s)
    Fair,
    /// Poor network (100KB-1MB/s)
    Poor,
    /// Very poor network (<100KB/s)
    VeryPoor,
}

impl NetworkCondition {
    /// Determine network condition from speed
    pub fn from_speed(speed_mbps: f64) -> Self {
        match speed_mbps {
            s if s >= 50.0 => NetworkCondition::Excellent,
            s if s >= 10.0 => NetworkCondition::Good,
            s if s >= 1.0 => NetworkCondition::Fair,
            s if s >= 0.1 => NetworkCondition::Poor,
            _ => NetworkCondition::VeryPoor,
        }
    }

    /// Get recommended concurrent connections for this condition
    pub fn recommended_concurrency(&self) -> u32 {
        match self {
            NetworkCondition::Excellent => 64,
            NetworkCondition::Good => 32,
            NetworkCondition::Fair => 16,
            NetworkCondition::Poor => 8,
            NetworkCondition::VeryPoor => 4,
        }
    }

    /// Get recommended chunk size for this condition
    pub fn recommended_chunk_size(&self) -> u64 {
        match self {
            NetworkCondition::Excellent => 8 * 1024 * 1024, // 8MB
            NetworkCondition::Good => 4 * 1024 * 1024,      // 4MB
            NetworkCondition::Fair => 2 * 1024 * 1024,      // 2MB
            NetworkCondition::Poor => 1024 * 1024,          // 1MB
            NetworkCondition::VeryPoor => 512 * 1024,       // 512KB
        }
    }
}

/// Adaptive download parameters
#[derive(Debug, Clone)]
pub struct AdaptiveParams {
    /// Current concurrent connections
    pub concurrent_connections: u32,
    /// Current chunk size
    pub chunk_size: u64,
    /// Current timeout
    pub timeout: Duration,
    /// Current retry attempts
    pub retry_attempts: u32,
    /// Network condition assessment
    pub network_condition: NetworkCondition,
    /// Confidence in current parameters (0.0 to 1.0)
    pub confidence: f64,
    /// Last adjustment time
    pub last_adjusted: Instant,
}

impl Default for AdaptiveParams {
    fn default() -> Self {
        Self {
            concurrent_connections: 16,
            chunk_size: 2 * 1024 * 1024, // 2MB
            timeout: Duration::from_secs(30),
            retry_attempts: 3,
            network_condition: NetworkCondition::Fair,
            confidence: 0.5,
            last_adjusted: Instant::now(),
        }
    }
}

/// Adaptive speed controller
pub struct AdaptiveSpeedController {
    /// Recent speed samples
    samples: Arc<RwLock<VecDeque<SpeedSample>>>,
    /// Current adaptive parameters
    params: Arc<RwLock<AdaptiveParams>>,
    /// Maximum samples to keep
    max_samples: usize,
    /// Minimum samples needed for adaptation
    min_samples_for_adaptation: usize,
    /// Adaptation interval
    adaptation_interval: Duration,
    /// Speed trend analyzer
    trend_analyzer: SpeedTrendAnalyzer,
}

impl AdaptiveSpeedController {
    /// Create a new adaptive speed controller
    pub fn new() -> Self {
        Self {
            samples: Arc::new(RwLock::new(VecDeque::new())),
            params: Arc::new(RwLock::new(AdaptiveParams::default())),
            max_samples: 100,
            min_samples_for_adaptation: 5,
            adaptation_interval: Duration::from_secs(10),
            trend_analyzer: SpeedTrendAnalyzer::new(),
        }
    }

    /// Record a speed sample
    pub async fn record_sample(&self, sample: SpeedSample) {
        let mut samples = self.samples.write().await;

        // Add new sample
        samples.push_back(sample.clone());

        // Remove old samples if we exceed max
        while samples.len() > self.max_samples {
            samples.pop_front();
        }

        debug!(
            "Recorded speed sample: {:.2} MB/s ({} bytes in {:?})",
            sample.speed / 1024.0 / 1024.0,
            sample.bytes,
            sample.duration
        );

        // Trigger adaptation if enough samples and time has passed
        let params = self.params.read().await;
        if samples.len() >= self.min_samples_for_adaptation
            && params.last_adjusted.elapsed() >= self.adaptation_interval
        {
            drop(params);
            drop(samples);
            self.adapt_parameters().await;
        }
    }

    /// Adapt parameters based on recent performance
    async fn adapt_parameters(&self) {
        let samples = self.samples.read().await;
        if samples.len() < self.min_samples_for_adaptation {
            return;
        }

        // Analyze recent performance
        let recent_samples: Vec<_> = samples.iter().rev().take(10).cloned().collect();
        let avg_speed =
            recent_samples.iter().map(|s| s.speed).sum::<f64>() / recent_samples.len() as f64;
        let speed_mbps = avg_speed / 1024.0 / 1024.0;

        // Determine network condition
        let network_condition = NetworkCondition::from_speed(speed_mbps);

        // Analyze speed trend
        let trend = self.trend_analyzer.analyze_trend(&recent_samples);

        let mut params = self.params.write().await;
        let old_params = params.clone();

        // Update network condition
        params.network_condition = network_condition;

        // Adapt concurrent connections based on performance
        let new_concurrency = self
            .calculate_optimal_concurrency(&recent_samples, &trend)
            .await;
        params.concurrent_connections = new_concurrency;

        // Adapt chunk size based on network condition and performance
        let new_chunk_size = self
            .calculate_optimal_chunk_size(&recent_samples, &trend)
            .await;
        params.chunk_size = new_chunk_size;

        // Adapt timeout based on response times
        let avg_response_time = recent_samples
            .iter()
            .map(|s| s.duration.as_millis() as f64)
            .sum::<f64>()
            / recent_samples.len() as f64;
        params.timeout =
            Duration::from_millis((avg_response_time * 3.0) as u64).max(Duration::from_secs(10));

        // Update confidence based on performance stability
        params.confidence = self.calculate_confidence(&recent_samples);
        params.last_adjusted = Instant::now();

        info!("Adapted parameters: concurrency {} -> {}, chunk_size {}KB -> {}KB, condition {:?}, confidence {:.2}",
              old_params.concurrent_connections, params.concurrent_connections,
              old_params.chunk_size / 1024, params.chunk_size / 1024,
              params.network_condition, params.confidence);
    }

    /// Calculate optimal concurrency based on recent performance
    async fn calculate_optimal_concurrency(
        &self,
        samples: &[SpeedSample],
        trend: &SpeedTrend,
    ) -> u32 {
        if samples.is_empty() {
            return 16; // Default
        }

        // Group samples by concurrency level
        let mut concurrency_performance: std::collections::HashMap<u32, Vec<f64>> =
            std::collections::HashMap::new();
        for sample in samples {
            concurrency_performance
                .entry(sample.concurrent_connections)
                .or_default()
                .push(sample.speed);
        }

        // Find the concurrency level with best average performance
        let mut best_concurrency = 16;
        let mut best_speed = 0.0;

        for (concurrency, speeds) in concurrency_performance {
            let avg_speed = speeds.iter().sum::<f64>() / speeds.len() as f64;
            if avg_speed > best_speed {
                best_speed = avg_speed;
                best_concurrency = concurrency;
            }
        }

        // Adjust based on trend
        match trend {
            SpeedTrend::Improving => (best_concurrency as f64 * 1.2) as u32,
            SpeedTrend::Declining => (best_concurrency as f64 * 0.8) as u32,
            SpeedTrend::Stable => best_concurrency,
        }
        .clamp(4, 128) // Reasonable bounds
    }

    /// Calculate optimal chunk size based on recent performance
    async fn calculate_optimal_chunk_size(
        &self,
        samples: &[SpeedSample],
        _trend: &SpeedTrend,
    ) -> u64 {
        if samples.is_empty() {
            return 2 * 1024 * 1024; // Default 2MB
        }

        // Group samples by chunk size
        let mut chunk_performance: std::collections::HashMap<u64, Vec<f64>> =
            std::collections::HashMap::new();
        for sample in samples {
            chunk_performance
                .entry(sample.chunk_size)
                .or_default()
                .push(sample.speed);
        }

        // Find the chunk size with best average performance
        let mut best_chunk_size = 2 * 1024 * 1024;
        let mut best_speed = 0.0;

        for (chunk_size, speeds) in chunk_performance {
            let avg_speed = speeds.iter().sum::<f64>() / speeds.len() as f64;
            if avg_speed > best_speed {
                best_speed = avg_speed;
                best_chunk_size = chunk_size;
            }
        }

        // Adjust based on network condition
        let params = self.params.read().await;
        let recommended = params.network_condition.recommended_chunk_size();

        // Blend optimal and recommended
        let blended = (best_chunk_size + recommended) / 2;
        blended.clamp(128 * 1024, 16 * 1024 * 1024) // 128KB to 16MB
    }

    /// Calculate confidence in current parameters
    fn calculate_confidence(&self, samples: &[SpeedSample]) -> f64 {
        if samples.len() < 3 {
            return 0.3; // Low confidence with few samples
        }

        // Calculate coefficient of variation (stability measure)
        let speeds: Vec<f64> = samples.iter().map(|s| s.speed).collect();
        let mean = speeds.iter().sum::<f64>() / speeds.len() as f64;
        let variance = speeds.iter().map(|s| (s - mean).powi(2)).sum::<f64>() / speeds.len() as f64;
        let std_dev = variance.sqrt();
        let cv = std_dev / mean;

        // Lower coefficient of variation = higher confidence
        (1.0 - cv.min(1.0)).max(0.1)
    }

    /// Get current adaptive parameters
    pub async fn get_params(&self) -> AdaptiveParams {
        self.params.read().await.clone()
    }

    /// Get current average speed
    pub async fn get_current_speed(&self) -> Option<f64> {
        let samples = self.samples.read().await;
        if samples.is_empty() {
            return None;
        }

        let recent_samples: Vec<_> = samples.iter().rev().take(5).collect();
        let avg_speed =
            recent_samples.iter().map(|s| s.speed).sum::<f64>() / recent_samples.len() as f64;
        Some(avg_speed)
    }

    /// Get speed statistics
    pub async fn get_speed_stats(&self) -> SpeedStats {
        let samples = self.samples.read().await;

        if samples.is_empty() {
            return SpeedStats::default();
        }

        let speeds: Vec<f64> = samples.iter().map(|s| s.speed).collect();
        let min_speed = speeds.iter().cloned().fold(f64::INFINITY, f64::min);
        let max_speed = speeds.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let avg_speed = speeds.iter().sum::<f64>() / speeds.len() as f64;

        SpeedStats {
            min_speed,
            max_speed,
            avg_speed,
            sample_count: samples.len(),
            latest_speed: samples.back().map(|s| s.speed),
        }
    }

    /// Clear all samples and reset parameters
    pub async fn reset(&self) {
        let mut samples = self.samples.write().await;
        samples.clear();

        let mut params = self.params.write().await;
        *params = AdaptiveParams::default();

        info!("Adaptive speed controller reset");
    }
}

/// Speed trend analysis
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpeedTrend {
    Improving,
    Stable,
    Declining,
}

/// Speed trend analyzer
struct SpeedTrendAnalyzer;

impl SpeedTrendAnalyzer {
    fn new() -> Self {
        Self
    }

    fn analyze_trend(&self, samples: &[SpeedSample]) -> SpeedTrend {
        if samples.len() < 3 {
            return SpeedTrend::Stable;
        }

        let speeds: Vec<f64> = samples.iter().map(|s| s.speed).collect();
        let first_half_avg =
            speeds[..speeds.len() / 2].iter().sum::<f64>() / (speeds.len() / 2) as f64;
        let second_half_avg = speeds[speeds.len() / 2..].iter().sum::<f64>()
            / (speeds.len() - speeds.len() / 2) as f64;

        let change_ratio = (second_half_avg - first_half_avg) / first_half_avg;

        if change_ratio > 0.1 {
            SpeedTrend::Improving
        } else if change_ratio < -0.1 {
            SpeedTrend::Declining
        } else {
            SpeedTrend::Stable
        }
    }
}

/// Speed statistics
#[derive(Debug, Clone)]
pub struct SpeedStats {
    pub min_speed: f64,
    pub max_speed: f64,
    pub avg_speed: f64,
    pub sample_count: usize,
    pub latest_speed: Option<f64>,
}

impl Default for SpeedStats {
    fn default() -> Self {
        Self {
            min_speed: 0.0,
            max_speed: 0.0,
            avg_speed: 0.0,
            sample_count: 0,
            latest_speed: None,
        }
    }
}

impl Default for AdaptiveSpeedController {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_speed_recording() {
        let controller = AdaptiveSpeedController::new();

        let sample = SpeedSample {
            timestamp: Instant::now(),
            speed: 10.0 * 1024.0 * 1024.0, // 10 MB/s
            bytes: 1024 * 1024,
            duration: Duration::from_millis(100),
            concurrent_connections: 16,
            chunk_size: 2 * 1024 * 1024,
            server_url: "https://example.com".to_string(),
        };

        controller.record_sample(sample).await;

        let stats = controller.get_speed_stats().await;
        assert_eq!(stats.sample_count, 1);
        assert!(stats.avg_speed > 0.0);
    }

    #[test]
    fn test_network_condition_classification() {
        assert_eq!(
            NetworkCondition::from_speed(100.0),
            NetworkCondition::Excellent
        );
        assert_eq!(NetworkCondition::from_speed(20.0), NetworkCondition::Good);
        assert_eq!(NetworkCondition::from_speed(5.0), NetworkCondition::Fair);
        assert_eq!(NetworkCondition::from_speed(0.5), NetworkCondition::Poor);
        assert_eq!(
            NetworkCondition::from_speed(0.05),
            NetworkCondition::VeryPoor
        );
    }
}
