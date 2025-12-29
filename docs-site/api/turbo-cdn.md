# TurboCdn

The main client for intelligent download acceleration.

## Overview

`TurboCdn` is the primary interface for all download operations. It handles:
- Geographic detection
- CDN quality assessment
- Smart download method selection
- Concurrent chunked downloads

## Creating a Client

### Default Configuration

```rust
use turbo_cdn::TurboCdn;

#[tokio::main]
async fn main() -> turbo_cdn::Result<()> {
    let downloader = TurboCdn::new().await?;
    Ok(())
}
```

### Builder Pattern

```rust
use turbo_cdn::*;

#[tokio::main]
async fn main() -> turbo_cdn::Result<()> {
    let downloader = TurboCdn::builder()
        .with_region(Region::China)
        .with_max_concurrent_downloads(16)
        .with_chunk_size(2 * 1024 * 1024)
        .with_timeout(60)
        .with_adaptive_chunking(true)
        .with_retry_attempts(5)
        .with_user_agent("my-app/1.0")
        .build()
        .await?;
    
    Ok(())
}
```

## Methods

### `new()`

Create a new client with default settings.

```rust
pub async fn new() -> Result<Self>
```

**Returns:** `Result<TurboCdn>` - The configured client

**Example:**
```rust
let downloader = TurboCdn::new().await?;
```

### `builder()`

Create a builder for custom configuration.

```rust
pub fn builder() -> TurboCdnBuilder
```

**Returns:** `TurboCdnBuilder` - A builder instance

**Example:**
```rust
let downloader = TurboCdn::builder()
    .with_timeout(120)
    .build()
    .await?;
```

### `download_from_url()`

Download a file with automatic CDN optimization.

```rust
pub async fn download_from_url(&self, url: &str) -> Result<DownloadResult>
```

**Parameters:**
- `url` - The URL to download

**Returns:** `Result<DownloadResult>` - Download result with path, size, speed, etc.

**Example:**
```rust
let result = downloader.download_from_url(
    "https://github.com/user/repo/releases/download/v1.0/file.zip"
).await?;
println!("Downloaded to: {}", result.path.display());
```

### `download_to_path()`

Download a file to a specific path.

```rust
pub async fn download_to_path(&self, url: &str, path: impl AsRef<Path>) -> Result<DownloadResult>
```

**Parameters:**
- `url` - The URL to download
- `path` - Destination file path

**Returns:** `Result<DownloadResult>` - Download result

**Example:**
```rust
let result = downloader.download_to_path(
    "https://example.com/file.zip",
    "./downloads/file.zip"
).await?;
```

### `download_with_options()`

Download with custom options.

```rust
pub async fn download_with_options(
    &self,
    url: &str,
    path: impl AsRef<Path>,
    options: DownloadOptions
) -> Result<DownloadResult>
```

**Parameters:**
- `url` - The URL to download
- `path` - Destination file path
- `options` - Custom download options

**Returns:** `Result<DownloadResult>` - Download result

**Example:**
```rust
let options = DownloadOptions::new()
    .with_resume(true)
    .with_chunk_size(4 * 1024 * 1024);

let result = downloader.download_with_options(
    "https://example.com/file.zip",
    "./downloads/file.zip",
    options
).await?;
```

### `download_smart()`

Download using smart mode (automatic method selection).

```rust
pub async fn download_smart(&self, url: &str) -> Result<DownloadResult>
```

**Parameters:**
- `url` - The URL to download

**Returns:** `Result<DownloadResult>` - Download result

**Example:**
```rust
let result = downloader.download_smart("https://example.com/file.zip").await?;
```

### `download_direct_from_url()`

Download directly without CDN optimization.

```rust
pub async fn download_direct_from_url(&self, url: &str) -> Result<DownloadResult>
```

**Parameters:**
- `url` - The URL to download

**Returns:** `Result<DownloadResult>` - Download result

**Example:**
```rust
let result = downloader.download_direct_from_url("https://example.com/file.zip").await?;
```

### `get_optimal_url()`

Get the optimal CDN URL without downloading.

```rust
pub async fn get_optimal_url(&self, url: &str) -> Result<String>
```

**Parameters:**
- `url` - The URL to optimize

**Returns:** `Result<String>` - The optimized URL

**Example:**
```rust
let optimal = downloader.get_optimal_url(
    "https://github.com/user/repo/releases/download/v1.0/file.zip"
).await?;
println!("Optimal URL: {}", optimal);
```

### `get_stats()`

Get download statistics.

```rust
pub async fn get_stats(&self) -> DownloadStats
```

**Returns:** `DownloadStats` - Statistics about downloads

**Example:**
```rust
let stats = downloader.get_stats().await;
println!("Total downloads: {}", stats.total_downloads);
println!("Success rate: {:.1}%", stats.success_rate());
```

### `get_performance_summary()`

Get performance summary.

```rust
pub fn get_performance_summary(&self) -> PerformanceSummary
```

**Returns:** `PerformanceSummary` - Performance metrics

**Example:**
```rust
let summary = downloader.get_performance_summary();
println!("Total servers: {}", summary.total_servers);
if let Some((url, score)) = summary.best_server {
    println!("Best server: {} (score: {:.2})", url, score);
}
```

## Builder Methods

### `with_region()`

Set the geographic region.

```rust
pub fn with_region(self, region: Region) -> Self
```

### `with_max_concurrent_downloads()`

Set maximum concurrent downloads.

```rust
pub fn with_max_concurrent_downloads(self, max: usize) -> Self
```

### `with_chunk_size()`

Set chunk size in bytes.

```rust
pub fn with_chunk_size(self, size: usize) -> Self
```

### `with_timeout()`

Set timeout in seconds.

```rust
pub fn with_timeout(self, seconds: u64) -> Self
```

### `with_adaptive_chunking()`

Enable/disable adaptive chunking.

```rust
pub fn with_adaptive_chunking(self, enabled: bool) -> Self
```

### `with_retry_attempts()`

Set retry attempts.

```rust
pub fn with_retry_attempts(self, attempts: u32) -> Self
```

### `with_user_agent()`

Set custom user agent.

```rust
pub fn with_user_agent(self, user_agent: &str) -> Self
```

### `build()`

Build the client.

```rust
pub async fn build(self) -> Result<TurboCdn>
```

## Types

### `Region`

Geographic region enum.

```rust
pub enum Region {
    China,
    AsiaPacific,
    Europe,
    NorthAmerica,
    Global,
}
```

### `DownloadResult`

Result of a download operation.

```rust
pub struct DownloadResult {
    pub path: PathBuf,      // Downloaded file path
    pub size: u64,          // File size in bytes
    pub speed: f64,         // Average speed in bytes/sec
    pub duration: Duration, // Total download time
    pub resumed: bool,      // Whether download was resumed
}
```

### `DownloadStats`

Download statistics.

```rust
pub struct DownloadStats {
    pub total_downloads: u64,
    pub successful_downloads: u64,
    pub failed_downloads: u64,
    pub total_bytes: u64,
    pub total_duration: Duration,
}

impl DownloadStats {
    pub fn success_rate(&self) -> f64;
    pub fn average_speed_mbps(&self) -> f64;
}
```

### `PerformanceSummary`

Performance summary.

```rust
pub struct PerformanceSummary {
    pub total_servers: usize,
    pub overall_success_rate: f64,
    pub best_server: Option<(String, f64)>,
}
```

## See Also

- [DownloadOptions](/api/download-options) - Download configuration options
- [API Overview](/api/) - Complete API reference
