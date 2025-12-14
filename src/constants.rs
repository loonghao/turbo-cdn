// Licensed under the MIT License
// Copyright (c) 2025 Hal <hal.long@outlook.com>

//! # Constants
//!
//! Centralized constants for the turbo-cdn library.
//! This module eliminates magic numbers and provides a single source of truth
//! for configurable values.

use std::time::Duration;

/// Default retry attempts for download operations
pub const DEFAULT_RETRY_ATTEMPTS: usize = 3;

/// Default retry delay base (exponential backoff: 2^n seconds)
pub const DEFAULT_RETRY_DELAY_BASE: u64 = 2;

/// Maximum servers to track in the server tracker
pub const MAX_SERVERS_TO_TRACK: usize = 100;

/// Number of extra servers to remove when cleaning up
pub const SERVER_CLEANUP_BUFFER: usize = 10;

/// Maximum URLs to try for download redundancy
pub const MAX_URLS_TO_TRY: usize = 8;

/// Number of recent speed samples to keep for averaging
pub const RECENT_SAMPLES_SIZE: usize = 10;

/// Default neutral score for untested servers
pub const DEFAULT_SERVER_SCORE: f64 = 0.5;

/// Excellent speed benchmark (10 MB/s)
pub const EXCELLENT_SPEED_BYTES_PER_SEC: f64 = 10.0 * 1024.0 * 1024.0;

/// Excellent latency benchmark (50ms)
pub const EXCELLENT_LATENCY_MS: f64 = 50.0;

/// Poor latency benchmark (1000ms)
pub const POOR_LATENCY_MS: f64 = 1000.0;

/// Performance score weight for speed
pub const WEIGHT_SPEED: f64 = 0.4;

/// Performance score weight for success rate
pub const WEIGHT_SUCCESS: f64 = 0.4;

/// Performance score weight for latency
pub const WEIGHT_LATENCY: f64 = 0.2;

/// Default response time for new servers
pub const DEFAULT_RESPONSE_TIME: Duration = Duration::from_millis(100);

/// HTTP/2 frame size for better throughput
pub const HTTP2_FRAME_SIZE: u32 = 16384;

/// Maximum redirect count
pub const MAX_REDIRECTS: usize = 10;

/// Large file threshold (100MB)
pub const LARGE_FILE_THRESHOLD: u64 = 100 * 1024 * 1024;

/// Medium file threshold (10MB)
pub const MEDIUM_FILE_THRESHOLD: u64 = 10 * 1024 * 1024;

/// Small file threshold (1MB)
pub const SMALL_FILE_THRESHOLD: u64 = 1024 * 1024;

/// Cache cleanup threshold (when to trigger cleanup)
pub const CACHE_CLEANUP_THRESHOLD: f64 = 0.8;

/// Default cache TTL in seconds (1 hour)
pub const DEFAULT_CACHE_TTL_SECS: u64 = 3600;

/// Default max cache entries
pub const DEFAULT_MAX_CACHE_ENTRIES: usize = 1000;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_weight_sum() {
        // Weights should sum to 1.0
        let total = WEIGHT_SPEED + WEIGHT_SUCCESS + WEIGHT_LATENCY;
        assert!((total - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_file_thresholds_ordering() {
        // Use runtime comparison to avoid clippy::assertions_on_constants
        let small = SMALL_FILE_THRESHOLD;
        let medium = MEDIUM_FILE_THRESHOLD;
        let large = LARGE_FILE_THRESHOLD;
        assert!(small < medium);
        assert!(medium < large);
    }
}
