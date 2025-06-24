//! # Async API Example
//!
//! This example demonstrates the async API features of Turbo CDN,
//! including quick functions and concurrent operations.

use turbo_cdn::{async_api, Result};
use std::time::Instant;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    turbo_cdn::init_tracing();

    println!("🚀 Turbo CDN - Async API Example");
    println!("================================");

    // Example 1: Quick URL optimization
    println!("\n🔍 Example 1: Quick URL Optimization");
    println!("------------------------------------");
    
    let url = "https://github.com/rust-lang/rust-analyzer/releases/download/2023-12-04/rust-analyzer-x86_64-pc-windows-msvc.gz";
    println!("Original URL: {}", url);
    
    let start = Instant::now();
    match async_api::quick::optimize_url(url).await {
        Ok(optimized_url) => {
            let duration = start.elapsed();
            println!("✅ Optimized URL: {}", optimized_url);
            println!("⏱️  Optimization took: {:.2}ms", duration.as_millis());
        }
        Err(e) => {
            println!("❌ Error: {}", e);
        }
    }

    // Example 2: Quick download
    println!("\n📥 Example 2: Quick Download");
    println!("---------------------------");
    
    let download_url = "https://github.com/sharkdp/hyperfine/releases/download/v1.18.0/hyperfine-v1.18.0-x86_64-pc-windows-msvc.zip";
    println!("Downloading: {}", download_url);
    
    let start = Instant::now();
    match async_api::quick::download_url(download_url).await {
        Ok(result) => {
            let duration = start.elapsed();
            println!("✅ Download completed!");
            println!("   📁 Path: {}", result.path.display());
            println!("   📊 Size: {} bytes", result.size);
            println!("   ⚡ Speed: {:.2} MB/s", result.speed / 1_000_000.0);
            println!("   ⏱️  Total time: {:.2}s", duration.as_secs_f64());
        }
        Err(e) => {
            println!("❌ Download failed: {}", e);
        }
    }

    // Example 3: Concurrent URL optimizations
    println!("\n🌐 Example 3: Concurrent URL Optimizations");
    println!("------------------------------------------");
    
    let urls = vec![
        "https://github.com/microsoft/vscode/releases/download/1.85.0/VSCode-linux-x64.tar.gz",
        "https://github.com/golang/go/releases/download/go1.21.5/go1.21.5.linux-amd64.tar.gz",
        "https://github.com/nodejs/node/releases/download/v20.10.0/node-v20.10.0-linux-x64.tar.xz",
        "https://registry.npmjs.org/react/-/react-18.2.0.tgz",
        "https://files.pythonhosted.org/packages/source/d/django/Django-4.2.7.tar.gz",
    ];
    
    println!("Optimizing {} URLs concurrently...", urls.len());
    let start = Instant::now();
    
    // Create concurrent tasks
    let tasks: Vec<_> = urls.into_iter().enumerate().map(|(i, url)| {
        tokio::spawn(async move {
            let result = async_api::quick::optimize_url(url).await;
            (i, url, result)
        })
    }).collect();
    
    // Wait for all tasks to complete
    let mut results = Vec::new();
    for task in tasks {
        match task.await {
            Ok(result) => results.push(result),
            Err(e) => println!("❌ Task failed: {}", e),
        }
    }
    
    let total_duration = start.elapsed();
    
    // Display results
    results.sort_by_key(|(i, _, _)| *i);
    for (i, url, result) in results {
        println!("\n{}. URL: {}", i + 1, url);
        match result {
            Ok(optimized_url) => {
                if optimized_url != url {
                    println!("   ✅ Optimized: {}", optimized_url);
                } else {
                    println!("   ℹ️  No optimization available");
                }
            }
            Err(e) => {
                println!("   ❌ Error: {}", e);
            }
        }
    }
    
    println!("\n⏱️  Total concurrent optimization time: {:.2}s", total_duration.as_secs_f64());

    // Example 4: Async download with progress simulation
    println!("\n📊 Example 4: Async Download with Progress Tracking");
    println!("--------------------------------------------------");
    
    let progress_url = "https://github.com/BurntSushi/ripgrep/releases/download/14.1.1/ripgrep-14.1.1-x86_64-unknown-linux-musl.tar.gz";
    println!("Starting download: {}", progress_url);
    
    // Start download task
    let download_task = tokio::spawn(async move {
        async_api::quick::download_url(progress_url).await
    });
    
    // Simulate progress monitoring
    let progress_task = tokio::spawn(async {
        let mut dots = 0;
        loop {
            print!("📥 Downloading");
            for _ in 0..dots {
                print!(".");
            }
            println!();
            dots = (dots + 1) % 4;
            sleep(Duration::from_millis(500)).await;
        }
    });
    
    // Wait for download to complete
    match download_task.await {
        Ok(Ok(result)) => {
            progress_task.abort(); // Stop progress animation
            println!("✅ Download completed!");
            println!("   📁 Path: {}", result.path.display());
            println!("   📊 Size: {} bytes", result.size);
            println!("   ⚡ Speed: {:.2} MB/s", result.speed / 1_000_000.0);
        }
        Ok(Err(e)) => {
            progress_task.abort();
            println!("❌ Download failed: {}", e);
        }
        Err(e) => {
            progress_task.abort();
            println!("❌ Task failed: {}", e);
        }
    }

    // Example 5: Batch processing with rate limiting
    println!("\n⚡ Example 5: Rate-Limited Batch Processing");
    println!("------------------------------------------");
    
    let batch_urls = vec![
        "https://github.com/cli/cli/releases/download/v2.40.1/gh_2.40.1_linux_amd64.tar.gz",
        "https://github.com/neovim/neovim/releases/download/v0.9.4/nvim-linux64.tar.gz",
        "https://github.com/sharkdp/fd/releases/download/v8.7.0/fd-v8.7.0-x86_64-unknown-linux-gnu.tar.gz",
    ];
    
    println!("Processing {} URLs with rate limiting...", batch_urls.len());
    
    for (i, url) in batch_urls.iter().enumerate() {
        println!("\n📋 Processing {}/{}: {}", i + 1, batch_urls.len(), url);
        
        let start = Instant::now();
        match async_api::quick::optimize_url(url).await {
            Ok(optimized_url) => {
                let duration = start.elapsed();
                if optimized_url != *url {
                    println!("   ✅ Optimized in {:.2}ms", duration.as_millis());
                } else {
                    println!("   ℹ️  No optimization available ({:.2}ms)", duration.as_millis());
                }
            }
            Err(e) => {
                println!("   ❌ Error: {}", e);
            }
        }
        
        // Rate limiting: wait between requests
        if i < batch_urls.len() - 1 {
            sleep(Duration::from_millis(100)).await;
        }
    }

    // Example 6: Error handling with retries
    println!("\n🛡️ Example 6: Error Handling with Retries");
    println!("-----------------------------------------");
    
    let unreliable_url = "https://httpstat.us/500"; // This will return 500 error
    println!("Testing error handling with: {}", unreliable_url);
    
    let max_retries = 3;
    for attempt in 1..=max_retries {
        println!("   Attempt {}/{}", attempt, max_retries);
        
        match async_api::quick::optimize_url(unreliable_url).await {
            Ok(url) => {
                println!("   ✅ Success: {}", url);
                break;
            }
            Err(e) => {
                println!("   ❌ Error: {}", e);
                if attempt < max_retries {
                    println!("   ⏳ Retrying in 1 second...");
                    sleep(Duration::from_secs(1)).await;
                } else {
                    println!("   💥 Max retries reached");
                }
            }
        }
    }

    println!("\n🎉 Async API examples completed!");
    
    Ok(())
}

/// Helper function to demonstrate custom async operations
async fn custom_download_with_timeout(url: &str, timeout_secs: u64) -> Result<String> {
    println!("🕐 Starting download with {}s timeout: {}", timeout_secs, url);
    
    let download_future = async_api::quick::optimize_url(url);
    let timeout_future = sleep(Duration::from_secs(timeout_secs));
    
    tokio::select! {
        result = download_future => {
            match result {
                Ok(optimized_url) => {
                    println!("✅ Completed within timeout");
                    Ok(optimized_url)
                }
                Err(e) => {
                    println!("❌ Download failed: {}", e);
                    Err(e)
                }
            }
        }
        _ = timeout_future => {
            println!("⏰ Operation timed out after {}s", timeout_secs);
            Err(turbo_cdn::Error::Timeout)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_quick_optimize() {
        let url = "https://github.com/BurntSushi/ripgrep/releases/download/14.1.1/ripgrep-14.1.1-x86_64-pc-windows-msvc.zip";
        let result = async_api::quick::optimize_url(url).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_concurrent_optimizations() {
        let urls = vec![
            "https://github.com/cli/cli/releases/download/v2.40.1/gh_2.40.1_windows_amd64.zip",
            "https://registry.npmjs.org/typescript/-/typescript-5.3.3.tgz",
        ];
        
        let tasks: Vec<_> = urls.into_iter().map(|url| {
            tokio::spawn(async move {
                async_api::quick::optimize_url(url).await
            })
        }).collect();
        
        for task in tasks {
            let result = task.await.unwrap();
            assert!(result.is_ok());
        }
    }

    #[tokio::test]
    async fn test_timeout_handling() {
        let result = custom_download_with_timeout(
            "https://github.com/BurntSushi/ripgrep/releases/download/14.1.1/ripgrep-14.1.1-x86_64-pc-windows-msvc.zip",
            30
        ).await;
        
        // Should complete within 30 seconds
        assert!(result.is_ok());
    }
}
