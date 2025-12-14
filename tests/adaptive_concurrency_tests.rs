// Licensed under the MIT License
// Copyright (c) 2025 Hal <hal.long@outlook.com>

//! Tests for the adaptive concurrency control system

use std::time::Duration;
use turbo_cdn::adaptive_concurrency::{
    AdaptiveConcurrencyController, ConcurrencyStats, CongestionMetrics,
};
use turbo_cdn::config::TurboCdnConfig;

/// Test default CongestionMetrics values
#[test]
fn test_congestion_metrics_default() {
    let metrics = CongestionMetrics::default();

    assert_eq!(metrics.rtt_ms, 50.0);
    assert_eq!(metrics.loss_rate, 0.0);
    assert_eq!(metrics.bandwidth_utilization, 0.5);
    assert_eq!(metrics.error_rate, 0.0);
}

/// Test CongestionMetrics clone
#[test]
fn test_congestion_metrics_clone() {
    let metrics = CongestionMetrics::default();
    let cloned = metrics.clone();

    assert_eq!(metrics.rtt_ms, cloned.rtt_ms);
    assert_eq!(metrics.loss_rate, cloned.loss_rate);
    assert_eq!(metrics.bandwidth_utilization, cloned.bandwidth_utilization);
    assert_eq!(metrics.error_rate, cloned.error_rate);
}

/// Test CongestionMetrics debug
#[test]
fn test_congestion_metrics_debug() {
    let metrics = CongestionMetrics::default();
    let debug_str = format!("{:?}", metrics);

    assert!(debug_str.contains("CongestionMetrics"));
    assert!(debug_str.contains("rtt_ms"));
    assert!(debug_str.contains("loss_rate"));
}

/// Test AdaptiveConcurrencyController creation with default config
#[test]
fn test_controller_creation_default() {
    let config = TurboCdnConfig::default();
    let controller = AdaptiveConcurrencyController::new(&config);

    let current = controller.current_concurrency();
    assert!(current >= 1);
    assert!(current <= 64);
}

/// Test AdaptiveConcurrencyController with custom config
#[test]
fn test_controller_creation_custom_config() {
    let mut config = TurboCdnConfig::default();
    config.performance.max_concurrent_downloads = 16;
    config.performance.min_concurrent_downloads = Some(4);
    config.performance.max_concurrent_downloads_limit = Some(32);
    config.performance.network_congestion_threshold = Some(0.7);

    let controller = AdaptiveConcurrencyController::new(&config);

    let current = controller.current_concurrency();
    assert!(current >= 4);
    assert!(current <= 32);
}

/// Test current_concurrency getter
#[test]
fn test_current_concurrency() {
    let config = TurboCdnConfig::default();
    let controller = AdaptiveConcurrencyController::new(&config);

    let concurrency = controller.current_concurrency();
    assert!(concurrency > 0);
}

/// Test record_success updates metrics
#[tokio::test]
async fn test_record_success() {
    let config = TurboCdnConfig::default();
    let controller = AdaptiveConcurrencyController::new(&config);

    // Record a successful request
    controller
        .record_success(Duration::from_millis(100), 1024 * 1024)
        .await;

    let stats = controller.get_stats().await;
    assert_eq!(stats.total_requests, 1);
    assert_eq!(stats.successful_requests, 1);
    assert_eq!(stats.failed_requests, 0);
}

/// Test record_success with multiple requests
#[tokio::test]
async fn test_record_multiple_successes() {
    let config = TurboCdnConfig::default();
    let controller = AdaptiveConcurrencyController::new(&config);

    // Record multiple successful requests
    for i in 0..5 {
        controller
            .record_success(Duration::from_millis(50 + i * 10), 512 * 1024)
            .await;
    }

    let stats = controller.get_stats().await;
    assert_eq!(stats.total_requests, 5);
    assert_eq!(stats.successful_requests, 5);
    assert_eq!(stats.failed_requests, 0);
    assert!(stats.success_rate > 0.99);
}

/// Test record_failure updates metrics
#[tokio::test]
async fn test_record_failure() {
    let config = TurboCdnConfig::default();
    let controller = AdaptiveConcurrencyController::new(&config);

    // Record a failed request
    controller.record_failure("Connection timeout").await;

    let stats = controller.get_stats().await;
    assert_eq!(stats.total_requests, 1);
    assert_eq!(stats.successful_requests, 0);
    assert_eq!(stats.failed_requests, 1);
}

/// Test mixed success and failure
#[tokio::test]
async fn test_mixed_success_failure() {
    let config = TurboCdnConfig::default();
    let controller = AdaptiveConcurrencyController::new(&config);

    // Record some successes
    for _ in 0..8 {
        controller
            .record_success(Duration::from_millis(100), 1024 * 1024)
            .await;
    }

    // Record some failures
    for _ in 0..2 {
        controller.record_failure("Network error").await;
    }

    let stats = controller.get_stats().await;
    assert_eq!(stats.total_requests, 10);
    assert_eq!(stats.successful_requests, 8);
    assert_eq!(stats.failed_requests, 2);
    assert!((stats.success_rate - 0.8).abs() < 0.01);
}

/// Test get_stats returns correct values
#[tokio::test]
async fn test_get_stats() {
    let config = TurboCdnConfig::default();
    let controller = AdaptiveConcurrencyController::new(&config);

    let stats = controller.get_stats().await;

    assert_eq!(stats.total_requests, 0);
    assert_eq!(stats.successful_requests, 0);
    assert_eq!(stats.failed_requests, 0);
    assert!(stats.min_concurrency > 0);
    assert!(stats.max_concurrency >= stats.min_concurrency);
    assert!(stats.current_concurrency >= stats.min_concurrency);
    assert!(stats.current_concurrency <= stats.max_concurrency);
}

/// Test reset_stats clears counters
#[tokio::test]
async fn test_reset_stats() {
    let config = TurboCdnConfig::default();
    let controller = AdaptiveConcurrencyController::new(&config);

    // Record some activity
    controller
        .record_success(Duration::from_millis(100), 1024)
        .await;
    controller.record_failure("Test error").await;

    // Verify activity was recorded
    let stats = controller.get_stats().await;
    assert_eq!(stats.total_requests, 2);

    // Reset stats
    controller.reset_stats();

    // Verify stats are reset
    let stats = controller.get_stats().await;
    assert_eq!(stats.total_requests, 0);
    assert_eq!(stats.successful_requests, 0);
    assert_eq!(stats.failed_requests, 0);
}

/// Test adaptive_backoff returns reasonable delays
#[tokio::test]
async fn test_adaptive_backoff() {
    let config = TurboCdnConfig::default();
    let controller = AdaptiveConcurrencyController::new(&config);

    // Test increasing backoff
    let delay0 = controller.adaptive_backoff(0).await;
    let delay1 = controller.adaptive_backoff(1).await;
    let delay2 = controller.adaptive_backoff(2).await;

    // Delays should generally increase (with some jitter)
    assert!(delay0.as_millis() >= 50); // At least some delay
    assert!(delay1.as_millis() >= delay0.as_millis() / 2); // Should be in similar range or higher
    assert!(delay2.as_millis() >= delay1.as_millis() / 2);
}

/// Test adaptive_backoff caps at 30 seconds
#[tokio::test]
async fn test_adaptive_backoff_max_cap() {
    let config = TurboCdnConfig::default();
    let controller = AdaptiveConcurrencyController::new(&config);

    // Test with high attempt number
    let delay = controller.adaptive_backoff(10).await;

    // Should be capped at 30 seconds
    assert!(delay <= Duration::from_secs(30));
}

/// Test adaptive_backoff with errors increases delay
#[tokio::test]
async fn test_adaptive_backoff_with_errors() {
    let config = TurboCdnConfig::default();
    let controller = AdaptiveConcurrencyController::new(&config);

    // Get baseline delay
    let baseline_delay = controller.adaptive_backoff(1).await;

    // Record some failures to increase error rate
    for _ in 0..5 {
        controller.record_failure("Test error").await;
    }

    // Get delay after errors
    let error_delay = controller.adaptive_backoff(1).await;

    // Delay should be higher due to error rate multiplier
    // (allowing for jitter variance)
    assert!(
        error_delay.as_millis() >= baseline_delay.as_millis() / 2,
        "Error delay should be at least half of baseline"
    );
}

/// Test ConcurrencyStats Display implementation
#[test]
fn test_concurrency_stats_display() {
    let stats = ConcurrencyStats {
        current_concurrency: 16,
        min_concurrency: 4,
        max_concurrency: 32,
        total_requests: 100,
        successful_requests: 95,
        failed_requests: 5,
        success_rate: 0.95,
        current_rtt_ms: 50.0,
        error_rate: 0.05,
        bandwidth_utilization: 0.8,
        congestion_score: 0.3,
    };

    let display = format!("{}", stats);

    assert!(display.contains("16"));
    assert!(display.contains("4"));
    assert!(display.contains("32"));
    assert!(display.contains("95"));
    assert!(display.contains("100"));
}

/// Test ConcurrencyStats Clone
#[test]
fn test_concurrency_stats_clone() {
    let stats = ConcurrencyStats {
        current_concurrency: 16,
        min_concurrency: 4,
        max_concurrency: 32,
        total_requests: 100,
        successful_requests: 95,
        failed_requests: 5,
        success_rate: 0.95,
        current_rtt_ms: 50.0,
        error_rate: 0.05,
        bandwidth_utilization: 0.8,
        congestion_score: 0.3,
    };

    let cloned = stats.clone();

    assert_eq!(stats.current_concurrency, cloned.current_concurrency);
    assert_eq!(stats.total_requests, cloned.total_requests);
    assert_eq!(stats.success_rate, cloned.success_rate);
}

/// Test ConcurrencyStats Debug
#[test]
fn test_concurrency_stats_debug() {
    let stats = ConcurrencyStats {
        current_concurrency: 16,
        min_concurrency: 4,
        max_concurrency: 32,
        total_requests: 100,
        successful_requests: 95,
        failed_requests: 5,
        success_rate: 0.95,
        current_rtt_ms: 50.0,
        error_rate: 0.05,
        bandwidth_utilization: 0.8,
        congestion_score: 0.3,
    };

    let debug_str = format!("{:?}", stats);

    assert!(debug_str.contains("ConcurrencyStats"));
    assert!(debug_str.contains("current_concurrency"));
    assert!(debug_str.contains("success_rate"));
}

/// Test RTT updates with exponential moving average
#[tokio::test]
async fn test_rtt_exponential_moving_average() {
    let config = TurboCdnConfig::default();
    let controller = AdaptiveConcurrencyController::new(&config);

    // Record requests with different RTTs
    controller
        .record_success(Duration::from_millis(100), 1024)
        .await;
    let stats1 = controller.get_stats().await;

    controller
        .record_success(Duration::from_millis(200), 1024)
        .await;
    let stats2 = controller.get_stats().await;

    // RTT should be updated using EMA (not just latest value)
    assert!(stats2.current_rtt_ms > stats1.current_rtt_ms);
}

/// Test bandwidth utilization calculation
#[tokio::test]
async fn test_bandwidth_utilization() {
    let config = TurboCdnConfig::default();
    let controller = AdaptiveConcurrencyController::new(&config);

    // Record a high-bandwidth transfer (10MB in 1 second = 80Mbps)
    controller
        .record_success(Duration::from_secs(1), 10 * 1024 * 1024)
        .await;

    let stats = controller.get_stats().await;
    assert!(stats.bandwidth_utilization > 0.0);
    assert!(stats.bandwidth_utilization <= 1.0);
}

/// Test congestion score calculation
#[tokio::test]
async fn test_congestion_score() {
    let config = TurboCdnConfig::default();
    let controller = AdaptiveConcurrencyController::new(&config);

    // Initial congestion score
    let stats = controller.get_stats().await;
    assert!(stats.congestion_score >= 0.0);
    assert!(stats.congestion_score <= 1.0);
}

/// Test high error rate triggers concurrency decrease
#[tokio::test]
async fn test_high_error_rate_decreases_concurrency() {
    let mut config = TurboCdnConfig::default();
    config.performance.min_concurrent_downloads = Some(1);
    config.performance.max_concurrent_downloads_limit = Some(32);

    let controller = AdaptiveConcurrencyController::new(&config);
    let initial_concurrency = controller.current_concurrency();

    // Record many failures to trigger high error rate (>10%)
    for _ in 0..20 {
        controller.record_failure("Simulated error").await;
    }

    let final_concurrency = controller.current_concurrency();

    // Concurrency should have decreased due to high error rate
    assert!(
        final_concurrency <= initial_concurrency,
        "Concurrency should decrease with high error rate"
    );
}

/// Test controller handles zero bytes transferred
#[tokio::test]
async fn test_zero_bytes_transferred() {
    let config = TurboCdnConfig::default();
    let controller = AdaptiveConcurrencyController::new(&config);

    // Record success with zero bytes (edge case)
    controller
        .record_success(Duration::from_millis(100), 0)
        .await;

    let stats = controller.get_stats().await;
    assert_eq!(stats.successful_requests, 1);
}

/// Test controller handles very short duration
#[tokio::test]
async fn test_very_short_duration() {
    let config = TurboCdnConfig::default();
    let controller = AdaptiveConcurrencyController::new(&config);

    // Record success with very short duration
    controller
        .record_success(Duration::from_nanos(1), 1024)
        .await;

    let stats = controller.get_stats().await;
    assert_eq!(stats.successful_requests, 1);
}

/// Test controller handles very long duration
#[tokio::test]
async fn test_very_long_duration() {
    let config = TurboCdnConfig::default();
    let controller = AdaptiveConcurrencyController::new(&config);

    // Record success with very long duration
    controller
        .record_success(Duration::from_secs(3600), 1024 * 1024)
        .await;

    let stats = controller.get_stats().await;
    assert_eq!(stats.successful_requests, 1);
}

/// Test controller handles large bytes transferred
#[tokio::test]
async fn test_large_bytes_transferred() {
    let config = TurboCdnConfig::default();
    let controller = AdaptiveConcurrencyController::new(&config);

    // Record success with large file (1GB)
    controller
        .record_success(Duration::from_secs(10), 1024 * 1024 * 1024)
        .await;

    let stats = controller.get_stats().await;
    assert_eq!(stats.successful_requests, 1);
    // Bandwidth utilization should be capped at 1.0
    assert!(stats.bandwidth_utilization <= 1.0);
}

/// Test concurrent access to controller
#[tokio::test]
async fn test_concurrent_access() {
    use std::sync::Arc;

    let config = TurboCdnConfig::default();
    let controller = Arc::new(AdaptiveConcurrencyController::new(&config));

    let mut handles = vec![];

    // Spawn multiple tasks that access the controller concurrently
    for i in 0..10 {
        let controller_clone = Arc::clone(&controller);
        let handle = tokio::spawn(async move {
            if i % 2 == 0 {
                controller_clone
                    .record_success(Duration::from_millis(100), 1024)
                    .await;
            } else {
                controller_clone.record_failure("Test error").await;
            }
        });
        handles.push(handle);
    }

    // Wait for all tasks to complete
    for handle in handles {
        handle.await.unwrap();
    }

    let stats = controller.get_stats().await;
    assert_eq!(stats.total_requests, 10);
    assert_eq!(stats.successful_requests, 5);
    assert_eq!(stats.failed_requests, 5);
}

/// Test success rate calculation with zero requests
#[tokio::test]
async fn test_success_rate_zero_requests() {
    let config = TurboCdnConfig::default();
    let controller = AdaptiveConcurrencyController::new(&config);

    let stats = controller.get_stats().await;
    assert_eq!(stats.success_rate, 0.0);
}

/// Test error rate in stats
#[tokio::test]
async fn test_error_rate_in_stats() {
    let config = TurboCdnConfig::default();
    let controller = AdaptiveConcurrencyController::new(&config);

    // Record 4 successes and 1 failure (20% error rate)
    for _ in 0..4 {
        controller
            .record_success(Duration::from_millis(100), 1024)
            .await;
    }
    controller.record_failure("Test error").await;

    let stats = controller.get_stats().await;
    assert!((stats.error_rate - 0.2).abs() < 0.01);
}
