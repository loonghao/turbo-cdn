// Licensed under the MIT License
// Copyright (c) 2025 Hal <hal.long@outlook.com>

//! Smart downloader with automatic method selection
//!
//! This module implements intelligent download method selection by
//! testing multiple approaches and choosing the fastest one.

use crate::concurrent_downloader::DownloadResult;
use crate::error::{Result, TurboCdnError};
use crate::{api_info, cli_info};
use std::time::{Duration, Instant};
use tokio::time::timeout;
// Note: tracing macros are used via crate::cli_info! and crate::api_info!
use std::sync::Arc;

/// Speed test result for a download method
#[derive(Debug, Clone)]
pub struct SpeedTestResult {
    pub method: DownloadMethod,
    pub speed_mbps: f64,
    pub response_time: Duration,
    pub success: bool,
    pub error: Option<String>,
}

/// Download method types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DownloadMethod {
    Direct,
    Cdn,
    CdnFast,
}

impl DownloadMethod {
    pub fn name(&self) -> &'static str {
        match self {
            DownloadMethod::Direct => "Direct",
            DownloadMethod::Cdn => "CDN",
            DownloadMethod::CdnFast => "CDN-Fast",
        }
    }
}

/// Smart download configuration
#[derive(Debug, Clone)]
pub struct SmartDownloadConfig {
    /// Test size in bytes for speed testing
    pub test_size: u64,
    /// Timeout for speed tests
    pub test_timeout: Duration,
    /// Minimum speed advantage to prefer CDN (e.g., 1.2 = 20% faster)
    pub cdn_advantage_threshold: f64,
    /// Maximum time to spend on speed testing
    pub max_test_time: Duration,
    /// Enable parallel testing
    pub parallel_testing: bool,
}

impl Default for SmartDownloadConfig {
    fn default() -> Self {
        Self {
            test_size: 64 * 1024, // 64KB test
            test_timeout: Duration::from_secs(5),
            cdn_advantage_threshold: 1.15, // CDN must be 15% faster
            max_test_time: Duration::from_secs(8),
            parallel_testing: true,
        }
    }
}

/// Smart downloader that automatically selects the best method
pub struct SmartDownloader {
    config: SmartDownloadConfig,
    turbo_cdn: Arc<crate::TurboCdn>,
    verbose: bool,
}

impl SmartDownloader {
    /// Create a new smart downloader
    pub async fn new() -> Result<Self> {
        Self::new_with_verbose(false).await
    }

    /// Create a new smart downloader with verbose flag
    pub async fn new_with_verbose(verbose: bool) -> Result<Self> {
        let turbo_cdn = Arc::new(crate::TurboCdn::new().await?);
        Ok(Self {
            config: SmartDownloadConfig::default(),
            turbo_cdn,
            verbose,
        })
    }

    /// Create a smart downloader with custom configuration
    pub async fn with_config(config: SmartDownloadConfig) -> Result<Self> {
        Self::with_config_and_verbose(config, false).await
    }

    /// Create a smart downloader with custom configuration and verbose flag
    pub async fn with_config_and_verbose(
        config: SmartDownloadConfig,
        verbose: bool,
    ) -> Result<Self> {
        let turbo_cdn = Arc::new(crate::TurboCdn::new().await?);
        Ok(Self {
            config,
            turbo_cdn,
            verbose,
        })
    }

    /// Smart download that automatically selects the best method
    pub async fn download_smart(&self, url: &str) -> Result<DownloadResult> {
        if self.verbose {
            cli_info!("🧠 Smart download starting for: {}", url);
        } else {
            api_info!("Smart download starting for: {}", url);
        }

        // Step 1: Quick speed test
        let test_results = if self.config.parallel_testing {
            self.parallel_speed_test(url).await?
        } else {
            self.sequential_speed_test(url).await?
        };

        // Step 2: Analyze results and select best method
        let best_method = self.select_best_method(&test_results);

        // Step 3: Show results to user
        self.display_test_results(&test_results, best_method, self.verbose);

        // Step 4: Download using the best method
        match best_method {
            DownloadMethod::Direct => {
                if self.verbose {
                    cli_info!("⚡ Using direct download");
                }
                self.turbo_cdn.download_direct_from_url(url).await
            }
            DownloadMethod::Cdn | DownloadMethod::CdnFast => {
                if self.verbose {
                    cli_info!("🌐 Using CDN download");
                }
                self.turbo_cdn.download_from_url(url).await
            }
        }
    }

    /// Download to specific path with smart method selection
    pub async fn download_smart_to_path<P: AsRef<std::path::Path>>(
        &self,
        url: &str,
        output_path: P,
    ) -> Result<DownloadResult> {
        if self.verbose {
            cli_info!("🧠 Smart download starting for: {}", url);
        } else {
            api_info!("Smart download starting for: {}", url);
        }

        let test_results = if self.config.parallel_testing {
            self.parallel_speed_test(url).await?
        } else {
            self.sequential_speed_test(url).await?
        };

        let best_method = self.select_best_method(&test_results);
        self.display_test_results(&test_results, best_method, self.verbose);

        match best_method {
            DownloadMethod::Direct => {
                if self.verbose {
                    cli_info!("⚡ Using direct download");
                }
                self.turbo_cdn
                    .download_direct_to_path(url, output_path)
                    .await
            }
            DownloadMethod::Cdn | DownloadMethod::CdnFast => {
                if self.verbose {
                    cli_info!("🌐 Using CDN download");
                }
                self.turbo_cdn.download_to_path(url, output_path).await
            }
        }
    }

    /// Perform parallel speed tests
    async fn parallel_speed_test(&self, url: &str) -> Result<Vec<SpeedTestResult>> {
        if self.verbose {
            cli_info!("🔍 Running parallel speed tests...");
        }

        let test_timeout = self.config.max_test_time;

        let results = timeout(test_timeout, async {
            let direct_test = self.test_direct_speed(url);
            let cdn_test = self.test_cdn_speed(url);

            // Run tests in parallel
            let (direct_result, cdn_result) = tokio::join!(direct_test, cdn_test);

            vec![direct_result, cdn_result]
        })
        .await
        .map_err(|_| TurboCdnError::network("Speed test timeout".to_string()))?;

        Ok(results)
    }

    /// Perform sequential speed tests
    async fn sequential_speed_test(&self, url: &str) -> Result<Vec<SpeedTestResult>> {
        if self.verbose {
            cli_info!("🔍 Running sequential speed tests...");
        }

        let mut results = Vec::new();

        // Test direct first (usually faster to test)
        results.push(self.test_direct_speed(url).await);

        // Test CDN only if direct test was successful
        if results[0].success {
            results.push(self.test_cdn_speed(url).await);
        } else {
            // If direct failed, still try CDN
            results.push(self.test_cdn_speed(url).await);
        }

        Ok(results)
    }

    /// Test direct download speed
    async fn test_direct_speed(&self, url: &str) -> SpeedTestResult {
        let start_time = Instant::now();

        match timeout(
            self.config.test_timeout,
            self.perform_range_request(url, true),
        )
        .await
        {
            Ok(Ok(bytes_downloaded)) => {
                let elapsed = start_time.elapsed();
                let speed_mbps =
                    (bytes_downloaded as f64 * 8.0) / (elapsed.as_secs_f64() * 1_000_000.0);

                SpeedTestResult {
                    method: DownloadMethod::Direct,
                    speed_mbps,
                    response_time: elapsed,
                    success: true,
                    error: None,
                }
            }
            Ok(Err(e)) => SpeedTestResult {
                method: DownloadMethod::Direct,
                speed_mbps: 0.0,
                response_time: start_time.elapsed(),
                success: false,
                error: Some(e.to_string()),
            },
            Err(_) => SpeedTestResult {
                method: DownloadMethod::Direct,
                speed_mbps: 0.0,
                response_time: self.config.test_timeout,
                success: false,
                error: Some("Timeout".to_string()),
            },
        }
    }

    /// Test CDN download speed
    async fn test_cdn_speed(&self, url: &str) -> SpeedTestResult {
        let start_time = Instant::now();

        match timeout(
            self.config.test_timeout,
            self.perform_range_request(url, false),
        )
        .await
        {
            Ok(Ok(bytes_downloaded)) => {
                let elapsed = start_time.elapsed();
                let speed_mbps =
                    (bytes_downloaded as f64 * 8.0) / (elapsed.as_secs_f64() * 1_000_000.0);

                SpeedTestResult {
                    method: DownloadMethod::Cdn,
                    speed_mbps,
                    response_time: elapsed,
                    success: true,
                    error: None,
                }
            }
            Ok(Err(e)) => SpeedTestResult {
                method: DownloadMethod::Cdn,
                speed_mbps: 0.0,
                response_time: start_time.elapsed(),
                success: false,
                error: Some(e.to_string()),
            },
            Err(_) => SpeedTestResult {
                method: DownloadMethod::Cdn,
                speed_mbps: 0.0,
                response_time: self.config.test_timeout,
                success: false,
                error: Some("Timeout".to_string()),
            },
        }
    }

    /// Perform a range request to test download speed
    async fn perform_range_request(&self, url: &str, direct: bool) -> Result<u64> {
        // Initialize rustls provider before creating reqwest client
        crate::init_rustls_provider();
        
        let client = reqwest::Client::new();
        let test_url = if direct {
            url.to_string()
        } else {
            // Get the first CDN URL
            match self.turbo_cdn.get_all_cdn_urls(url).await {
                Ok(urls) if !urls.is_empty() => urls[0].clone(),
                _ => url.to_string(), // Fallback to direct
            }
        };

        let range_header = format!("bytes=0-{}", self.config.test_size - 1);

        let response = client
            .get(&test_url)
            .header("Range", range_header)
            .send()
            .await
            .map_err(|e| TurboCdnError::network(format!("Request failed: {e}")))?;

        if !response.status().is_success() && response.status() != 206 {
            return Err(TurboCdnError::network(format!(
                "HTTP error: {}",
                response.status()
            )));
        }

        let bytes = response
            .bytes()
            .await
            .map_err(|e| TurboCdnError::network(format!("Failed to read response: {e}")))?;

        Ok(bytes.len() as u64)
    }

    /// Select the best download method based on test results
    fn select_best_method(&self, results: &[SpeedTestResult]) -> DownloadMethod {
        let mut best_method = DownloadMethod::Direct;
        let mut best_speed = 0.0;

        // Find the fastest successful method
        for result in results {
            if result.success && result.speed_mbps > best_speed {
                best_speed = result.speed_mbps;
                best_method = result.method;
            }
        }

        // Apply CDN advantage threshold
        if let (Some(direct), Some(cdn)) = (
            results
                .iter()
                .find(|r| r.method == DownloadMethod::Direct && r.success),
            results
                .iter()
                .find(|r| r.method == DownloadMethod::Cdn && r.success),
        ) {
            // CDN must be significantly faster to be chosen
            if cdn.speed_mbps > direct.speed_mbps * self.config.cdn_advantage_threshold {
                return DownloadMethod::Cdn;
            } else {
                return DownloadMethod::Direct;
            }
        }

        best_method
    }

    /// Display test results to user
    fn display_test_results(
        &self,
        results: &[SpeedTestResult],
        selected: DownloadMethod,
        verbose: bool,
    ) {
        // Convert to format expected by cli_progress
        let display_results: Vec<(String, f64, bool)> = results
            .iter()
            .map(|r| {
                let speed = if r.success { r.speed_mbps / 8.0 } else { 0.0 }; // Convert to MB/s
                let is_selected = r.method == selected;
                (r.method.name().to_string(), speed, is_selected)
            })
            .collect();

        crate::cli_progress::display_speed_test_results(&display_results, verbose);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_download_method_name() {
        assert_eq!(DownloadMethod::Direct.name(), "Direct");
        assert_eq!(DownloadMethod::Cdn.name(), "CDN");
        assert_eq!(DownloadMethod::CdnFast.name(), "CDN-Fast");
    }

    #[test]
    fn test_smart_download_config_default() {
        let config = SmartDownloadConfig::default();
        assert_eq!(config.test_size, 64 * 1024);
        assert_eq!(config.cdn_advantage_threshold, 1.15);
        assert!(config.parallel_testing);
    }
}
