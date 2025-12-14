// Licensed under the MIT License
// Copyright (c) 2025 Hal <hal.long@outlook.com>

//! Server performance tracking and intelligent selection
//!
//! This module provides server performance tracking similar to aria2's feedback
//! URI selector, allowing the downloader to learn from historical performance
//! and select the fastest servers automatically.

use crate::constants::{
    DEFAULT_RESPONSE_TIME, DEFAULT_SERVER_SCORE, EXCELLENT_SPEED_BYTES_PER_SEC,
    MAX_SERVERS_TO_TRACK, POOR_LATENCY_MS, RECENT_SAMPLES_SIZE, SERVER_CLEANUP_BUFFER,
    WEIGHT_LATENCY, WEIGHT_SPEED, WEIGHT_SUCCESS,
};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tracing::{debug, info, warn};

/// Server performance statistics
#[derive(Debug, Clone)]
pub struct ServerStats {
    /// Average download speed in bytes per second
    pub average_speed: f64,
    /// Success rate (0.0 to 1.0)
    pub success_rate: f64,
    /// Average response time
    pub average_response_time: Duration,
    /// Total number of attempts
    pub total_attempts: u32,
    /// Number of successful downloads
    pub successful_downloads: u32,
    /// Number of failed downloads
    pub failed_downloads: u32,
    /// Last update timestamp
    pub last_updated: Instant,
    /// Recent speed samples (for calculating moving average)
    pub recent_speeds: Vec<f64>,
    /// Recent response times
    pub recent_response_times: Vec<Duration>,
}

impl Default for ServerStats {
    fn default() -> Self {
        Self {
            average_speed: 0.0,
            success_rate: 1.0, // Start optimistic
            average_response_time: DEFAULT_RESPONSE_TIME,
            total_attempts: 0,
            successful_downloads: 0,
            failed_downloads: 0,
            last_updated: Instant::now(),
            recent_speeds: Vec::new(),
            recent_response_times: Vec::new(),
        }
    }
}

impl ServerStats {
    /// Calculate performance score (0.0 to 1.0, higher is better)
    pub fn performance_score(&self) -> f64 {
        if self.total_attempts == 0 {
            return DEFAULT_SERVER_SCORE;
        }

        // Normalize speed (assume 10MB/s is excellent)
        let speed_score = (self.average_speed / EXCELLENT_SPEED_BYTES_PER_SEC).min(1.0);

        // Success rate is already normalized
        let success_score = self.success_rate;

        // Normalize latency (assume 50ms is excellent, 1000ms is poor)
        let latency_ms = self.average_response_time.as_millis() as f64;
        let latency_score = (POOR_LATENCY_MS - latency_ms.min(POOR_LATENCY_MS)) / POOR_LATENCY_MS;

        WEIGHT_SPEED * speed_score + WEIGHT_SUCCESS * success_score + WEIGHT_LATENCY * latency_score
    }

    /// Update statistics with new download result
    pub fn update_with_result(&mut self, speed: f64, response_time: Duration, success: bool) {
        self.total_attempts += 1;
        self.last_updated = Instant::now();

        if success {
            self.successful_downloads += 1;

            // Update speed statistics
            self.recent_speeds.push(speed);
            if self.recent_speeds.len() > RECENT_SAMPLES_SIZE {
                self.recent_speeds.remove(0);
            }
            self.average_speed =
                self.recent_speeds.iter().sum::<f64>() / self.recent_speeds.len() as f64;
        } else {
            self.failed_downloads += 1;
        }

        // Update response time statistics
        self.recent_response_times.push(response_time);
        if self.recent_response_times.len() > RECENT_SAMPLES_SIZE {
            self.recent_response_times.remove(0);
        }
        self.average_response_time = Duration::from_millis(
            (self
                .recent_response_times
                .iter()
                .map(|d| d.as_millis())
                .sum::<u128>()
                / self.recent_response_times.len() as u128) as u64,
        );

        // Update success rate
        self.success_rate = self.successful_downloads as f64 / self.total_attempts as f64;

        debug!(
            "Updated server stats: speed={:.2} MB/s, success_rate={:.2}, response_time={:?}, score={:.3}",
            self.average_speed / 1024.0 / 1024.0,
            self.success_rate,
            self.average_response_time,
            self.performance_score()
        );
    }
}

/// Server performance tracker with intelligent selection
#[derive(Debug)]
pub struct ServerTracker {
    server_stats: HashMap<String, ServerStats>,
    max_servers_to_track: usize,
}

impl ServerTracker {
    /// Create a new server tracker
    pub fn new() -> Self {
        Self {
            server_stats: HashMap::new(),
            max_servers_to_track: MAX_SERVERS_TO_TRACK,
        }
    }

    /// Create a new server tracker with custom capacity
    pub fn with_capacity(max_servers: usize) -> Self {
        Self {
            server_stats: HashMap::new(),
            max_servers_to_track: max_servers,
        }
    }

    /// Select the best servers from a list of URLs
    pub fn select_best_servers(&self, urls: &[String], max_count: usize) -> Vec<String> {
        if urls.is_empty() {
            return Vec::new();
        }

        let default_stats = ServerStats::default();
        let mut scored_urls: Vec<(String, f64)> = urls
            .iter()
            .map(|url| {
                let stats = self.server_stats.get(url).unwrap_or(&default_stats);
                let score = stats.performance_score();
                (url.clone(), score)
            })
            .collect();

        // Sort by score (descending)
        scored_urls.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        let selected: Vec<String> = scored_urls
            .into_iter()
            .take(max_count)
            .map(|(url, score)| {
                debug!("Selected server: {} (score: {:.3})", url, score);
                url
            })
            .collect();

        info!(
            "Selected {} servers from {} candidates",
            selected.len(),
            urls.len()
        );
        selected
    }

    /// Record a successful download
    pub fn record_success(&mut self, url: &str, speed: f64, response_time: Duration) {
        self.ensure_capacity();
        let stats = self.server_stats.entry(url.to_string()).or_default();
        stats.update_with_result(speed, response_time, true);
    }

    /// Record a failed download
    pub fn record_failure(&mut self, url: &str, response_time: Duration) {
        self.ensure_capacity();
        let stats = self.server_stats.entry(url.to_string()).or_default();
        stats.update_with_result(0.0, response_time, false);
    }

    /// Get statistics for a specific server
    pub fn get_stats(&self, url: &str) -> Option<&ServerStats> {
        self.server_stats.get(url)
    }

    /// Get all server statistics
    pub fn get_all_stats(&self) -> &HashMap<String, ServerStats> {
        &self.server_stats
    }

    /// Clear old statistics to prevent memory bloat
    fn ensure_capacity(&mut self) {
        if self.server_stats.len() >= self.max_servers_to_track {
            // Collect URLs to remove (oldest entries by last_updated)
            let mut entries: Vec<_> = self
                .server_stats
                .iter()
                .map(|(url, stats)| (url.clone(), stats.last_updated))
                .collect();
            entries.sort_by_key(|(_, last_updated)| *last_updated);

            let to_remove = entries.len() - self.max_servers_to_track + SERVER_CLEANUP_BUFFER;
            let urls_to_remove: Vec<String> = entries
                .into_iter()
                .take(to_remove)
                .map(|(url, _)| url)
                .collect();

            for url in urls_to_remove {
                self.server_stats.remove(&url);
            }

            warn!(
                "Cleaned up old server statistics, now tracking {} servers",
                self.server_stats.len()
            );
        }
    }

    /// Get performance summary
    pub fn get_performance_summary(&self) -> PerformanceSummary {
        if self.server_stats.is_empty() {
            return PerformanceSummary::default();
        }

        let total_servers = self.server_stats.len();
        let total_attempts: u32 = self.server_stats.values().map(|s| s.total_attempts).sum();
        let total_successes: u32 = self
            .server_stats
            .values()
            .map(|s| s.successful_downloads)
            .sum();
        let average_speed: f64 = self
            .server_stats
            .values()
            .filter(|s| s.average_speed > 0.0)
            .map(|s| s.average_speed)
            .sum::<f64>()
            / self
                .server_stats
                .values()
                .filter(|s| s.average_speed > 0.0)
                .count()
                .max(1) as f64;

        let best_server = self
            .server_stats
            .iter()
            .max_by(|(_, a), (_, b)| {
                a.performance_score()
                    .partial_cmp(&b.performance_score())
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(url, stats)| (url.clone(), stats.performance_score()));

        PerformanceSummary {
            total_servers,
            total_attempts,
            total_successes,
            overall_success_rate: if total_attempts > 0 {
                total_successes as f64 / total_attempts as f64
            } else {
                0.0
            },
            average_speed,
            best_server,
        }
    }
}

impl Default for ServerTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// Performance summary for reporting
#[derive(Debug, Clone)]
pub struct PerformanceSummary {
    pub total_servers: usize,
    pub total_attempts: u32,
    pub total_successes: u32,
    pub overall_success_rate: f64,
    pub average_speed: f64,
    pub best_server: Option<(String, f64)>,
}

impl Default for PerformanceSummary {
    fn default() -> Self {
        Self {
            total_servers: 0,
            total_attempts: 0,
            total_successes: 0,
            overall_success_rate: 0.0,
            average_speed: 0.0,
            best_server: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_stats_performance_score() {
        let mut stats = ServerStats::default();

        // Test with good performance
        stats.update_with_result(5.0 * 1024.0 * 1024.0, Duration::from_millis(50), true);
        assert!(stats.performance_score() > 0.5);

        // Test with poor performance
        stats.update_with_result(100.0 * 1024.0, Duration::from_millis(500), false);
        assert!(stats.performance_score() < 0.8);
    }

    #[test]
    fn test_server_tracker_selection() {
        let mut tracker = ServerTracker::new();

        // Record some performance data
        tracker.record_success(
            "http://fast.example.com",
            10.0 * 1024.0 * 1024.0,
            Duration::from_millis(50),
        );
        tracker.record_success(
            "http://slow.example.com",
            1.0 * 1024.0 * 1024.0,
            Duration::from_millis(200),
        );
        tracker.record_failure("http://bad.example.com", Duration::from_millis(1000));

        let urls = vec![
            "http://fast.example.com".to_string(),
            "http://slow.example.com".to_string(),
            "http://bad.example.com".to_string(),
        ];

        let selected = tracker.select_best_servers(&urls, 2);
        assert_eq!(selected.len(), 2);
        assert_eq!(selected[0], "http://fast.example.com");
    }
}
