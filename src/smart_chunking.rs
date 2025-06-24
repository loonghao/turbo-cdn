// Licensed under the MIT License
// Copyright (c) 2025 Hal <hal.long@outlook.com>

//! Smart Chunking Algorithm
//!
//! This module implements an intelligent chunking system that adapts chunk sizes
//! based on file size, network conditions, and server capabilities.

use crate::config::TurboCdnConfig;
use std::time::Duration;
use tracing::{debug, info};

/// Chunk strategy based on file characteristics
#[derive(Debug, Clone, PartialEq)]
pub enum ChunkStrategy {
    /// Single chunk for small files
    Single,
    /// Fixed size chunks for medium files
    Fixed { chunk_size: u64 },
    /// Dynamic chunks that adapt to network conditions
    Dynamic { base_size: u64, max_chunks: u32 },
    /// Aggressive chunking for large files
    Aggressive { chunk_size: u64, max_chunks: u32 },
}

/// Chunk performance metrics
#[derive(Debug, Clone)]
pub struct ChunkMetrics {
    /// Chunk index
    pub index: u32,
    /// Chunk size in bytes
    pub size: u64,
    /// Download duration
    pub duration: Duration,
    /// Number of retry attempts
    pub retry_count: u32,
    /// Success status
    pub success: bool,
    /// Error message if failed
    pub error: Option<String>,
}

/// Smart chunking calculator
#[derive(Debug)]
pub struct SmartChunking {
    #[allow(dead_code)]
    config: TurboCdnConfig,
    /// Minimum chunk size
    min_chunk_size: u64,
    /// Maximum chunk size
    max_chunk_size: u64,
    /// Optimal chunk size based on recent performance
    optimal_chunk_size: u64,
    /// Recent chunk performance history
    performance_history: Vec<ChunkMetrics>,
    /// Maximum history entries to keep
    max_history: usize,
}

impl SmartChunking {
    /// Create a new smart chunking calculator
    pub fn new(config: TurboCdnConfig) -> Self {
        let min_chunk_size = config.performance.min_chunk_size;
        let max_chunk_size = config.performance.max_chunk_size;
        let optimal_chunk_size = config.performance.chunk_size;

        Self {
            config,
            min_chunk_size,
            max_chunk_size,
            optimal_chunk_size,
            performance_history: Vec::new(),
            max_history: 100,
        }
    }

    /// Calculate optimal chunking strategy for a file
    pub fn calculate_strategy(
        &self,
        file_size: u64,
        server_supports_ranges: bool,
    ) -> ChunkStrategy {
        if !server_supports_ranges {
            info!("Server doesn't support range requests, using single chunk");
            return ChunkStrategy::Single;
        }

        // File size thresholds
        let small_file_threshold = 1024 * 1024; // 1MB
        let medium_file_threshold = 50 * 1024 * 1024; // 50MB
        let large_file_threshold = 500 * 1024 * 1024; // 500MB

        match file_size {
            // Small files: single chunk or minimal chunking
            size if size <= small_file_threshold => {
                if size <= self.min_chunk_size {
                    ChunkStrategy::Single
                } else {
                    ChunkStrategy::Fixed {
                        chunk_size: self.min_chunk_size,
                    }
                }
            }

            // Medium files: fixed chunking
            size if size <= medium_file_threshold => {
                let chunk_size = self.calculate_optimal_chunk_size(size);
                ChunkStrategy::Fixed { chunk_size }
            }

            // Large files: dynamic chunking
            size if size <= large_file_threshold => {
                let base_size = self.calculate_optimal_chunk_size(size);
                let max_chunks = self.calculate_max_chunks(size, base_size);
                ChunkStrategy::Dynamic {
                    base_size,
                    max_chunks,
                }
            }

            // Very large files: aggressive chunking
            size => {
                let chunk_size = self.max_chunk_size;
                let max_chunks = self.calculate_max_chunks(size, chunk_size);
                ChunkStrategy::Aggressive {
                    chunk_size,
                    max_chunks,
                }
            }
        }
    }

    /// Calculate optimal chunk size based on file size and performance history
    fn calculate_optimal_chunk_size(&self, file_size: u64) -> u64 {
        // Start with configured chunk size
        let mut chunk_size = self.optimal_chunk_size;

        // Adjust based on file size
        if file_size < 10 * 1024 * 1024 {
            // < 10MB
            chunk_size = chunk_size.min(file_size / 4);
        } else if file_size > 100 * 1024 * 1024 {
            // > 100MB
            chunk_size = chunk_size.max(5 * 1024 * 1024); // At least 5MB chunks
        }

        // Adjust based on recent performance
        if let Some(avg_performance) = self.calculate_average_performance() {
            if avg_performance.success_rate < 0.8 {
                // High failure rate: use smaller chunks
                chunk_size = (chunk_size / 2).max(self.min_chunk_size);
            } else if avg_performance.success_rate > 0.95
                && avg_performance.avg_speed > 5.0 * 1024.0 * 1024.0
            {
                // High success rate and good speed: use larger chunks
                chunk_size = (chunk_size * 2).min(self.max_chunk_size);
            }
        }

        // Ensure chunk size is within bounds
        chunk_size.clamp(self.min_chunk_size, self.max_chunk_size)
    }

    /// Calculate maximum number of chunks
    fn calculate_max_chunks(&self, file_size: u64, chunk_size: u64) -> u32 {
        let calculated_chunks = file_size.div_ceil(chunk_size);
        let max_concurrent = self.config.performance.max_concurrent_downloads as u64;

        // Limit chunks to reasonable number
        calculated_chunks.min(max_concurrent * 2).max(1) as u32
    }

    /// Generate chunk ranges for a file
    pub fn generate_chunks(&self, file_size: u64, strategy: &ChunkStrategy) -> Vec<ChunkRange> {
        match strategy {
            ChunkStrategy::Single => {
                vec![ChunkRange {
                    index: 0,
                    start: 0,
                    end: file_size - 1,
                    size: file_size,
                }]
            }

            ChunkStrategy::Fixed { chunk_size } => {
                self.generate_fixed_chunks(file_size, *chunk_size)
            }

            ChunkStrategy::Dynamic {
                base_size,
                max_chunks,
            } => self.generate_dynamic_chunks(file_size, *base_size, *max_chunks),

            ChunkStrategy::Aggressive {
                chunk_size,
                max_chunks,
            } => self.generate_aggressive_chunks(file_size, *chunk_size, *max_chunks),
        }
    }

    /// Generate fixed-size chunks
    fn generate_fixed_chunks(&self, file_size: u64, chunk_size: u64) -> Vec<ChunkRange> {
        let mut chunks = Vec::new();
        let mut start = 0;
        let mut index = 0;

        while start < file_size {
            let end = (start + chunk_size - 1).min(file_size - 1);
            let size = end - start + 1;

            chunks.push(ChunkRange {
                index,
                start,
                end,
                size,
            });

            start = end + 1;
            index += 1;
        }

        debug!(
            "Generated {} fixed chunks of ~{}KB each",
            chunks.len(),
            chunk_size / 1024
        );
        chunks
    }

    /// Generate dynamic chunks that adapt to performance
    fn generate_dynamic_chunks(
        &self,
        file_size: u64,
        base_size: u64,
        max_chunks: u32,
    ) -> Vec<ChunkRange> {
        let optimal_chunks = file_size.div_ceil(base_size).min(max_chunks as u64) as u32;

        let actual_chunk_size = file_size / optimal_chunks as u64;
        self.generate_fixed_chunks(file_size, actual_chunk_size)
    }

    /// Generate aggressive chunks for large files
    fn generate_aggressive_chunks(
        &self,
        file_size: u64,
        chunk_size: u64,
        max_chunks: u32,
    ) -> Vec<ChunkRange> {
        // Use smaller chunks initially, then larger chunks
        let mut chunks = Vec::new();
        let mut start = 0;
        let mut index = 0;
        let initial_small_chunks = 8_u32.min(max_chunks / 2);

        // First few chunks are smaller for quick start
        let small_chunk_size = chunk_size / 2;
        for _ in 0..initial_small_chunks {
            if start >= file_size {
                break;
            }

            let end = (start + small_chunk_size - 1).min(file_size - 1);
            let size = end - start + 1;

            chunks.push(ChunkRange {
                index,
                start,
                end,
                size,
            });

            start = end + 1;
            index += 1;
        }

        // Remaining chunks are larger
        while start < file_size && index < max_chunks {
            let end = (start + chunk_size - 1).min(file_size - 1);
            let size = end - start + 1;

            chunks.push(ChunkRange {
                index,
                start,
                end,
                size,
            });

            start = end + 1;
            index += 1;
        }

        debug!(
            "Generated {} aggressive chunks ({} small + {} large)",
            chunks.len(),
            initial_small_chunks,
            chunks.len().saturating_sub(initial_small_chunks as usize)
        );
        chunks
    }

    /// Record chunk performance for future optimization
    pub fn record_chunk_performance(&mut self, metrics: ChunkMetrics) {
        self.performance_history.push(metrics);

        // Keep only recent history
        if self.performance_history.len() > self.max_history {
            self.performance_history.remove(0);
        }

        // Update optimal chunk size based on performance
        self.update_optimal_chunk_size();
    }

    /// Update optimal chunk size based on performance history
    fn update_optimal_chunk_size(&mut self) {
        if let Some(performance) = self.calculate_average_performance() {
            if performance.success_rate > 0.9 && performance.avg_speed > 2.0 * 1024.0 * 1024.0 {
                // Good performance: try larger chunks
                self.optimal_chunk_size = (self.optimal_chunk_size + 512 * 1024) // Increase by 512KB
                    .min(self.max_chunk_size);
            } else if performance.success_rate < 0.7 {
                // Poor performance: use smaller chunks
                self.optimal_chunk_size = (self.optimal_chunk_size - 256 * 1024) // Decrease by 256KB
                    .max(self.min_chunk_size);
            }
        }
    }

    /// Calculate average performance from history
    fn calculate_average_performance(&self) -> Option<PerformanceSummary> {
        if self.performance_history.is_empty() {
            return None;
        }

        let total_chunks = self.performance_history.len();
        let successful_chunks = self
            .performance_history
            .iter()
            .filter(|m| m.success)
            .count();

        let avg_speed = self
            .performance_history
            .iter()
            .filter(|m| m.success && m.duration.as_secs_f64() > 0.0)
            .map(|m| m.size as f64 / m.duration.as_secs_f64())
            .sum::<f64>()
            / successful_chunks.max(1) as f64;

        Some(PerformanceSummary {
            success_rate: successful_chunks as f64 / total_chunks as f64,
            avg_speed,
            total_chunks,
            successful_chunks,
        })
    }

    /// Get current performance statistics
    pub fn get_performance_stats(&self) -> Option<PerformanceSummary> {
        self.calculate_average_performance()
    }
}

/// Chunk range definition
#[derive(Debug, Clone)]
pub struct ChunkRange {
    /// Chunk index
    pub index: u32,
    /// Start byte position (inclusive)
    pub start: u64,
    /// End byte position (inclusive)
    pub end: u64,
    /// Chunk size in bytes
    pub size: u64,
}

impl ChunkRange {
    /// Get HTTP Range header value
    pub fn to_range_header(&self) -> String {
        format!("bytes={}-{}", self.start, self.end)
    }
}

/// Performance summary
#[derive(Debug, Clone)]
pub struct PerformanceSummary {
    pub success_rate: f64,
    pub avg_speed: f64, // bytes per second
    pub total_chunks: usize,
    pub successful_chunks: usize,
}

impl std::fmt::Display for PerformanceSummary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Success: {:.1}% ({}/{}) | Avg Speed: {:.1} MB/s",
            self.success_rate * 100.0,
            self.successful_chunks,
            self.total_chunks,
            self.avg_speed / (1024.0 * 1024.0)
        )
    }
}
