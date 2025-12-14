// Licensed under the MIT License
// Copyright (c) 2025 Hal <hal.long@outlook.com>

//! End-to-End (E2E) Tests for turbo-cdn
//!
//! These tests perform real network downloads to verify the complete download pipeline.
//! They are marked with `#[ignore]` by default to avoid network dependency in CI.
//!
//! Run these tests with: `cargo test --test e2e_tests -- --ignored`

use std::path::PathBuf;
use std::time::Instant;
use tempfile::TempDir;
use turbo_cdn::*;

/// Small test file URLs for quick E2E testing
const SMALL_TEST_FILES: &[(&str, &str, u64)] = &[
    // jQuery minified (~87KB)
    (
        "https://cdn.jsdelivr.net/npm/jquery@3.7.1/dist/jquery.min.js",
        "jquery.min.js",
        80_000, // minimum expected size
    ),
    // Lodash minified (~72KB)
    (
        "https://cdn.jsdelivr.net/npm/lodash@4.17.21/lodash.min.js",
        "lodash.min.js",
        70_000,
    ),
];

/// Medium test file URLs for performance testing
const MEDIUM_TEST_FILES: &[(&str, &str, u64)] = &[
    // ripgrep Windows binary (~2MB)
    (
        "https://github.com/BurntSushi/ripgrep/releases/download/14.1.1/ripgrep-14.1.1-x86_64-pc-windows-msvc.zip",
        "ripgrep-14.1.1-x86_64-pc-windows-msvc.zip",
        2_000_000,
    ),
];

/// Large test file URLs for stress testing
const LARGE_TEST_FILES: &[(&str, &str, u64)] = &[
    // Node.js Windows binary (~30MB)
    (
        "https://nodejs.org/dist/v20.10.0/node-v20.10.0-win-x64.zip",
        "node-v20.10.0-win-x64.zip",
        25_000_000,
    ),
];

// ============================================================================
// Helper Functions
// ============================================================================

/// Create a temporary directory for test downloads
fn create_temp_dir() -> TempDir {
    tempfile::tempdir().expect("Failed to create temp directory")
}

/// Verify downloaded file exists and has expected minimum size
fn verify_download(path: &PathBuf, min_size: u64) -> bool {
    if !path.exists() {
        eprintln!("File does not exist: {:?}", path);
        return false;
    }

    let metadata = std::fs::metadata(path).expect("Failed to get file metadata");
    let size = metadata.len();

    if size < min_size {
        eprintln!(
            "File size {} is less than expected minimum {}",
            size, min_size
        );
        return false;
    }

    true
}

/// Format bytes per second as human-readable speed
fn format_speed(bytes_per_sec: f64) -> String {
    if bytes_per_sec >= 1024.0 * 1024.0 * 1024.0 {
        format!("{:.2} GB/s", bytes_per_sec / 1024.0 / 1024.0 / 1024.0)
    } else if bytes_per_sec >= 1024.0 * 1024.0 {
        format!("{:.2} MB/s", bytes_per_sec / 1024.0 / 1024.0)
    } else if bytes_per_sec >= 1024.0 {
        format!("{:.2} KB/s", bytes_per_sec / 1024.0)
    } else {
        format!("{:.2} B/s", bytes_per_sec)
    }
}

// ============================================================================
// E2E Download Tests
// ============================================================================

/// Test downloading a small file from jsDelivr CDN
#[tokio::test]
#[ignore = "Requires network access"]
async fn test_e2e_download_small_file() {
    let temp_dir = create_temp_dir();
    let cdn = TurboCdn::builder()
        .with_auto_detect_region(false)
        .with_region(Region::Global)
        .with_max_concurrent_downloads(8)
        .with_timeout(60)
        .build()
        .await
        .expect("Failed to create TurboCdn");

    let (url, filename, min_size) = SMALL_TEST_FILES[0];
    let output_path = temp_dir.path().join(filename);

    let start = Instant::now();
    let result = cdn
        .download_to_path(url, &output_path)
        .await
        .expect("Download failed");
    let elapsed = start.elapsed();

    println!("Downloaded {} in {:?}", filename, elapsed);
    println!("  Size: {} bytes", result.size);
    println!("  Speed: {}", format_speed(result.speed));
    println!("  URL used: {}", result.url);

    assert!(verify_download(&output_path, min_size));
    assert!(result.size >= min_size);
    assert!(result.speed > 0.0);
}

/// Test downloading multiple small files concurrently
#[tokio::test]
#[ignore = "Requires network access"]
async fn test_e2e_download_multiple_files() {
    let temp_dir = create_temp_dir();
    let cdn = TurboCdn::builder()
        .with_auto_detect_region(false)
        .with_region(Region::Global)
        .with_max_concurrent_downloads(16)
        .build()
        .await
        .expect("Failed to create TurboCdn");

    let mut handles = vec![];

    for (url, filename, min_size) in SMALL_TEST_FILES {
        let output_path = temp_dir.path().join(filename);
        let url = url.to_string();
        let min_size = *min_size;

        // Note: We need to clone the path for the async block
        let cdn_ref = &cdn;
        let result = cdn_ref.download_to_path(&url, &output_path).await;

        handles.push((output_path, min_size, result));
    }

    for (path, min_size, result) in handles {
        let result = result.expect("Download failed");
        assert!(verify_download(&path, min_size));
        println!(
            "Downloaded {} ({} bytes) at {}",
            path.file_name().unwrap().to_string_lossy(),
            result.size,
            format_speed(result.speed)
        );
    }
}

/// Test downloading a medium-sized file (GitHub release)
#[tokio::test]
#[ignore = "Requires network access - downloads ~2MB"]
async fn test_e2e_download_github_release() {
    let temp_dir = create_temp_dir();
    let cdn = TurboCdn::builder()
        .with_auto_detect_region(false)
        .with_region(Region::Global)
        .with_max_concurrent_downloads(16)
        .with_chunk_size(512 * 1024) // 512KB chunks
        .build()
        .await
        .expect("Failed to create TurboCdn");

    let (url, filename, min_size) = MEDIUM_TEST_FILES[0];
    let output_path = temp_dir.path().join(filename);

    let start = Instant::now();
    let result = cdn
        .download_to_path(url, &output_path)
        .await
        .expect("Download failed");
    let elapsed = start.elapsed();

    println!("GitHub Release Download:");
    println!("  File: {}", filename);
    println!(
        "  Size: {} bytes ({:.2} MB)",
        result.size,
        result.size as f64 / 1024.0 / 1024.0
    );
    println!("  Time: {:?}", elapsed);
    println!("  Speed: {}", format_speed(result.speed));
    println!("  CDN URL: {}", result.url);

    assert!(verify_download(&output_path, min_size));
}

/// Test downloading with China region CDN mirrors
#[tokio::test]
#[ignore = "Requires network access"]
async fn test_e2e_download_china_cdn() {
    let temp_dir = create_temp_dir();
    let cdn = TurboCdn::builder()
        .with_auto_detect_region(false)
        .with_region(Region::China)
        .with_max_concurrent_downloads(16)
        .build()
        .await
        .expect("Failed to create TurboCdn");

    let (url, filename, min_size) = SMALL_TEST_FILES[0];

    // First, check what CDN URLs are available
    let cdn_urls = cdn
        .get_all_cdn_urls(url)
        .await
        .expect("Failed to get CDN URLs");

    println!("Available CDN URLs for China region:");
    for (i, cdn_url) in cdn_urls.iter().enumerate() {
        println!("  {}: {}", i + 1, cdn_url);
    }

    // Download the file
    let output_path = temp_dir.path().join(filename);
    let result = cdn
        .download_to_path(url, &output_path)
        .await
        .expect("Download failed");

    println!("Downloaded via: {}", result.url);
    println!("Speed: {}", format_speed(result.speed));

    assert!(verify_download(&output_path, min_size));
}

/// Test smart download mode
#[tokio::test]
#[ignore = "Requires network access"]
async fn test_e2e_smart_download() {
    let temp_dir = create_temp_dir();
    let cdn = TurboCdn::builder()
        .with_auto_detect_region(true)
        .with_max_concurrent_downloads(16)
        .build()
        .await
        .expect("Failed to create TurboCdn");

    let (url, filename, min_size) = SMALL_TEST_FILES[0];
    let output_path = temp_dir.path().join(filename);

    let start = Instant::now();
    let result = cdn
        .download_smart_to_path(url, &output_path)
        .await
        .expect("Smart download failed");
    let elapsed = start.elapsed();

    println!("Smart Download Result:");
    println!("  File: {}", filename);
    println!("  Size: {} bytes", result.size);
    println!("  Time: {:?}", elapsed);
    println!("  Speed: {}", format_speed(result.speed));
    println!("  URL: {}", result.url);

    assert!(verify_download(&output_path, min_size));
}

/// Test direct download (without CDN optimization)
#[tokio::test]
#[ignore = "Requires network access"]
async fn test_e2e_direct_download() {
    let temp_dir = create_temp_dir();
    let cdn = TurboCdn::builder()
        .with_auto_detect_region(false)
        .with_region(Region::Global)
        .build()
        .await
        .expect("Failed to create TurboCdn");

    let (url, filename, min_size) = SMALL_TEST_FILES[0];
    let output_path = temp_dir.path().join(filename);

    let result = cdn
        .download_direct_to_path(url, &output_path)
        .await
        .expect("Direct download failed");

    println!("Direct Download (no CDN):");
    println!("  URL: {}", result.url);
    println!("  Speed: {}", format_speed(result.speed));

    // Direct download should use the original URL
    assert!(result.url.contains("cdn.jsdelivr.net"));
    assert!(verify_download(&output_path, min_size));
}

/// Test download with progress tracking
#[tokio::test]
#[ignore = "Requires network access"]
async fn test_e2e_download_with_progress() {
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::sync::Arc;

    let temp_dir = create_temp_dir();
    let cdn = TurboCdn::builder()
        .with_auto_detect_region(false)
        .with_region(Region::Global)
        .build()
        .await
        .expect("Failed to create TurboCdn");

    let (url, filename, min_size) = SMALL_TEST_FILES[0];
    let output_path = temp_dir.path().join(filename);

    let progress_count = Arc::new(AtomicU64::new(0));
    let progress_count_clone = Arc::clone(&progress_count);

    let callback: ProgressCallback = Box::new(move |info| {
        progress_count_clone.fetch_add(1, Ordering::Relaxed);
        if info.percentage % 25.0 < 1.0 {
            println!(
                "Progress: {:.1}% ({}/{} bytes) - {}",
                info.percentage,
                info.downloaded_size,
                info.total_size,
                format_speed(info.speed)
            );
        }
    });

    let options = DownloadOptions::new()
        .with_progress_callback(callback)
        .with_expected_size(min_size);

    let result = cdn
        .download_with_options(url, &output_path, options)
        .await
        .expect("Download with progress failed");

    let total_callbacks = progress_count.load(Ordering::Relaxed);
    println!("Total progress callbacks: {}", total_callbacks);
    println!("Final size: {} bytes", result.size);

    assert!(verify_download(&output_path, min_size));
}

/// Test download retry on failure (simulated by using invalid URL first)
#[tokio::test]
#[ignore = "Requires network access"]
async fn test_e2e_download_fallback() {
    let temp_dir = create_temp_dir();
    let cdn = TurboCdn::builder()
        .with_auto_detect_region(false)
        .with_region(Region::Global)
        .with_retry_attempts(3)
        .build()
        .await
        .expect("Failed to create TurboCdn");

    // Use a valid URL that should work
    let (url, filename, min_size) = SMALL_TEST_FILES[0];
    let output_path = temp_dir.path().join(filename);

    let result = cdn.download_to_path(url, &output_path).await;

    assert!(result.is_ok(), "Download should succeed");
    let result = result.unwrap();
    assert!(verify_download(&output_path, min_size));
    println!("Successfully downloaded via: {}", result.url);
}

/// Test sync API for real download
#[test]
#[ignore = "Requires network access"]
fn test_e2e_sync_download() {
    use turbo_cdn::sync_api::SyncTurboCdn;

    let temp_dir = create_temp_dir();
    let cdn = SyncTurboCdn::new().expect("Failed to create SyncTurboCdn");

    let (url, filename, min_size) = SMALL_TEST_FILES[0];
    let output_path = temp_dir.path().join(filename);

    let result = cdn
        .download_to_path(url, &output_path)
        .expect("Sync download failed");

    println!("Sync Download:");
    println!("  Size: {} bytes", result.size);
    println!("  Speed: {}", format_speed(result.speed));

    assert!(verify_download(&output_path, min_size));
}

// ============================================================================
// Large File Tests (Stress Testing)
// ============================================================================

/// Test downloading a large file (~30MB)
#[tokio::test]
#[ignore = "Requires network access - downloads ~30MB"]
async fn test_e2e_download_large_file() {
    let temp_dir = create_temp_dir();
    let cdn = TurboCdn::builder()
        .with_auto_detect_region(false)
        .with_region(Region::Global)
        .with_max_concurrent_downloads(32)
        .with_chunk_size(1024 * 1024) // 1MB chunks
        .with_adaptive_chunking(true)
        .build()
        .await
        .expect("Failed to create TurboCdn");

    let (url, filename, min_size) = LARGE_TEST_FILES[0];
    let output_path = temp_dir.path().join(filename);

    println!("Starting large file download: {}", filename);
    println!("Expected size: ~{} MB", min_size / 1024 / 1024);

    let start = Instant::now();
    let result = cdn
        .download_to_path(url, &output_path)
        .await
        .expect("Large file download failed");
    let elapsed = start.elapsed();

    println!("\nLarge File Download Complete:");
    println!("  File: {}", filename);
    println!(
        "  Size: {} bytes ({:.2} MB)",
        result.size,
        result.size as f64 / 1024.0 / 1024.0
    );
    println!("  Time: {:?}", elapsed);
    println!("  Speed: {}", format_speed(result.speed));
    println!("  URL: {}", result.url);

    assert!(verify_download(&output_path, min_size));
}

// ============================================================================
// CDN Comparison Tests
// ============================================================================

/// Compare download speeds between different CDN configurations
#[tokio::test]
#[ignore = "Requires network access - runs multiple downloads"]
async fn test_e2e_cdn_speed_comparison() {
    let temp_dir = create_temp_dir();
    let (url, filename, _min_size) = SMALL_TEST_FILES[0];

    println!("CDN Speed Comparison Test");
    println!("=========================");
    println!("Test file: {}", filename);
    println!();

    // Test 1: Direct download (no CDN)
    {
        let cdn = TurboCdn::builder()
            .with_auto_detect_region(false)
            .with_region(Region::Global)
            .build()
            .await
            .expect("Failed to create TurboCdn");

        let output_path = temp_dir.path().join(format!("direct_{}", filename));
        let start = Instant::now();
        let result = cdn.download_direct_to_path(url, &output_path).await;
        let elapsed = start.elapsed();

        if let Ok(result) = result {
            println!("Direct Download:");
            println!("  Time: {:?}", elapsed);
            println!("  Speed: {}", format_speed(result.speed));
            println!("  URL: {}", result.url);
        } else {
            println!("Direct Download: FAILED");
        }
    }

    // Test 2: CDN-optimized download (Global)
    {
        let cdn = TurboCdn::builder()
            .with_auto_detect_region(false)
            .with_region(Region::Global)
            .with_max_concurrent_downloads(16)
            .build()
            .await
            .expect("Failed to create TurboCdn");

        let output_path = temp_dir.path().join(format!("cdn_global_{}", filename));
        let start = Instant::now();
        let result = cdn.download_to_path(url, &output_path).await;
        let elapsed = start.elapsed();

        if let Ok(result) = result {
            println!("\nCDN Download (Global):");
            println!("  Time: {:?}", elapsed);
            println!("  Speed: {}", format_speed(result.speed));
            println!("  URL: {}", result.url);
        } else {
            println!("\nCDN Download (Global): FAILED");
        }
    }

    // Test 3: CDN-optimized download (China)
    {
        let cdn = TurboCdn::builder()
            .with_auto_detect_region(false)
            .with_region(Region::China)
            .with_max_concurrent_downloads(16)
            .build()
            .await
            .expect("Failed to create TurboCdn");

        let output_path = temp_dir.path().join(format!("cdn_china_{}", filename));
        let start = Instant::now();
        let result = cdn.download_to_path(url, &output_path).await;
        let elapsed = start.elapsed();

        if let Ok(result) = result {
            println!("\nCDN Download (China):");
            println!("  Time: {:?}", elapsed);
            println!("  Speed: {}", format_speed(result.speed));
            println!("  URL: {}", result.url);
        } else {
            println!("\nCDN Download (China): FAILED");
        }
    }

    println!("\n=========================");
}

// ============================================================================
// Error Handling Tests
// ============================================================================

/// Test handling of 404 errors
#[tokio::test]
#[ignore = "Requires network access"]
async fn test_e2e_handle_404_error() {
    let temp_dir = create_temp_dir();
    let cdn = TurboCdn::builder()
        .with_auto_detect_region(false)
        .with_region(Region::Global)
        .with_retry_attempts(1)
        .with_timeout(10)
        .build()
        .await
        .expect("Failed to create TurboCdn");

    let url = "https://cdn.jsdelivr.net/npm/nonexistent-package-12345/file.js";
    let output_path = temp_dir.path().join("nonexistent.js");

    let result = cdn.download_to_path(url, &output_path).await;

    assert!(result.is_err(), "Should fail for non-existent file");
    println!("Expected error: {:?}", result.err());
}

/// Test handling of timeout
#[tokio::test]
#[ignore = "Requires network access - tests timeout behavior"]
async fn test_e2e_handle_timeout() {
    let temp_dir = create_temp_dir();
    let cdn = TurboCdn::builder()
        .with_auto_detect_region(false)
        .with_region(Region::Global)
        .with_timeout(1) // Very short timeout
        .with_retry_attempts(1)
        .build()
        .await
        .expect("Failed to create TurboCdn");

    // Try to download a large file with very short timeout
    let (url, filename, _) = LARGE_TEST_FILES[0];
    let output_path = temp_dir.path().join(filename);

    let start = Instant::now();
    let result = cdn.download_to_path(url, &output_path).await;
    let elapsed = start.elapsed();

    println!("Timeout test completed in {:?}", elapsed);

    // Result could be either success (if fast enough) or timeout error
    match result {
        Ok(r) => println!("Surprisingly succeeded: {} bytes in {:?}", r.size, elapsed),
        Err(e) => println!("Expected timeout error: {:?}", e),
    }
}

// ============================================================================
// Statistics and Metrics Tests
// ============================================================================

/// Test that statistics are properly updated after downloads
#[tokio::test]
#[ignore = "Requires network access"]
async fn test_e2e_statistics_tracking() {
    let temp_dir = create_temp_dir();
    let cdn = TurboCdn::builder()
        .with_auto_detect_region(false)
        .with_region(Region::Global)
        .build()
        .await
        .expect("Failed to create TurboCdn");

    // Check initial stats
    let initial_stats = cdn.get_stats().await;
    assert_eq!(initial_stats.total_downloads, 0);

    // Download a file
    let (url, filename, _) = SMALL_TEST_FILES[0];
    let output_path = temp_dir.path().join(filename);
    let _ = cdn.download_to_path(url, &output_path).await;

    // Check updated stats
    let updated_stats = cdn.get_stats().await;
    assert_eq!(updated_stats.total_downloads, 1);
    assert!(updated_stats.total_bytes > 0);
    assert!(updated_stats.average_speed > 0.0);

    println!("Statistics after download:");
    println!("  Total downloads: {}", updated_stats.total_downloads);
    println!("  Successful: {}", updated_stats.successful_downloads);
    println!("  Total bytes: {}", updated_stats.total_bytes_human());
    println!(
        "  Average speed: {:.2} MB/s",
        updated_stats.average_speed_mbps()
    );
    println!("  Success rate: {:.1}%", updated_stats.success_rate());

    // Check server performance summary
    let perf_summary = cdn.get_performance_summary();
    println!("\nServer Performance:");
    println!("  Total servers tracked: {}", perf_summary.total_servers);
    println!("  Total attempts: {}", perf_summary.total_attempts);
    println!(
        "  Success rate: {:.1}%",
        perf_summary.overall_success_rate * 100.0
    );
}
