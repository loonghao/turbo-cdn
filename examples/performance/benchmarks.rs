//! # Performance Benchmarks Example
//!
//! This example demonstrates how to benchmark Turbo CDN performance
//! and compare it with other download methods.

use turbo_cdn::{TurboCdn, DownloadOptions, async_api, Result};
use std::time::{Instant, Duration};
use std::path::PathBuf;
use std::collections::HashMap;

/// Benchmark result for a single download
#[derive(Debug, Clone)]
pub struct BenchmarkResult {
    pub method: String,
    pub url: String,
    pub file_size: u64,
    pub download_time: Duration,
    pub speed_mbps: f64,
    pub success: bool,
    pub error: Option<String>,
}

impl BenchmarkResult {
    pub fn new(method: &str, url: &str) -> Self {
        Self {
            method: method.to_string(),
            url: url.to_string(),
            file_size: 0,
            download_time: Duration::from_secs(0),
            speed_mbps: 0.0,
            success: false,
            error: None,
        }
    }

    pub fn with_success(mut self, file_size: u64, download_time: Duration) -> Self {
        self.file_size = file_size;
        self.download_time = download_time;
        self.speed_mbps = (file_size as f64 * 8.0) / (download_time.as_secs_f64() * 1_000_000.0);
        self.success = true;
        self
    }

    pub fn with_error(mut self, error: String) -> Self {
        self.error = Some(error);
        self.success = false;
        self
    }
}

/// Benchmark suite for comparing download methods
pub struct DownloadBenchmark {
    test_urls: Vec<String>,
    output_dir: PathBuf,
}

impl DownloadBenchmark {
    pub fn new() -> Self {
        Self {
            test_urls: vec![
                "https://github.com/BurntSushi/ripgrep/releases/download/14.1.1/ripgrep-14.1.1-x86_64-pc-windows-msvc.zip".to_string(),
                "https://github.com/sharkdp/fd/releases/download/v8.7.0/fd-v8.7.0-x86_64-pc-windows-msvc.zip".to_string(),
                "https://github.com/sharkdp/bat/releases/download/v0.24.0/bat-v0.24.0-x86_64-pc-windows-msvc.zip".to_string(),
                "https://registry.npmjs.org/typescript/-/typescript-5.3.3.tgz".to_string(),
            ],
            output_dir: PathBuf::from("./benchmark-downloads"),
        }
    }

    pub fn with_urls(mut self, urls: Vec<String>) -> Self {
        self.test_urls = urls;
        self
    }

    pub fn with_output_dir(mut self, dir: PathBuf) -> Self {
        self.output_dir = dir;
        self
    }

    /// Run comprehensive benchmarks
    pub async fn run_comprehensive_benchmark(&self) -> Result<Vec<BenchmarkResult>> {
        println!("🚀 Turbo CDN - Comprehensive Performance Benchmark");
        println!("=================================================");
        
        // Ensure output directory exists
        std::fs::create_dir_all(&self.output_dir)?;
        
        let mut all_results = Vec::new();
        
        for (i, url) in self.test_urls.iter().enumerate() {
            println!("\n📋 Test {}/{}: {}", i + 1, self.test_urls.len(), url);
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            
            // Test different methods
            let methods = vec![
                ("Turbo CDN (Default)", Self::benchmark_turbo_cdn_default),
                ("Turbo CDN (Optimized)", Self::benchmark_turbo_cdn_optimized),
                ("Turbo CDN (Conservative)", Self::benchmark_turbo_cdn_conservative),
                ("Async API (Quick)", Self::benchmark_async_api),
            ];
            
            for (method_name, benchmark_fn) in methods {
                println!("\n🔍 Testing: {}", method_name);
                
                let result = benchmark_fn(url, &self.output_dir).await;
                
                match &result {
                    Ok(bench_result) if bench_result.success => {
                        println!("   ✅ Success: {:.2} MB/s ({:.2}s)", 
                            bench_result.speed_mbps, 
                            bench_result.download_time.as_secs_f64()
                        );
                    }
                    Ok(bench_result) => {
                        println!("   ❌ Failed: {}", 
                            bench_result.error.as_ref().unwrap_or(&"Unknown error".to_string())
                        );
                    }
                    Err(e) => {
                        println!("   ❌ Error: {}", e);
                    }
                }
                
                if let Ok(bench_result) = result {
                    all_results.push(bench_result);
                }
                
                // Small delay between tests
                tokio::time::sleep(Duration::from_millis(500)).await;
            }
        }
        
        // Print summary
        self.print_benchmark_summary(&all_results);
        
        Ok(all_results)
    }

    /// Benchmark Turbo CDN with default settings
    async fn benchmark_turbo_cdn_default(url: &str, output_dir: &PathBuf) -> Result<BenchmarkResult> {
        let mut result = BenchmarkResult::new("Turbo CDN (Default)", url);
        
        let start = Instant::now();
        
        match TurboCdn::new().await {
            Ok(turbo_cdn) => {
                match turbo_cdn.download_from_url(url).await {
                    Ok(download_result) => {
                        let duration = start.elapsed();
                        result = result.with_success(download_result.size, duration);
                    }
                    Err(e) => {
                        result = result.with_error(e.to_string());
                    }
                }
            }
            Err(e) => {
                result = result.with_error(format!("Failed to create client: {}", e));
            }
        }
        
        Ok(result)
    }

    /// Benchmark Turbo CDN with optimized settings
    async fn benchmark_turbo_cdn_optimized(url: &str, output_dir: &PathBuf) -> Result<BenchmarkResult> {
        let mut result = BenchmarkResult::new("Turbo CDN (Optimized)", url);
        
        let options = DownloadOptions {
            max_concurrent_chunks: 16,
            chunk_size: 4 * 1024 * 1024, // 4MB
            enable_resume: true,
            custom_headers: None,
            timeout_override: Some(Duration::from_secs(300)),
            verify_integrity: false, // Skip for speed
            expected_size: None,
            progress_callback: None,
        };
        
        let start = Instant::now();
        
        match TurboCdn::new().await {
            Ok(turbo_cdn) => {
                match turbo_cdn.download_from_url_with_options(url, options).await {
                    Ok(download_result) => {
                        let duration = start.elapsed();
                        result = result.with_success(download_result.size, duration);
                    }
                    Err(e) => {
                        result = result.with_error(e.to_string());
                    }
                }
            }
            Err(e) => {
                result = result.with_error(format!("Failed to create client: {}", e));
            }
        }
        
        Ok(result)
    }

    /// Benchmark Turbo CDN with conservative settings
    async fn benchmark_turbo_cdn_conservative(url: &str, output_dir: &PathBuf) -> Result<BenchmarkResult> {
        let mut result = BenchmarkResult::new("Turbo CDN (Conservative)", url);
        
        let options = DownloadOptions {
            max_concurrent_chunks: 2,
            chunk_size: 512 * 1024, // 512KB
            enable_resume: true,
            custom_headers: None,
            timeout_override: Some(Duration::from_secs(120)),
            verify_integrity: true,
            expected_size: None,
            progress_callback: None,
        };
        
        let start = Instant::now();
        
        match TurboCdn::new().await {
            Ok(turbo_cdn) => {
                match turbo_cdn.download_from_url_with_options(url, options).await {
                    Ok(download_result) => {
                        let duration = start.elapsed();
                        result = result.with_success(download_result.size, duration);
                    }
                    Err(e) => {
                        result = result.with_error(e.to_string());
                    }
                }
            }
            Err(e) => {
                result = result.with_error(format!("Failed to create client: {}", e));
            }
        }
        
        Ok(result)
    }

    /// Benchmark async API
    async fn benchmark_async_api(url: &str, output_dir: &PathBuf) -> Result<BenchmarkResult> {
        let mut result = BenchmarkResult::new("Async API (Quick)", url);
        
        let start = Instant::now();
        
        match async_api::quick::download_url(url).await {
            Ok(download_result) => {
                let duration = start.elapsed();
                result = result.with_success(download_result.size, duration);
            }
            Err(e) => {
                result = result.with_error(e.to_string());
            }
        }
        
        Ok(result)
    }

    /// Print benchmark summary
    fn print_benchmark_summary(&self, results: &[BenchmarkResult]) {
        println!("\n📊 Benchmark Summary");
        println!("===================");
        
        // Group results by method
        let mut method_results: HashMap<String, Vec<&BenchmarkResult>> = HashMap::new();
        for result in results {
            method_results.entry(result.method.clone()).or_insert_with(Vec::new).push(result);
        }
        
        // Calculate statistics for each method
        for (method, method_results) in method_results {
            let successful_results: Vec<_> = method_results.iter()
                .filter(|r| r.success)
                .collect();
            
            if successful_results.is_empty() {
                println!("\n❌ {}: No successful downloads", method);
                continue;
            }
            
            let avg_speed = successful_results.iter()
                .map(|r| r.speed_mbps)
                .sum::<f64>() / successful_results.len() as f64;
            
            let avg_time = successful_results.iter()
                .map(|r| r.download_time.as_secs_f64())
                .sum::<f64>() / successful_results.len() as f64;
            
            let total_size = successful_results.iter()
                .map(|r| r.file_size)
                .sum::<u64>();
            
            let success_rate = (successful_results.len() as f64 / method_results.len() as f64) * 100.0;
            
            println!("\n🚀 {}", method);
            println!("   📊 Success rate: {:.1}% ({}/{})", 
                success_rate, successful_results.len(), method_results.len());
            println!("   ⚡ Average speed: {:.2} MB/s", avg_speed);
            println!("   ⏱️  Average time: {:.2}s", avg_time);
            println!("   📦 Total downloaded: {:.2} MB", total_size as f64 / 1_000_000.0);
        }
        
        // Find best performing method
        if let Some(best_method) = self.find_best_method(results) {
            println!("\n🏆 Best Performing Method: {}", best_method);
        }
    }

    /// Find the best performing method
    fn find_best_method(&self, results: &[BenchmarkResult]) -> Option<String> {
        let mut method_speeds: HashMap<String, Vec<f64>> = HashMap::new();
        
        for result in results {
            if result.success {
                method_speeds.entry(result.method.clone())
                    .or_insert_with(Vec::new)
                    .push(result.speed_mbps);
            }
        }
        
        method_speeds.into_iter()
            .map(|(method, speeds)| {
                let avg_speed = speeds.iter().sum::<f64>() / speeds.len() as f64;
                (method, avg_speed)
            })
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .map(|(method, _)| method)
    }

    /// Run concurrent download benchmark
    pub async fn run_concurrent_benchmark(&self, concurrency: usize) -> Result<Vec<BenchmarkResult>> {
        println!("\n🔄 Concurrent Download Benchmark (Concurrency: {})", concurrency);
        println!("================================================");
        
        let mut tasks = Vec::new();
        
        for (i, url) in self.test_urls.iter().enumerate() {
            let url = url.clone();
            let output_dir = self.output_dir.clone();
            
            let task = tokio::spawn(async move {
                println!("🚀 Starting concurrent download {}: {}", i + 1, url);
                Self::benchmark_turbo_cdn_default(&url, &output_dir).await
            });
            
            tasks.push(task);
            
            // Limit concurrency
            if tasks.len() >= concurrency {
                break;
            }
        }
        
        let start = Instant::now();
        let mut results = Vec::new();
        
        for task in tasks {
            match task.await {
                Ok(Ok(result)) => results.push(result),
                Ok(Err(e)) => println!("❌ Task error: {}", e),
                Err(e) => println!("❌ Join error: {}", e),
            }
        }
        
        let total_time = start.elapsed();
        
        println!("\n📊 Concurrent Benchmark Results:");
        println!("   ⏱️  Total time: {:.2}s", total_time.as_secs_f64());
        println!("   📊 Completed downloads: {}", results.len());
        
        if !results.is_empty() {
            let total_size: u64 = results.iter().map(|r| r.file_size).sum();
            let overall_speed = (total_size as f64 * 8.0) / (total_time.as_secs_f64() * 1_000_000.0);
            println!("   ⚡ Overall speed: {:.2} MB/s", overall_speed);
        }
        
        Ok(results)
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    turbo_cdn::init_tracing();

    println!("🏁 Turbo CDN - Performance Benchmarks");
    println!("=====================================");

    // Create benchmark suite
    let benchmark = DownloadBenchmark::new();

    // Run comprehensive benchmark
    println!("\n🔍 Running comprehensive benchmark...");
    let _results = benchmark.run_comprehensive_benchmark().await?;

    // Run concurrent benchmark
    println!("\n🔄 Running concurrent benchmark...");
    let _concurrent_results = benchmark.run_concurrent_benchmark(4).await?;

    // Custom benchmark with specific URLs
    println!("\n🎯 Running custom benchmark...");
    let custom_urls = vec![
        "https://github.com/microsoft/vscode/releases/download/1.85.0/VSCode-linux-x64.tar.gz".to_string(),
        "https://nodejs.org/dist/v20.10.0/node-v20.10.0-linux-x64.tar.xz".to_string(),
    ];
    
    let custom_benchmark = DownloadBenchmark::new()
        .with_urls(custom_urls)
        .with_output_dir(PathBuf::from("./custom-benchmark"));
    
    let _custom_results = custom_benchmark.run_comprehensive_benchmark().await?;

    println!("\n🎉 All benchmarks completed!");
    println!("   📁 Results saved to benchmark directories");
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_benchmark_creation() {
        let benchmark = DownloadBenchmark::new();
        assert!(!benchmark.test_urls.is_empty());
    }

    #[tokio::test]
    async fn test_benchmark_result() {
        let result = BenchmarkResult::new("Test", "https://example.com")
            .with_success(1000000, Duration::from_secs(1));
        
        assert!(result.success);
        assert_eq!(result.file_size, 1000000);
        assert!(result.speed_mbps > 0.0);
    }

    #[tokio::test]
    async fn test_single_benchmark() {
        let url = "https://github.com/BurntSushi/ripgrep/releases/download/14.1.1/ripgrep-14.1.1-x86_64-pc-windows-msvc.zip";
        let output_dir = PathBuf::from("./test-benchmark");
        
        let result = DownloadBenchmark::benchmark_async_api(url, &output_dir).await;
        assert!(result.is_ok());
    }
}
