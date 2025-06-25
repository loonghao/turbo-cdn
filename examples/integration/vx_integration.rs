//! # vx Integration Example
//!
//! This example demonstrates how to integrate Turbo CDN with the vx tool
//! for optimal download performance and URL optimization.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Instant;
use turbo_cdn::{async_api, Result};

/// Configuration for vx integration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VxConfig {
    pub enable_cdn_optimization: bool,
    pub max_concurrent_downloads: usize,
    pub cache_directory: PathBuf,
    pub timeout_seconds: u64,
    pub verify_checksums: bool,
}

impl Default for VxConfig {
    fn default() -> Self {
        Self {
            enable_cdn_optimization: true,
            max_concurrent_downloads: 4,
            cache_directory: PathBuf::from("./cache"),
            timeout_seconds: 300,
            verify_checksums: true,
        }
    }
}

/// vx CDN Manager - integrates Turbo CDN with vx
pub struct VxCdnManager {
    config: VxConfig,
}

impl Default for VxCdnManager {
    fn default() -> Self {
        Self::new(VxConfig::default())
    }
}

impl VxCdnManager {
    /// Create a new vx CDN manager
    pub fn new(config: VxConfig) -> Self {
        Self { config }
    }



    /// Optimize a URL for vx downloads
    pub async fn optimize_url_for_vx(&self, url: &str) -> Result<String> {
        if !self.config.enable_cdn_optimization {
            return Ok(url.to_string());
        }

        println!("🔍 vx: Optimizing URL for faster download...");
        let start = Instant::now();

        match async_api::quick::optimize_url(url).await {
            Ok(optimized_url) => {
                let duration = start.elapsed();
                if optimized_url != url {
                    println!(
                        "✅ vx: CDN optimization found ({:.2}ms)",
                        duration.as_millis()
                    );
                    println!("   Original: {}", url);
                    println!("   Optimized: {}", optimized_url);
                } else {
                    println!(
                        "ℹ️  vx: No CDN optimization available ({:.2}ms)",
                        duration.as_millis()
                    );
                }
                Ok(optimized_url)
            }
            Err(e) => {
                println!("⚠️  vx: CDN optimization failed, using original URL: {}", e);
                Ok(url.to_string())
            }
        }
    }

    /// Download a file for vx with optimization
    pub async fn download_for_vx(
        &self,
        url: &str,
        output_path: Option<PathBuf>,
    ) -> Result<VxDownloadResult> {
        println!("📥 vx: Starting optimized download...");

        // First, optimize the URL
        let optimized_url = self.optimize_url_for_vx(url).await?;

        // Download using the optimized URL
        let start = Instant::now();
        let result = if let Some(path) = output_path {
            // Ensure parent directory exists
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            async_api::quick::download_url_to_path(&optimized_url, &path).await?
        } else {
            async_api::quick::download_url(&optimized_url).await?
        };

        let total_duration = start.elapsed();

        let was_optimized = optimized_url != url;
        let vx_result = VxDownloadResult {
            original_url: url.to_string(),
            optimized_url,
            path: result.path,
            size: result.size,
            speed: result.speed,
            duration: total_duration,
            was_optimized,
        };

        println!("✅ vx: Download completed!");
        println!("   📁 Path: {}", vx_result.path.display());
        println!("   📊 Size: {} bytes", vx_result.size);
        println!("   ⚡ Speed: {:.2} MB/s", vx_result.speed / 1_000_000.0);
        println!("   ⏱️  Duration: {:.2}s", vx_result.duration.as_secs_f64());
        println!("   🚀 CDN optimized: {}", vx_result.was_optimized);

        Ok(vx_result)
    }

    /// Batch download multiple URLs for vx
    pub async fn batch_download_for_vx(&self, urls: Vec<String>) -> Result<Vec<VxDownloadResult>> {
        println!("📦 vx: Starting batch download of {} files...", urls.len());

        let mut results = Vec::new();

        for (i, url) in urls.iter().enumerate() {
            println!("\n📋 vx: Processing {}/{}: {}", i + 1, urls.len(), url);

            match self.download_for_vx(url, None).await {
                Ok(result) => {
                    results.push(result);
                    println!("✅ vx: File {}/{} completed", i + 1, urls.len());
                }
                Err(e) => {
                    println!("❌ vx: File {}/{} failed: {}", i + 1, urls.len(), e);
                    // Continue with other downloads
                }
            }
        }

        println!("\n📊 vx: Batch download summary:");
        println!("   ✅ Successful: {}/{}", results.len(), urls.len());
        println!(
            "   ❌ Failed: {}/{}",
            urls.len() - results.len(),
            urls.len()
        );

        Ok(results)
    }

    /// Check if a URL can be optimized
    pub async fn can_optimize_url(&self, url: &str) -> bool {
        if !self.config.enable_cdn_optimization {
            return false;
        }

        match self.optimize_url_for_vx(url).await {
            Ok(optimized_url) => optimized_url != url,
            Err(_) => false,
        }
    }

    /// Get optimization statistics for multiple URLs
    pub async fn get_optimization_stats(&self, urls: Vec<String>) -> VxOptimizationStats {
        let mut stats = VxOptimizationStats {
            total_urls: urls.len(),
            ..Default::default()
        };

        for url in urls {
            if self.can_optimize_url(&url).await {
                stats.optimizable_urls += 1;
            }
        }

        stats.optimization_rate = if stats.total_urls > 0 {
            (stats.optimizable_urls as f64 / stats.total_urls as f64) * 100.0
        } else {
            0.0
        };

        stats
    }
}

/// Result of a vx download operation
#[derive(Debug, Clone)]
pub struct VxDownloadResult {
    pub original_url: String,
    pub optimized_url: String,
    pub path: PathBuf,
    pub size: u64,
    pub speed: f64,
    pub duration: std::time::Duration,
    pub was_optimized: bool,
}

/// Optimization statistics for vx
#[derive(Debug, Default)]
pub struct VxOptimizationStats {
    pub total_urls: usize,
    pub optimizable_urls: usize,
    pub optimization_rate: f64,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    let _ = turbo_cdn::logging::init_api_logging();

    println!("🔗 Turbo CDN - vx Integration Example");
    println!("====================================");

    // Example 1: Basic vx integration
    println!("\n🚀 Example 1: Basic vx Integration");
    println!("---------------------------------");

    let vx_manager = VxCdnManager::default();

    let test_url = "https://github.com/BurntSushi/ripgrep/releases/download/14.1.1/ripgrep-14.1.1-x86_64-pc-windows-msvc.zip";

    match vx_manager.download_for_vx(test_url, None).await {
        Ok(result) => {
            println!("🎉 vx integration successful!");
            if result.was_optimized {
                println!("   🚀 CDN optimization provided faster download");
            }
        }
        Err(e) => {
            println!("❌ vx integration failed: {}", e);
        }
    }

    // Example 2: Custom vx configuration
    println!("\n⚙️ Example 2: Custom vx Configuration");
    println!("------------------------------------");

    let custom_config = VxConfig {
        enable_cdn_optimization: true,
        max_concurrent_downloads: 8,
        cache_directory: PathBuf::from("./vx-cache"),
        timeout_seconds: 180,
        verify_checksums: true,
    };

    let custom_vx_manager = VxCdnManager::new(custom_config);

    println!("Custom vx configuration:");
    println!(
        "  🔧 CDN optimization: {}",
        custom_vx_manager.config.enable_cdn_optimization
    );
    println!(
        "  📊 Max concurrent: {}",
        custom_vx_manager.config.max_concurrent_downloads
    );
    println!(
        "  📁 Cache directory: {}",
        custom_vx_manager.config.cache_directory.display()
    );
    println!(
        "  ⏱️  Timeout: {}s",
        custom_vx_manager.config.timeout_seconds
    );

    // Example 3: Batch download for vx
    println!("\n📦 Example 3: Batch Download for vx");
    println!("----------------------------------");

    let batch_urls = vec![
        "https://github.com/cli/cli/releases/download/v2.40.1/gh_2.40.1_windows_amd64.zip".to_string(),
        "https://github.com/sharkdp/fd/releases/download/v8.7.0/fd-v8.7.0-x86_64-pc-windows-msvc.zip".to_string(),
        "https://github.com/sharkdp/bat/releases/download/v0.24.0/bat-v0.24.0-x86_64-pc-windows-msvc.zip".to_string(),
    ];

    match vx_manager.batch_download_for_vx(batch_urls).await {
        Ok(results) => {
            println!("📊 vx batch download results:");
            for (i, result) in results.iter().enumerate() {
                println!(
                    "  {}. {} - {:.2} MB/s (CDN: {})",
                    i + 1,
                    result.path.file_name().unwrap().to_string_lossy(),
                    result.speed / 1_000_000.0,
                    result.was_optimized
                );
            }
        }
        Err(e) => {
            println!("❌ vx batch download failed: {}", e);
        }
    }

    // Example 4: URL optimization checking
    println!("\n🔍 Example 4: URL Optimization Analysis");
    println!("--------------------------------------");

    let analysis_urls = vec![
        "https://github.com/microsoft/vscode/releases/download/1.85.0/VSCode-linux-x64.tar.gz"
            .to_string(),
        "https://registry.npmjs.org/typescript/-/typescript-5.3.3.tgz".to_string(),
        "https://files.pythonhosted.org/packages/source/r/requests/requests-2.31.0.tar.gz"
            .to_string(),
        "https://golang.org/dl/go1.21.5.linux-amd64.tar.gz".to_string(),
    ];

    let stats = vx_manager
        .get_optimization_stats(analysis_urls.clone())
        .await;

    println!("📊 vx optimization analysis:");
    println!("  📋 Total URLs analyzed: {}", stats.total_urls);
    println!("  🚀 Optimizable URLs: {}", stats.optimizable_urls);
    println!("  📈 Optimization rate: {:.1}%", stats.optimization_rate);

    // Show individual results
    for url in analysis_urls {
        let can_optimize = vx_manager.can_optimize_url(&url).await;
        println!("  {} {}", if can_optimize { "✅" } else { "❌" }, url);
    }

    // Example 5: vx-style command simulation
    println!("\n💻 Example 5: vx Command Simulation");
    println!("-----------------------------------");

    // Simulate: vx install node@20.10.0
    let node_url = "https://nodejs.org/dist/v20.10.0/node-v20.10.0-win-x64.zip";
    println!("🔧 Simulating: vx install node@20.10.0");
    println!("   Resolving to: {}", node_url);

    match vx_manager.optimize_url_for_vx(node_url).await {
        Ok(optimized_url) => {
            println!("   🚀 vx would use optimized URL for faster installation");
            if optimized_url != node_url {
                println!("   📈 Expected performance improvement from CDN");
            }
        }
        Err(e) => {
            println!("   ⚠️  vx would fall back to original URL: {}", e);
        }
    }

    println!("\n🎉 vx integration examples completed!");
    println!("   Integration ready for production use in vx tool");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_vx_manager_creation() {
        let manager = VxCdnManager::default();
        assert!(manager.config.enable_cdn_optimization);
    }

    #[tokio::test]
    async fn test_url_optimization() {
        let manager = VxCdnManager::default();
        let url = "https://github.com/BurntSushi/ripgrep/releases/download/14.1.1/ripgrep-14.1.1-x86_64-pc-windows-msvc.zip";

        let result = manager.optimize_url_for_vx(url).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_optimization_stats() {
        let manager = VxCdnManager::default();
        let urls = vec![
            "https://github.com/cli/cli/releases/download/v2.40.1/gh_2.40.1_windows_amd64.zip"
                .to_string(),
        ];

        let stats = manager.get_optimization_stats(urls).await;
        assert_eq!(stats.total_urls, 1);
    }

    #[tokio::test]
    async fn test_disabled_optimization() {
        let config = VxConfig {
            enable_cdn_optimization: false,
            ..Default::default()
        };
        let manager = VxCdnManager::new(config);

        let url = "https://example.com/file.zip";
        let result = manager.optimize_url_for_vx(url).await.unwrap();
        assert_eq!(result, url);
    }
}
