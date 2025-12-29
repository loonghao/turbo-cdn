# Smart Chunking

Turbo CDN uses intelligent chunk sizing for optimal download performance.

## Overview

Smart chunking dynamically determines the optimal chunk size based on:
- File size
- Network bandwidth
- Historical performance
- Server capabilities

## How It Works

### Chunk Size Selection

```
┌─────────────────────────────────────────────────────────────┐
│                   Smart Chunking Algorithm                   │
├─────────────────────────────────────────────────────────────┤
│  1. File Analysis                                            │
│     ├─ Get file size                                        │
│     └─ Check server range support                           │
│                                                              │
│  2. Initial Chunk Size                                       │
│     ├─ Small files (< 1MB): Single chunk                    │
│     ├─ Medium files (1-100MB): 1MB chunks                   │
│     └─ Large files (> 100MB): 2-4MB chunks                  │
│                                                              │
│  3. Dynamic Adjustment                                       │
│     ├─ Monitor chunk completion times                       │
│     ├─ Adjust based on throughput                           │
│     └─ Factor in network conditions                         │
│                                                              │
│  4. Performance Learning                                     │
│     └─ Store optimal sizes for future downloads             │
└─────────────────────────────────────────────────────────────┘
```

## Default Chunk Sizes

| File Size | Default Chunk | Rationale |
|-----------|---------------|-----------|
| < 1 MB | Entire file | Overhead not worth splitting |
| 1-10 MB | 512 KB | Balance parallelism and overhead |
| 10-100 MB | 1 MB | Good for most connections |
| 100 MB - 1 GB | 2 MB | Reduce chunk count |
| > 1 GB | 4 MB | Minimize overhead for large files |

## Configuration

### Builder Pattern

```rust
use turbo_cdn::*;

#[tokio::main]
async fn main() -> turbo_cdn::Result<()> {
    let downloader = TurboCdn::builder()
        .with_chunk_size(2 * 1024 * 1024)    // 2MB fixed chunk size
        .with_adaptive_chunking(true)         // Enable smart adjustment
        .build()
        .await?;
    
    Ok(())
}
```

### Download Options

```rust
use turbo_cdn::*;

let options = DownloadOptions::new()
    .with_chunk_size(4 * 1024 * 1024);  // 4MB chunks

let result = downloader.download_with_options(
    "https://example.com/large-file.zip",
    "./file.zip",
    options
).await?;
```

## Adaptive Chunking

When enabled, chunk sizes adjust during download:

### Increase Chunk Size When

- Chunks complete faster than expected
- Network bandwidth is high
- Low error rate

### Decrease Chunk Size When

- Chunks timeout frequently
- Network appears congested
- High error rate

### Example Adaptation

```
Download: 500MB file
Initial chunk size: 2MB (250 chunks)

Progress:
  0-10%:  2MB chunks, 15 MB/s → Increase to 4MB
  10-50%: 4MB chunks, 18 MB/s → Stable
  50-80%: 4MB chunks, 12 MB/s → Decrease to 2MB (congestion)
  80-100%: 2MB chunks, 14 MB/s → Complete
```

## Performance Impact

### Chunk Size vs Performance

| Chunk Size | Overhead | Parallelism | Best For |
|------------|----------|-------------|----------|
| 256 KB | High | High | Slow, unstable networks |
| 1 MB | Medium | Medium | Most scenarios |
| 4 MB | Low | Lower | Fast, stable networks |
| 8 MB | Very Low | Low | Very fast connections |

### Benchmark Results

For a 100MB file:

| Chunk Size | Time (Fast Net) | Time (Slow Net) |
|------------|-----------------|-----------------|
| 256 KB | 12.5s | 85s |
| 1 MB | 10.2s | 82s |
| 4 MB | 9.8s | 95s |
| Adaptive | 9.5s | 78s |

## Memory Considerations

### Memory Usage

```
Memory = Chunk Size × Concurrent Chunks × Buffer Factor
```

Example:
- 4MB chunks × 8 concurrent × 1.5 buffer = 48MB

### Optimization

For memory-constrained environments:

```rust
let downloader = TurboCdn::builder()
    .with_chunk_size(1024 * 1024)         // 1MB chunks
    .with_max_concurrent_downloads(4)      // Fewer concurrent
    .build()
    .await?;
```

## Server Compatibility

### Range Request Support

Smart chunking requires HTTP Range support. Turbo CDN:

1. Checks `Accept-Ranges` header
2. Falls back to single-chunk if unsupported
3. Handles partial content responses

### Detection

```rust
// Automatic detection during download
let result = downloader.download_from_url("https://example.com/file.zip").await?;

// Check if chunked download was used
if result.chunks_used > 1 {
    println!("Downloaded in {} chunks", result.chunks_used);
}
```

## Best Practices

### Large Files (> 100MB)

```rust
let options = DownloadOptions::new()
    .with_chunk_size(4 * 1024 * 1024)      // 4MB chunks
    .with_max_concurrent_chunks(16)
    .with_adaptive_chunking(true);
```

### Small Files (< 10MB)

```rust
let options = DownloadOptions::new()
    .with_chunk_size(512 * 1024)           // 512KB chunks
    .with_max_concurrent_chunks(8);
```

### Unstable Networks

```rust
let options = DownloadOptions::new()
    .with_chunk_size(256 * 1024)           // Small chunks
    .with_max_concurrent_chunks(4)
    .with_resume(true);                     // Enable resume
```

## Troubleshooting

### Slow Downloads

If downloads are slower than expected:

1. **Check chunk size**: May be too small or large
2. **Check concurrency**: May need adjustment
3. **Enable adaptive**: Let the system optimize

```rust
let downloader = TurboCdn::builder()
    .with_adaptive_chunking(true)
    .build()
    .await?;
```

### Memory Issues

If running out of memory:

```rust
let downloader = TurboCdn::builder()
    .with_chunk_size(512 * 1024)           // Smaller chunks
    .with_max_concurrent_downloads(4)       // Fewer concurrent
    .build()
    .await?;
```

### Debug Logging

```bash
RUST_LOG=turbo_cdn::smart_chunking=debug turbo-cdn dl "https://example.com/file.zip"
```

## Next Steps

- [DNS Cache](/guide/dns-cache) - High-performance DNS resolution
- [API Reference](/api/) - Complete API documentation
