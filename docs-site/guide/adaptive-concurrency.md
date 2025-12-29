# Adaptive Concurrency

Turbo CDN dynamically adjusts download concurrency based on network conditions.

## Overview

Traditional download tools use fixed concurrency, which can:
- **Overload slow networks** → Increased failures
- **Underutilize fast networks** → Slower downloads

Turbo CDN's adaptive concurrency controller solves this by:
- Monitoring network conditions in real-time
- Detecting congestion patterns
- Dynamically adjusting parallelization

## How It Works

### Concurrency Control Flow

```
┌─────────────────────────────────────────────────────────────┐
│               Adaptive Concurrency Controller                │
├─────────────────────────────────────────────────────────────┤
│  1. Initial Assessment                                       │
│     ├─ Test network bandwidth                               │
│     └─ Set initial concurrency level                        │
│                                                              │
│  2. Continuous Monitoring                                    │
│     ├─ Track chunk completion times                         │
│     ├─ Monitor error rates                                  │
│     └─ Detect congestion patterns                           │
│                                                              │
│  3. Dynamic Adjustment                                       │
│     ├─ Increase concurrency if network is fast              │
│     ├─ Decrease if congestion detected                      │
│     └─ Maintain optimal throughput                          │
│                                                              │
│  4. Stabilization                                            │
│     └─ Converge to optimal level                            │
└─────────────────────────────────────────────────────────────┘
```

## Congestion Detection

### Indicators

| Indicator | Threshold | Action |
|-----------|-----------|--------|
| Chunk timeout | > 2 consecutive | Reduce concurrency |
| Error rate | > 10% | Reduce concurrency |
| Latency spike | > 2x baseline | Reduce concurrency |
| Fast completion | < 50% expected time | Increase concurrency |

### Algorithm

```rust
// Simplified congestion detection logic
if error_rate > 0.1 || consecutive_timeouts > 2 {
    concurrency = max(concurrency - 2, MIN_CONCURRENCY);
} else if avg_chunk_time < expected_time * 0.5 {
    concurrency = min(concurrency + 1, MAX_CONCURRENCY);
}
```

## Configuration

### Default Settings

| Parameter | Default | Description |
|-----------|---------|-------------|
| Min Concurrency | 2 | Minimum parallel chunks |
| Max Concurrency | 32 | Maximum parallel chunks |
| Initial Concurrency | 8 | Starting level |
| Adjustment Interval | 1 second | How often to adjust |

### Custom Configuration

```rust
use turbo_cdn::*;

#[tokio::main]
async fn main() -> turbo_cdn::Result<()> {
    let downloader = TurboCdn::builder()
        .with_max_concurrent_downloads(16)    // Maximum concurrency
        .with_adaptive_concurrency(true)      // Enable adaptive control
        .build()
        .await?;
    
    Ok(())
}
```

### Download Options

```rust
use turbo_cdn::*;

let options = DownloadOptions::new()
    .with_max_concurrent_chunks(16)    // Max concurrent chunks
    .with_adaptive_concurrency(true);  // Enable adaptive control

let result = downloader.download_with_options(
    "https://example.com/large-file.zip",
    "./file.zip",
    options
).await?;
```

## Performance Impact

### Benchmark Results

| Network Type | Fixed (8) | Adaptive | Improvement |
|--------------|-----------|----------|-------------|
| Fast (100 Mbps) | 8.5 MB/s | 11.2 MB/s | +32% |
| Medium (20 Mbps) | 2.1 MB/s | 2.4 MB/s | +14% |
| Slow (5 Mbps) | 0.4 MB/s | 0.6 MB/s | +50% |
| Unstable | 0.2 MB/s | 0.5 MB/s | +150% |

### Key Benefits

1. **Fast Networks**: Maximizes throughput by increasing concurrency
2. **Slow Networks**: Prevents overload by reducing concurrency
3. **Unstable Networks**: Adapts to changing conditions
4. **Congested Networks**: Detects and responds to congestion

## Monitoring

### Real-time Statistics

```rust
use turbo_cdn::*;

#[tokio::main]
async fn main() -> turbo_cdn::Result<()> {
    let downloader = TurboCdn::new().await?;
    
    // During download, concurrency is tracked
    let result = downloader.download_from_url("https://example.com/file.zip").await?;
    
    // Get performance summary
    let summary = downloader.get_performance_summary();
    println!("Average concurrency: {}", summary.avg_concurrency);
    
    Ok(())
}
```

### CLI Verbose Output

```bash
turbo-cdn dl "https://example.com/large-file.zip" --verbose
```

Output includes:
```
Adaptive concurrency: 8 → 12 (network fast)
Adaptive concurrency: 12 → 10 (slight congestion)
Adaptive concurrency: 10 → 10 (stable)
```

## Best Practices

### When to Enable

Adaptive concurrency is **enabled by default** and recommended for:
- Large file downloads (> 10 MB)
- Unknown network conditions
- Variable network quality

### When to Disable

Consider fixed concurrency when:
- Network is known and stable
- Server has strict rate limits
- Debugging download issues

```rust
let downloader = TurboCdn::builder()
    .with_adaptive_concurrency(false)
    .with_max_concurrent_downloads(4)  // Fixed at 4
    .build()
    .await?;
```

## Troubleshooting

### Too Aggressive

If downloads cause network issues:

```rust
let downloader = TurboCdn::builder()
    .with_max_concurrent_downloads(8)  // Lower maximum
    .build()
    .await?;
```

### Too Conservative

If downloads are slow on fast networks:

```rust
let downloader = TurboCdn::builder()
    .with_max_concurrent_downloads(32)  // Higher maximum
    .build()
    .await?;
```

### Debug Logging

```bash
RUST_LOG=turbo_cdn::adaptive_concurrency=debug turbo-cdn dl "https://example.com/file.zip"
```

## Next Steps

- [Smart Chunking](/guide/smart-chunking) - Intelligent chunk sizing
- [DNS Cache](/guide/dns-cache) - High-performance DNS resolution
