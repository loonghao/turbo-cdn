// Licensed under the MIT License
// Copyright (c) 2025 Hal <hal.long@outlook.com>

//! Tests for the DNS cache module

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::time::Duration;
use turbo_cdn::dns_cache::{DnsCache, DnsCacheEntry, DnsCacheStats};

// ============================================================================
// DnsCacheEntry Tests
// ============================================================================

#[test]
fn test_dns_cache_entry_is_valid() {
    let entry = DnsCacheEntry {
        addresses: vec![IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8))],
        created_at: std::time::Instant::now(),
        ttl: Duration::from_secs(300),
        hit_count: 0,
        last_accessed: std::time::Instant::now(),
    };

    assert!(entry.is_valid());
    assert!(!entry.is_expired());
}

#[test]
fn test_dns_cache_entry_is_expired() {
    let entry = DnsCacheEntry {
        addresses: vec![IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8))],
        created_at: std::time::Instant::now() - Duration::from_secs(400),
        ttl: Duration::from_secs(300),
        hit_count: 0,
        last_accessed: std::time::Instant::now(),
    };

    // Note: This test may be flaky due to timing, but with 400s elapsed and 300s TTL,
    // the entry should be expired
    assert!(entry.is_expired());
    assert!(!entry.is_valid());
}

#[test]
fn test_dns_cache_entry_mark_accessed() {
    let mut entry = DnsCacheEntry {
        addresses: vec![IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8))],
        created_at: std::time::Instant::now(),
        ttl: Duration::from_secs(300),
        hit_count: 0,
        last_accessed: std::time::Instant::now(),
    };

    assert_eq!(entry.hit_count, 0);

    entry.mark_accessed();
    assert_eq!(entry.hit_count, 1);

    entry.mark_accessed();
    assert_eq!(entry.hit_count, 2);
}

#[test]
fn test_dns_cache_entry_multiple_addresses() {
    let entry = DnsCacheEntry {
        addresses: vec![
            IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8)),
            IpAddr::V4(Ipv4Addr::new(8, 8, 4, 4)),
            IpAddr::V6(Ipv6Addr::new(0x2001, 0x4860, 0x4860, 0, 0, 0, 0, 0x8888)),
        ],
        created_at: std::time::Instant::now(),
        ttl: Duration::from_secs(300),
        hit_count: 0,
        last_accessed: std::time::Instant::now(),
    };

    assert_eq!(entry.addresses.len(), 3);
}

#[test]
fn test_dns_cache_entry_clone() {
    let entry = DnsCacheEntry {
        addresses: vec![IpAddr::V4(Ipv4Addr::new(1, 2, 3, 4))],
        created_at: std::time::Instant::now(),
        ttl: Duration::from_secs(60),
        hit_count: 5,
        last_accessed: std::time::Instant::now(),
    };

    let cloned = entry.clone();
    assert_eq!(cloned.addresses, entry.addresses);
    assert_eq!(cloned.ttl, entry.ttl);
    assert_eq!(cloned.hit_count, entry.hit_count);
}

#[test]
fn test_dns_cache_entry_debug() {
    let entry = DnsCacheEntry {
        addresses: vec![IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8))],
        created_at: std::time::Instant::now(),
        ttl: Duration::from_secs(300),
        hit_count: 0,
        last_accessed: std::time::Instant::now(),
    };

    let debug_str = format!("{:?}", entry);
    assert!(debug_str.contains("DnsCacheEntry"));
    assert!(debug_str.contains("8.8.8.8"));
}

// ============================================================================
// DnsCache Tests
// ============================================================================

#[test]
fn test_dns_cache_new() {
    let cache = DnsCache::new(Duration::from_secs(300), 1000);
    assert!(cache.is_empty());
    assert_eq!(cache.len(), 0);
}

#[test]
fn test_dns_cache_with_defaults() {
    let cache = DnsCache::with_defaults();
    assert!(cache.is_empty());
}

#[tokio::test]
async fn test_dns_cache_insert_and_get() {
    let cache = DnsCache::new(Duration::from_secs(300), 1000);

    let addresses = vec![
        IpAddr::V4(Ipv4Addr::new(93, 184, 216, 34)),
        IpAddr::V4(Ipv4Addr::new(93, 184, 216, 35)),
    ];

    cache.insert("example.com", addresses.clone(), None).await;

    assert!(!cache.is_empty());
    assert_eq!(cache.len(), 1);
    assert!(cache.contains("example.com"));

    let entry = cache.get_entry("example.com");
    assert!(entry.is_some());
    assert_eq!(entry.unwrap().addresses, addresses);
}

#[tokio::test]
async fn test_dns_cache_insert_with_custom_ttl() {
    let cache = DnsCache::new(Duration::from_secs(300), 1000);

    let addresses = vec![IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1))];
    let custom_ttl = Duration::from_secs(60);

    cache
        .insert("cloudflare.com", addresses, Some(custom_ttl))
        .await;

    let entry = cache.get_entry("cloudflare.com");
    assert!(entry.is_some());
    assert_eq!(entry.unwrap().ttl, custom_ttl);
}

#[tokio::test]
async fn test_dns_cache_clear() {
    let cache = DnsCache::new(Duration::from_secs(300), 1000);

    cache
        .insert(
            "example.com",
            vec![IpAddr::V4(Ipv4Addr::new(1, 2, 3, 4))],
            None,
        )
        .await;
    cache
        .insert(
            "test.com",
            vec![IpAddr::V4(Ipv4Addr::new(5, 6, 7, 8))],
            None,
        )
        .await;

    assert_eq!(cache.len(), 2);

    cache.clear().await;

    assert!(cache.is_empty());
    assert_eq!(cache.len(), 0);
}

#[tokio::test]
async fn test_dns_cache_cleanup_expired() {
    let cache = DnsCache::new(Duration::from_millis(1), 1000); // Very short TTL

    cache
        .insert(
            "short-lived.com",
            vec![IpAddr::V4(Ipv4Addr::new(1, 2, 3, 4))],
            None,
        )
        .await;

    // Wait for TTL to expire
    tokio::time::sleep(Duration::from_millis(10)).await;

    cache.cleanup_expired().await;

    // Entry should be removed after cleanup
    assert!(cache.is_empty());
}

#[tokio::test]
async fn test_dns_cache_get_stats() {
    let cache = DnsCache::new(Duration::from_secs(300), 1000);

    cache
        .insert(
            "example.com",
            vec![IpAddr::V4(Ipv4Addr::new(1, 2, 3, 4))],
            None,
        )
        .await;

    let stats = cache.get_stats().await;
    assert_eq!(stats.total_entries, 1);
}

#[tokio::test]
async fn test_dns_cache_contains() {
    let cache = DnsCache::new(Duration::from_secs(300), 1000);

    assert!(!cache.contains("nonexistent.com"));

    cache
        .insert(
            "example.com",
            vec![IpAddr::V4(Ipv4Addr::new(1, 2, 3, 4))],
            None,
        )
        .await;

    assert!(cache.contains("example.com"));
    assert!(!cache.contains("other.com"));
}

#[tokio::test]
async fn test_dns_cache_multiple_entries() {
    let cache = DnsCache::new(Duration::from_secs(300), 1000);

    for i in 0..10 {
        let hostname = format!("host{}.example.com", i);
        let ip = IpAddr::V4(Ipv4Addr::new(10, 0, 0, i as u8));
        cache.insert(&hostname, vec![ip], None).await;
    }

    assert_eq!(cache.len(), 10);

    for i in 0..10 {
        let hostname = format!("host{}.example.com", i);
        assert!(cache.contains(&hostname));
    }
}

// ============================================================================
// DnsCacheStats Tests
// ============================================================================

#[test]
fn test_dns_cache_stats_display() {
    let stats = DnsCacheStats {
        total_entries: 10,
        cache_hits: 80,
        cache_misses: 20,
        hit_ratio: 0.8,
        expired_cleaned: 5,
        time_saved_ms: 4000,
    };

    let display = format!("{}", stats);
    assert!(display.contains("10 entries"));
    assert!(display.contains("80.0%"));
    assert!(display.contains("80/100"));
    assert!(display.contains("4000ms"));
}

#[test]
fn test_dns_cache_stats_clone() {
    let stats = DnsCacheStats {
        total_entries: 5,
        cache_hits: 100,
        cache_misses: 10,
        hit_ratio: 0.909,
        expired_cleaned: 2,
        time_saved_ms: 5000,
    };

    let cloned = stats.clone();
    assert_eq!(cloned.total_entries, stats.total_entries);
    assert_eq!(cloned.cache_hits, stats.cache_hits);
    assert_eq!(cloned.cache_misses, stats.cache_misses);
}

#[test]
fn test_dns_cache_stats_debug() {
    let stats = DnsCacheStats {
        total_entries: 1,
        cache_hits: 10,
        cache_misses: 2,
        hit_ratio: 0.833,
        expired_cleaned: 0,
        time_saved_ms: 500,
    };

    let debug_str = format!("{:?}", stats);
    assert!(debug_str.contains("DnsCacheStats"));
}

// ============================================================================
// Edge Case Tests
// ============================================================================

#[tokio::test]
async fn test_dns_cache_empty_addresses() {
    let cache = DnsCache::new(Duration::from_secs(300), 1000);

    // Insert entry with empty addresses
    cache.insert("empty.com", vec![], None).await;

    let entry = cache.get_entry("empty.com");
    assert!(entry.is_some());
    assert!(entry.unwrap().addresses.is_empty());
}

#[tokio::test]
async fn test_dns_cache_overwrite_entry() {
    let cache = DnsCache::new(Duration::from_secs(300), 1000);

    let addresses1 = vec![IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1))];
    let addresses2 = vec![IpAddr::V4(Ipv4Addr::new(2, 2, 2, 2))];

    cache.insert("example.com", addresses1, None).await;
    cache.insert("example.com", addresses2.clone(), None).await;

    let entry = cache.get_entry("example.com");
    assert!(entry.is_some());
    assert_eq!(entry.unwrap().addresses, addresses2);
}

#[tokio::test]
async fn test_dns_cache_ipv6_addresses() {
    let cache = DnsCache::new(Duration::from_secs(300), 1000);

    let addresses = vec![
        IpAddr::V6(Ipv6Addr::new(0x2001, 0x4860, 0x4860, 0, 0, 0, 0, 0x8888)),
        IpAddr::V6(Ipv6Addr::new(0x2001, 0x4860, 0x4860, 0, 0, 0, 0, 0x8844)),
    ];

    cache.insert("google.com", addresses.clone(), None).await;

    let entry = cache.get_entry("google.com");
    assert!(entry.is_some());
    assert_eq!(entry.unwrap().addresses, addresses);
}

#[tokio::test]
async fn test_dns_cache_mixed_ip_versions() {
    let cache = DnsCache::new(Duration::from_secs(300), 1000);

    let addresses = vec![
        IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8)),
        IpAddr::V6(Ipv6Addr::new(0x2001, 0x4860, 0x4860, 0, 0, 0, 0, 0x8888)),
    ];

    cache
        .insert("dual-stack.com", addresses.clone(), None)
        .await;

    let entry = cache.get_entry("dual-stack.com");
    assert!(entry.is_some());
    assert_eq!(entry.unwrap().addresses.len(), 2);
}

#[test]
fn test_dns_cache_debug() {
    let cache = DnsCache::new(Duration::from_secs(300), 1000);
    let debug_str = format!("{:?}", cache);
    assert!(debug_str.contains("DnsCache"));
}

// ============================================================================
// Concurrent Access Tests
// ============================================================================

#[tokio::test]
async fn test_dns_cache_concurrent_inserts() {
    use std::sync::Arc;

    let cache = Arc::new(DnsCache::new(Duration::from_secs(300), 1000));

    let mut handles = vec![];

    for i in 0..100 {
        let cache_clone = Arc::clone(&cache);
        let handle = tokio::spawn(async move {
            let hostname = format!("host{}.test.com", i);
            let ip = IpAddr::V4(Ipv4Addr::new(10, 0, (i / 256) as u8, (i % 256) as u8));
            cache_clone.insert(&hostname, vec![ip], None).await;
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await.unwrap();
    }

    assert_eq!(cache.len(), 100);
}

#[tokio::test]
async fn test_dns_cache_concurrent_reads() {
    use std::sync::Arc;

    let cache = Arc::new(DnsCache::new(Duration::from_secs(300), 1000));

    // Pre-populate cache
    for i in 0..10 {
        let hostname = format!("host{}.test.com", i);
        let ip = IpAddr::V4(Ipv4Addr::new(10, 0, 0, i as u8));
        cache.insert(&hostname, vec![ip], None).await;
    }

    let mut handles = vec![];

    for i in 0..100 {
        let cache_clone = Arc::clone(&cache);
        let handle = tokio::spawn(async move {
            let hostname = format!("host{}.test.com", i % 10);
            cache_clone.contains(&hostname)
        });
        handles.push(handle);
    }

    for handle in handles {
        let result = handle.await.unwrap();
        assert!(result);
    }
}
