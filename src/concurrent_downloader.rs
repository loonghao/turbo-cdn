// Licensed under the MIT License
// Copyright (c) 2025 Hal <hal.long@outlook.com>

//! Concurrent download engine with dynamic segmentation
//!
//! This module provides high-performance concurrent downloads with features like:
//! - Range request support detection
//! - Dynamic chunk size adjustment
//! - Resume capability
//! - Progress tracking

use crate::constants::{
    DEFAULT_RETRY_ATTEMPTS, DEFAULT_RETRY_DELAY_BASE, HTTP2_FRAME_SIZE, MAX_REDIRECTS,
    MAX_URLS_TO_TRY,
};
use crate::error::{Result, TurboCdnError};
use crate::progress::ProgressTracker;
use crate::server_tracker::ServerTracker;
use reqwest::Client;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::fs::{File, OpenOptions};
use tokio::io::{AsyncSeekExt, AsyncWriteExt, SeekFrom};
use tokio::sync::Semaphore;
use tracing::{debug, info, warn};

/// Chunk information for concurrent downloads
#[derive(Debug, Clone)]
pub struct ChunkInfo {
    /// Start byte position
    pub start: u64,
    /// End byte position (inclusive)
    pub end: u64,
    /// Chunk index
    pub index: usize,
}

/// Download result information
#[derive(Debug, Clone)]
pub struct DownloadResult {
    /// Path to downloaded file
    pub path: PathBuf,
    /// Total bytes downloaded
    pub size: u64,
    /// Download duration
    pub duration: Duration,
    /// Average download speed (bytes/sec)
    pub speed: f64,
    /// Source URL used
    pub url: String,
    /// Whether resume was used
    pub resumed: bool,
}

/// High-performance concurrent downloader with dynamic segmentation
///
/// Provides intelligent chunked downloads with adaptive sizing, resume support,
/// and server performance tracking for optimal download speeds.
#[derive(Debug)]
pub struct ConcurrentDownloader {
    http_client: Client,
    max_concurrent_chunks: usize,
    initial_chunk_size: u64,
    min_chunk_size: u64,
    max_chunk_size: u64,
    #[allow(dead_code)]
    request_timeout: Duration,
    adaptive_chunking_enabled: bool,
    speed_threshold_bytes_per_sec: u64,
    server_performance_tracker: std::sync::Arc<std::sync::Mutex<ServerTracker>>,
}

impl ConcurrentDownloader {
    /// Create a new concurrent downloader
    pub fn new() -> Result<Self> {
        let config = crate::config::TurboCdnConfig::default();
        Self::with_config(&config)
    }

    /// Create a new concurrent downloader with configuration
    pub fn with_config(config: &crate::config::TurboCdnConfig) -> Result<Self> {
        let mut client_builder = Client::builder()
            .timeout(Duration::from_secs(config.performance.timeout))
            .user_agent(&config.general.user_agent)
            .pool_max_idle_per_host(config.performance.pool_max_idle_per_host)
            .pool_idle_timeout(Duration::from_secs(config.performance.pool_idle_timeout))
            .tcp_keepalive(Duration::from_secs(config.performance.tcp_keepalive))
            .tcp_nodelay(true) // Disable Nagle's algorithm for lower latency
            .connection_verbose(config.general.debug) // Enable connection debugging if debug mode
            .redirect(reqwest::redirect::Policy::limited(MAX_REDIRECTS));

        if config.performance.http2_prior_knowledge {
            client_builder = client_builder.http2_prior_knowledge();
        }

        // Configure TLS settings for better performance
        client_builder = client_builder
            .http2_adaptive_window(true)
            .http2_max_frame_size(Some(HTTP2_FRAME_SIZE));

        let client = client_builder
            .build()
            .map_err(|e| TurboCdnError::network(format!("Failed to create HTTP client: {e}")))?;

        Ok(Self {
            http_client: client,
            max_concurrent_chunks: config.performance.max_concurrent_downloads,
            initial_chunk_size: config.performance.chunk_size,
            min_chunk_size: config.performance.min_chunk_size,
            max_chunk_size: config.performance.max_chunk_size,
            request_timeout: Duration::from_secs(config.performance.timeout),
            adaptive_chunking_enabled: config.performance.adaptive_chunking,
            speed_threshold_bytes_per_sec: config.performance.speed_threshold_bytes_per_sec,
            server_performance_tracker: std::sync::Arc::new(std::sync::Mutex::new(
                ServerTracker::new(),
            )),
        })
    }

    /// Download a file from URL with automatic optimization and retry logic
    pub async fn download<P: AsRef<Path>>(
        &self,
        urls: &[String],
        output_path: P,
        progress_tracker: Option<Arc<ProgressTracker>>,
    ) -> Result<DownloadResult> {
        let output_path = output_path.as_ref();
        let start_time = Instant::now();

        // Use intelligent server selection - select more URLs for better redundancy
        let max_urls_to_try = (urls.len()).min(MAX_URLS_TO_TRY);
        let selected_urls = {
            let tracker = self.server_performance_tracker.lock().unwrap();
            tracker.select_best_servers(urls, max_urls_to_try)
        };

        info!(
            "Selected {} URLs for download from {} candidates (max_concurrent_chunks: {})",
            selected_urls.len(),
            urls.len(),
            self.max_concurrent_chunks
        );

        // Get retry attempts from config or use default
        let retry_attempts = DEFAULT_RETRY_ATTEMPTS;

        // Try each URL with retry logic
        for (index, url) in selected_urls.iter().enumerate() {
            debug!("Trying URL {}/{}: {}", index + 1, selected_urls.len(), url);

            // Retry logic for each URL
            for retry_attempt in 0..=retry_attempts {
                let url_start_time = Instant::now();

                if retry_attempt > 0 {
                    debug!("Retry attempt {} for URL: {}", retry_attempt, url);
                    // Exponential backoff
                    let delay =
                        Duration::from_secs(DEFAULT_RETRY_DELAY_BASE.pow(retry_attempt as u32 - 1));
                    tokio::time::sleep(delay).await;
                }

                match self
                    .download_single_url(url, output_path, progress_tracker.clone())
                    .await
                {
                    Ok(mut result) => {
                        let url_duration = url_start_time.elapsed();
                        result.duration = start_time.elapsed();
                        result.speed = result.size as f64 / result.duration.as_secs_f64();

                        // Record successful download
                        {
                            let mut tracker = self.server_performance_tracker.lock().unwrap();
                            tracker.record_success(url, result.speed, url_duration);
                        }

                        info!(
                            "Download completed: {} bytes in {:.2}s ({:.2} MB/s) from {} (attempt {})",
                            result.size,
                            result.duration.as_secs_f64(),
                            result.speed / 1024.0 / 1024.0,
                            url,
                            retry_attempt + 1
                        );
                        return Ok(result);
                    }
                    Err(e) => {
                        let url_duration = url_start_time.elapsed();

                        // Record failed download
                        {
                            let mut tracker = self.server_performance_tracker.lock().unwrap();
                            tracker.record_failure(url, url_duration);
                        }

                        // Check if we should skip retries for this URL
                        let should_skip_retries = !e.is_retryable();
                        let is_not_found = e.status_code() == Some(404);

                        if is_not_found {
                            warn!("HTTP 404 Not Found for {}, trying next mirror...", url);
                            break; // Skip retries, try next URL immediately
                        } else if should_skip_retries {
                            warn!(
                                "Non-retryable error for {}: {}, trying next mirror...",
                                url, e
                            );
                            break; // Skip retries, try next URL
                        } else {
                            warn!("Attempt {} failed for {}: {}", retry_attempt + 1, url, e);
                        }

                        // If this was the last retry for this URL, try next URL
                        if retry_attempt == retry_attempts {
                            break;
                        }
                    }
                }
            }

            // If we've exhausted all retries for all URLs, return error
            if index == selected_urls.len() - 1 {
                return Err(TurboCdnError::download(
                    "All download URLs failed after retries".to_string(),
                ));
            }
        }

        Err(TurboCdnError::download(
            "All download URLs failed".to_string(),
        ))
    }

    /// Download from a single URL
    async fn download_single_url<P: AsRef<Path>>(
        &self,
        url: &str,
        output_path: P,
        progress_tracker: Option<Arc<ProgressTracker>>,
    ) -> Result<DownloadResult> {
        let output_path = output_path.as_ref();

        // Check if file already exists and get its size
        let existing_size = if output_path.exists() {
            tokio::fs::metadata(output_path)
                .await
                .map(|m| m.len())
                .unwrap_or(0)
        } else {
            0
        };

        // Get file info from server
        let file_info = self.get_file_info(url).await?;

        // Check if file is already complete
        if existing_size == file_info.total_size {
            info!(
                "File already exists and is complete: {}",
                output_path.display()
            );
            return Ok(DownloadResult {
                path: output_path.to_path_buf(),
                size: existing_size,
                duration: Duration::from_secs(0),
                speed: 0.0,
                url: url.to_string(),
                resumed: false,
            });
        }

        // Determine download strategy
        if file_info.supports_ranges && file_info.total_size > self.min_chunk_size * 2 {
            // Use concurrent chunked download
            self.download_with_chunks(
                url,
                output_path,
                &file_info,
                existing_size,
                progress_tracker,
            )
            .await
        } else {
            // Use single-threaded download
            self.download_single_thread(url, output_path, existing_size, progress_tracker)
                .await
        }
    }

    /// Get file information from server
    async fn get_file_info(&self, url: &str) -> Result<FileInfo> {
        debug!("Getting file info for: {}", url);

        let response = self
            .http_client
            .head(url)
            .send()
            .await
            .map_err(|e| TurboCdnError::network(format!("Failed to get file info: {e}")))?;

        let status = response.status();
        if !status.is_success() {
            let status_code = status.as_u16();
            return Err(TurboCdnError::from_status_code(status_code, url));
        }

        let total_size = response
            .headers()
            .get("content-length")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(0);

        let supports_ranges = response
            .headers()
            .get("accept-ranges")
            .map(|v| v.to_str().unwrap_or("").contains("bytes"))
            .unwrap_or(false);

        debug!(
            "File info: size={}, supports_ranges={}",
            total_size, supports_ranges
        );

        Ok(FileInfo {
            total_size,
            supports_ranges,
        })
    }

    /// Download with concurrent chunks
    async fn download_with_chunks<P: AsRef<Path>>(
        &self,
        url: &str,
        output_path: P,
        file_info: &FileInfo,
        existing_size: u64,
        progress_tracker: Option<Arc<ProgressTracker>>,
    ) -> Result<DownloadResult> {
        let output_path = output_path.as_ref();
        let remaining_size = file_info.total_size - existing_size;

        info!(
            "Starting chunked download: {} bytes remaining, {} chunks",
            remaining_size, self.max_concurrent_chunks
        );

        // Calculate chunks
        let chunks = self.calculate_chunks(existing_size, file_info.total_size);
        debug!("Created {} chunks", chunks.len());

        // Create or open output file
        let file = if existing_size > 0 {
            OpenOptions::new()
                .write(true)
                .open(output_path)
                .await
                .map_err(|e| TurboCdnError::io(format!("Failed to open file: {e}")))?
        } else {
            File::create(output_path)
                .await
                .map_err(|e| TurboCdnError::io(format!("Failed to create file: {e}")))?
        };

        let file = Arc::new(tokio::sync::Mutex::new(file));

        // Limit concurrent downloads
        let semaphore = Arc::new(Semaphore::new(self.max_concurrent_chunks));

        // Download chunks concurrently
        let mut tasks = Vec::new();
        for chunk in chunks {
            let client = self.http_client.clone();
            let url = url.to_string();
            let file = file.clone();
            let semaphore = semaphore.clone();
            let progress_tracker = progress_tracker.clone();

            let task = tokio::spawn(async move {
                let _permit = semaphore.acquire().await.unwrap();
                Self::download_chunk(client, &url, chunk, file, progress_tracker).await
            });

            tasks.push(task);
        }

        // Wait for all chunks to complete
        for task in tasks {
            task.await
                .map_err(|e| TurboCdnError::network(format!("Chunk download failed: {e}")))?
                .map_err(|e| TurboCdnError::network(format!("Chunk processing failed: {e}")))?;
        }

        Ok(DownloadResult {
            path: output_path.to_path_buf(),
            size: file_info.total_size,
            duration: Duration::from_secs(0), // Will be set by caller
            speed: 0.0,                       // Will be set by caller
            url: url.to_string(),
            resumed: existing_size > 0,
        })
    }

    /// Calculate optimal chunks for download with adaptive sizing
    fn calculate_chunks(&self, start_offset: u64, total_size: u64) -> Vec<ChunkInfo> {
        self.calculate_adaptive_chunks(start_offset, total_size, None)
    }

    /// Calculate adaptive chunks based on current download speed
    fn calculate_adaptive_chunks(
        &self,
        start_offset: u64,
        total_size: u64,
        current_speed: Option<u64>,
    ) -> Vec<ChunkInfo> {
        let remaining_size = total_size - start_offset;

        // Determine optimal chunk size based on speed and file size
        let optimal_chunk_size = if self.adaptive_chunking_enabled {
            self.calculate_optimal_chunk_size(remaining_size, current_speed)
        } else {
            self.initial_chunk_size
        };

        // Ensure chunk size is within bounds
        let chunk_size = optimal_chunk_size
            .max(self.min_chunk_size)
            .min(self.max_chunk_size);

        // Calculate number of chunks, but don't exceed max_concurrent_chunks
        let ideal_chunk_count = (remaining_size / chunk_size).max(1);
        let actual_chunk_count = ideal_chunk_count.min(self.max_concurrent_chunks as u64);
        let adjusted_chunk_size = remaining_size / actual_chunk_count;

        let mut chunks = Vec::new();
        let mut current_start = start_offset;
        let mut index = 0;

        while current_start < total_size && index < actual_chunk_count {
            let chunk_end = if index == actual_chunk_count - 1 {
                // Last chunk gets any remaining bytes
                total_size - 1
            } else {
                (current_start + adjusted_chunk_size - 1).min(total_size - 1)
            };

            chunks.push(ChunkInfo {
                start: current_start,
                end: chunk_end,
                index: index as usize,
            });

            current_start = chunk_end + 1;
            index += 1;
        }

        debug!(
            "Calculated {} chunks with size ~{} KB each (adaptive: {})",
            chunks.len(),
            adjusted_chunk_size / 1024,
            self.adaptive_chunking_enabled
        );

        chunks
    }

    /// Calculate optimal chunk size based on file size and current speed
    fn calculate_optimal_chunk_size(&self, file_size: u64, current_speed: Option<u64>) -> u64 {
        // 更激进的分块策略 - 优先考虑并发度而不是块大小
        let size_based_chunk = if file_size > 100 * 1024 * 1024 {
            // Large files (>100MB): use smaller chunks for maximum parallelism
            self.initial_chunk_size / 2
        } else if file_size > 10 * 1024 * 1024 {
            // Medium files (10-100MB): use smaller chunks
            self.initial_chunk_size / 2
        } else if file_size > 1024 * 1024 {
            // Small files (1-10MB): use even smaller chunks for turbo speed
            self.initial_chunk_size / 4
        } else {
            // Very small files (<1MB): use minimum chunk size for maximum concurrency
            self.min_chunk_size
        };

        // Adjust based on current download speed if available
        if let Some(speed) = current_speed {
            if speed > self.speed_threshold_bytes_per_sec * 4 {
                // Very fast connection: still use smaller chunks for better parallelism
                size_based_chunk
            } else if speed > self.speed_threshold_bytes_per_sec * 2 {
                // Fast connection: use smaller chunks
                size_based_chunk
            } else if speed < self.speed_threshold_bytes_per_sec / 4 {
                // Very slow connection: use much smaller chunks for maximum parallelism
                size_based_chunk / 2
            } else if speed < self.speed_threshold_bytes_per_sec / 2 {
                // Slow connection: use smaller chunks
                size_based_chunk / 2
            } else {
                // Normal speed: use size-based chunk
                size_based_chunk
            }
        } else {
            // Default to smaller chunks for turbo speed
            size_based_chunk
        }
    }

    /// Download a single chunk
    async fn download_chunk(
        client: Client,
        url: &str,
        chunk: ChunkInfo,
        file: Arc<tokio::sync::Mutex<File>>,
        _progress_tracker: Option<Arc<ProgressTracker>>,
    ) -> Result<()> {
        debug!(
            "Downloading chunk {}: bytes {}-{}",
            chunk.index, chunk.start, chunk.end
        );

        let range_header = format!("bytes={}-{}", chunk.start, chunk.end);
        let response = client
            .get(url)
            .header("Range", range_header)
            .send()
            .await
            .map_err(|e| TurboCdnError::network(format!("Failed to download chunk: {e}")))?;

        let status = response.status();
        if !status.is_success() && status.as_u16() != 206 {
            let status_code = status.as_u16();
            return Err(TurboCdnError::from_status_code(status_code, url));
        }

        let bytes = response
            .bytes()
            .await
            .map_err(|e| TurboCdnError::network(format!("Failed to read chunk data: {e}")))?;

        // Write chunk to file
        let mut file_guard = file.lock().await;
        file_guard
            .seek(SeekFrom::Start(chunk.start))
            .await
            .map_err(|e| TurboCdnError::io(format!("Failed to seek in file: {e}")))?;

        file_guard
            .write_all(&bytes)
            .await
            .map_err(|e| TurboCdnError::io(format!("Failed to write chunk: {e}")))?;

        debug!("Completed chunk {}: {} bytes", chunk.index, bytes.len());
        Ok(())
    }

    /// Download with single thread (fallback)
    async fn download_single_thread<P: AsRef<Path>>(
        &self,
        url: &str,
        output_path: P,
        existing_size: u64,
        _progress_tracker: Option<Arc<ProgressTracker>>,
    ) -> Result<DownloadResult> {
        info!("Starting single-threaded download");

        let mut request = self.http_client.get(url);

        // Add range header for resume if file exists
        if existing_size > 0 {
            request = request.header("Range", format!("bytes={existing_size}-"));
        }

        let response = request
            .send()
            .await
            .map_err(|e| TurboCdnError::network(format!("Failed to start download: {e}")))?;

        let status = response.status();
        if !status.is_success() && status.as_u16() != 206 {
            let status_code = status.as_u16();
            return Err(TurboCdnError::from_status_code(status_code, url));
        }

        // Open or create file
        let mut file = if existing_size > 0 {
            OpenOptions::new()
                .append(true)
                .open(output_path.as_ref())
                .await
                .map_err(|e| TurboCdnError::io(format!("Failed to open file: {e}")))?
        } else {
            File::create(output_path.as_ref())
                .await
                .map_err(|e| TurboCdnError::io(format!("Failed to create file: {e}")))?
        };

        // Stream download
        let mut stream = response.bytes_stream();
        let mut downloaded_bytes = existing_size;

        use futures_util::StreamExt;
        while let Some(chunk) = stream.next().await {
            let chunk =
                chunk.map_err(|e| TurboCdnError::network(format!("Failed to read chunk: {e}")))?;

            file.write_all(&chunk)
                .await
                .map_err(|e| TurboCdnError::io(format!("Failed to write chunk: {e}")))?;

            downloaded_bytes += chunk.len() as u64;
        }

        file.flush()
            .await
            .map_err(|e| TurboCdnError::io(format!("Failed to flush file: {e}")))?;

        Ok(DownloadResult {
            path: output_path.as_ref().to_path_buf(),
            size: downloaded_bytes,
            duration: Duration::from_secs(0), // Will be set by caller
            speed: 0.0,                       // Will be set by caller
            url: url.to_string(),
            resumed: existing_size > 0,
        })
    }

    /// Get server performance statistics
    pub fn get_server_stats(&self) -> crate::server_tracker::PerformanceSummary {
        let tracker = self.server_performance_tracker.lock().unwrap();
        tracker.get_performance_summary()
    }

    /// Get detailed stats for a specific server
    pub fn get_server_detail(&self, url: &str) -> Option<crate::server_tracker::ServerStats> {
        let tracker = self.server_performance_tracker.lock().unwrap();
        tracker.get_stats(url).cloned()
    }
}

/// File information from server
#[derive(Debug, Clone)]
struct FileInfo {
    total_size: u64,
    supports_ranges: bool,
}

impl Default for ConcurrentDownloader {
    fn default() -> Self {
        Self::new().expect("Failed to create default ConcurrentDownloader")
    }
}
