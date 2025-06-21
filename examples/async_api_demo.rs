// Licensed under the MIT License
// Copyright (c) 2025 Hal <hal.long@outlook.com>

//! # Async API Demo for vx Integration
//! 
//! This example demonstrates the async API interfaces designed specifically
//! for integration with external tools like vx.

use turbo_cdn::async_api::{AsyncTurboCdn, AsyncTurboCdnBuilder};
use turbo_cdn::{DownloadOptions, Region, Source};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    turbo_cdn::init_tracing();

    println!("🚀 Turbo CDN Async API Demo for vx Integration");
    println!("==============================================\n");

    // Example 1: Quick async operations
    println!("📦 Example 1: Quick Async Operations");
    println!("------------------------------------");

    // Quick URL optimization
    let url = "https://github.com/rust-lang/mdBook/releases/download/v0.4.21/mdbook-v0.4.21-x86_64-unknown-linux-gnu.tar.gz";
    println!("🔗 Original URL: {}", url);

    match turbo_cdn::async_api::quick::optimize_url(url).await {
        Ok(optimal_url) => {
            println!("⚡ Optimal URL: {}", optimal_url);
        }
        Err(e) => {
            println!("❌ Optimization failed: {}", e);
        }
    }

    // Quick URL parsing
    match turbo_cdn::async_api::quick::parse_url(url).await {
        Ok(parsed) => {
            println!("📋 Repository: {}", parsed.repository);
            println!("🏷️  Version: {}", parsed.version);
            println!("📄 Filename: {}", parsed.filename);
            println!("🔍 Source Type: {:?}", parsed.source_type);
        }
        Err(e) => {
            println!("❌ Parsing failed: {}", e);
        }
    }

    println!();

    // Example 2: Async client with custom configuration
    println!("📦 Example 2: Async Client with Custom Configuration");
    println!("---------------------------------------------------");

    let async_client = AsyncTurboCdnBuilder::new()
        .with_region(Region::Global)
        .with_cache(true)
        .with_max_concurrent_downloads(4)
        .with_sources(&[
            Source::github(),
            Source::jsdelivr(),
            Source::fastly(),
            Source::cloudflare(),
        ])
        .build()
        .await?;

    // Test multiple URLs with the same client
    let test_urls = vec![
        "https://cdn.jsdelivr.net/gh/jquery/jquery@3.6.0/dist/jquery.min.js",
        "https://registry.npmjs.org/express/-/express-4.18.2.tgz",
        "https://crates.io/api/v1/crates/tokio/1.28.0/download",
    ];

    for test_url in test_urls {
        println!("🔗 Testing URL: {}", test_url);
        
        match async_client.parse_url_async(test_url).await {
            Ok(parsed) => {
                println!("   ✅ Parsed: {} v{}", parsed.repository, parsed.version);
                
                // Try to get optimal URL
                match async_client.get_optimal_url_async(test_url).await {
                    Ok(optimal) => {
                        if optimal != test_url {
                            println!("   ⚡ Optimized to: {}", optimal);
                        } else {
                            println!("   ℹ️  Already optimal");
                        }
                    }
                    Err(e) => {
                        println!("   ⚠️  Optimization failed: {}", e);
                    }
                }
            }
            Err(e) => {
                println!("   ❌ Parse failed: {}", e);
            }
        }
        println!();
    }

    // Example 3: Concurrent operations
    println!("📦 Example 3: Concurrent Async Operations");
    println!("-----------------------------------------");

    let client = AsyncTurboCdn::new().await?;
    
    // Clone the client for concurrent use
    let client1 = client.clone();
    let client2 = client.clone();
    let client3 = client.clone();

    // Spawn concurrent tasks
    let task1 = tokio::spawn(async move {
        client1.parse_url_async("https://github.com/microsoft/vscode/releases/download/1.85.0/VSCode-linux-x64.tar.gz").await
    });

    let task2 = tokio::spawn(async move {
        client2.get_optimal_url_async("https://cdn.jsdelivr.net/gh/lodash/lodash@4.17.21/lodash.min.js").await
    });

    let task3 = tokio::spawn(async move {
        client3.extract_version_from_filename_async("myapp-v2.1.0-beta.zip").await
    });

    // Wait for all tasks to complete
    let (result1, result2, result3) = tokio::join!(task1, task2, task3);

    match result1 {
        Ok(Ok(parsed)) => {
            println!("✅ Task 1 - Parsed VSCode: {} v{}", parsed.repository, parsed.version);
        }
        Ok(Err(e)) => {
            println!("❌ Task 1 failed: {}", e);
        }
        Err(e) => {
            println!("❌ Task 1 panicked: {}", e);
        }
    }

    match result2 {
        Ok(Ok(optimal_url)) => {
            println!("✅ Task 2 - Optimal URL: {}", optimal_url);
        }
        Ok(Err(e)) => {
            println!("❌ Task 2 failed: {}", e);
        }
        Err(e) => {
            println!("❌ Task 2 panicked: {}", e);
        }
    }

    match result3 {
        Ok(version) => {
            if let Some(v) = version {
                println!("✅ Task 3 - Extracted version: {}", v);
            } else {
                println!("ℹ️  Task 3 - No version found");
            }
        }
        Err(e) => {
            println!("❌ Task 3 panicked: {}", e);
        }
    }

    println!();

    // Example 4: Download with progress (simulated for demo)
    println!("📦 Example 4: Async Download with Custom Options");
    println!("------------------------------------------------");

    let download_options = DownloadOptions {
        max_concurrent_chunks: 4,
        chunk_size: 512 * 1024, // 512KB chunks
        timeout: Duration::from_secs(30),
        use_cache: true,
        verify_checksum: false, // Skip for demo
        ..Default::default()
    };

    // For demo purposes, we'll just show the setup
    // In a real scenario, you would call:
    // let result = client.download_from_url_async(url, Some(download_options)).await?;
    
    println!("🔧 Download options configured:");
    println!("   📊 Max chunks: {}", download_options.max_concurrent_chunks);
    println!("   📦 Chunk size: {} KB", download_options.chunk_size / 1024);
    println!("   ⏱️  Timeout: {:?}", download_options.timeout);
    println!("   💾 Use cache: {}", download_options.use_cache);
    println!("   🔐 Verify checksum: {}", download_options.verify_checksum);

    println!("\n💡 Integration Tips for vx:");
    println!("==========================");
    println!("1. Use AsyncTurboCdn::new() for simple cases");
    println!("2. Use AsyncTurboCdnBuilder for custom configuration");
    println!("3. Clone the client for concurrent operations");
    println!("4. Use quick:: functions for one-off operations");
    println!("5. All methods are thread-safe and async-ready");
    println!("6. Error handling follows Rust Result patterns");

    println!("\n🔧 Example vx Integration:");
    println!("==========================");
    println!("```rust");
    println!("use turbo_cdn::async_api::AsyncTurboCdn;");
    println!("");
    println!("// In your vx application");
    println!("let cdn_client = AsyncTurboCdn::new().await?;");
    println!("");
    println!("// Optimize any URL");
    println!("let optimal_url = cdn_client.get_optimal_url_async(original_url).await?;");
    println!("");
    println!("// Download with optimization");
    println!("let result = cdn_client.download_from_url_async(url, None).await?;");
    println!("```");

    Ok(())
}

/// Example of how vx might integrate the async API
#[allow(dead_code)]
async fn vx_integration_example() -> Result<(), Box<dyn std::error::Error>> {
    // This is how vx might use the async API
    let cdn = AsyncTurboCdn::new().await?;
    
    // Example: Optimize a GitHub release URL
    let github_url = "https://github.com/oven-sh/bun/releases/download/bun-v1.2.9/bun-bun-v1.2.9.zip";
    
    // Parse the URL to get information
    let parsed = cdn.parse_url_async(github_url).await?;
    println!("Repository: {}", parsed.repository);
    println!("Version: {}", parsed.version);
    
    // Get the optimal URL for user's location
    let optimal_url = cdn.get_optimal_url_async(github_url).await?;
    println!("Optimal URL: {}", optimal_url);
    
    // Download using the optimal URL
    let result = cdn.download_from_url_async(&optimal_url, None).await?;
    println!("Downloaded to: {}", result.path.display());
    println!("Speed: {:.2} MB/s", result.speed / 1_000_000.0);
    
    Ok(())
}
