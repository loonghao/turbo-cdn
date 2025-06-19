// Licensed under the MIT License
// Copyright (c) 2025 Hal <hal.long@outlook.com>

use indicatif::{ProgressBar, ProgressStyle};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::debug;

/// Progress tracker for downloads
#[derive(Debug)]
pub struct ProgressTracker {
    inner: Arc<RwLock<ProgressTrackerInner>>,
}

struct ProgressTrackerInner {
    progress_bar: Option<ProgressBar>,
    start_time: Instant,
    total_size: u64,
    downloaded_size: u64,
    chunks: Vec<ChunkProgress>,
    speed_samples: Vec<SpeedSample>,
    callback: Option<Box<dyn Fn(ProgressInfo) + Send + Sync>>,
}

impl std::fmt::Debug for ProgressTrackerInner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProgressTrackerInner")
            .field("progress_bar", &self.progress_bar)
            .field("start_time", &self.start_time)
            .field("total_size", &self.total_size)
            .field("downloaded_size", &self.downloaded_size)
            .field("chunks", &self.chunks)
            .field("speed_samples", &self.speed_samples)
            .field("callback", &"<callback>")
            .finish()
    }
}

/// Progress information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressInfo {
    /// Total file size in bytes
    pub total_size: u64,

    /// Downloaded size in bytes
    pub downloaded_size: u64,

    /// Download percentage (0.0 to 100.0)
    pub percentage: f64,

    /// Download speed in bytes per second
    pub speed: f64,

    /// Estimated time remaining
    pub eta: Option<Duration>,

    /// Elapsed time since download started
    pub elapsed: Duration,

    /// Number of active chunks
    pub active_chunks: usize,

    /// Whether the download is complete
    pub complete: bool,
}

/// Progress information for a single chunk
#[derive(Debug, Clone)]
pub struct ChunkProgress {
    pub chunk_id: usize,
    pub start_byte: u64,
    pub end_byte: u64,
    pub downloaded: u64,
    pub active: bool,
}

/// Speed sample for calculating average speed
#[derive(Debug, Clone)]
struct SpeedSample {
    timestamp: Instant,
    bytes_downloaded: u64,
}

/// Progress callback type
pub type ProgressCallback = Box<dyn Fn(ProgressInfo) + Send + Sync>;

impl ProgressTracker {
    /// Create a new progress tracker
    pub fn new(total_size: u64) -> Self {
        let inner = ProgressTrackerInner {
            progress_bar: None,
            start_time: Instant::now(),
            total_size,
            downloaded_size: 0,
            chunks: Vec::new(),
            speed_samples: Vec::new(),
            callback: None,
        };

        Self {
            inner: Arc::new(RwLock::new(inner)),
        }
    }

    /// Create a new progress tracker with visual progress bar
    pub fn with_progress_bar(total_size: u64) -> Self {
        let progress_bar = ProgressBar::new(total_size);
        progress_bar.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})")
                .unwrap()
                .progress_chars("#>-"),
        );

        let inner = ProgressTrackerInner {
            progress_bar: Some(progress_bar),
            start_time: Instant::now(),
            total_size,
            downloaded_size: 0,
            chunks: Vec::new(),
            speed_samples: Vec::new(),
            callback: None,
        };

        Self {
            inner: Arc::new(RwLock::new(inner)),
        }
    }

    /// Set a progress callback
    pub async fn set_callback<F>(&self, callback: F)
    where
        F: Fn(ProgressInfo) + Send + Sync + 'static,
    {
        let mut inner = self.inner.write().await;
        inner.callback = Some(Box::new(callback));
    }

    /// Initialize chunks for parallel downloading
    pub async fn init_chunks(&self, chunk_ranges: Vec<(u64, u64)>) {
        let mut inner = self.inner.write().await;
        inner.chunks = chunk_ranges
            .into_iter()
            .enumerate()
            .map(|(id, (start, end))| ChunkProgress {
                chunk_id: id,
                start_byte: start,
                end_byte: end,
                downloaded: 0,
                active: false,
            })
            .collect();
    }

    /// Update progress for a specific chunk
    pub async fn update_chunk(&self, chunk_id: usize, bytes_downloaded: u64) {
        let mut inner = self.inner.write().await;

        if let Some(chunk) = inner.chunks.get_mut(chunk_id) {
            let old_downloaded = chunk.downloaded;
            chunk.downloaded = bytes_downloaded;
            chunk.active = true;

            // Update total downloaded size
            let delta = bytes_downloaded.saturating_sub(old_downloaded);
            inner.downloaded_size += delta;

            // Update progress bar
            if let Some(ref pb) = inner.progress_bar {
                pb.set_position(inner.downloaded_size);
            }

            // Add speed sample
            let bytes_downloaded = inner.downloaded_size;
            inner.speed_samples.push(SpeedSample {
                timestamp: Instant::now(),
                bytes_downloaded,
            });

            // Keep only recent samples (last 10 seconds)
            let cutoff = Instant::now() - Duration::from_secs(10);
            inner
                .speed_samples
                .retain(|sample| sample.timestamp > cutoff);

            // Trigger callback
            if let Some(ref callback) = inner.callback {
                let progress_info = Self::calculate_progress_info(&inner);
                callback(progress_info);
            }
        }
    }

    /// Mark a chunk as complete
    pub async fn complete_chunk(&self, chunk_id: usize) {
        let mut inner = self.inner.write().await;

        if let Some(chunk) = inner.chunks.get_mut(chunk_id) {
            chunk.active = false;
            debug!(
                "Chunk {} completed: {}/{} bytes",
                chunk_id,
                chunk.downloaded,
                chunk.end_byte - chunk.start_byte + 1
            );
        }
    }

    /// Update total progress (for single-threaded downloads)
    pub async fn update(&self, bytes_downloaded: u64) {
        let mut inner = self.inner.write().await;
        inner.downloaded_size = bytes_downloaded;

        // Update progress bar
        if let Some(ref pb) = inner.progress_bar {
            pb.set_position(bytes_downloaded);
        }

        // Add speed sample
        inner.speed_samples.push(SpeedSample {
            timestamp: Instant::now(),
            bytes_downloaded,
        });

        // Keep only recent samples
        let cutoff = Instant::now() - Duration::from_secs(10);
        inner
            .speed_samples
            .retain(|sample| sample.timestamp > cutoff);

        // Trigger callback
        if let Some(ref callback) = inner.callback {
            let progress_info = Self::calculate_progress_info(&inner);
            callback(progress_info);
        }
    }

    /// Mark download as complete
    pub async fn complete(&self) {
        let mut inner = self.inner.write().await;
        inner.downloaded_size = inner.total_size;

        if let Some(ref pb) = inner.progress_bar {
            pb.finish_with_message("Download completed");
        }

        // Trigger final callback
        if let Some(ref callback) = inner.callback {
            let mut progress_info = Self::calculate_progress_info(&inner);
            progress_info.complete = true;
            callback(progress_info);
        }
    }

    /// Get current progress information
    pub async fn get_progress(&self) -> ProgressInfo {
        let inner = self.inner.read().await;
        Self::calculate_progress_info(&inner)
    }

    /// Abort the download and clean up
    pub async fn abort(&self) {
        let inner = self.inner.read().await;
        if let Some(ref pb) = inner.progress_bar {
            pb.abandon_with_message("Download aborted");
        }
    }

    // Private helper methods

    fn calculate_progress_info(inner: &ProgressTrackerInner) -> ProgressInfo {
        let percentage = if inner.total_size > 0 {
            (inner.downloaded_size as f64 / inner.total_size as f64) * 100.0
        } else {
            0.0
        };

        let elapsed = inner.start_time.elapsed();
        let speed = Self::calculate_speed(&inner.speed_samples);
        let eta = Self::calculate_eta(inner.total_size, inner.downloaded_size, speed);
        let active_chunks = inner.chunks.iter().filter(|c| c.active).count();

        ProgressInfo {
            total_size: inner.total_size,
            downloaded_size: inner.downloaded_size,
            percentage,
            speed,
            eta,
            elapsed,
            active_chunks,
            complete: false,
        }
    }

    fn calculate_speed(samples: &[SpeedSample]) -> f64 {
        if samples.len() < 2 {
            return 0.0;
        }

        let first = &samples[0];
        let last = &samples[samples.len() - 1];

        let time_diff = last.timestamp.duration_since(first.timestamp).as_secs_f64();
        let bytes_diff = last.bytes_downloaded.saturating_sub(first.bytes_downloaded);

        if time_diff > 0.0 {
            bytes_diff as f64 / time_diff
        } else {
            0.0
        }
    }

    fn calculate_eta(total_size: u64, downloaded_size: u64, speed: f64) -> Option<Duration> {
        if speed > 0.0 && downloaded_size < total_size {
            let remaining_bytes = total_size - downloaded_size;
            let eta_seconds = remaining_bytes as f64 / speed;
            Some(Duration::from_secs_f64(eta_seconds))
        } else {
            None
        }
    }
}

impl ProgressInfo {
    /// Get percentage as a value between 0.0 and 1.0
    pub fn percentage_normalized(&self) -> f64 {
        self.percentage / 100.0
    }

    /// Get speed in MB/s
    pub fn speed_mbps(&self) -> f64 {
        self.speed / 1_000_000.0
    }

    /// Get a human-readable speed string
    pub fn speed_human(&self) -> String {
        if self.speed >= 1_000_000_000.0 {
            format!("{:.2} GB/s", self.speed / 1_000_000_000.0)
        } else if self.speed >= 1_000_000.0 {
            format!("{:.2} MB/s", self.speed / 1_000_000.0)
        } else if self.speed >= 1_000.0 {
            format!("{:.2} KB/s", self.speed / 1_000.0)
        } else {
            format!("{:.0} B/s", self.speed)
        }
    }

    /// Get a human-readable ETA string
    pub fn eta_human(&self) -> String {
        match self.eta {
            Some(eta) => {
                let total_seconds = eta.as_secs();
                let hours = total_seconds / 3600;
                let minutes = (total_seconds % 3600) / 60;
                let seconds = total_seconds % 60;

                if hours > 0 {
                    format!("{}h {}m {}s", hours, minutes, seconds)
                } else if minutes > 0 {
                    format!("{}m {}s", minutes, seconds)
                } else {
                    format!("{}s", seconds)
                }
            }
            None => "Unknown".to_string(),
        }
    }

    /// Get a human-readable size string
    pub fn size_human(&self) -> String {
        Self::format_bytes(self.downloaded_size, self.total_size)
    }

    fn format_bytes(downloaded: u64, total: u64) -> String {
        let format_size = |size: u64| -> String {
            if size >= 1_000_000_000 {
                format!("{:.2} GB", size as f64 / 1_000_000_000.0)
            } else if size >= 1_000_000 {
                format!("{:.2} MB", size as f64 / 1_000_000.0)
            } else if size >= 1_000 {
                format!("{:.2} KB", size as f64 / 1_000.0)
            } else {
                format!("{} B", size)
            }
        };

        format!("{} / {}", format_size(downloaded), format_size(total))
    }
}

/// Simple progress reporter that prints to console
pub struct ConsoleProgressReporter;

impl ConsoleProgressReporter {
    pub fn default_callback() -> ProgressCallback {
        Box::new(|progress: ProgressInfo| {
            println!(
                "Progress: {:.1}% ({}) - {} - ETA: {}",
                progress.percentage,
                progress.size_human(),
                progress.speed_human(),
                progress.eta_human()
            );
        })
    }
}

impl Default for ConsoleProgressReporter {
    fn default() -> Self {
        let _tracker = Self::default_callback();
        Self
    }
}
