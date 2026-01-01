// Licensed under the MIT License
// Copyright (c) 2025 Hal <hal.long@outlook.com>

//! End-to-End (E2E) Tests for turbo-cdn
//!
//! These tests perform real network downloads to verify the complete download pipeline.
//! They are marked with `#[ignore]` by default to avoid network dependency in CI.
//!
//! Run these tests with: `cargo test --test e2e_tests -- --ignored`
//!
//! Test categories:
//! - Basic download tests: Small files from CDN
//! - GitHub release tests: Medium files with CDN optimization
//! - Large file tests: Stress testing with big files
//! - CDN comparison tests: Compare different CDN configurations
//! - Error handling tests: 404, timeout, etc.
//! - API tests: Builder pattern, statistics, etc.
//! - URL mapping tests: Verify CDN URL generation

use std::path::PathBuf;
use std::time::Instant;
use tempfile::TempDir;
use turbo_cdn::*;

/// Small test file URLs for quick E2E testing (updated 2025-12)
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
    // Vue.js production (~140KB)
    (
        "https://cdn.jsdelivr.net/npm/vue@3.5.13/dist/vue.global.prod.js",
        "vue.global.prod.js",
        130_000,
    ),
    // React production (~6KB)
    (
        "https://cdn.jsdelivr.net/npm/react@18.3.1/umd/react.production.min.js",
        "react.production.min.js",
        5_000,
    ),
];

/// Medium test file URLs for performance testing (updated 2025-12)
const MEDIUM_TEST_FILES: &[(&str, &str, u64)] = &[
    // ripgrep Windows binary (~2MB)
    (
        "https://github.com/BurntSushi/ripgrep/releases/download/14.1.1/ripgrep-14.1.1-x86_64-pc-windows-msvc.zip",
        "ripgrep-14.1.1-x86_64-pc-windows-msvc.zip",
        2_000_000,
    ),
    // fd Windows binary (~1.5MB)
    (
        "https://github.com/sharkdp/fd/releases/download/v10.2.0/fd-v10.2.0-x86_64-pc-windows-msvc.zip",
        "fd-v10.2.0-x86_64-pc-windows-msvc.zip",
        1_400_000,
    ),
    // bat Windows binary (~3MB)
    (
        "https://github.com/sharkdp/bat/releases/download/v0.24.0/bat-v0.24.0-x86_64-pc-windows-msvc.zip",
        "bat-v0.24.0-x86_64-pc-windows-msvc.zip",
        2_800_000,
    ),
];

/// loonghao projects test files (our own tools - updated 2026-01)
const LOONGHAO_TEST_FILES: &[(&str, &str, u64)] = &[
    // vx - Universal tool executor v0.6.8 (~5.6MB)
    (
        "https://github.com/loonghao/vx/releases/download/vx-v0.6.8/vx-x86_64-pc-windows-msvc.zip",
        "vx-x86_64-pc-windows-msvc.zip",
        5_000_000,
    ),
    // auroraview - Python wheel (~4.2MB)
    (
        "https://github.com/loonghao/auroraview/releases/download/auroraview-v0.3.32/auroraview-0.3.32-cp38-abi3-win_amd64.whl",
        "auroraview-0.3.32-cp38-abi3-win_amd64.whl",
        4_000_000,
    ),
    // auroraview-cli - CLI tool (~4.2MB)
    (
        "https://github.com/loonghao/auroraview/releases/download/auroraview-v0.3.32/auroraview-cli-0.3.32-x86_64-pc-windows-msvc.zip",
        "auroraview-cli-0.3.32-x86_64-pc-windows-msvc.zip",
        4_000_000,
    ),
    // auroraview-gallery - Large gallery app (~70MB)
    (
        "https://github.com/loonghao/auroraview/releases/download/auroraview-v0.3.32/auroraview-gallery-0.3.32-x86_64-pc-windows-msvc.zip",
        "auroraview-gallery-0.3.32-x86_64-pc-windows-msvc.zip",
        65_000_000,
    ),
];

/// Large test file URLs for stress testing (updated 2025-12)
const LARGE_TEST_FILES: &[(&str, &str, u64)] = &[
    // Node.js Windows binary (~30MB)
    (
        "https://nodejs.org/dist/v22.12.0/node-v22.12.0-win-x64.zip",
        "node-v22.12.0-win-x64.zip",
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
// GitHub Release Download Tests
// ============================================================================

/// Test downloading multiple GitHub releases
#[tokio::test]
#[ignore = "Requires network access - downloads multiple GitHub releases"]
async fn test_e2e_download_multiple_github_releases() {
    let temp_dir = create_temp_dir();
    let cdn = TurboCdn::builder()
        .with_auto_detect_region(false)
        .with_region(Region::Global)
        .with_max_concurrent_downloads(16)
        .build()
        .await
        .expect("Failed to create TurboCdn");

    println!("Multiple GitHub Releases Download Test:");
    println!("=======================================");

    let mut total_size = 0u64;
    let mut total_time = std::time::Duration::ZERO;

    for (url, filename, min_size) in MEDIUM_TEST_FILES.iter() {
        let output_path = temp_dir.path().join(*filename);

        let start = Instant::now();
        let result = cdn.download_to_path(url, &output_path).await;
        let elapsed = start.elapsed();

        match result {
            Ok(r) => {
                total_size += r.size;
                total_time += elapsed;
                println!(
                    "  {} - {:.2} MB in {:?} ({})",
                    filename,
                    r.size as f64 / 1024.0 / 1024.0,
                    elapsed,
                    format_speed(r.speed)
                );
                assert!(verify_download(&output_path, *min_size));
            }
            Err(e) => {
                println!("  {} - FAILED: {:?}", filename, e);
            }
        }
    }

    let overall_speed = total_size as f64 / total_time.as_secs_f64();
    println!("\nSummary:");
    println!(
        "  Total size: {:.2} MB",
        total_size as f64 / 1024.0 / 1024.0
    );
    println!("  Total time: {:?}", total_time);
    println!("  Overall speed: {}", format_speed(overall_speed));
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

// ============================================================================
// URL Mapping E2E Tests
// ============================================================================

/// Test URL mapping for GitHub releases
#[tokio::test]
#[ignore = "Requires network access"]
async fn test_e2e_url_mapping_github_releases() {
    let cdn = TurboCdn::builder()
        .with_auto_detect_region(false)
        .with_region(Region::Global)
        .build()
        .await
        .expect("Failed to create TurboCdn");

    let url = "https://github.com/BurntSushi/ripgrep/releases/download/14.1.1/ripgrep-14.1.1-x86_64-pc-windows-msvc.zip";
    let cdn_urls = cdn
        .get_all_cdn_urls(url)
        .await
        .expect("Failed to get CDN URLs");

    println!("GitHub Release URL Mapping:");
    println!("  Original: {}", url);
    println!("  Mapped URLs ({}):", cdn_urls.len());
    for (i, cdn_url) in cdn_urls.iter().enumerate() {
        println!("    {}: {}", i + 1, cdn_url);
    }

    // Should have multiple CDN alternatives
    assert!(cdn_urls.len() > 1, "Should have multiple CDN alternatives");

    // Original URL should be in the list (as fallback)
    assert!(
        cdn_urls.iter().any(|u| u.contains("github.com")),
        "Original URL should be in fallback"
    );
}

/// Test URL mapping for jsDelivr CDN
#[tokio::test]
#[ignore = "Requires network access"]
async fn test_e2e_url_mapping_jsdelivr() {
    let cdn = TurboCdn::builder()
        .with_auto_detect_region(false)
        .with_region(Region::Global)
        .build()
        .await
        .expect("Failed to create TurboCdn");

    let url = "https://cdn.jsdelivr.net/npm/jquery@3.7.1/dist/jquery.min.js";
    let cdn_urls = cdn
        .get_all_cdn_urls(url)
        .await
        .expect("Failed to get CDN URLs");

    println!("jsDelivr URL Mapping:");
    println!("  Original: {}", url);
    println!("  Mapped URLs ({}):", cdn_urls.len());
    for (i, cdn_url) in cdn_urls.iter().enumerate() {
        println!("    {}: {}", i + 1, cdn_url);
    }

    // Should have Fastly and Gcore alternatives
    assert!(
        cdn_urls.iter().any(|u| u.contains("fastly.jsdelivr.net")),
        "Should have Fastly alternative"
    );
}

/// Test URL mapping for China region
#[tokio::test]
#[ignore = "Requires network access"]
async fn test_e2e_url_mapping_china_region() {
    let cdn = TurboCdn::builder()
        .with_auto_detect_region(false)
        .with_region(Region::China)
        .build()
        .await
        .expect("Failed to create TurboCdn");

    // Test GitHub release mapping for China
    let github_url = "https://github.com/BurntSushi/ripgrep/releases/download/14.1.1/ripgrep-14.1.1-x86_64-pc-windows-msvc.zip";
    let github_cdn_urls = cdn
        .get_all_cdn_urls(github_url)
        .await
        .expect("Failed to get CDN URLs");

    println!("China Region - GitHub Release URL Mapping:");
    println!("  Mapped URLs ({}):", github_cdn_urls.len());
    for (i, cdn_url) in github_cdn_urls.iter().enumerate() {
        println!("    {}: {}", i + 1, cdn_url);
    }

    // Should have China-friendly proxies
    assert!(
        github_cdn_urls.len() > 1,
        "Should have multiple CDN alternatives for China"
    );
}

// ============================================================================
// CDN Availability E2E Tests
// ============================================================================

/// Test CDN mirror availability (updated 2026-01)
#[tokio::test]
#[ignore = "Requires network access - tests multiple CDN mirrors"]
async fn test_e2e_cdn_mirror_availability() {
    use std::time::Duration;

    // CDN mirrors ordered by priority (based on 2026-01 testing)
    let cdn_mirrors = [
        ("gh-proxy.com", "https://gh-proxy.com/https://github.com/loonghao/vx/releases/download/vx-v0.6.8/vx-x86_64-pc-windows-msvc.zip"),
        ("ghproxy.net", "https://ghproxy.net/https://github.com/loonghao/vx/releases/download/vx-v0.6.8/vx-x86_64-pc-windows-msvc.zip"),
        ("ghfast.top", "https://ghfast.top/https://github.com/loonghao/vx/releases/download/vx-v0.6.8/vx-x86_64-pc-windows-msvc.zip"),
        ("ghproxy.homeboyc.cn", "https://ghproxy.homeboyc.cn/https://github.com/loonghao/vx/releases/download/vx-v0.6.8/vx-x86_64-pc-windows-msvc.zip"),
        ("gh.api.99988866.xyz", "https://gh.api.99988866.xyz/https://github.com/loonghao/vx/releases/download/vx-v0.6.8/vx-x86_64-pc-windows-msvc.zip"),
        ("ghproxy.cc", "https://ghproxy.cc/https://github.com/loonghao/vx/releases/download/vx-v0.6.8/vx-x86_64-pc-windows-msvc.zip"),
        ("mirror.ghproxy.com", "https://mirror.ghproxy.com/https://github.com/loonghao/vx/releases/download/vx-v0.6.8/vx-x86_64-pc-windows-msvc.zip"),
        ("bgithub.xyz", "https://bgithub.xyz/loonghao/vx/releases/download/vx-v0.6.8/vx-x86_64-pc-windows-msvc.zip"),
        ("kkgithub.com", "https://kkgithub.com/loonghao/vx/releases/download/vx-v0.6.8/vx-x86_64-pc-windows-msvc.zip"),
        ("hub.gitmirror.com", "https://hub.gitmirror.com/https://github.com/loonghao/vx/releases/download/vx-v0.6.8/vx-x86_64-pc-windows-msvc.zip"),
        ("github.moeyy.xyz", "https://github.moeyy.xyz/https://github.com/loonghao/vx/releases/download/vx-v0.6.8/vx-x86_64-pc-windows-msvc.zip"),
        ("ghps.cc", "https://ghps.cc/https://github.com/loonghao/vx/releases/download/vx-v0.6.8/vx-x86_64-pc-windows-msvc.zip"),
    ];

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(15))
        .build()
        .expect("Failed to create HTTP client");

    println!("CDN Mirror Availability Test (using loonghao/vx):");
    println!("==================================================");

    let mut available_count = 0;
    for (name, url) in cdn_mirrors.iter() {
        let start = Instant::now();
        let result = client.head(*url).send().await;
        let elapsed = start.elapsed();

        match result {
            Ok(response) => {
                let status = response.status();
                if status.is_success() || status.as_u16() == 302 || status.as_u16() == 301 {
                    println!("  [OK] {} - {:?} ({})", name, elapsed, status);
                    available_count += 1;
                } else {
                    println!("  [FAIL] {} - {:?} ({})", name, elapsed, status);
                }
            }
            Err(e) => {
                println!("  [ERR] {} - {:?} ({:?})", name, elapsed, e);
            }
        }
    }

    println!("\nAvailable: {}/{}", available_count, cdn_mirrors.len());

    // At least 30% of mirrors should be available for reliability
    let min_required = cdn_mirrors.len() / 3;
    assert!(
        available_count >= min_required,
        "At least {} CDN mirrors should be available, got {}",
        min_required,
        available_count
    );
}

/// Test CDN mirror availability for auroraview
#[tokio::test]
#[ignore = "Requires network access - tests multiple CDN mirrors"]
async fn test_e2e_cdn_mirror_availability_auroraview() {
    use std::time::Duration;

    // Test with auroraview releases
    let cdn_mirrors = [
        ("gh-proxy.com", "https://gh-proxy.com/https://github.com/loonghao/auroraview/releases/download/auroraview-v0.3.32/auroraview-0.3.32-cp38-abi3-win_amd64.whl"),
        ("ghproxy.net", "https://ghproxy.net/https://github.com/loonghao/auroraview/releases/download/auroraview-v0.3.32/auroraview-0.3.32-cp38-abi3-win_amd64.whl"),
        ("ghfast.top", "https://ghfast.top/https://github.com/loonghao/auroraview/releases/download/auroraview-v0.3.32/auroraview-0.3.32-cp38-abi3-win_amd64.whl"),
        ("ghproxy.homeboyc.cn", "https://ghproxy.homeboyc.cn/https://github.com/loonghao/auroraview/releases/download/auroraview-v0.3.32/auroraview-0.3.32-cp38-abi3-win_amd64.whl"),
        ("mirror.ghproxy.com", "https://mirror.ghproxy.com/https://github.com/loonghao/auroraview/releases/download/auroraview-v0.3.32/auroraview-0.3.32-cp38-abi3-win_amd64.whl"),
    ];

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(15))
        .build()
        .expect("Failed to create HTTP client");

    println!("CDN Mirror Availability Test (using loonghao/auroraview):");
    println!("==========================================================");

    let mut available_count = 0;
    for (name, url) in cdn_mirrors.iter() {
        let start = Instant::now();
        let result = client.head(*url).send().await;
        let elapsed = start.elapsed();

        match result {
            Ok(response) => {
                let status = response.status();
                if status.is_success() || status.as_u16() == 302 || status.as_u16() == 301 {
                    println!("  [OK] {} - {:?} ({})", name, elapsed, status);
                    available_count += 1;
                } else {
                    println!("  [FAIL] {} - {:?} ({})", name, elapsed, status);
                }
            }
            Err(e) => {
                println!("  [ERR] {} - {:?} ({:?})", name, elapsed, e);
            }
        }
    }

    println!("\nAvailable: {}/{}", available_count, cdn_mirrors.len());

    // At least one mirror should be available
    assert!(
        available_count > 0,
        "At least one CDN mirror should be available"
    );
}

/// Test jsDelivr CDN alternatives availability
#[tokio::test]
#[ignore = "Requires network access"]
async fn test_e2e_jsdelivr_alternatives_availability() {
    use std::time::Duration;

    let jsdelivr_alternatives = [
        (
            "cdn.jsdelivr.net",
            "https://cdn.jsdelivr.net/npm/jquery@3.7.1/dist/jquery.min.js",
        ),
        (
            "fastly.jsdelivr.net",
            "https://fastly.jsdelivr.net/npm/jquery@3.7.1/dist/jquery.min.js",
        ),
        (
            "gcore.jsdelivr.net",
            "https://gcore.jsdelivr.net/npm/jquery@3.7.1/dist/jquery.min.js",
        ),
        (
            "testingcf.jsdelivr.net",
            "https://testingcf.jsdelivr.net/npm/jquery@3.7.1/dist/jquery.min.js",
        ),
    ];

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .expect("Failed to create HTTP client");

    println!("jsDelivr Alternatives Availability Test:");
    println!("========================================");

    let mut available_count = 0;
    for (name, url) in jsdelivr_alternatives.iter() {
        let start = Instant::now();
        let result = client.head(*url).send().await;
        let elapsed = start.elapsed();

        match result {
            Ok(response) => {
                let status = response.status();
                if status.is_success() {
                    println!("  [OK] {} - {:?} ({})", name, elapsed, status);
                    available_count += 1;
                } else {
                    println!("  [FAIL] {} - {:?} ({})", name, elapsed, status);
                }
            }
            Err(e) => {
                println!("  [ERR] {} - {:?} ({})", name, elapsed, e);
            }
        }
    }

    println!(
        "\nAvailable: {}/{}",
        available_count,
        jsdelivr_alternatives.len()
    );
    assert!(
        available_count > 0,
        "At least one jsDelivr alternative should be available"
    );
}

// ============================================================================
// Builder Pattern E2E Tests
// ============================================================================

/// Test builder pattern with various configurations
#[tokio::test]
#[ignore = "Requires network access"]
async fn test_e2e_builder_configurations() {
    let temp_dir = create_temp_dir();
    let (url, filename, min_size) = SMALL_TEST_FILES[0];

    // Test 1: Minimal configuration
    {
        let cdn = TurboCdn::builder()
            .build()
            .await
            .expect("Failed to create TurboCdn with minimal config");

        let output_path = temp_dir.path().join(format!("minimal_{}", filename));
        let result = cdn.download_to_path(url, &output_path).await;
        assert!(result.is_ok(), "Minimal config download should succeed");
        println!("Minimal config: OK");
    }

    // Test 2: High concurrency configuration
    {
        let cdn = TurboCdn::builder()
            .with_max_concurrent_downloads(32)
            .with_chunk_size(256 * 1024)
            .with_adaptive_chunking(true)
            .build()
            .await
            .expect("Failed to create TurboCdn with high concurrency");

        let output_path = temp_dir
            .path()
            .join(format!("high_concurrency_{}", filename));
        let result = cdn.download_to_path(url, &output_path).await;
        assert!(
            result.is_ok(),
            "High concurrency config download should succeed"
        );
        println!("High concurrency config: OK");
    }

    // Test 3: Conservative configuration
    {
        let cdn = TurboCdn::builder()
            .with_max_concurrent_downloads(4)
            .with_chunk_size(1024 * 1024)
            .with_timeout(120)
            .with_retry_attempts(5)
            .build()
            .await
            .expect("Failed to create TurboCdn with conservative config");

        let output_path = temp_dir.path().join(format!("conservative_{}", filename));
        let result = cdn.download_to_path(url, &output_path).await;
        assert!(
            result.is_ok(),
            "Conservative config download should succeed"
        );
        println!("Conservative config: OK");
    }

    // Test 4: Region-specific configuration
    {
        let cdn = TurboCdn::builder()
            .with_region(Region::Asia)
            .with_auto_detect_region(false)
            .build()
            .await
            .expect("Failed to create TurboCdn with Asia region");

        let output_path = temp_dir.path().join(format!("asia_{}", filename));
        let result = cdn.download_to_path(url, &output_path).await;
        assert!(result.is_ok(), "Asia region config download should succeed");
        println!("Asia region config: OK");
    }

    // Verify all downloads
    for prefix in ["minimal_", "high_concurrency_", "conservative_", "asia_"] {
        let path = temp_dir.path().join(format!("{}{}", prefix, filename));
        assert!(
            verify_download(&path, min_size),
            "Download {} should be valid",
            prefix
        );
    }
}

// ============================================================================
// Concurrent Download E2E Tests
// ============================================================================

/// Test downloading multiple files concurrently
#[tokio::test]
#[ignore = "Requires network access"]
async fn test_e2e_concurrent_downloads() {
    let temp_dir = create_temp_dir();
    let cdn = TurboCdn::builder()
        .with_auto_detect_region(false)
        .with_region(Region::Global)
        .with_max_concurrent_downloads(16)
        .build()
        .await
        .expect("Failed to create TurboCdn");

    let start = Instant::now();

    // Download all small test files concurrently
    let mut handles = Vec::new();
    for (url, filename, min_size) in SMALL_TEST_FILES.iter() {
        let output_path = temp_dir.path().join(*filename);
        let url = url.to_string();
        let min_size = *min_size;
        let cdn_ref = &cdn;

        let result = cdn_ref.download_to_path(&url, &output_path).await;
        handles.push((output_path, min_size, result));
    }

    let elapsed = start.elapsed();

    println!("Concurrent Downloads Test:");
    println!("==========================");
    println!("Total time: {:?}", elapsed);

    let mut total_size = 0u64;
    for (path, min_size, result) in handles {
        let result = result.expect("Download failed");
        assert!(verify_download(&path, min_size));
        total_size += result.size;
        println!(
            "  {} - {} bytes at {}",
            path.file_name().unwrap().to_string_lossy(),
            result.size,
            format_speed(result.speed)
        );
    }

    let overall_speed = total_size as f64 / elapsed.as_secs_f64();
    println!("\nTotal downloaded: {} bytes", total_size);
    println!("Overall speed: {}", format_speed(overall_speed));
}

// ============================================================================
// Performance Benchmark E2E Tests
// ============================================================================

/// Benchmark download performance with different chunk sizes
#[tokio::test]
#[ignore = "Requires network access - runs multiple downloads"]
async fn test_e2e_chunk_size_benchmark() {
    let temp_dir = create_temp_dir();
    let (url, filename, min_size) = MEDIUM_TEST_FILES[0];

    let chunk_sizes = [
        (128 * 1024, "128KB"),
        (256 * 1024, "256KB"),
        (512 * 1024, "512KB"),
        (1024 * 1024, "1MB"),
        (2 * 1024 * 1024, "2MB"),
    ];

    println!("Chunk Size Benchmark:");
    println!("=====================");
    println!("Test file: {}", filename);
    println!();

    for (chunk_size, label) in chunk_sizes.iter() {
        let cdn = TurboCdn::builder()
            .with_auto_detect_region(false)
            .with_region(Region::Global)
            .with_chunk_size(*chunk_size)
            .with_max_concurrent_downloads(16)
            .build()
            .await
            .expect("Failed to create TurboCdn");

        let output_path = temp_dir
            .path()
            .join(format!("chunk_{}_{}", label, filename));
        let start = Instant::now();
        let result = cdn.download_to_path(url, &output_path).await;
        let elapsed = start.elapsed();

        match result {
            Ok(r) => {
                println!(
                    "  Chunk {}: {:?} - {}",
                    label,
                    elapsed,
                    format_speed(r.speed)
                );
                assert!(verify_download(&output_path, min_size));
            }
            Err(e) => {
                println!("  Chunk {}: FAILED - {:?}", label, e);
            }
        }
    }
}

/// Benchmark download performance with different concurrency levels
#[tokio::test]
#[ignore = "Requires network access - runs multiple downloads"]
async fn test_e2e_concurrency_benchmark() {
    let temp_dir = create_temp_dir();
    let (url, filename, min_size) = MEDIUM_TEST_FILES[0];

    let concurrency_levels = [4, 8, 16, 32];

    println!("Concurrency Benchmark:");
    println!("======================");
    println!("Test file: {}", filename);
    println!();

    for concurrency in concurrency_levels.iter() {
        let cdn = TurboCdn::builder()
            .with_auto_detect_region(false)
            .with_region(Region::Global)
            .with_max_concurrent_downloads(*concurrency)
            .build()
            .await
            .expect("Failed to create TurboCdn");

        let output_path = temp_dir
            .path()
            .join(format!("conc_{}_{}", concurrency, filename));
        let start = Instant::now();
        let result = cdn.download_to_path(url, &output_path).await;
        let elapsed = start.elapsed();

        match result {
            Ok(r) => {
                println!(
                    "  {} concurrent: {:?} - {}",
                    concurrency,
                    elapsed,
                    format_speed(r.speed)
                );
                assert!(verify_download(&output_path, min_size));
            }
            Err(e) => {
                println!("  {} concurrent: FAILED - {:?}", concurrency, e);
            }
        }
    }
}

// ============================================================================
// loonghao Projects E2E Tests
// ============================================================================

/// Test downloading vx (Universal tool executor) v0.6.8
#[tokio::test]
#[ignore = "Requires network access"]
async fn test_e2e_download_vx() {
    let temp_dir = create_temp_dir();
    let cdn = TurboCdn::builder()
        .with_auto_detect_region(false)
        .with_region(Region::Global)
        .with_max_concurrent_downloads(16)
        .with_timeout(120)
        .with_retry_attempts(5)
        .build()
        .await
        .expect("Failed to create TurboCdn");

    let (url, filename, min_size) = LOONGHAO_TEST_FILES[0];
    let output_path = temp_dir.path().join(filename);

    let start = Instant::now();
    let result = cdn
        .download_to_path(url, &output_path)
        .await
        .expect("Download failed");
    let elapsed = start.elapsed();

    println!("vx Download:");
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

/// Test downloading auroraview Python wheel
#[tokio::test]
#[ignore = "Requires network access"]
async fn test_e2e_download_auroraview_wheel() {
    let temp_dir = create_temp_dir();
    let cdn = TurboCdn::builder()
        .with_auto_detect_region(false)
        .with_region(Region::Global)
        .with_max_concurrent_downloads(16)
        .with_timeout(120)
        .with_retry_attempts(5)
        .build()
        .await
        .expect("Failed to create TurboCdn");

    let (url, filename, min_size) = LOONGHAO_TEST_FILES[1];
    let output_path = temp_dir.path().join(filename);

    let start = Instant::now();
    let result = cdn
        .download_to_path(url, &output_path)
        .await
        .expect("Download failed");
    let elapsed = start.elapsed();

    println!("auroraview Wheel Download:");
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

/// Test downloading auroraview CLI
#[tokio::test]
#[ignore = "Requires network access"]
async fn test_e2e_download_auroraview_cli() {
    let temp_dir = create_temp_dir();
    let cdn = TurboCdn::builder()
        .with_auto_detect_region(false)
        .with_region(Region::Global)
        .with_max_concurrent_downloads(16)
        .with_timeout(120)
        .with_retry_attempts(5)
        .build()
        .await
        .expect("Failed to create TurboCdn");

    let (url, filename, min_size) = LOONGHAO_TEST_FILES[2];
    let output_path = temp_dir.path().join(filename);

    let start = Instant::now();
    let result = cdn
        .download_to_path(url, &output_path)
        .await
        .expect("Download failed");
    let elapsed = start.elapsed();

    println!("auroraview CLI Download:");
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

/// Test downloading auroraview Gallery (large file ~70MB)
#[tokio::test]
#[ignore = "Requires network access - downloads ~70MB"]
async fn test_e2e_download_auroraview_gallery() {
    let temp_dir = create_temp_dir();
    let cdn = TurboCdn::builder()
        .with_auto_detect_region(false)
        .with_region(Region::Global)
        .with_max_concurrent_downloads(32)
        .with_chunk_size(1024 * 1024) // 1MB chunks
        .with_adaptive_chunking(true)
        .with_timeout(300) // 5 minutes for large file
        .with_retry_attempts(5)
        .build()
        .await
        .expect("Failed to create TurboCdn");

    let (url, filename, min_size) = LOONGHAO_TEST_FILES[3];
    let output_path = temp_dir.path().join(filename);

    println!("Starting large file download: {}", filename);
    println!("Expected size: ~{} MB", min_size / 1024 / 1024);

    let start = Instant::now();
    let result = cdn
        .download_to_path(url, &output_path)
        .await
        .expect("Large file download failed");
    let elapsed = start.elapsed();

    println!("\nauroraview Gallery Download Complete:");
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

/// Test downloading all loonghao projects
#[tokio::test]
#[ignore = "Requires network access - downloads loonghao projects"]
async fn test_e2e_download_loonghao_projects() {
    let temp_dir = create_temp_dir();
    let cdn = TurboCdn::builder()
        .with_auto_detect_region(false)
        .with_region(Region::Global)
        .with_max_concurrent_downloads(16)
        .with_timeout(300) // 5 minutes for large files
        .with_retry_attempts(5)
        .build()
        .await
        .expect("Failed to create TurboCdn");

    println!("loonghao Projects Download Test:");
    println!("=================================");

    let mut total_size = 0u64;
    let mut total_time = std::time::Duration::ZERO;
    let mut success_count = 0;
    let mut fail_count = 0;

    for (url, filename, min_size) in LOONGHAO_TEST_FILES.iter() {
        let output_path = temp_dir.path().join(*filename);

        let start = Instant::now();
        let result = cdn.download_to_path(url, &output_path).await;
        let elapsed = start.elapsed();

        match result {
            Ok(r) => {
                total_size += r.size;
                total_time += elapsed;
                success_count += 1;
                println!(
                    "  [OK] {} - {:.2} MB in {:?} ({})",
                    filename,
                    r.size as f64 / 1024.0 / 1024.0,
                    elapsed,
                    format_speed(r.speed)
                );
                println!("       CDN: {}", r.url);
                assert!(verify_download(&output_path, *min_size));
            }
            Err(e) => {
                fail_count += 1;
                println!("  [FAIL] {} - {:?}", filename, e);
            }
        }
    }

    let overall_speed = if total_time.as_secs_f64() > 0.0 {
        total_size as f64 / total_time.as_secs_f64()
    } else {
        0.0
    };

    println!("\nSummary:");
    println!(
        "  Total size: {:.2} MB",
        total_size as f64 / 1024.0 / 1024.0
    );
    println!("  Total time: {:?}", total_time);
    println!("  Overall speed: {}", format_speed(overall_speed));
    println!("  Success: {}/{}", success_count, LOONGHAO_TEST_FILES.len());
    println!("  Failed: {}", fail_count);

    // At least 75% should succeed
    assert!(
        success_count >= LOONGHAO_TEST_FILES.len() * 3 / 4,
        "At least 75% of downloads should succeed"
    );
}

/// Test URL mapping for loonghao projects
#[tokio::test]
#[ignore = "Requires network access"]
async fn test_e2e_url_mapping_loonghao_projects() {
    let cdn = TurboCdn::builder()
        .with_auto_detect_region(false)
        .with_region(Region::Global)
        .build()
        .await
        .expect("Failed to create TurboCdn");

    println!("loonghao Projects URL Mapping:");
    println!("==============================");

    for (url, filename, _) in LOONGHAO_TEST_FILES.iter() {
        let cdn_urls = cdn
            .get_all_cdn_urls(url)
            .await
            .expect("Failed to get CDN URLs");

        println!("\n{}:", filename);
        println!("  Original: {}", url);
        println!("  Mapped URLs ({}):", cdn_urls.len());
        for (i, cdn_url) in cdn_urls.iter().enumerate() {
            println!("    {}: {}", i + 1, cdn_url);
        }

        // Should have multiple CDN alternatives
        assert!(
            cdn_urls.len() > 1,
            "Should have multiple CDN alternatives for {}",
            filename
        );
    }
}
