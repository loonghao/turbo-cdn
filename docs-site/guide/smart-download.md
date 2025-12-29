# Smart Download

Turbo CDN's smart download mode automatically selects the best download method.

## Overview

Smart download is the **default mode** in Turbo CDN. It automatically:

1. Tests direct download performance
2. Tests CDN mirror performance
3. Compares results
4. Selects the fastest option
5. Downloads using the optimal method

## How It Works

### Decision Flow

```
┌─────────────────────────────────────────────────────────────┐
│                    Smart Download Mode                       │
├─────────────────────────────────────────────────────────────┤
│  1. URL Analysis                                             │
│     └─ Determine if CDN optimization is available           │
│                                                              │
│  2. Parallel Testing                                         │
│     ├─ Test direct URL (HEAD request)                       │
│     └─ Test CDN mirrors (HEAD requests)                     │
│                                                              │
│  3. Performance Comparison                                   │
│     ├─ Compare latencies                                    │
│     └─ Factor in historical performance                     │
│                                                              │
│  4. Method Selection                                         │
│     ├─ If CDN is 20%+ faster → Use CDN                      │
│     └─ Otherwise → Use direct                               │
│                                                              │
│  5. Download Execution                                       │
│     └─ Download with selected method                        │
└─────────────────────────────────────────────────────────────┘
```

## Usage

### CLI (Default)

```bash
# Smart mode is the default
turbo-cdn dl "https://github.com/user/repo/releases/download/v1.0/file.zip"

# Explicitly enable smart mode (same as default)
turbo-cdn dl "https://github.com/user/repo/releases/download/v1.0/file.zip" --smart

# Verbose output shows decision process
turbo-cdn dl "https://github.com/user/repo/releases/download/v1.0/file.zip" --verbose
```

### Library

```rust
use turbo_cdn::*;

#[tokio::main]
async fn main() -> turbo_cdn::Result<()> {
    let downloader = TurboCdn::new().await?;
    
    // Smart download (default behavior)
    let result = downloader.download_smart("https://github.com/user/repo/releases/download/v1.0/file.zip").await?;
    
    // With verbose output
    let result = downloader.download_smart_with_verbose(
        "https://github.com/user/repo/releases/download/v1.0/file.zip",
        true  // verbose
    ).await?;
    
    Ok(())
}
```

## Download Modes

### Smart Mode (Default)

Automatically selects the best method:

```bash
turbo-cdn dl "https://example.com/file.zip"
```

### Direct Mode

Bypasses all optimization, downloads directly from source:

```bash
turbo-cdn dl "https://example.com/file.zip" --no-cdn
```

```rust
let result = downloader.download_direct_from_url("https://example.com/file.zip").await?;
```

### CDN Mode

Forces CDN optimization (skips comparison):

```bash
turbo-cdn dl "https://example.com/file.zip" --force-cdn
```

```rust
let result = downloader.download_from_url("https://example.com/file.zip").await?;
```

## Decision Criteria

### When CDN is Selected

- CDN latency is **20% or more** faster than direct
- Direct URL is unreachable or slow
- Historical data shows CDN performs better

### When Direct is Selected

- Direct URL is fast (< 100ms latency)
- CDN provides minimal improvement (< 20%)
- No CDN mirrors available for the URL

## Verbose Output

With `--verbose`, you can see the decision process:

```
🧠 Smart Download (Auto-Select Best Method) - Default Mode
=========================================================
Source URL: https://github.com/user/repo/releases/download/v1.0/file.zip
Mode: Smart mode (testing and selecting fastest method)

✓ TurboCdn initialized in smart mode (auto-selecting best method)

Testing download methods...
  Direct URL: 150ms latency
  CDN (ghfast.top): 80ms latency
  
Selected: CDN (ghfast.top) - 47% faster

🎉 Download completed successfully!
   📁 ./file.zip
   📊 25.50 MB (12.30 MB/s)
```

## Performance Optimization

### Test Timeout

Smart mode uses short timeouts for testing:

| Test Type | Timeout |
|-----------|---------|
| HEAD request | 3 seconds |
| Initial bytes | 5 seconds |

### Caching

Test results are cached to avoid repeated testing:

- **Cache Duration**: 5 minutes
- **Cache Key**: URL + region
- **Invalidation**: On network change

## When to Override

### Use `--no-cdn` when:

- You know the direct source is fastest
- CDN mirrors are known to be outdated
- Debugging download issues

### Use `--force-cdn` when:

- Direct source is blocked or slow
- You want consistent CDN behavior
- Testing CDN performance

## Troubleshooting

### Smart Mode Chooses Wrong Method

If smart mode consistently picks the slower option:

1. **Check verbose output**: See actual latencies
2. **Clear cache**: Results may be stale
3. **Report issue**: May indicate a bug

```bash
# See detailed decision process
turbo-cdn dl "https://example.com/file.zip" --verbose

# Force specific mode for comparison
turbo-cdn dl "https://example.com/file.zip" --no-cdn --verbose
turbo-cdn dl "https://example.com/file.zip" --force-cdn --verbose
```

### Slow Testing Phase

If the testing phase takes too long:

- Network may be unstable
- Consider using `--force-cdn` or `--no-cdn` directly

## Next Steps

- [Adaptive Concurrency](/guide/adaptive-concurrency) - Dynamic parallelization
- [Smart Chunking](/guide/smart-chunking) - Intelligent chunk sizing
