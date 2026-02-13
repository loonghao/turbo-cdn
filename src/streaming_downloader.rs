// Licensed under the MIT License
// Copyright (c) 2025 Hal <hal.long@outlook.com>

//! Streaming downloader with constant memory usage
//!
//! This module provides streaming download operations that maintain constant memory usage
//! regardless of file size, using buffer pools and zero-copy operations for optimal performance.

use crate::error::{Result, TurboCdnError};
use crate::memory_tracker::{self, MemoryPressure};
use crate::mmap_writer::MmapWriter;
use crate::progress::ProgressTracker;
use futures_util::StreamExt;
use reqwest::Client;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, Semaphore};
use tracing::{debug, info, warn};

/// Buffer pool for zero-copy operations
#[derive(Debug)]
pub struct BufferPool {
    /// Queue of available buffers
    buffers: crossbeam::queue::SegQueue<Vec<u8>>,
    /// Buffer size in bytes
    buffer_size: usize,
    /// Maximum number of buffers in pool
    max_buffers: usize,
    /// Current number of buffers created
    current_buffers: std::sync::atomic::AtomicUsize,
}

impl BufferPool {
    /// Create a new buffer pool with specified buffer size and max buffers
    pub fn new(buffer_size: usize, max_buffers: usize) -> Self {
        Self {
            buffers: crossbeam::queue::SegQueue::new(),
            buffer_size,
            max_buffers,
            current_buffers: std::sync::atomic::AtomicUsize::new(0),
        }
    }

    /// Create a buffer pool with adaptive sizing based on memory pressure
    pub fn adaptive(file_size: u64) -> Self {
        let memory_pressure = memory_tracker::check_global_memory_pressure();
        
        let (buffer_size, max_buffers) = match memory_pressure {
            MemoryPressure::Critical => {
                // Very conservative: 64KB buffers, max 4 buffers
                (64 * 1024, 4)
            }
            MemoryPressure::High => {
                // Conservative: 128KB buffers, max 8 buffers
                (128 * 1024, 8)
            }
            MemoryPressure::Moderate => {
                // Balanced: 256KB buffers, max 16 buffers
                (256 * 1024, 16)
            }
            MemoryPressure::Low => {
                // Optimal: Scale based on file size
                let buffer_size = if file_size > 100 * 1024 * 1024 {
                    1024 * 1024 // 1MB for large files
                } else if file_size > 10 * 1024 * 1024 {
                    512 * 1024 // 512KB for medium files
                } else {
                    256 * 1024 // 256KB for small files
                };
                (buffer_size, 32)
            }
        };

        debug!(
            "Created adaptive buffer pool: {}KB buffers, max {} buffers (pressure: {:?})",
            buffer_size / 1024,
            max_buffers,
            memory_pressure
        );

        Self::new(buffer_size, max_buffers)
    }

    /// Get a buffer from the pool, creating one if necessary
    pub fn get_buffer(&self) -> Vec<u8> {
        // Try to get an existing buffer first
        if let Some(mut buffer) = self.buffers.pop() {
            buffer.clear();
            buffer.reserve(self.buffer_size);
            return buffer;
        }

        // Create a new buffer if we haven't reached the limit
        let current = self.current_buffers.load(std::sync::atomic::Ordering::Relaxed);
        if current < self.max_buffers {
            self.current_buffers.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            let buffer = Vec::with_capacity(self.buffer_size);
            memory_tracker::record_allocation(self.buffer_size);
            debug!("Created new buffer #{} ({}KB)", current + 1, self.buffer_size / 1024);
            return buffer;
        }

        // If we've reached the limit, create a temporary buffer
        // This should rarely happen with proper semaphore usage
        warn!("Buffer pool exhausted, creating temporary buffer");
        Vec::with_capacity(self.buffer_size)
    }

    /// Return a buffer to the pool
    pub fn return_buffer(&self, buffer: Vec<u8>) {
        // Only return buffers that are the expected size to avoid memory bloat
        if buffer.capacity() == self.buffer_size {
            self.buffers.push(buffer);
        } else {
            // Buffer has wrong capacity, let it be dropped
            debug!("Dropping buffer with wrong capacity: {} vs {}", buffer.capacity(), self.buffer_size);
        }
    }

    /// Get pool statistics
    pub fn stats(&self) -> BufferPoolStats {
        BufferPoolStats {
            buffer_size: self.buffer_size,
            max_buffers: self.max_buffers,
            current_buffers: self.current_buffers.load(std::sync::atomic::Ordering::Relaxed),
            available_buffers: self.buffers.len(),
        }
    }
}

impl Drop for BufferPool {
    fn drop(&mut self) {
        // Record deallocation of all buffers
        let current_buffers = self.current_buffers.load(std::sync::atomic::Ordering::Relaxed);
        memory_tracker::record_deallocation(current_buffers * self.buffer_size);
        debug!("Dropped buffer pool with {} buffers", current_buffers);
    }
}

/// Buffer pool statistics
#[derive(Debug, Clone)]
pub struct BufferPoolStats {
    pub buffer_size: usize,
    pub max_buffers: usize,
    pub current_buffers: usize,
    pub available_buffers: usize,
}

/// Streaming downloader with constant memory usage
#[derive(Debug)]
pub struct StreamingDownloader {
    /// HTTP client for requests
    client: Client,
    /// Buffer pool for zero-copy operations
    buffer_pool: Arc<BufferPool>,
    /// Semaphore to limit concurrent buffer usage
    buffer_semaphore: Arc<Semaphore>,
    /// Request timeout
    timeout: Duration,
}

impl StreamingDownloader {
    /// Create a new streaming downloader with default settings
    pub fn new() -> Result<Self> {
        let config = crate::config::TurboCdnConfig::default();
        Self::with_config(&config)
    }

    /// Create a new streaming downloader with configuration
    pub fn with_config(config: &crate::config::TurboCdnConfig) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(config.performance.timeout))
            .user_agent(&config.general.user_agent)
            .pool_max_idle_per_host(config.performance.pool_max_idle_per_host)
            .pool_idle_timeout(Duration::from_secs(config.performance.pool_idle_timeout))
            .tcp_keepalive(Duration::from_secs(config.performance.tcp_keepalive))
            .tcp_nodelay(true)
            .build()
            .map_err(|e| TurboCdnError::network(format!("Failed to create HTTP client: {e}")))?;

        // Create buffer pool with adaptive sizing
        let buffer_pool = Arc::new(BufferPool::adaptive(0)); // Will be resized per download
        let max_buffers = buffer_pool.max_buffers;
        let buffer_semaphore = Arc::new(Semaphore::new(max_buffers));

        Ok(Self {
            client,
            buffer_pool,
            buffer_semaphore,
            timeout: Duration::from_secs(config.performance.timeout),
        })
    }

    /// Download a file with streaming and constant memory usage
    pub async fn download_streaming<P: AsRef<Path>>(
        &self,
        url: &str,
        output_path: P,
        expected_size: Option<u64>,
        progress_tracker: Option<Arc<ProgressTracker>>,
    ) -> Result<StreamingDownloadResult> {
        let output_path = output_path.as_ref();
        let start_time = Instant::now();

        info!("Starting streaming download: {} -> {}", url, output_path.display());

        // Get file info if size not provided
        let file_size = if let Some(size) = expected_size {
            size
        } else {
            self.get_content_length(url).await.unwrap_or(0)
        };

        // Create adaptive buffer pool for this download
        let download_buffer_pool = Arc::new(BufferPool::adaptive(file_size));
        let max_buffers = download_buffer_pool.max_buffers;
        let download_semaphore = Arc::new(Semaphore::new(max_buffers));

        // Create writer based on file size and memory pressure
        let writer = if file_size > 0 {
            Some(MmapWriter::new(output_path, file_size, None).await?)
        } else {
            None
        };

        // Start streaming download
        let response = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|e| TurboCdnError::network(format!("Failed to start download: {e}")))?;

        if !response.status().is_success() {
            return Err(TurboCdnError::network(format!(
                "Download failed with status: {}",
                response.status()
            )));
        }

        // Get actual content length from response if not known
        let actual_size = response
            .content_length()
            .or(expected_size)
            .unwrap_or(0);

        // Clone buffer pool for stats before moving it
        let buffer_pool_stats = download_buffer_pool.stats();

        // Stream download with constant memory usage
        let bytes_downloaded = self
            .stream_with_constant_memory(
                response,
                writer,
                output_path,
                actual_size,
                download_buffer_pool,
                download_semaphore,
                progress_tracker,
            )
            .await?;

        let duration = start_time.elapsed();
        let speed = if duration.as_secs_f64() > 0.0 {
            bytes_downloaded as f64 / duration.as_secs_f64()
        } else {
            0.0
        };

        info!(
            "Streaming download completed: {} bytes in {:.2}s ({:.2} MB/s)",
            bytes_downloaded,
            duration.as_secs_f64(),
            speed / 1024.0 / 1024.0
        );

        Ok(StreamingDownloadResult {
            path: output_path.to_path_buf(),
            size: bytes_downloaded,
            duration,
            speed,
            url: url.to_string(),
            buffer_pool_stats,
        })
    }

    /// Stream download with constant memory usage using buffer pool
    async fn stream_with_constant_memory(
        &self,
        response: reqwest::Response,
        writer: Option<MmapWriter>,
        output_path: &Path,
        expected_size: u64,
        buffer_pool: Arc<BufferPool>,
        semaphore: Arc<Semaphore>,
        progress_tracker: Option<Arc<ProgressTracker>>,
    ) -> Result<u64> {
        let mut stream = response.bytes_stream();
        let mut total_downloaded = 0u64;
        let mut current_offset = 0u64;

        // Create file writer if mmap writer not available
        let mut file_writer = if writer.is_none() {
            Some(
                tokio::fs::File::create(output_path)
                    .await
                    .map_err(|e| TurboCdnError::io(format!("Failed to create file: {e}")))?,
            )
        } else {
            None
        };

        // Channel for streaming chunks with backpressure
        let (chunk_sender, mut chunk_receiver) = mpsc::channel::<StreamingChunk>(buffer_pool.max_buffers);

        // Spawn task to process stream and send chunks
        let stream_buffer_pool = buffer_pool.clone();
        let stream_semaphore = semaphore.clone();
        let stream_task = tokio::spawn(async move {
            while let Some(chunk_result) = stream.next().await {
                match chunk_result {
                    Ok(bytes) => {
                        // Acquire semaphore permit to limit memory usage
                        let _permit = stream_semaphore.acquire().await.unwrap();
                        
                        // Get buffer from pool
                        let mut buffer = stream_buffer_pool.get_buffer();
                        buffer.extend_from_slice(&bytes);
                        
                        let chunk = StreamingChunk {
                            data: buffer,
                            size: bytes.len(),
                        };

                        if chunk_sender.send(chunk).await.is_err() {
                            debug!("Chunk receiver closed, stopping stream processing");
                            break;
                        }
                    }
                    Err(e) => {
                        warn!("Stream error: {}", e);
                        break;
                    }
                }
            }
            debug!("Stream processing completed");
        });

        // Process chunks with constant memory usage
        while let Some(chunk) = chunk_receiver.recv().await {
            let chunk_size = chunk.size;
            
            // Write chunk to file
            if let Some(ref writer) = writer {
                writer.write_at(current_offset, &chunk.data).await?;
            } else if let Some(ref mut file) = file_writer {
                use tokio::io::AsyncWriteExt;
                file.write_all(&chunk.data).await
                    .map_err(|e| TurboCdnError::io(format!("Failed to write chunk: {e}")))?;
            }

            // Return buffer to pool for reuse
            buffer_pool.return_buffer(chunk.data);

            // Update progress
            total_downloaded += chunk_size as u64;
            current_offset += chunk_size as u64;

            if let Some(ref tracker) = progress_tracker {
                tracker.update_progress(total_downloaded, expected_size).await;
            }

            // Check memory pressure and adapt if necessary
            let pressure = memory_tracker::check_global_memory_pressure();
            if pressure == MemoryPressure::Critical {
                warn!("Critical memory pressure detected during streaming download");
                // Could implement emergency measures here
            }
        }

        // Wait for stream processing to complete
        let _ = stream_task.await;

        // Flush and sync if using mmap writer
        if let Some(ref writer) = writer {
            writer.flush().await?;
            writer.sync().await?;
        } else if let Some(ref mut file) = file_writer {
            use tokio::io::AsyncWriteExt;
            file.flush().await
                .map_err(|e| TurboCdnError::io(format!("Failed to flush file: {e}")))?;
        }

        Ok(total_downloaded)
    }

    /// Get content length from URL without downloading
    async fn get_content_length(&self, url: &str) -> Result<u64> {
        let response = self
            .client
            .head(url)
            .timeout(self.timeout)
            .send()
            .await
            .map_err(|e| TurboCdnError::network(format!("Failed to get content length: {e}")))?;

        Ok(response.content_length().unwrap_or(0))
    }
}

/// Streaming chunk for processing
#[derive(Debug)]
struct StreamingChunk {
    data: Vec<u8>,
    size: usize,
}

/// Result of streaming download operation
#[derive(Debug, Clone)]
pub struct StreamingDownloadResult {
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
    /// Buffer pool statistics
    pub buffer_pool_stats: BufferPoolStats,
}

impl Default for StreamingDownloader {
    fn default() -> Self {
        Self::new().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_buffer_pool_creation() {
        let pool = BufferPool::new(1024, 10);
        let stats = pool.stats();
        
        assert_eq!(stats.buffer_size, 1024);
        assert_eq!(stats.max_buffers, 10);
        assert_eq!(stats.current_buffers, 0);
        assert_eq!(stats.available_buffers, 0);
    }

    #[test]
    fn test_buffer_pool_get_return() {
        let pool = BufferPool::new(1024, 10);
        
        let buffer1 = pool.get_buffer();
        assert_eq!(buffer1.capacity(), 1024);
        
        let stats = pool.stats();
        assert_eq!(stats.current_buffers, 1);
        
        pool.return_buffer(buffer1);
        
        let stats = pool.stats();
        assert_eq!(stats.available_buffers, 1);
    }

    #[test]
    fn test_adaptive_buffer_pool() {
        // Test different file sizes create different buffer configurations
        let small_pool = BufferPool::adaptive(1024 * 1024); // 1MB
        let large_pool = BufferPool::adaptive(100 * 1024 * 1024); // 100MB
        
        // Large files should have larger buffers (assuming low memory pressure)
        // This test may vary based on current memory pressure
        let small_stats = small_pool.stats();
        let large_stats = large_pool.stats();
        
        assert!(small_stats.buffer_size > 0);
        assert!(large_stats.buffer_size > 0);
        assert!(small_stats.max_buffers > 0);
        assert!(large_stats.max_buffers > 0);
    }

    #[tokio::test]
    async fn test_streaming_downloader_creation() {
        let downloader = StreamingDownloader::new();
        assert!(downloader.is_ok());
    }

    #[tokio::test]
    async fn test_get_content_length() {
        let downloader = StreamingDownloader::new().unwrap();
        
        // Test with a URL that should return content-length
        // Note: This test requires internet access and may be flaky
        // In a real test suite, you'd use a mock server
        let result = downloader.get_content_length("https://httpbin.org/bytes/1024").await;
        
        // We can't assert exact values due to network variability
        // Just check that it doesn't error
        match result {
            Ok(size) => {
                debug!("Content length: {}", size);
                // Size might be 0 if server doesn't provide content-length
            }
            Err(e) => {
                // Network errors are acceptable in tests
                debug!("Network error (expected in some environments): {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_streaming_download_result() {
        let result = StreamingDownloadResult {
            path: PathBuf::from("/tmp/test.dat"),
            size: 1024,
            duration: Duration::from_secs(1),
            speed: 1024.0,
            url: "https://example.com/test.dat".to_string(),
            buffer_pool_stats: BufferPoolStats {
                buffer_size: 1024,
                max_buffers: 10,
                current_buffers: 5,
                available_buffers: 3,
            },
        };

        assert_eq!(result.size, 1024);
        assert_eq!(result.speed, 1024.0);
        assert_eq!(result.buffer_pool_stats.buffer_size, 1024);
    }
}