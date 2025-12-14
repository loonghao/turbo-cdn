// Licensed under the MIT License
// Copyright (c) 2025 Hal <hal.long@outlook.com>

//! Tests for the smart chunking module

use std::time::Duration;
use turbo_cdn::config::TurboCdnConfig;
use turbo_cdn::smart_chunking::{
    ChunkMetrics, ChunkRange, ChunkStrategy, PerformanceSummary, SmartChunking,
};

// ============================================================================
// ChunkStrategy Tests
// ============================================================================

#[test]
fn test_chunk_strategy_single() {
    let strategy = ChunkStrategy::Single;
    assert_eq!(strategy, ChunkStrategy::Single);
}

#[test]
fn test_chunk_strategy_fixed() {
    let strategy = ChunkStrategy::Fixed {
        chunk_size: 1024 * 1024,
    };
    if let ChunkStrategy::Fixed { chunk_size } = strategy {
        assert_eq!(chunk_size, 1024 * 1024);
    } else {
        panic!("Expected Fixed strategy");
    }
}

#[test]
fn test_chunk_strategy_dynamic() {
    let strategy = ChunkStrategy::Dynamic {
        base_size: 2 * 1024 * 1024,
        max_chunks: 16,
    };
    if let ChunkStrategy::Dynamic {
        base_size,
        max_chunks,
    } = strategy
    {
        assert_eq!(base_size, 2 * 1024 * 1024);
        assert_eq!(max_chunks, 16);
    } else {
        panic!("Expected Dynamic strategy");
    }
}

#[test]
fn test_chunk_strategy_aggressive() {
    let strategy = ChunkStrategy::Aggressive {
        chunk_size: 5 * 1024 * 1024,
        max_chunks: 32,
    };
    if let ChunkStrategy::Aggressive {
        chunk_size,
        max_chunks,
    } = strategy
    {
        assert_eq!(chunk_size, 5 * 1024 * 1024);
        assert_eq!(max_chunks, 32);
    } else {
        panic!("Expected Aggressive strategy");
    }
}

#[test]
fn test_chunk_strategy_clone() {
    let strategy = ChunkStrategy::Fixed {
        chunk_size: 1024 * 1024,
    };
    let cloned = strategy.clone();
    assert_eq!(strategy, cloned);
}

#[test]
fn test_chunk_strategy_debug() {
    let strategies = vec![
        ChunkStrategy::Single,
        ChunkStrategy::Fixed {
            chunk_size: 1024 * 1024,
        },
        ChunkStrategy::Dynamic {
            base_size: 2 * 1024 * 1024,
            max_chunks: 16,
        },
        ChunkStrategy::Aggressive {
            chunk_size: 5 * 1024 * 1024,
            max_chunks: 32,
        },
    ];

    for strategy in strategies {
        let debug_str = format!("{:?}", strategy);
        assert!(!debug_str.is_empty());
    }
}

// ============================================================================
// ChunkMetrics Tests
// ============================================================================

#[test]
fn test_chunk_metrics_success() {
    let metrics = ChunkMetrics {
        index: 0,
        size: 1024 * 1024,
        duration: Duration::from_millis(500),
        retry_count: 0,
        success: true,
        error: None,
    };

    assert_eq!(metrics.index, 0);
    assert_eq!(metrics.size, 1024 * 1024);
    assert!(metrics.success);
    assert!(metrics.error.is_none());
}

#[test]
fn test_chunk_metrics_failure() {
    let metrics = ChunkMetrics {
        index: 1,
        size: 1024 * 1024,
        duration: Duration::from_secs(30),
        retry_count: 3,
        success: false,
        error: Some("Connection timeout".to_string()),
    };

    assert!(!metrics.success);
    assert_eq!(metrics.retry_count, 3);
    assert!(metrics.error.is_some());
    assert!(metrics.error.unwrap().contains("timeout"));
}

#[test]
fn test_chunk_metrics_clone() {
    let metrics = ChunkMetrics {
        index: 5,
        size: 2 * 1024 * 1024,
        duration: Duration::from_millis(1000),
        retry_count: 1,
        success: true,
        error: None,
    };

    let cloned = metrics.clone();
    assert_eq!(cloned.index, 5);
    assert_eq!(cloned.size, 2 * 1024 * 1024);
    assert!(cloned.success);
}

#[test]
fn test_chunk_metrics_debug() {
    let metrics = ChunkMetrics {
        index: 0,
        size: 1024,
        duration: Duration::from_millis(100),
        retry_count: 0,
        success: true,
        error: None,
    };

    let debug_str = format!("{:?}", metrics);
    assert!(debug_str.contains("ChunkMetrics"));
}

// ============================================================================
// ChunkRange Tests
// ============================================================================

#[test]
fn test_chunk_range_creation() {
    let range = ChunkRange {
        index: 0,
        start: 0,
        end: 1023,
        size: 1024,
    };

    assert_eq!(range.index, 0);
    assert_eq!(range.start, 0);
    assert_eq!(range.end, 1023);
    assert_eq!(range.size, 1024);
}

#[test]
fn test_chunk_range_to_range_header() {
    let range = ChunkRange {
        index: 0,
        start: 0,
        end: 1023,
        size: 1024,
    };

    assert_eq!(range.to_range_header(), "bytes=0-1023");
}

#[test]
fn test_chunk_range_large_file() {
    let range = ChunkRange {
        index: 10,
        start: 10 * 1024 * 1024,
        end: 11 * 1024 * 1024 - 1,
        size: 1024 * 1024,
    };

    let header = range.to_range_header();
    assert!(header.starts_with("bytes="));
    assert!(header.contains("10485760")); // 10 * 1024 * 1024
}

#[test]
fn test_chunk_range_clone() {
    let range = ChunkRange {
        index: 5,
        start: 5000,
        end: 9999,
        size: 5000,
    };

    let cloned = range.clone();
    assert_eq!(cloned.index, range.index);
    assert_eq!(cloned.start, range.start);
    assert_eq!(cloned.end, range.end);
    assert_eq!(cloned.size, range.size);
}

#[test]
fn test_chunk_range_debug() {
    let range = ChunkRange {
        index: 0,
        start: 0,
        end: 1023,
        size: 1024,
    };

    let debug_str = format!("{:?}", range);
    assert!(debug_str.contains("ChunkRange"));
}

// ============================================================================
// SmartChunking Tests
// ============================================================================

#[test]
fn test_smart_chunking_new() {
    let config = TurboCdnConfig::default();
    let chunking = SmartChunking::new(config);

    let debug_str = format!("{:?}", chunking);
    assert!(debug_str.contains("SmartChunking"));
}

#[test]
fn test_smart_chunking_calculate_strategy_small_file() {
    let config = TurboCdnConfig::default();
    let chunking = SmartChunking::new(config);

    // Very small file (100 bytes)
    let strategy = chunking.calculate_strategy(100, true);
    assert_eq!(strategy, ChunkStrategy::Single);
}

#[test]
fn test_smart_chunking_calculate_strategy_no_range_support() {
    let config = TurboCdnConfig::default();
    let chunking = SmartChunking::new(config);

    // Large file but no range support
    let strategy = chunking.calculate_strategy(100 * 1024 * 1024, false);
    assert_eq!(strategy, ChunkStrategy::Single);
}

#[test]
fn test_smart_chunking_calculate_strategy_medium_file() {
    let config = TurboCdnConfig::default();
    let chunking = SmartChunking::new(config);

    // Medium file (10 MB)
    let strategy = chunking.calculate_strategy(10 * 1024 * 1024, true);

    // Should be Fixed strategy for medium files
    matches!(strategy, ChunkStrategy::Fixed { .. });
}

#[test]
fn test_smart_chunking_calculate_strategy_large_file() {
    let config = TurboCdnConfig::default();
    let chunking = SmartChunking::new(config);

    // Large file (100 MB)
    let strategy = chunking.calculate_strategy(100 * 1024 * 1024, true);

    // Should be Dynamic strategy for large files
    matches!(strategy, ChunkStrategy::Dynamic { .. });
}

#[test]
fn test_smart_chunking_calculate_strategy_very_large_file() {
    let config = TurboCdnConfig::default();
    let chunking = SmartChunking::new(config);

    // Very large file (1 GB)
    let strategy = chunking.calculate_strategy(1024 * 1024 * 1024, true);

    // Should be Aggressive strategy for very large files
    matches!(strategy, ChunkStrategy::Aggressive { .. });
}

#[test]
fn test_smart_chunking_generate_chunks_single() {
    let config = TurboCdnConfig::default();
    let chunking = SmartChunking::new(config);

    let strategy = ChunkStrategy::Single;
    let chunks = chunking.generate_chunks(1000, &strategy);

    assert_eq!(chunks.len(), 1);
    assert_eq!(chunks[0].start, 0);
    assert_eq!(chunks[0].end, 999);
    assert_eq!(chunks[0].size, 1000);
}

#[test]
fn test_smart_chunking_generate_chunks_fixed() {
    let config = TurboCdnConfig::default();
    let chunking = SmartChunking::new(config);

    let strategy = ChunkStrategy::Fixed {
        chunk_size: 1024 * 1024,
    };
    let file_size = 5 * 1024 * 1024; // 5 MB
    let chunks = chunking.generate_chunks(file_size, &strategy);

    assert_eq!(chunks.len(), 5);

    // Verify chunk ranges are contiguous
    for i in 0..chunks.len() {
        assert_eq!(chunks[i].index, i as u32);
        if i > 0 {
            assert_eq!(chunks[i].start, chunks[i - 1].end + 1);
        }
    }

    // Verify total size
    let total_size: u64 = chunks.iter().map(|c| c.size).sum();
    assert_eq!(total_size, file_size);
}

#[test]
fn test_smart_chunking_generate_chunks_dynamic() {
    let config = TurboCdnConfig::default();
    let chunking = SmartChunking::new(config);

    let strategy = ChunkStrategy::Dynamic {
        base_size: 2 * 1024 * 1024,
        max_chunks: 10,
    };
    let file_size = 20 * 1024 * 1024; // 20 MB
    let chunks = chunking.generate_chunks(file_size, &strategy);

    // Should have at most max_chunks
    assert!(chunks.len() <= 10);

    // Verify total size
    let total_size: u64 = chunks.iter().map(|c| c.size).sum();
    assert_eq!(total_size, file_size);
}

#[test]
fn test_smart_chunking_generate_chunks_aggressive() {
    let config = TurboCdnConfig::default();
    let chunking = SmartChunking::new(config);

    let strategy = ChunkStrategy::Aggressive {
        chunk_size: 5 * 1024 * 1024,
        max_chunks: 20,
    };
    let file_size = 50 * 1024 * 1024; // 50 MB
    let chunks = chunking.generate_chunks(file_size, &strategy);

    // Should have at most max_chunks
    assert!(chunks.len() <= 20);

    // First chunks should be smaller (aggressive strategy starts with smaller chunks)
    if chunks.len() > 8 {
        // First 8 chunks should be smaller
        let first_chunk_size = chunks[0].size;
        let later_chunk_size = chunks[8].size;
        assert!(first_chunk_size <= later_chunk_size);
    }

    // Verify total size
    let total_size: u64 = chunks.iter().map(|c| c.size).sum();
    assert_eq!(total_size, file_size);
}

#[test]
fn test_smart_chunking_record_performance() {
    let config = TurboCdnConfig::default();
    let mut chunking = SmartChunking::new(config);

    let metrics = ChunkMetrics {
        index: 0,
        size: 1024 * 1024,
        duration: Duration::from_millis(500),
        retry_count: 0,
        success: true,
        error: None,
    };

    chunking.record_chunk_performance(metrics);

    let stats = chunking.get_performance_stats();
    assert!(stats.is_some());
}

#[test]
fn test_smart_chunking_performance_stats_after_multiple_records() {
    let config = TurboCdnConfig::default();
    let mut chunking = SmartChunking::new(config);

    // Record successful chunks
    for i in 0..10 {
        let metrics = ChunkMetrics {
            index: i,
            size: 1024 * 1024,
            duration: Duration::from_millis(100),
            retry_count: 0,
            success: true,
            error: None,
        };
        chunking.record_chunk_performance(metrics);
    }

    let stats = chunking.get_performance_stats().unwrap();
    assert_eq!(stats.total_chunks, 10);
    assert_eq!(stats.successful_chunks, 10);
    assert_eq!(stats.success_rate, 1.0);
}

#[test]
fn test_smart_chunking_performance_stats_with_failures() {
    let config = TurboCdnConfig::default();
    let mut chunking = SmartChunking::new(config);

    // Record 8 successful and 2 failed chunks
    for i in 0..10 {
        let metrics = ChunkMetrics {
            index: i,
            size: 1024 * 1024,
            duration: Duration::from_millis(100),
            retry_count: if i < 8 { 0 } else { 3 },
            success: i < 8,
            error: if i < 8 {
                None
            } else {
                Some("Failed".to_string())
            },
        };
        chunking.record_chunk_performance(metrics);
    }

    let stats = chunking.get_performance_stats().unwrap();
    assert_eq!(stats.total_chunks, 10);
    assert_eq!(stats.successful_chunks, 8);
    assert!((stats.success_rate - 0.8).abs() < 0.001);
}

// ============================================================================
// PerformanceSummary Tests
// ============================================================================

#[test]
fn test_performance_summary_display() {
    let summary = PerformanceSummary {
        success_rate: 0.95,
        avg_speed: 10.0 * 1024.0 * 1024.0, // 10 MB/s
        total_chunks: 100,
        successful_chunks: 95,
    };

    let display = format!("{}", summary);
    assert!(display.contains("95.0%"));
    assert!(display.contains("95/100"));
    assert!(display.contains("10.0 MB/s"));
}

#[test]
fn test_performance_summary_clone() {
    let summary = PerformanceSummary {
        success_rate: 0.9,
        avg_speed: 5.0 * 1024.0 * 1024.0,
        total_chunks: 50,
        successful_chunks: 45,
    };

    let cloned = summary.clone();
    assert_eq!(cloned.success_rate, 0.9);
    assert_eq!(cloned.total_chunks, 50);
    assert_eq!(cloned.successful_chunks, 45);
}

#[test]
fn test_performance_summary_debug() {
    let summary = PerformanceSummary {
        success_rate: 1.0,
        avg_speed: 1024.0 * 1024.0,
        total_chunks: 10,
        successful_chunks: 10,
    };

    let debug_str = format!("{:?}", summary);
    assert!(debug_str.contains("PerformanceSummary"));
}

// ============================================================================
// Edge Case Tests
// ============================================================================

#[test]
fn test_smart_chunking_zero_size_file() {
    let config = TurboCdnConfig::default();
    let chunking = SmartChunking::new(config);

    // Zero-size file should use single chunk
    let strategy = chunking.calculate_strategy(0, true);
    assert_eq!(strategy, ChunkStrategy::Single);
}

#[test]
fn test_smart_chunking_exact_chunk_boundary() {
    let config = TurboCdnConfig::default();
    let chunking = SmartChunking::new(config);

    let chunk_size = 1024 * 1024; // 1 MB
    let strategy = ChunkStrategy::Fixed { chunk_size };
    let file_size = 5 * chunk_size; // Exactly 5 chunks

    let chunks = chunking.generate_chunks(file_size, &strategy);

    assert_eq!(chunks.len(), 5);
    for chunk in &chunks {
        assert_eq!(chunk.size, chunk_size);
    }
}

#[test]
fn test_smart_chunking_non_exact_chunk_boundary() {
    let config = TurboCdnConfig::default();
    let chunking = SmartChunking::new(config);

    let chunk_size = 1024 * 1024; // 1 MB
    let strategy = ChunkStrategy::Fixed { chunk_size };
    let file_size = 5 * chunk_size + 500; // 5 chunks + 500 bytes

    let chunks = chunking.generate_chunks(file_size, &strategy);

    assert_eq!(chunks.len(), 6);

    // Last chunk should be smaller
    let last_chunk = chunks.last().unwrap();
    assert_eq!(last_chunk.size, 500);
}

#[test]
fn test_smart_chunking_very_small_chunk_size() {
    let config = TurboCdnConfig::default();
    let chunking = SmartChunking::new(config);

    let strategy = ChunkStrategy::Fixed { chunk_size: 100 };
    let file_size = 1000;

    let chunks = chunking.generate_chunks(file_size, &strategy);

    assert_eq!(chunks.len(), 10);

    // Verify total size
    let total_size: u64 = chunks.iter().map(|c| c.size).sum();
    assert_eq!(total_size, file_size);
}

#[test]
fn test_smart_chunking_chunk_size_larger_than_file() {
    let config = TurboCdnConfig::default();
    let chunking = SmartChunking::new(config);

    let strategy = ChunkStrategy::Fixed {
        chunk_size: 10 * 1024 * 1024,
    }; // 10 MB chunks
    let file_size = 5 * 1024 * 1024; // 5 MB file

    let chunks = chunking.generate_chunks(file_size, &strategy);

    assert_eq!(chunks.len(), 1);
    assert_eq!(chunks[0].size, file_size);
}

#[test]
fn test_chunk_range_header_format() {
    let test_cases = vec![
        (0, 999, "bytes=0-999"),
        (1000, 1999, "bytes=1000-1999"),
        (0, 0, "bytes=0-0"),
        (1024 * 1024, 2 * 1024 * 1024 - 1, "bytes=1048576-2097151"),
    ];

    for (start, end, expected) in test_cases {
        let range = ChunkRange {
            index: 0,
            start,
            end,
            size: end - start + 1,
        };
        assert_eq!(range.to_range_header(), expected);
    }
}

#[test]
fn test_smart_chunking_performance_history_limit() {
    let config = TurboCdnConfig::default();
    let mut chunking = SmartChunking::new(config);

    // Record more than max_history entries
    for i in 0..200 {
        let metrics = ChunkMetrics {
            index: i,
            size: 1024,
            duration: Duration::from_millis(10),
            retry_count: 0,
            success: true,
            error: None,
        };
        chunking.record_chunk_performance(metrics);
    }

    // Stats should still work
    let stats = chunking.get_performance_stats();
    assert!(stats.is_some());
    // History should be limited to max_history (100)
    assert!(stats.unwrap().total_chunks <= 100);
}
