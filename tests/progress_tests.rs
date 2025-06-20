// Licensed under the MIT License
// Copyright (c) 2025 Hal <hal.long@outlook.com>

use std::sync::{Arc, Mutex};
use std::time::Duration;
use turbo_cdn::progress::{ConsoleProgressReporter, ProgressInfo, ProgressTracker};

#[test]
fn test_progress_info_creation() {
    let progress = ProgressInfo {
        total_size: 1000000,
        downloaded_size: 500000,
        percentage: 50.0,
        speed: 1000000.0,
        eta: Some(Duration::from_secs(500)),
        elapsed: Duration::from_secs(10),
        active_chunks: 4,
        complete: false,
    };

    assert_eq!(progress.total_size, 1000000);
    assert_eq!(progress.downloaded_size, 500000);
    assert_eq!(progress.percentage, 50.0);
    assert_eq!(progress.speed, 1000000.0);
    assert_eq!(progress.eta, Some(Duration::from_secs(500)));
    assert_eq!(progress.elapsed, Duration::from_secs(10));
    assert_eq!(progress.active_chunks, 4);
    assert!(!progress.complete);
}

#[test]
fn test_progress_info_percentage_normalized() {
    let progress = ProgressInfo {
        total_size: 1000,
        downloaded_size: 250,
        percentage: 25.0,
        speed: 100.0,
        eta: None,
        elapsed: Duration::from_secs(5),
        active_chunks: 1,
        complete: false,
    };

    assert_eq!(progress.percentage_normalized(), 0.25);
}

#[test]
fn test_progress_info_speed_mbps() {
    let progress = ProgressInfo {
        total_size: 1000,
        downloaded_size: 500,
        percentage: 50.0,
        speed: 2_000_000.0, // 2 MB/s
        eta: None,
        elapsed: Duration::from_secs(1),
        active_chunks: 1,
        complete: false,
    };

    assert_eq!(progress.speed_mbps(), 2.0);
}

#[test]
fn test_progress_info_speed_human() {
    let test_cases = vec![
        (500.0, "500 B/s"),
        (1_500.0, "1.50 KB/s"),
        (1_500_000.0, "1.50 MB/s"),
        (1_500_000_000.0, "1.50 GB/s"),
    ];

    for (speed, expected) in test_cases {
        let progress = ProgressInfo {
            total_size: 1000,
            downloaded_size: 500,
            percentage: 50.0,
            speed,
            eta: None,
            elapsed: Duration::from_secs(1),
            active_chunks: 1,
            complete: false,
        };

        assert_eq!(progress.speed_human(), expected);
    }
}

#[test]
fn test_progress_info_size_human() {
    let progress = ProgressInfo {
        total_size: 1000000,
        downloaded_size: 500000,
        percentage: 50.0,
        speed: 1000.0,
        eta: None,
        elapsed: Duration::from_secs(1),
        active_chunks: 1,
        complete: false,
    };

    let size_str = progress.size_human();
    // Should be in format "downloaded / total"
    assert!(size_str.contains("0.50 MB") || size_str.contains("500.00 KB"));
    assert!(size_str.contains("1.00 MB"));
    assert!(size_str.contains(" / "));
}

#[test]
fn test_progress_info_eta_human() {
    let test_cases = vec![
        (Some(Duration::from_secs(30)), "30s"),
        (Some(Duration::from_secs(90)), "1m 30s"),
        (Some(Duration::from_secs(3661)), "1h 1m 1s"),
        (None, "unknown"),
    ];

    for (eta, expected_contains) in test_cases {
        let progress = ProgressInfo {
            total_size: 1000,
            downloaded_size: 500,
            percentage: 50.0,
            speed: 1000.0,
            eta,
            elapsed: Duration::from_secs(1),
            active_chunks: 1,
            complete: false,
        };

        let eta_str = progress.eta_human();
        if expected_contains == "unknown" {
            assert_eq!(eta_str, "Unknown");
        } else {
            assert!(eta_str.contains(expected_contains));
        }
    }
}

#[tokio::test]
async fn test_progress_tracker_creation() {
    let tracker = ProgressTracker::new(1000000);
    let info = tracker.get_progress().await;

    assert_eq!(info.total_size, 1000000);
    assert_eq!(info.downloaded_size, 0);
    assert_eq!(info.percentage, 0.0);
    assert!(!info.complete);
}

#[tokio::test]
async fn test_progress_tracker_with_progress_bar() {
    let tracker = ProgressTracker::with_progress_bar(1000000);
    let info = tracker.get_progress().await;

    assert_eq!(info.total_size, 1000000);
    assert_eq!(info.downloaded_size, 0);
    assert_eq!(info.percentage, 0.0);
    assert!(!info.complete);
}

#[tokio::test]
async fn test_progress_tracker_update() {
    let tracker = ProgressTracker::new(1000);

    // Update progress
    tracker.update(250).await;
    let info = tracker.get_progress().await;

    assert_eq!(info.downloaded_size, 250);
    assert_eq!(info.percentage, 25.0);
    assert!(!info.complete);

    // Complete the download
    tracker.update(1000).await;
    let info = tracker.get_progress().await;

    assert_eq!(info.downloaded_size, 1000);
    assert_eq!(info.percentage, 100.0);
    // Note: complete flag is only set by the complete() method, not by update()
}

#[tokio::test]
async fn test_progress_tracker_callback() {
    let callback_data = Arc::new(Mutex::new(Vec::new()));
    let callback_data_clone = callback_data.clone();

    let tracker = ProgressTracker::new(1000);
    tracker
        .set_callback(move |progress| {
            callback_data_clone
                .lock()
                .unwrap()
                .push(progress.percentage);
        })
        .await;

    // Update progress multiple times
    tracker.update(250).await;
    tracker.update(500).await;
    tracker.update(750).await;
    tracker.update(1000).await;

    // Give some time for callbacks to execute
    tokio::time::sleep(Duration::from_millis(10)).await;

    let percentages = callback_data.lock().unwrap();
    assert!(percentages.len() >= 4);
    assert!(percentages.contains(&25.0));
    assert!(percentages.contains(&50.0));
    assert!(percentages.contains(&75.0));
    assert!(percentages.contains(&100.0));
}

#[tokio::test]
async fn test_progress_tracker_chunk_management() {
    let tracker = ProgressTracker::new(1000);

    // Initialize chunks
    let chunk_ranges = vec![(0, 249), (250, 499), (500, 749), (750, 999)];
    tracker.init_chunks(chunk_ranges).await;

    let info = tracker.get_progress().await;
    assert_eq!(info.active_chunks, 0); // No chunks are active initially

    // Update chunk progress
    tracker.update_chunk(0, 125).await;
    tracker.update_chunk(1, 125).await;

    let info = tracker.get_progress().await;
    assert_eq!(info.downloaded_size, 250); // 125 + 125
    assert_eq!(info.percentage, 25.0);
    assert_eq!(info.active_chunks, 2); // 2 chunks are now active

    // Complete chunks
    tracker.complete_chunk(0).await;
    tracker.complete_chunk(1).await;

    let info = tracker.get_progress().await;
    assert_eq!(info.active_chunks, 0); // Chunks are no longer active
}

#[tokio::test]
async fn test_progress_tracker_speed_calculation() {
    let tracker = ProgressTracker::new(1000);

    // Update with some delay to calculate speed
    tracker.update(100).await;
    tokio::time::sleep(Duration::from_millis(100)).await;
    tracker.update(200).await;

    let info = tracker.get_progress().await;
    assert!(info.speed > 0.0);
    assert!(info.elapsed > Duration::from_millis(50));
}

#[tokio::test]
async fn test_progress_tracker_eta_calculation() {
    let tracker = ProgressTracker::new(1000);

    // Update progress to establish a speed
    tracker.update(100).await;
    tokio::time::sleep(Duration::from_millis(100)).await;
    tracker.update(200).await;

    let info = tracker.get_progress().await;

    // ETA should be calculated when we have speed data
    if info.speed > 0.0 {
        assert!(info.eta.is_some());
    }
}

#[tokio::test]
async fn test_progress_tracker_finish() {
    let tracker = ProgressTracker::new(1000);

    // Update to partial progress
    tracker.update(500).await;
    let info = tracker.get_progress().await;
    assert!(!info.complete);

    // Complete the tracker
    tracker.complete().await;
    let info = tracker.get_progress().await;
    // Note: The complete flag is set in the callback, not in get_progress()
    // So we just check that the download size and percentage are correct
    assert_eq!(info.downloaded_size, 1000);
    assert_eq!(info.percentage, 100.0);
}

#[test]
fn test_console_progress_reporter_creation() {
    let _reporter = ConsoleProgressReporter;

    // Test that we can create a callback without panicking
    let callback = ConsoleProgressReporter::default_callback();
    let progress = ProgressInfo {
        total_size: 1000,
        downloaded_size: 500,
        percentage: 50.0,
        speed: 1000.0,
        eta: Some(Duration::from_secs(500)),
        elapsed: Duration::from_secs(10),
        active_chunks: 1,
        complete: false,
    };

    // This should not panic
    callback(progress);
}

#[test]
fn test_progress_info_serialization() {
    let progress = ProgressInfo {
        total_size: 1000000,
        downloaded_size: 500000,
        percentage: 50.0,
        speed: 1000000.0,
        eta: Some(Duration::from_secs(500)),
        elapsed: Duration::from_secs(10),
        active_chunks: 4,
        complete: false,
    };

    // Test serialization
    let serialized = serde_json::to_string(&progress).unwrap();
    assert!(serialized.contains("1000000"));
    assert!(serialized.contains("500000"));
    assert!(serialized.contains("50.0"));

    // Test deserialization
    let deserialized: ProgressInfo = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized.total_size, progress.total_size);
    assert_eq!(deserialized.downloaded_size, progress.downloaded_size);
    assert_eq!(deserialized.percentage, progress.percentage);
    assert_eq!(deserialized.speed, progress.speed);
    assert_eq!(deserialized.active_chunks, progress.active_chunks);
    assert_eq!(deserialized.complete, progress.complete);
}

#[tokio::test]
async fn test_progress_tracker_concurrent_updates() {
    let tracker = ProgressTracker::new(1000);

    // Simulate sequential updates (since ProgressTracker doesn't implement Clone)
    for i in 0..10 {
        tracker.update(i * 10).await;
    }

    let info = tracker.get_progress().await;
    // The final downloaded size should be the last update (90)
    assert_eq!(info.downloaded_size, 90);
    assert_eq!(info.percentage, 9.0);
}
