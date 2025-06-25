//! # Advanced Configuration Example
//!
//! This example demonstrates advanced configuration options for Turbo CDN,
//! including custom settings, performance tuning, and specialized use cases.

use std::collections::HashMap;

use std::time::Duration;
use turbo_cdn::{DownloadOptions, Result, TurboCdn};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing with custom level
    std::env::set_var("RUST_LOG", "turbo_cdn=debug");
    turbo_cdn::init_tracing();

    println!("🔧 Turbo CDN - Advanced Configuration Example");
    println!("=============================================");

    // Example 1: Custom download options
    println!("\n⚙️ Example 1: Custom Download Options");
    println!("------------------------------------");

    let mut custom_headers = HashMap::new();
    custom_headers.insert("User-Agent".to_string(), "MyApp/1.0 (Custom)".to_string());
    custom_headers.insert("Accept".to_string(), "application/octet-stream".to_string());

    let download_options = DownloadOptions {
        max_concurrent_chunks: Some(8),
        chunk_size: Some(2 * 1024 * 1024), // 2MB chunks
        enable_resume: true,
        custom_headers: Some(custom_headers),
        timeout_override: Some(Duration::from_secs(120)),
        verify_integrity: true,
        expected_size: None,
        progress_callback: Some(Box::new(|progress| {
            println!(
                "📊 Progress: {:.1}% ({} bytes)",
                progress.percentage, progress.downloaded_size
            );
        })),
    };

    println!("Custom options configured:");
    println!(
        "  🧩 Max concurrent chunks: {}",
        download_options.max_concurrent_chunks.unwrap_or(4)
    );
    println!(
        "  📦 Chunk size: {} MB",
        download_options.chunk_size.unwrap_or(1024 * 1024) / (1024 * 1024)
    );
    println!("  🔄 Resume enabled: {}", download_options.enable_resume);
    println!(
        "  ⏱️  Timeout: {}s",
        download_options.timeout_override.unwrap().as_secs()
    );
    println!(
        "  🛡️ Integrity verification: {}",
        download_options.verify_integrity
    );

    // Create client and download with custom options
    let turbo_cdn = TurboCdn::new().await?;
    let url = "https://github.com/sharkdp/hyperfine/releases/download/v1.18.0/hyperfine-v1.18.0-x86_64-pc-windows-msvc.zip";

    println!("\n📥 Downloading with custom options: {}", url);
    match turbo_cdn
        .download_with_options(url, std::env::temp_dir().join("download"), download_options)
        .await
    {
        Ok(result) => {
            println!("✅ Download completed with custom options!");
            println!("   📁 Path: {}", result.path.display());
            println!("   📊 Size: {} bytes", result.size);
            println!("   ⚡ Speed: {:.2} MB/s", result.speed / 1_000_000.0);
        }
        Err(e) => {
            println!("❌ Download failed: {}", e);
        }
    }

    // Example 2: Performance-optimized configuration
    println!("\n🚀 Example 2: Performance-Optimized Configuration");
    println!("------------------------------------------------");

    let performance_options = DownloadOptions {
        max_concurrent_chunks: Some(16),   // More aggressive chunking
        chunk_size: Some(4 * 1024 * 1024), // 4MB chunks for better throughput
        enable_resume: true,
        custom_headers: None,
        timeout_override: Some(Duration::from_secs(300)), // Longer timeout for large files
        verify_integrity: false,                          // Skip verification for speed
        expected_size: None,
        progress_callback: Some(Box::new(|progress| {
            if progress.percentage as u32 % 10 == 0 {
                println!(
                    "🚀 High-speed download: {:.0}% complete",
                    progress.percentage
                );
            }
        })),
    };

    println!("Performance-optimized settings:");
    println!(
        "  ⚡ Max concurrent chunks: {}",
        performance_options.max_concurrent_chunks.unwrap_or(16)
    );
    println!(
        "  📦 Chunk size: {} MB",
        performance_options.chunk_size.unwrap_or(4 * 1024 * 1024) / (1024 * 1024)
    );
    println!(
        "  🛡️ Integrity verification: {}",
        performance_options.verify_integrity
    );

    // Example 3: Conservative/reliable configuration
    println!("\n🛡️ Example 3: Conservative/Reliable Configuration");
    println!("------------------------------------------------");

    let conservative_options = DownloadOptions {
        max_concurrent_chunks: Some(2), // Fewer connections to be gentle on servers
        chunk_size: Some(512 * 1024),   // 512KB chunks
        enable_resume: true,
        custom_headers: Some({
            let mut headers = HashMap::new();
            headers.insert(
                "User-Agent".to_string(),
                "TurboCDN/1.0 (Conservative Mode)".to_string(),
            );
            headers
        }),
        timeout_override: Some(Duration::from_secs(60)),
        verify_integrity: true, // Always verify
        expected_size: None,
        progress_callback: Some(Box::new(|progress| {
            println!(
                "🐌 Conservative download: {:.1}% - {:.2} MB/s",
                progress.percentage,
                progress.speed / 1_000_000.0
            );
        })),
    };

    println!("Conservative settings:");
    println!(
        "  🐌 Max concurrent chunks: {}",
        conservative_options.max_concurrent_chunks.unwrap_or(2)
    );
    println!(
        "  📦 Chunk size: {} KB",
        conservative_options.chunk_size.unwrap_or(512 * 1024) / 1024
    );
    println!(
        "  🛡️ Integrity verification: {}",
        conservative_options.verify_integrity
    );

    // Example 4: Environment-based configuration
    println!("\n🌍 Example 4: Environment-Based Configuration");
    println!("--------------------------------------------");

    let env_options = create_environment_based_options();
    println!("Environment-based configuration loaded:");
    println!("  📊 Chunks: {}", env_options.max_concurrent_chunks.unwrap_or(4));
    println!("  📦 Chunk size: {} bytes", env_options.chunk_size.unwrap_or(1024 * 1024));
    println!(
        "  ⏱️  Timeout: {}s",
        env_options
            .timeout_override
            .unwrap_or(Duration::from_secs(30))
            .as_secs()
    );

    // Example 5: Specialized configurations for different file types
    println!("\n📋 Example 5: File Type-Specific Configurations");
    println!("----------------------------------------------");

    let file_configs = get_file_type_configurations();

    for (file_type, config) in file_configs {
        println!("\n📄 {} files configuration:", file_type);
        println!("   🧩 Chunks: {}", config.max_concurrent_chunks.unwrap_or(4));
        println!("   📦 Chunk size: {} KB", config.chunk_size.unwrap_or(1024 * 1024) / 1024);
        println!("   🛡️ Verify integrity: {}", config.verify_integrity);
    }

    // Example 6: Dynamic configuration based on network conditions
    println!("\n📡 Example 6: Network-Adaptive Configuration");
    println!("-------------------------------------------");

    let network_config = create_network_adaptive_config().await;
    println!("Network-adaptive configuration:");
    println!(
        "  📊 Detected optimal chunks: {}",
        network_config.max_concurrent_chunks.unwrap_or(8)
    );
    println!(
        "  📦 Optimal chunk size: {} KB",
        network_config.chunk_size.unwrap_or(2 * 1024 * 1024) / 1024
    );

    // Example 7: Configuration validation
    println!("\n✅ Example 7: Configuration Validation");
    println!("-------------------------------------");

    let test_configs = vec![
        ("Valid config", create_valid_config()),
        ("High-performance config", create_high_performance_config()),
        ("Low-bandwidth config", create_low_bandwidth_config()),
    ];

    for (name, config) in test_configs {
        println!("\n🔍 Validating {}: ", name);
        match validate_download_options(&config) {
            Ok(_) => println!("   ✅ Configuration is valid"),
            Err(e) => println!("   ❌ Configuration error: {}", e),
        }
    }

    println!("\n🎉 Advanced configuration examples completed!");

    Ok(())
}

/// Create configuration based on environment variables
fn create_environment_based_options() -> DownloadOptions {
    let max_chunks = std::env::var("TURBO_CDN_MAX_CHUNKS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(4);

    let chunk_size = std::env::var("TURBO_CDN_CHUNK_SIZE")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(1024 * 1024); // 1MB default

    let timeout = std::env::var("TURBO_CDN_TIMEOUT")
        .ok()
        .and_then(|s| s.parse().ok())
        .map(Duration::from_secs)
        .unwrap_or(Duration::from_secs(60));

    DownloadOptions {
        max_concurrent_chunks: Some(max_chunks),
        chunk_size: Some(chunk_size),
        enable_resume: true,
        custom_headers: None,
        timeout_override: Some(timeout),
        verify_integrity: true,
        expected_size: None,
        progress_callback: None,
    }
}

/// Get configurations optimized for different file types
fn get_file_type_configurations() -> HashMap<String, DownloadOptions> {
    let mut configs = HashMap::new();

    // Large binary files (videos, ISOs, etc.)
    configs.insert(
        "Large Binary".to_string(),
        DownloadOptions {
            max_concurrent_chunks: Some(16),
            chunk_size: Some(8 * 1024 * 1024), // 8MB
            enable_resume: true,
            custom_headers: None,
            timeout_override: Some(Duration::from_secs(600)), // 10 minutes
            verify_integrity: false,                          // Skip for speed
            expected_size: None,
            progress_callback: None,
        },
    );

    // Source code archives
    configs.insert(
        "Source Code".to_string(),
        DownloadOptions {
            max_concurrent_chunks: Some(4),
            chunk_size: Some(1024 * 1024), // 1MB
            enable_resume: true,
            custom_headers: None,
            timeout_override: Some(Duration::from_secs(120)),
            verify_integrity: true, // Always verify source code
            expected_size: None,
            progress_callback: None,
        },
    );

    // Small utilities and tools
    configs.insert(
        "Small Tools".to_string(),
        DownloadOptions {
            max_concurrent_chunks: Some(2),
            chunk_size: Some(256 * 1024), // 256KB
            enable_resume: false,   // Not needed for small files
            custom_headers: None,
            timeout_override: Some(Duration::from_secs(30)),
            verify_integrity: true,
            expected_size: None,
            progress_callback: None,
        },
    );

    configs
}

/// Create network-adaptive configuration (simulated)
async fn create_network_adaptive_config() -> DownloadOptions {
    // Simulate network speed detection
    let simulated_bandwidth_mbps = 50.0; // Assume 50 Mbps connection

    let optimal_chunks = if simulated_bandwidth_mbps > 100.0 {
        16 // High-speed connection
    } else if simulated_bandwidth_mbps > 25.0 {
        8 // Medium-speed connection
    } else {
        4 // Lower-speed connection
    };

    let optimal_chunk_size = if simulated_bandwidth_mbps > 100.0 {
        4 * 1024 * 1024 // 4MB for high-speed
    } else if simulated_bandwidth_mbps > 25.0 {
        2 * 1024 * 1024 // 2MB for medium-speed
    } else {
        1024 * 1024 // 1MB for lower-speed
    };

    DownloadOptions {
        max_concurrent_chunks: Some(optimal_chunks),
        chunk_size: Some(optimal_chunk_size),
        enable_resume: true,
        custom_headers: None,
        timeout_override: Some(Duration::from_secs(120)),
        verify_integrity: true,
        expected_size: None,
        progress_callback: None,
    }
}

/// Create a valid configuration for testing
fn create_valid_config() -> DownloadOptions {
    DownloadOptions {
        max_concurrent_chunks: Some(4),
        chunk_size: Some(1024 * 1024),
        enable_resume: true,
        custom_headers: None,
        timeout_override: Some(Duration::from_secs(60)),
        verify_integrity: true,
        expected_size: None,
        progress_callback: None,
    }
}

/// Create a high-performance configuration
fn create_high_performance_config() -> DownloadOptions {
    DownloadOptions {
        max_concurrent_chunks: Some(32),
        chunk_size: Some(16 * 1024 * 1024),
        enable_resume: true,
        custom_headers: None,
        timeout_override: Some(Duration::from_secs(300)),
        verify_integrity: false,
        expected_size: None,
        progress_callback: None,
    }
}

/// Create a low-bandwidth configuration
fn create_low_bandwidth_config() -> DownloadOptions {
    DownloadOptions {
        max_concurrent_chunks: Some(1),
        chunk_size: Some(128 * 1024),
        enable_resume: true,
        custom_headers: None,
        timeout_override: Some(Duration::from_secs(180)),
        verify_integrity: true,
        expected_size: None,
        progress_callback: None,
    }
}

/// Validate download options
fn validate_download_options(options: &DownloadOptions) -> std::result::Result<(), String> {
    if let Some(chunks) = options.max_concurrent_chunks {
        if chunks == 0 {
            return Err("max_concurrent_chunks must be greater than 0".to_string());
        }

        if chunks > 64 {
            return Err("max_concurrent_chunks should not exceed 64 for most use cases".to_string());
        }
    }

    if let Some(chunk_size) = options.chunk_size {
        if chunk_size < 1024 {
            return Err("chunk_size should be at least 1KB".to_string());
        }

        if chunk_size > 32 * 1024 * 1024 {
            return Err("chunk_size should not exceed 32MB".to_string());
        }
    }

    if let Some(timeout) = options.timeout_override {
        if timeout.as_secs() < 5 {
            return Err("timeout should be at least 5 seconds".to_string());
        }
        if timeout.as_secs() > 3600 {
            return Err("timeout should not exceed 1 hour".to_string());
        }
    }

    Ok(())
}
