//! # Performance Monitoring Example
//!
//! This example demonstrates how to monitor and track Turbo CDN performance
//! in real-time, including metrics collection and analysis.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use turbo_cdn::{DownloadOptions, Result, TurboCdn};

/// Performance metrics for a single download
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadMetrics {
    pub timestamp: u64,
    pub url: String,
    pub file_size: u64,
    pub download_time: f64,
    pub speed_mbps: f64,
    pub chunks_used: usize,
    pub chunk_size: usize,
    pub cdn_optimized: bool,
    pub resume_count: usize,
    pub error: Option<String>,
}

impl DownloadMetrics {
    pub fn new(url: String) -> Self {
        Self {
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            url,
            file_size: 0,
            download_time: 0.0,
            speed_mbps: 0.0,
            chunks_used: 0,
            chunk_size: 0,
            cdn_optimized: false,
            resume_count: 0,
            error: None,
        }
    }
}

/// Aggregated performance statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceStats {
    pub total_downloads: usize,
    pub successful_downloads: usize,
    pub failed_downloads: usize,
    pub total_bytes: u64,
    pub total_time: f64,
    pub average_speed: f64,
    pub peak_speed: f64,
    pub cdn_optimization_rate: f64,
    pub error_rate: f64,
}

impl Default for PerformanceStats {
    fn default() -> Self {
        Self {
            total_downloads: 0,
            successful_downloads: 0,
            failed_downloads: 0,
            total_bytes: 0,
            total_time: 0.0,
            average_speed: 0.0,
            peak_speed: 0.0,
            cdn_optimization_rate: 0.0,
            error_rate: 0.0,
        }
    }
}

/// Real-time performance monitor
pub struct PerformanceMonitor {
    metrics: Arc<Mutex<VecDeque<DownloadMetrics>>>,
    max_history: usize,
}

impl PerformanceMonitor {
    pub fn new(max_history: usize) -> Self {
        Self {
            metrics: Arc::new(Mutex::new(VecDeque::new())),
            max_history,
        }
    }

    /// Record a download metric
    pub fn record_download(&self, metric: DownloadMetrics) {
        let mut metrics = self.metrics.lock().unwrap();

        // Add new metric
        metrics.push_back(metric);

        // Maintain max history size
        while metrics.len() > self.max_history {
            metrics.pop_front();
        }
    }

    /// Get current performance statistics
    pub fn get_stats(&self) -> PerformanceStats {
        let metrics = self.metrics.lock().unwrap();

        if metrics.is_empty() {
            return PerformanceStats::default();
        }

        let total_downloads = metrics.len();
        let successful_downloads = metrics.iter().filter(|m| m.error.is_none()).count();
        let failed_downloads = total_downloads - successful_downloads;

        let successful_metrics: Vec<_> = metrics.iter().filter(|m| m.error.is_none()).collect();

        let total_bytes = successful_metrics.iter().map(|m| m.file_size).sum();
        let total_time = successful_metrics.iter().map(|m| m.download_time).sum();

        let average_speed = if !successful_metrics.is_empty() {
            successful_metrics.iter().map(|m| m.speed_mbps).sum::<f64>()
                / successful_metrics.len() as f64
        } else {
            0.0
        };

        let peak_speed = successful_metrics
            .iter()
            .map(|m| m.speed_mbps)
            .fold(0.0, f64::max);

        let cdn_optimized_count = metrics.iter().filter(|m| m.cdn_optimized).count();
        let cdn_optimization_rate = (cdn_optimized_count as f64 / total_downloads as f64) * 100.0;

        let error_rate = (failed_downloads as f64 / total_downloads as f64) * 100.0;

        PerformanceStats {
            total_downloads,
            successful_downloads,
            failed_downloads,
            total_bytes,
            total_time,
            average_speed,
            peak_speed,
            cdn_optimization_rate,
            error_rate,
        }
    }

    /// Get recent metrics (last N entries)
    pub fn get_recent_metrics(&self, count: usize) -> Vec<DownloadMetrics> {
        let metrics = self.metrics.lock().unwrap();
        metrics.iter().rev().take(count).cloned().collect()
    }

    /// Get metrics within time range
    pub fn get_metrics_in_range(&self, start_time: u64, end_time: u64) -> Vec<DownloadMetrics> {
        let metrics = self.metrics.lock().unwrap();
        metrics
            .iter()
            .filter(|m| m.timestamp >= start_time && m.timestamp <= end_time)
            .cloned()
            .collect()
    }

    /// Export metrics to JSON
    pub fn export_metrics(&self) -> Result<String> {
        let metrics = self.metrics.lock().unwrap();
        let json = serde_json::to_string_pretty(&*metrics)?;
        Ok(json)
    }

    /// Clear all metrics
    pub fn clear_metrics(&self) {
        let mut metrics = self.metrics.lock().unwrap();
        metrics.clear();
    }
}

/// Enhanced TurboCdn client with monitoring
pub struct MonitoredTurboCdn {
    client: TurboCdn,
    monitor: Arc<PerformanceMonitor>,
}

impl MonitoredTurboCdn {
    pub async fn new() -> Result<Self> {
        let client = TurboCdn::new().await?;
        let monitor = Arc::new(PerformanceMonitor::new(1000)); // Keep last 1000 downloads

        Ok(Self { client, monitor })
    }

    pub fn get_monitor(&self) -> Arc<PerformanceMonitor> {
        self.monitor.clone()
    }

    /// Download with performance monitoring
    pub async fn download_with_monitoring(&self, url: &str) -> Result<turbo_cdn::DownloadResult> {
        let mut metric = DownloadMetrics::new(url.to_string());
        let start = Instant::now();

        // Check if URL can be optimized
        match self.client.get_optimal_url(url).await {
            Ok(optimal_url) => {
                metric.cdn_optimized = optimal_url != url;
            }
            Err(_) => {
                metric.cdn_optimized = false;
            }
        }

        // Perform download
        match self
            .client
            .download_to_path(url, std::env::temp_dir().join("monitoring_basic"))
            .await
        {
            Ok(result) => {
                let duration = start.elapsed();

                metric.file_size = result.size;
                metric.download_time = duration.as_secs_f64();
                metric.speed_mbps =
                    (result.size as f64 * 8.0) / (duration.as_secs_f64() * 1_000_000.0);

                self.monitor.record_download(metric);
                Ok(result)
            }
            Err(e) => {
                metric.error = Some(e.to_string());
                self.monitor.record_download(metric);
                Err(e)
            }
        }
    }

    /// Download with custom options and monitoring
    pub async fn download_with_options_and_monitoring(
        &self,
        url: &str,
        options: DownloadOptions,
    ) -> Result<turbo_cdn::DownloadResult> {
        let mut metric = DownloadMetrics::new(url.to_string());
        let start = Instant::now();

        // Record configuration
        metric.chunks_used = options.max_concurrent_chunks.unwrap_or(4);
        metric.chunk_size = options.chunk_size.unwrap_or(1024 * 1024) as usize;

        // Check CDN optimization
        match self.client.get_optimal_url(url).await {
            Ok(optimal_url) => {
                metric.cdn_optimized = optimal_url != url;
            }
            Err(_) => {
                metric.cdn_optimized = false;
            }
        }

        // Perform download
        match self
            .client
            .download_with_options(
                url,
                std::env::temp_dir().join("monitoring_download"),
                options,
            )
            .await
        {
            Ok(result) => {
                let duration = start.elapsed();

                metric.file_size = result.size;
                metric.download_time = duration.as_secs_f64();
                metric.speed_mbps =
                    (result.size as f64 * 8.0) / (duration.as_secs_f64() * 1_000_000.0);

                self.monitor.record_download(metric);
                Ok(result)
            }
            Err(e) => {
                metric.error = Some(e.to_string());
                self.monitor.record_download(metric);
                Err(e)
            }
        }
    }
}

/// Real-time performance dashboard
pub struct PerformanceDashboard {
    monitor: Arc<PerformanceMonitor>,
}

impl PerformanceDashboard {
    pub fn new(monitor: Arc<PerformanceMonitor>) -> Self {
        Self { monitor }
    }

    /// Display current performance dashboard
    pub fn display_dashboard(&self) {
        let stats = self.monitor.get_stats();
        let recent_metrics = self.monitor.get_recent_metrics(5);

        println!("📊 Turbo CDN Performance Dashboard");
        println!("==================================");

        // Overall statistics
        println!("\n📈 Overall Statistics:");
        println!("   📋 Total downloads: {}", stats.total_downloads);
        println!(
            "   ✅ Successful: {} ({:.1}%)",
            stats.successful_downloads,
            if stats.total_downloads > 0 {
                (stats.successful_downloads as f64 / stats.total_downloads as f64) * 100.0
            } else {
                0.0
            }
        );
        println!(
            "   ❌ Failed: {} ({:.1}%)",
            stats.failed_downloads, stats.error_rate
        );
        println!(
            "   📦 Total data: {:.2} MB",
            stats.total_bytes as f64 / 1_000_000.0
        );
        println!("   ⏱️  Total time: {:.2}s", stats.total_time);

        // Performance metrics
        println!("\n⚡ Performance Metrics:");
        println!("   📊 Average speed: {:.2} MB/s", stats.average_speed);
        println!("   🚀 Peak speed: {:.2} MB/s", stats.peak_speed);
        println!(
            "   🌐 CDN optimization rate: {:.1}%",
            stats.cdn_optimization_rate
        );

        // Recent downloads
        if !recent_metrics.is_empty() {
            println!("\n📋 Recent Downloads:");
            for (i, metric) in recent_metrics.iter().enumerate() {
                let status = if metric.error.is_none() { "✅" } else { "❌" };
                let cdn_status = if metric.cdn_optimized { "🚀" } else { "📡" };

                println!(
                    "   {}. {} {} {:.2} MB/s - {} {}",
                    i + 1,
                    status,
                    cdn_status,
                    metric.speed_mbps,
                    metric.url.split('/').last().unwrap_or("unknown"),
                    if let Some(ref error) = metric.error {
                        format!("({})", error)
                    } else {
                        String::new()
                    }
                );
            }
        }

        println!("==================================");
    }

    /// Start real-time monitoring
    pub async fn start_realtime_monitoring(&self, interval_secs: u64) {
        println!("🔄 Starting real-time performance monitoring...");
        println!("Press Ctrl+C to stop");

        let mut interval = tokio::time::interval(Duration::from_secs(interval_secs));

        loop {
            interval.tick().await;

            // Clear screen (ANSI escape code)
            print!("\x1B[2J\x1B[1;1H");

            self.display_dashboard();

            // Show timestamp
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
            println!(
                "🕐 Last updated: {}",
                chrono::DateTime::from_timestamp(now as i64, 0)
                    .unwrap()
                    .format("%Y-%m-%d %H:%M:%S")
            );
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    turbo_cdn::init_tracing();

    println!("📊 Turbo CDN - Performance Monitoring Example");
    println!("=============================================");

    // Create monitored client
    let monitored_client = MonitoredTurboCdn::new().await?;
    let monitor = monitored_client.get_monitor();

    // Create dashboard
    let dashboard = PerformanceDashboard::new(monitor.clone());

    // Test URLs for monitoring
    let test_urls = vec![
        "https://github.com/BurntSushi/ripgrep/releases/download/14.1.1/ripgrep-14.1.1-x86_64-pc-windows-msvc.zip",
        "https://github.com/sharkdp/fd/releases/download/v8.7.0/fd-v8.7.0-x86_64-pc-windows-msvc.zip",
        "https://github.com/sharkdp/bat/releases/download/v0.24.0/bat-v0.24.0-x86_64-pc-windows-msvc.zip",
        "https://registry.npmjs.org/typescript/-/typescript-5.3.3.tgz",
    ];

    // Perform monitored downloads
    println!("\n🚀 Performing monitored downloads...");
    for (i, url) in test_urls.iter().enumerate() {
        println!("\n📥 Download {}/{}: {}", i + 1, test_urls.len(), url);

        match monitored_client.download_with_monitoring(url).await {
            Ok(result) => {
                println!("   ✅ Success: {:.2} MB/s", result.speed / 1_000_000.0);
            }
            Err(e) => {
                println!("   ❌ Failed: {}", e);
            }
        }

        // Show updated dashboard after each download
        dashboard.display_dashboard();

        // Small delay between downloads
        tokio::time::sleep(Duration::from_secs(1)).await;
    }

    // Test different configurations
    println!("\n🔧 Testing different configurations...");

    let configs = vec![
        (
            "High Performance",
            DownloadOptions {
                max_concurrent_chunks: Some(16),
                chunk_size: Some(4 * 1024 * 1024),
                enable_resume: true,
                custom_headers: None,
                timeout_override: Some(Duration::from_secs(300)),
                verify_integrity: false,
                expected_size: None,
                progress_callback: None,
            },
        ),
        (
            "Conservative",
            DownloadOptions {
                max_concurrent_chunks: Some(2),
                chunk_size: Some(512 * 1024),
                enable_resume: true,
                custom_headers: None,
                timeout_override: Some(Duration::from_secs(120)),
                verify_integrity: true,
                expected_size: None,
                progress_callback: None,
            },
        ),
    ];

    for (config_name, options) in configs {
        println!("\n⚙️  Testing {} configuration:", config_name);
        let test_url =
            "https://github.com/cli/cli/releases/download/v2.40.1/gh_2.40.1_windows_amd64.zip";

        match monitored_client
            .download_with_options_and_monitoring(test_url, options)
            .await
        {
            Ok(result) => {
                println!(
                    "   ✅ {}: {:.2} MB/s",
                    config_name,
                    result.speed / 1_000_000.0
                );
            }
            Err(e) => {
                println!("   ❌ {}: {}", config_name, e);
            }
        }
    }

    // Final dashboard
    println!("\n📊 Final Performance Report:");
    dashboard.display_dashboard();

    // Export metrics
    println!("\n💾 Exporting metrics...");
    match monitor.export_metrics() {
        Ok(json) => {
            std::fs::write("performance_metrics.json", json)?;
            println!("   ✅ Metrics exported to performance_metrics.json");
        }
        Err(e) => {
            println!("   ❌ Failed to export metrics: {}", e);
        }
    }

    println!("\n🎉 Performance monitoring completed!");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_performance_monitor() {
        let monitor = PerformanceMonitor::new(10);

        let metric = DownloadMetrics::new("https://example.com".to_string());
        monitor.record_download(metric);

        let stats = monitor.get_stats();
        assert_eq!(stats.total_downloads, 1);
    }

    #[test]
    fn test_metrics_history_limit() {
        let monitor = PerformanceMonitor::new(2);

        for i in 0..5 {
            let metric = DownloadMetrics::new(format!("https://example{}.com", i));
            monitor.record_download(metric);
        }

        let stats = monitor.get_stats();
        assert_eq!(stats.total_downloads, 2); // Should only keep last 2
    }

    #[tokio::test]
    async fn test_monitored_client() {
        let client = MonitoredTurboCdn::new().await;
        assert!(client.is_ok());
    }
}
