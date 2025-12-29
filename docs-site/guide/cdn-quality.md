# CDN Quality Assessment

Turbo CDN continuously monitors CDN performance to select the best mirrors.

## Quality Scoring System

### Scoring Algorithm

Each CDN mirror receives a quality score from 0-100 based on:

| Factor | Weight | Description |
|--------|--------|-------------|
| Latency | 40% | Response time to first byte |
| Bandwidth | 35% | Download speed capability |
| Availability | 25% | Success rate of requests |

### Score Calculation

```
Quality Score = (Latency Score × 0.4) + (Bandwidth Score × 0.35) + (Availability Score × 0.25)
```

Where:
- **Latency Score**: `100 - min(latency_ms / 10, 100)`
- **Bandwidth Score**: `min(bandwidth_mbps × 10, 100)`
- **Availability Score**: `success_rate × 100`

## Real-time Monitoring

### Assessment Flow

```
┌─────────────────────────────────────────────────────────────┐
│                  CDN Quality Assessment                      │
├─────────────────────────────────────────────────────────────┤
│  1. Background Testing                                       │
│     ├─ Periodic health checks                               │
│     ├─ Latency measurement                                  │
│     └─ Bandwidth sampling                                   │
│                                                              │
│  2. Score Calculation                                        │
│     ├─ Weighted scoring algorithm                           │
│     └─ Historical averaging                                 │
│                                                              │
│  3. Dynamic Ranking                                          │
│     ├─ Sort mirrors by score                                │
│     └─ Update selection priority                            │
│                                                              │
│  4. Smart Caching                                            │
│     ├─ Cache scores with TTL                                │
│     └─ Avoid redundant testing                              │
└─────────────────────────────────────────────────────────────┘
```

## Mirror Selection

### Selection Process

1. **Get Available Mirrors**: Based on URL pattern and region
2. **Retrieve Scores**: From cache or perform live assessment
3. **Rank by Score**: Sort mirrors by quality score
4. **Select Best**: Choose highest-scoring mirror
5. **Fallback**: If best fails, try next in ranking

### Example Selection

For a GitHub release download in China:

| Mirror | Latency | Bandwidth | Availability | Score |
|--------|---------|-----------|--------------|-------|
| ghfast.top | 50ms | 10 MB/s | 99% | 87.25 |
| gh.con.sh | 80ms | 8 MB/s | 98% | 79.50 |
| cors.isteed.cc | 100ms | 6 MB/s | 95% | 70.75 |

Result: `ghfast.top` is selected as the primary mirror.

## Configuration

### Default Behavior

```rust
use turbo_cdn::*;

#[tokio::main]
async fn main() -> turbo_cdn::Result<()> {
    // Quality assessment is enabled by default
    let downloader = TurboCdn::new().await?;
    
    // Get optimal URL with quality-based selection
    let optimal = downloader.get_optimal_url("https://github.com/user/repo/releases/download/v1.0/file.zip").await?;
    
    Ok(())
}
```

### Custom Assessment

```rust
use turbo_cdn::*;

#[tokio::main]
async fn main() -> turbo_cdn::Result<()> {
    let downloader = TurboCdn::builder()
        .with_quality_assessment(true)      // Enable (default)
        .with_assessment_timeout(5)         // 5 second timeout per test
        .with_assessment_interval(300)      // Re-assess every 5 minutes
        .build()
        .await?;
    
    Ok(())
}
```

## Performance Statistics

### Accessing Statistics

```rust
use turbo_cdn::*;

#[tokio::main]
async fn main() -> turbo_cdn::Result<()> {
    let downloader = TurboCdn::new().await?;
    
    // Get performance summary
    let summary = downloader.get_performance_summary();
    
    println!("Total servers tracked: {}", summary.total_servers);
    println!("Overall success rate: {:.1}%", summary.overall_success_rate * 100.0);
    
    if let Some((url, score)) = summary.best_server {
        println!("Best server: {} (score: {:.2})", url, score);
    }
    
    Ok(())
}
```

### CLI Statistics

```bash
turbo-cdn stats
```

## Quality Thresholds

| Score Range | Quality | Action |
|-------------|---------|--------|
| 80-100 | Excellent | Primary choice |
| 60-79 | Good | Acceptable fallback |
| 40-59 | Fair | Use if no better option |
| 0-39 | Poor | Avoid if possible |

## Caching Strategy

### Cache Parameters

| Parameter | Default | Description |
|-----------|---------|-------------|
| TTL | 5 minutes | Score validity period |
| Max Entries | 100 | Maximum cached mirrors |
| Refresh | Background | Non-blocking updates |

### Cache Behavior

- **Hit**: Return cached score immediately
- **Miss**: Perform live assessment
- **Stale**: Return cached, refresh in background
- **Expired**: Block until fresh assessment

## Troubleshooting

### Poor Performance

If downloads are slow despite quality assessment:

1. **Check network**: Local network may be the bottleneck
2. **Force refresh**: Clear cached scores
3. **Manual selection**: Override with known-good mirror

```rust
// Force direct download to bypass CDN selection
let downloader = TurboCdn::new().await?;
let result = downloader.download_direct_from_url("https://example.com/file.zip").await?;
```

### Debug Logging

```bash
RUST_LOG=turbo_cdn::cdn_quality=debug turbo-cdn dl "https://example.com/file.zip"
```

## Next Steps

- [Smart Download](/guide/smart-download) - Automatic method selection
- [Adaptive Concurrency](/guide/adaptive-concurrency) - Dynamic parallelization
