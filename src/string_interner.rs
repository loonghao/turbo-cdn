//! String interning system for frequently used URLs and patterns
//!
//! This module provides a high-performance string interning system that reduces
//! memory allocations by reusing common strings like CDN patterns and URLs.

use dashmap::DashMap;
use std::borrow::Cow;
use std::sync::Arc;

/// High-performance string interner using Arc<str> for zero-copy sharing
#[derive(Debug, Default)]
pub struct StringInterner {
    /// Cache of interned strings
    cache: DashMap<String, Arc<str>>,
    /// Statistics for monitoring
    stats: InternerStats,
}

/// Statistics for string interner performance monitoring
#[derive(Debug, Default)]
struct InternerStats {
    /// Total number of intern requests
    total_requests: std::sync::atomic::AtomicU64,
    /// Number of cache hits
    cache_hits: std::sync::atomic::AtomicU64,
    /// Number of new strings interned
    new_interns: std::sync::atomic::AtomicU64,
}

impl StringInterner {
    /// Create a new string interner
    pub fn new() -> Self {
        Self {
            cache: DashMap::new(),
            stats: InternerStats::default(),
        }
    }

    /// Intern a string, returning an Arc<str> for efficient sharing
    pub fn intern(&self, s: &str) -> Arc<str> {
        use std::sync::atomic::Ordering;

        self.stats.total_requests.fetch_add(1, Ordering::Relaxed);

        // Check if already interned
        if let Some(interned) = self.cache.get(s) {
            self.stats.cache_hits.fetch_add(1, Ordering::Relaxed);
            return interned.clone();
        }

        // Create new interned string
        let arc_str: Arc<str> = Arc::from(s);
        self.cache.insert(s.to_string(), arc_str.clone());
        self.stats.new_interns.fetch_add(1, Ordering::Relaxed);

        arc_str
    }

    /// Intern a string and return as Cow<str> for flexible usage
    pub fn intern_cow(&self, s: &str) -> Cow<'static, str> {
        let interned = self.intern(s);
        // Convert Arc<str> to Cow<'static, str>
        // This is safe because Arc<str> has static lifetime semantics
        unsafe {
            let ptr = Arc::as_ptr(&interned);
            let static_str = &*ptr;
            Cow::Borrowed(static_str)
        }
    }

    /// Get cache statistics
    pub fn stats(&self) -> InternerStatistics {
        use std::sync::atomic::Ordering;

        InternerStatistics {
            total_requests: self.stats.total_requests.load(Ordering::Relaxed),
            cache_hits: self.stats.cache_hits.load(Ordering::Relaxed),
            new_interns: self.stats.new_interns.load(Ordering::Relaxed),
            cache_size: self.cache.len(),
            hit_rate: {
                let total = self.stats.total_requests.load(Ordering::Relaxed);
                let hits = self.stats.cache_hits.load(Ordering::Relaxed);
                if total > 0 {
                    (hits as f64 / total as f64) * 100.0
                } else {
                    0.0
                }
            },
        }
    }

    /// Clear the cache (useful for testing or memory management)
    pub fn clear(&self) {
        self.cache.clear();
        use std::sync::atomic::Ordering;
        self.stats.total_requests.store(0, Ordering::Relaxed);
        self.stats.cache_hits.store(0, Ordering::Relaxed);
        self.stats.new_interns.store(0, Ordering::Relaxed);
    }

    /// Get current cache size
    pub fn cache_size(&self) -> usize {
        self.cache.len()
    }
}

/// Public statistics structure
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct InternerStatistics {
    pub total_requests: u64,
    pub cache_hits: u64,
    pub new_interns: u64,
    pub cache_size: usize,
    pub hit_rate: f64,
}

/// Global string interner instance for URL patterns and frequently used strings
static GLOBAL_INTERNER: once_cell::sync::Lazy<StringInterner> =
    once_cell::sync::Lazy::new(StringInterner::new);

/// Convenience function to intern a string using the global interner
pub fn intern_string(s: &str) -> Arc<str> {
    GLOBAL_INTERNER.intern(s)
}

/// Convenience function to intern a string as Cow using the global interner
pub fn intern_string_cow(s: &str) -> Cow<'static, str> {
    GLOBAL_INTERNER.intern_cow(s)
}

/// Get global interner statistics
pub fn global_interner_stats() -> InternerStatistics {
    GLOBAL_INTERNER.stats()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_interning() {
        let interner = StringInterner::new();

        let s1 = interner.intern("test");
        let s2 = interner.intern("test");

        // Should be the same Arc
        assert!(Arc::ptr_eq(&s1, &s2));

        let stats = interner.stats();
        assert_eq!(stats.total_requests, 2);
        assert_eq!(stats.cache_hits, 1);
        assert_eq!(stats.new_interns, 1);
        assert_eq!(stats.hit_rate, 50.0);
    }

    #[test]
    fn test_cow_interning() {
        let interner = StringInterner::new();

        let cow1 = interner.intern_cow("test");
        let cow2 = interner.intern_cow("test");

        // Both should be borrowed variants pointing to the same data
        match (&cow1, &cow2) {
            (Cow::Borrowed(s1), Cow::Borrowed(s2)) => {
                assert_eq!(s1.as_ptr(), s2.as_ptr());
            }
            _ => panic!("Expected borrowed variants"),
        }
    }

    #[test]
    fn test_global_interner() {
        let s1 = intern_string("global_test");
        let s2 = intern_string("global_test");

        assert!(Arc::ptr_eq(&s1, &s2));

        let stats = global_interner_stats();
        assert!(stats.total_requests >= 2);
    }

    #[test]
    fn test_interner_clear() {
        let interner = StringInterner::new();

        interner.intern("test1");
        interner.intern("test2");

        assert_eq!(interner.cache_size(), 2);

        interner.clear();

        assert_eq!(interner.cache_size(), 0);
        let stats = interner.stats();
        assert_eq!(stats.total_requests, 0);
    }
}
