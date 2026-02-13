//! Memory usage tracking and metrics
//!
//! This module provides comprehensive memory usage tracking, including heap allocations,
//! memory pressure detection, and adaptive behavior based on memory usage patterns.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::{debug, info, warn};

/// Memory usage statistics
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MemoryUsage {
    /// Current heap allocated bytes
    pub heap_allocated: u64,
    /// Current heap deallocated bytes
    pub heap_deallocated: u64,
    /// Peak memory usage during session
    pub peak_memory: u64,
    /// Current memory usage (allocated - deallocated)
    pub current_memory: u64,
    /// Number of allocations
    pub allocation_count: u64,
    /// Number of deallocations
    pub deallocation_count: u64,
    /// Average allocation size
    pub avg_allocation_size: f64,
}

impl Default for MemoryUsage {
    fn default() -> Self {
        Self {
            heap_allocated: 0,
            heap_deallocated: 0,
            peak_memory: 0,
            current_memory: 0,
            allocation_count: 0,
            deallocation_count: 0,
            avg_allocation_size: 0.0,
        }
    }
}

impl MemoryUsage {
    /// Calculate memory efficiency ratio (deallocated / allocated)
    pub fn efficiency_ratio(&self) -> f64 {
        if self.heap_allocated > 0 {
            self.heap_deallocated as f64 / self.heap_allocated as f64
        } else {
            0.0
        }
    }

    /// Check if memory usage is considered high
    pub fn is_high_usage(&self, threshold_mb: u64) -> bool {
        self.current_memory > threshold_mb * 1024 * 1024
    }

    /// Get memory usage in MB
    pub fn current_memory_mb(&self) -> f64 {
        self.current_memory as f64 / (1024.0 * 1024.0)
    }

    /// Get peak memory usage in MB
    pub fn peak_memory_mb(&self) -> f64 {
        self.peak_memory as f64 / (1024.0 * 1024.0)
    }
}

/// Memory pressure levels for adaptive behavior
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum MemoryPressure {
    /// Low memory usage - normal operation
    Low,
    /// Moderate memory usage - start optimizing
    Moderate,
    /// High memory usage - aggressive optimization
    High,
    /// Critical memory usage - emergency measures
    Critical,
}

impl MemoryPressure {
    /// Determine memory pressure level based on current usage
    pub fn from_usage(current_mb: f64, peak_mb: f64) -> Self {
        let usage_ratio = if peak_mb > 0.0 { current_mb / peak_mb } else { 0.0 };
        
        match current_mb {
            mb if mb > 1000.0 => Self::Critical,  // > 1GB
            mb if mb > 500.0 => Self::High,       // > 500MB
            mb if mb > 200.0 => Self::Moderate,   // > 200MB
            _ if usage_ratio > 0.8 => Self::High, // High ratio to peak
            _ => Self::Low,
        }
    }

    /// Get recommended action for this pressure level
    pub fn recommended_action(&self) -> &'static str {
        match self {
            Self::Low => "Normal operation",
            Self::Moderate => "Start cache cleanup and reduce buffer sizes",
            Self::High => "Aggressive cache cleanup and limit concurrent operations",
            Self::Critical => "Emergency cleanup and pause non-essential operations",
        }
    }
}

/// Memory tracker for monitoring heap allocations and providing adaptive behavior
#[derive(Debug)]
pub struct MemoryTracker {
    /// Total bytes allocated
    allocated: AtomicU64,
    /// Total bytes deallocated
    deallocated: AtomicU64,
    /// Peak memory usage
    peak_memory: AtomicU64,
    /// Number of allocations
    allocation_count: AtomicU64,
    /// Number of deallocations
    deallocation_count: AtomicU64,
    /// Total allocation size for average calculation
    total_allocation_size: AtomicU64,
    /// Last memory check timestamp
    last_check: std::sync::Mutex<Instant>,
    /// Memory pressure thresholds in MB
    moderate_threshold: u64,
    high_threshold: u64,
    critical_threshold: u64,
}

impl Default for MemoryTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl MemoryTracker {
    /// Create a new memory tracker with default thresholds
    pub fn new() -> Self {
        Self::with_thresholds(200, 500, 1000) // 200MB, 500MB, 1GB
    }

    /// Create a new memory tracker with custom thresholds (in MB)
    pub fn with_thresholds(moderate: u64, high: u64, critical: u64) -> Self {
        Self {
            allocated: AtomicU64::new(0),
            deallocated: AtomicU64::new(0),
            peak_memory: AtomicU64::new(0),
            allocation_count: AtomicU64::new(0),
            deallocation_count: AtomicU64::new(0),
            total_allocation_size: AtomicU64::new(0),
            last_check: std::sync::Mutex::new(Instant::now()),
            moderate_threshold: moderate,
            high_threshold: high,
            critical_threshold: critical,
        }
    }

    /// Record a memory allocation
    pub fn record_allocation(&self, size: usize) {
        let size_u64 = size as u64;
        
        self.allocated.fetch_add(size_u64, Ordering::Relaxed);
        self.allocation_count.fetch_add(1, Ordering::Relaxed);
        self.total_allocation_size.fetch_add(size_u64, Ordering::Relaxed);
        
        // Update peak memory if necessary
        let current = self.current_memory();
        let mut peak = self.peak_memory.load(Ordering::Relaxed);
        while current > peak {
            match self.peak_memory.compare_exchange_weak(
                peak,
                current,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(new_peak) => peak = new_peak,
            }
        }

        // Log large allocations
        if size > 10 * 1024 * 1024 {  // > 10MB
            debug!("Large allocation: {:.2} MB", size as f64 / (1024.0 * 1024.0));
        }
    }

    /// Record a memory deallocation
    pub fn record_deallocation(&self, size: usize) {
        let size_u64 = size as u64;
        
        self.deallocated.fetch_add(size_u64, Ordering::Relaxed);
        self.deallocation_count.fetch_add(1, Ordering::Relaxed);
    }

    /// Get current memory usage statistics
    pub fn get_usage(&self) -> MemoryUsage {
        let allocated = self.allocated.load(Ordering::Relaxed);
        let deallocated = self.deallocated.load(Ordering::Relaxed);
        let allocation_count = self.allocation_count.load(Ordering::Relaxed);
        let total_allocation_size = self.total_allocation_size.load(Ordering::Relaxed);

        MemoryUsage {
            heap_allocated: allocated,
            heap_deallocated: deallocated,
            peak_memory: self.peak_memory.load(Ordering::Relaxed),
            current_memory: allocated.saturating_sub(deallocated),
            allocation_count,
            deallocation_count: self.deallocation_count.load(Ordering::Relaxed),
            avg_allocation_size: if allocation_count > 0 {
                total_allocation_size as f64 / allocation_count as f64
            } else {
                0.0
            },
        }
    }

    /// Get current memory usage in bytes
    pub fn current_memory(&self) -> u64 {
        let allocated = self.allocated.load(Ordering::Relaxed);
        let deallocated = self.deallocated.load(Ordering::Relaxed);
        allocated.saturating_sub(deallocated)
    }

    /// Get current memory pressure level
    pub fn memory_pressure(&self) -> MemoryPressure {
        let usage = self.get_usage();
        let current_mb = usage.current_memory_mb();
        
        // Use thresholds to determine pressure level
        match current_mb as u64 {
            mb if mb >= self.critical_threshold => MemoryPressure::Critical,
            mb if mb >= self.high_threshold => MemoryPressure::High,
            mb if mb >= self.moderate_threshold => MemoryPressure::Moderate,
            _ => MemoryPressure::Low,
        }
    }

    /// Check if memory pressure has changed and log if necessary
    pub fn check_memory_pressure(&self) -> MemoryPressure {
        let pressure = self.memory_pressure();
        let usage = self.get_usage();
        
        // Only check periodically to avoid spam
        if let Ok(mut last_check) = self.last_check.lock() {
            if last_check.elapsed() > Duration::from_secs(30) {
            match pressure {
                MemoryPressure::Critical => {
                    warn!(
                        "Critical memory pressure: {:.2} MB current, {:.2} MB peak - {}",
                        usage.current_memory_mb(),
                        usage.peak_memory_mb(),
                        pressure.recommended_action()
                    );
                }
                MemoryPressure::High => {
                    warn!(
                        "High memory pressure: {:.2} MB current, {:.2} MB peak - {}",
                        usage.current_memory_mb(),
                        usage.peak_memory_mb(),
                        pressure.recommended_action()
                    );
                }
                MemoryPressure::Moderate => {
                    info!(
                        "Moderate memory usage: {:.2} MB current, {:.2} MB peak",
                        usage.current_memory_mb(),
                        usage.peak_memory_mb()
                    );
                }
                MemoryPressure::Low => {
                    debug!(
                        "Low memory usage: {:.2} MB current, {:.2} MB peak",
                        usage.current_memory_mb(),
                        usage.peak_memory_mb()
                    );
                }
            }
            *last_check = Instant::now();
            }
        }
        
        pressure
    }

    /// Reset all statistics
    pub fn reset(&self) {
        self.allocated.store(0, Ordering::Relaxed);
        self.deallocated.store(0, Ordering::Relaxed);
        self.peak_memory.store(0, Ordering::Relaxed);
        self.allocation_count.store(0, Ordering::Relaxed);
        self.deallocation_count.store(0, Ordering::Relaxed);
        self.total_allocation_size.store(0, Ordering::Relaxed);
        if let Ok(mut last_check) = self.last_check.lock() {
            *last_check = Instant::now();
        }
    }

    /// Get memory efficiency metrics
    pub fn efficiency_metrics(&self) -> MemoryEfficiencyMetrics {
        let usage = self.get_usage();
        
        MemoryEfficiencyMetrics {
            efficiency_ratio: usage.efficiency_ratio(),
            fragmentation_ratio: self.calculate_fragmentation_ratio(),
            allocation_rate: self.calculate_allocation_rate(),
            pressure_level: self.memory_pressure(),
            recommendations: self.get_recommendations(),
        }
    }

    /// Calculate fragmentation ratio (estimate)
    fn calculate_fragmentation_ratio(&self) -> f64 {
        let allocation_count = self.allocation_count.load(Ordering::Relaxed);
        let deallocation_count = self.deallocation_count.load(Ordering::Relaxed);
        
        if allocation_count > 0 {
            let active_allocations = allocation_count.saturating_sub(deallocation_count);
            active_allocations as f64 / allocation_count as f64
        } else {
            0.0
        }
    }

    /// Calculate allocation rate (allocations per second, estimated)
    fn calculate_allocation_rate(&self) -> f64 {
        if let Ok(last_check) = self.last_check.lock() {
            let elapsed = last_check.elapsed().as_secs_f64();
            
            if elapsed > 0.0 {
                let allocation_count = self.allocation_count.load(Ordering::Relaxed);
                allocation_count as f64 / elapsed
            } else {
                0.0
            }
        } else {
            0.0 // Return 0 if lock acquisition fails
        }
    }

    /// Get optimization recommendations based on current state
    fn get_recommendations(&self) -> Vec<String> {
        let mut recommendations = Vec::new();
        let usage = self.get_usage();
        let pressure = self.memory_pressure();
        
        match pressure {
            MemoryPressure::Critical => {
                recommendations.push("Immediately reduce concurrent operations".to_string());
                recommendations.push("Clear all non-essential caches".to_string());
                recommendations.push("Consider reducing buffer sizes".to_string());
            }
            MemoryPressure::High => {
                recommendations.push("Reduce concurrent chunk downloads".to_string());
                recommendations.push("Enable aggressive cache cleanup".to_string());
                recommendations.push("Consider smaller buffer sizes".to_string());
            }
            MemoryPressure::Moderate => {
                recommendations.push("Enable periodic cache cleanup".to_string());
                recommendations.push("Monitor allocation patterns".to_string());
            }
            MemoryPressure::Low => {
                if usage.efficiency_ratio() < 0.8 {
                    recommendations.push("Consider optimizing memory reuse".to_string());
                }
            }
        }
        
        if usage.avg_allocation_size > 1024.0 * 1024.0 {  // > 1MB average
            recommendations.push("Large average allocation size detected - consider buffer pooling".to_string());
        }
        
        recommendations
    }
}

/// Memory efficiency metrics for performance analysis
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MemoryEfficiencyMetrics {
    /// Ratio of deallocated to allocated memory
    pub efficiency_ratio: f64,
    /// Estimated fragmentation ratio
    pub fragmentation_ratio: f64,
    /// Allocation rate (allocations per second)
    pub allocation_rate: f64,
    /// Current memory pressure level
    pub pressure_level: MemoryPressure,
    /// Optimization recommendations
    pub recommendations: Vec<String>,
}

/// Global memory tracker instance
static GLOBAL_MEMORY_TRACKER: once_cell::sync::Lazy<Arc<MemoryTracker>> = 
    once_cell::sync::Lazy::new(|| Arc::new(MemoryTracker::new()));

/// Get the global memory tracker instance
pub fn global_memory_tracker() -> Arc<MemoryTracker> {
    GLOBAL_MEMORY_TRACKER.clone()
}

/// Record a global memory allocation
pub fn record_allocation(size: usize) {
    GLOBAL_MEMORY_TRACKER.record_allocation(size);
}

/// Record a global memory deallocation
pub fn record_deallocation(size: usize) {
    GLOBAL_MEMORY_TRACKER.record_deallocation(size);
}

/// Get global memory usage statistics
pub fn global_memory_usage() -> MemoryUsage {
    GLOBAL_MEMORY_TRACKER.get_usage()
}

/// Get global memory pressure level
pub fn global_memory_pressure() -> MemoryPressure {
    GLOBAL_MEMORY_TRACKER.memory_pressure()
}

/// Check global memory pressure and log if necessary
pub fn check_global_memory_pressure() -> MemoryPressure {
    GLOBAL_MEMORY_TRACKER.check_memory_pressure()
}

/// Custom allocator wrapper that tracks memory usage
/// 
/// This allocator wraps mimalloc and automatically records all allocations
/// and deallocations to the global memory tracker for comprehensive monitoring.
#[derive(Debug)]
pub struct TrackingAllocator;

unsafe impl std::alloc::GlobalAlloc for TrackingAllocator {
    unsafe fn alloc(&self, layout: std::alloc::Layout) -> *mut u8 {
        let ptr = mimalloc::MiMalloc.alloc(layout);
        if !ptr.is_null() {
            record_allocation(layout.size());
        }
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: std::alloc::Layout) {
        record_deallocation(layout.size());
        mimalloc::MiMalloc.dealloc(ptr, layout);
    }

    unsafe fn alloc_zeroed(&self, layout: std::alloc::Layout) -> *mut u8 {
        let ptr = mimalloc::MiMalloc.alloc_zeroed(layout);
        if !ptr.is_null() {
            record_allocation(layout.size());
        }
        ptr
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: std::alloc::Layout, new_size: usize) -> *mut u8 {
        let new_ptr = mimalloc::MiMalloc.realloc(ptr, layout, new_size);
        if !new_ptr.is_null() {
            // Record deallocation of old size and allocation of new size
            record_deallocation(layout.size());
            record_allocation(new_size);
        }
        new_ptr
    }
}

/// Enable memory tracking by using the TrackingAllocator as the global allocator
/// 
/// Add this to your main.rs or lib.rs:
/// ```rust
/// #[global_allocator]
/// static GLOBAL: turbo_cdn::memory_tracker::TrackingAllocator = turbo_cdn::memory_tracker::TrackingAllocator;
/// ```
pub static TRACKING_ALLOCATOR: TrackingAllocator = TrackingAllocator;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_tracker_creation() {
        let tracker = MemoryTracker::new();
        let usage = tracker.get_usage();
        
        assert_eq!(usage.heap_allocated, 0);
        assert_eq!(usage.heap_deallocated, 0);
        assert_eq!(usage.current_memory, 0);
        assert_eq!(usage.allocation_count, 0);
    }

    #[test]
    fn test_memory_allocation_tracking() {
        let tracker = MemoryTracker::new();
        
        tracker.record_allocation(1024);
        tracker.record_allocation(2048);
        
        let usage = tracker.get_usage();
        assert_eq!(usage.heap_allocated, 3072);
        assert_eq!(usage.allocation_count, 2);
        assert_eq!(usage.current_memory, 3072);
        assert_eq!(usage.avg_allocation_size, 1536.0);
    }

    #[test]
    fn test_memory_deallocation_tracking() {
        let tracker = MemoryTracker::new();
        
        tracker.record_allocation(2048);
        tracker.record_deallocation(1024);
        
        let usage = tracker.get_usage();
        assert_eq!(usage.heap_allocated, 2048);
        assert_eq!(usage.heap_deallocated, 1024);
        assert_eq!(usage.current_memory, 1024);
        assert_eq!(usage.deallocation_count, 1);
    }

    #[test]
    fn test_peak_memory_tracking() {
        let tracker = MemoryTracker::new();
        
        tracker.record_allocation(1024);
        tracker.record_allocation(2048);
        tracker.record_deallocation(1024);
        
        let usage = tracker.get_usage();
        assert_eq!(usage.peak_memory, 3072);
        assert_eq!(usage.current_memory, 2048);
    }

    #[test]
    fn test_memory_pressure_levels() {
        assert_eq!(MemoryPressure::from_usage(50.0, 100.0), MemoryPressure::Low);
        assert_eq!(MemoryPressure::from_usage(250.0, 300.0), MemoryPressure::Moderate);
        assert_eq!(MemoryPressure::from_usage(600.0, 700.0), MemoryPressure::High);
        assert_eq!(MemoryPressure::from_usage(1200.0, 1300.0), MemoryPressure::Critical);
    }

    #[test]
    fn test_memory_efficiency_ratio() {
        let usage = MemoryUsage {
            heap_allocated: 1000,
            heap_deallocated: 800,
            ..Default::default()
        };
        
        assert_eq!(usage.efficiency_ratio(), 0.8);
    }

    #[test]
    fn test_global_memory_tracker() {
        let tracker1 = global_memory_tracker();
        let tracker2 = global_memory_tracker();
        
        // Should be the same instance
        assert!(Arc::ptr_eq(&tracker1, &tracker2));
    }

    #[test]
    fn test_memory_usage_mb_conversion() {
        let usage = MemoryUsage {
            current_memory: 1024 * 1024, // 1MB
            peak_memory: 2 * 1024 * 1024, // 2MB
            ..Default::default()
        };
        
        assert_eq!(usage.current_memory_mb(), 1.0);
        assert_eq!(usage.peak_memory_mb(), 2.0);
    }

    #[test]
    fn test_memory_tracker_reset() {
        let tracker = MemoryTracker::new();
        
        tracker.record_allocation(1024);
        tracker.record_deallocation(512);
        
        let usage_before = tracker.get_usage();
        assert!(usage_before.heap_allocated > 0);
        
        tracker.reset();
        
        let usage_after = tracker.get_usage();
        assert_eq!(usage_after.heap_allocated, 0);
        assert_eq!(usage_after.heap_deallocated, 0);
        assert_eq!(usage_after.allocation_count, 0);
    }
}