# DNS Cache

Turbo CDN includes a high-performance DNS caching system for reduced latency.

## Overview

DNS resolution can add significant latency to downloads:
- Each new connection requires DNS lookup
- Public DNS servers may be slow
- Repeated lookups waste time

Turbo CDN's DNS cache:
- Caches resolved addresses
- Uses hickory-dns for fast resolution
- Implements TTL-based expiration
- Provides automatic cleanup

## How It Works

### DNS Resolution Flow

```
┌─────────────────────────────────────────────────────────────┐
│                    DNS Cache System                          │
├─────────────────────────────────────────────────────────────┤
│  1. Cache Lookup                                             │
│     ├─ Check if hostname is cached                          │
│     └─ If valid entry exists, return immediately            │
│                                                              │
│  2. DNS Resolution (on cache miss)                          │
│     ├─ Use hickory-dns resolver                             │
│     ├─ Query configured DNS servers                         │
│     └─ Handle A/AAAA records                                │
│                                                              │
│  3. Cache Storage                                            │
│     ├─ Store resolved addresses                             │
│     ├─ Set TTL from DNS response                            │
│     └─ Limit cache size (LRU eviction)                      │
│                                                              │
│  4. Background Maintenance                                   │
│     ├─ Periodic cleanup of expired entries                  │
│     └─ Pre-fetch for frequently used hosts                  │
└─────────────────────────────────────────────────────────────┘
```

## Performance Impact

### Latency Reduction

| Scenario | Without Cache | With Cache | Improvement |
|----------|---------------|------------|-------------|
| First request | 50-200ms | 50-200ms | - |
| Subsequent requests | 50-200ms | < 1ms | 99%+ |
| Parallel downloads | 50-200ms each | < 1ms each | 99%+ |

### Real-World Benefits

For a download with 16 parallel chunks:
- **Without cache**: 16 × 100ms = 1.6s DNS overhead
- **With cache**: 100ms + 15 × 0ms = 100ms DNS overhead
- **Savings**: 1.5 seconds per download

## Configuration

### Default Settings

| Parameter | Default | Description |
|-----------|---------|-------------|
| Cache Size | 1000 entries | Maximum cached hosts |
| Default TTL | 300 seconds | TTL when not specified |
| Min TTL | 60 seconds | Minimum cache time |
| Max TTL | 3600 seconds | Maximum cache time |
| Cleanup Interval | 60 seconds | How often to clean expired |

### Custom Configuration

```rust
use turbo_cdn::*;

#[tokio::main]
async fn main() -> turbo_cdn::Result<()> {
    let downloader = TurboCdn::builder()
        .with_dns_cache(true)              // Enable (default)
        .with_dns_cache_size(2000)         // Increase cache size
        .with_dns_cache_ttl(600)           // 10 minute default TTL
        .build()
        .await?;
    
    Ok(())
}
```

## DNS Resolver

### hickory-dns Integration

Turbo CDN uses [hickory-dns](https://github.com/hickory-dns/hickory-dns) (formerly trust-dns) for:

- Pure Rust implementation
- Async/await support
- DNS-over-TLS support
- DNSSEC validation (optional)

### Resolver Configuration

```rust
// Default resolver uses system DNS
let downloader = TurboCdn::new().await?;

// Custom DNS servers (future feature)
let downloader = TurboCdn::builder()
    .with_dns_servers(vec!["8.8.8.8", "1.1.1.1"])
    .build()
    .await?;
```

## Cache Behavior

### TTL Handling

1. **DNS Response TTL**: Respected from server response
2. **Minimum TTL**: Prevents too-short caching
3. **Maximum TTL**: Prevents stale entries

### Cache Eviction

When cache is full:
1. Expired entries removed first
2. LRU (Least Recently Used) eviction
3. Maintains frequently accessed entries

### Pre-fetching

For known CDN hosts, Turbo CDN can pre-fetch DNS:

```rust
// Pre-fetch DNS for known mirrors
downloader.prefetch_dns(&[
    "ghfast.top",
    "gh.con.sh",
    "github.com",
]).await?;
```

## Monitoring

### Cache Statistics

```rust
use turbo_cdn::*;

#[tokio::main]
async fn main() -> turbo_cdn::Result<()> {
    let downloader = TurboCdn::new().await?;
    
    // Perform some downloads...
    
    // Get DNS cache stats
    let stats = downloader.get_dns_cache_stats();
    println!("Cache hits: {}", stats.hits);
    println!("Cache misses: {}", stats.misses);
    println!("Hit rate: {:.1}%", stats.hit_rate() * 100.0);
    println!("Cached entries: {}", stats.entries);
    
    Ok(())
}
```

### CLI Statistics

```bash
turbo-cdn stats
```

Includes DNS cache metrics.

## Best Practices

### Enable for Production

DNS caching is enabled by default and recommended for:
- All production deployments
- Repeated downloads from same hosts
- High-concurrency scenarios

### Disable for Debugging

When troubleshooting DNS issues:

```rust
let downloader = TurboCdn::builder()
    .with_dns_cache(false)
    .build()
    .await?;
```

### Large-Scale Deployments

For applications with many unique hosts:

```rust
let downloader = TurboCdn::builder()
    .with_dns_cache_size(5000)     // Larger cache
    .with_dns_cache_ttl(900)       // Longer TTL
    .build()
    .await?;
```

## Troubleshooting

### DNS Resolution Failures

If DNS resolution fails:

1. **Check network**: Ensure internet connectivity
2. **Check DNS servers**: System DNS may be misconfigured
3. **Try alternative DNS**: Configure custom DNS servers

### Stale Cache Entries

If cached entries become stale:

```rust
// Clear DNS cache
downloader.clear_dns_cache().await;
```

### Debug Logging

```bash
RUST_LOG=turbo_cdn::dns_cache=debug turbo-cdn dl "https://example.com/file.zip"
```

## Security Considerations

### DNS Spoofing Protection

- Use HTTPS for downloads (TLS validates server)
- Consider DNS-over-TLS for sensitive environments
- DNSSEC validation available (optional)

### Cache Poisoning

- TTL limits prevent long-term poisoning
- Regular cache cleanup
- Validation against known CDN hosts

## Next Steps

- [API Reference](/api/) - Complete API documentation
- [Performance Guide](/guide/smart-download) - Optimization tips
