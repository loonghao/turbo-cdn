//! # Basic Usage Example
//!
//! This example demonstrates the basic usage of the Turbo CDN library.
//! It shows how to create a client, download files, and optimize URLs.

use turbo_cdn::{Result, TurboCdn};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing for better debugging
    turbo_cdn::init_tracing();

    println!("🚀 Turbo CDN - Basic Usage Example");
    println!("==================================");

    // Create a new TurboCdn client
    println!("📡 Initializing Turbo CDN client...");
    let turbo_cdn = TurboCdn::new().await?;
    println!("✅ Client initialized successfully!");

    // Example 1: Get optimal URL
    println!("\n🔍 Example 1: URL Optimization");
    println!("------------------------------");

    let original_url = "https://github.com/BurntSushi/ripgrep/releases/download/14.1.1/ripgrep-14.1.1-x86_64-pc-windows-msvc.zip";
    println!("Original URL: {}", original_url);

    match turbo_cdn.get_optimal_url(original_url).await {
        Ok(optimal_url) => {
            if optimal_url != original_url {
                println!("✅ Optimized URL: {}", optimal_url);
                println!("🚀 CDN optimization available!");
            } else {
                println!("ℹ️  No CDN optimization available for this URL");
            }
        }
        Err(e) => {
            println!("❌ Error optimizing URL: {}", e);
        }
    }

    // Example 2: Simple download
    println!("\n📥 Example 2: Simple Download");
    println!("-----------------------------");

    let download_url = "https://github.com/sharkdp/fd/releases/download/v8.7.0/fd-v8.7.0-x86_64-pc-windows-msvc.zip";
    println!("Downloading: {}", download_url);

    match turbo_cdn.download_from_url(download_url).await {
        Ok(result) => {
            println!("✅ Download completed!");
            println!("   📁 Path: {}", result.path.display());
            println!("   📊 Size: {} bytes", result.size);
            println!("   ⚡ Speed: {:.2} MB/s", result.speed / 1_000_000.0);
            println!("   ⏱️  Duration: {:.2}s", result.duration.as_secs_f64());
        }
        Err(e) => {
            println!("❌ Download failed: {}", e);
        }
    }

    // Example 3: Download with custom output path
    println!("\n📁 Example 3: Download to Custom Path");
    println!("-------------------------------------");

    let custom_url = "https://github.com/sharkdp/bat/releases/download/v0.24.0/bat-v0.24.0-x86_64-pc-windows-msvc.zip";
    let custom_path = std::path::PathBuf::from("./downloads/bat.zip");

    // Create downloads directory if it doesn't exist
    if let Some(parent) = custom_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    println!("Downloading: {}", custom_url);
    println!("Output path: {}", custom_path.display());

    match turbo_cdn.download_to_path(custom_url, &custom_path).await {
        Ok(result) => {
            println!("✅ Download completed!");
            println!("   📁 Path: {}", result.path.display());
            println!("   📊 Size: {} bytes", result.size);
            println!("   ⚡ Speed: {:.2} MB/s", result.speed / 1_000_000.0);
        }
        Err(e) => {
            println!("❌ Download failed: {}", e);
        }
    }

    // Example 4: Multiple URL optimizations
    println!("\n🌐 Example 4: Multiple URL Optimizations");
    println!("----------------------------------------");

    let urls = vec![
        "https://github.com/cli/cli/releases/download/v2.40.1/gh_2.40.1_windows_amd64.zip",
        "https://registry.npmjs.org/typescript/-/typescript-5.3.3.tgz",
        "https://files.pythonhosted.org/packages/source/r/requests/requests-2.31.0.tar.gz",
    ];

    for (i, url) in urls.iter().enumerate() {
        println!("\n{}. Testing URL: {}", i + 1, url);
        match turbo_cdn.get_optimal_url(url).await {
            Ok(optimal_url) => {
                if optimal_url != *url {
                    println!("   ✅ Optimized: {}", optimal_url);
                } else {
                    println!("   ℹ️  No optimization available");
                }
            }
            Err(e) => {
                println!("   ❌ Error: {}", e);
            }
        }
    }

    // Example 5: Error handling
    println!("\n🛡️ Example 5: Error Handling");
    println!("----------------------------");

    let invalid_url = "https://invalid-domain-that-does-not-exist.com/file.zip";
    println!("Testing invalid URL: {}", invalid_url);

    match turbo_cdn.get_optimal_url(invalid_url).await {
        Ok(optimal_url) => {
            println!("   Optimal URL: {}", optimal_url);
        }
        Err(e) => {
            println!("   ❌ Expected error: {}", e);
            println!("   ℹ️  This demonstrates proper error handling");
        }
    }

    println!("\n🎉 Basic usage examples completed!");
    println!("   Check the ./downloads/ directory for downloaded files");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_client_creation() {
        let result = TurboCdn::new().await;
        assert!(result.is_ok(), "Failed to create TurboCdn client");
    }

    #[tokio::test]
    async fn test_url_optimization() {
        let turbo_cdn = TurboCdn::new().await.unwrap();
        let url = "https://github.com/BurntSushi/ripgrep/releases/download/14.1.1/ripgrep-14.1.1-x86_64-pc-windows-msvc.zip";

        let result = turbo_cdn.get_optimal_url(url).await;
        assert!(result.is_ok(), "URL optimization should not fail");

        let optimal_url = result.unwrap();
        assert!(!optimal_url.is_empty(), "Optimal URL should not be empty");
    }

    #[tokio::test]
    async fn test_invalid_url_handling() {
        let turbo_cdn = TurboCdn::new().await.unwrap();
        let invalid_url = "not-a-valid-url";

        let result = turbo_cdn.get_optimal_url(invalid_url).await;
        // Should either return an error or the original URL
        match result {
            Ok(url) => assert_eq!(url, invalid_url),
            Err(_) => {} // Error is also acceptable
        }
    }
}
