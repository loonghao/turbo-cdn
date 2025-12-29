# DownloadOptions

Configuration options for individual download operations.

## Overview

`DownloadOptions` allows fine-grained control over download behavior for specific operations, separate from the global `TurboCdn` configuration.

## Creating Options

### Default Options

```rust
use turbo_cdn::DownloadOptions;

let options = DownloadOptions::new();
```

### Chained Configuration

```rust
use turbo_cdn::DownloadOptions;
use std::time::Duration;

let options = DownloadOptions::new()
    .with_max_concurrent_chunks(16)
    .with_chunk_size(2 * 1024 * 1024)
    .with_resume(true)
    .with_timeout(Duration::from_secs(120))
    .with_integrity_verification(true);
```

## Methods

### `new()`

Create new options with defaults.

```rust
pub fn new() -> Self
```

**Returns:** `DownloadOptions` - Options with default values

**Example:**
```rust
let options = DownloadOptions::new();
```

### `with_max_concurrent_chunks()`

Set maximum concurrent chunks.

```rust
pub fn with_max_concurrent_chunks(self, max: usize) -> Self
```

**Parameters:**
- `max` - Maximum number of concurrent chunk downloads

**Default:** 8

**Example:**
```rust
let options = DownloadOptions::new()
    .with_max_concurrent_chunks(16);
```

### `with_chunk_size()`

Set chunk size in bytes.

```rust
pub fn with_chunk_size(self, size: usize) -> Self
```

**Parameters:**
- `size` - Chunk size in bytes

**Default:** 1 MB (1,048,576 bytes)

**Example:**
```rust
let options = DownloadOptions::new()
    .with_chunk_size(4 * 1024 * 1024);  // 4 MB
```

### `with_resume()`

Enable/disable resume support.

```rust
pub fn with_resume(self, enabled: bool) -> Self
```

**Parameters:**
- `enabled` - Whether to enable resume support

**Default:** true

**Example:**
```rust
let options = DownloadOptions::new()
    .with_resume(true);
```

### `with_timeout()`

Set operation timeout.

```rust
pub fn with_timeout(self, timeout: Duration) -> Self
```

**Parameters:**
- `timeout` - Timeout duration

**Default:** 30 seconds

**Example:**
```rust
use std::time::Duration;

let options = DownloadOptions::new()
    .with_timeout(Duration::from_secs(120));
```

### `with_integrity_verification()`

Enable/disable integrity verification.

```rust
pub fn with_integrity_verification(self, enabled: bool) -> Self
```

**Parameters:**
- `enabled` - Whether to verify file integrity

**Default:** false

**Example:**
```rust
let options = DownloadOptions::new()
    .with_integrity_verification(true);
```

### `with_header()`

Add a custom HTTP header.

```rust
pub fn with_header(self, name: &str, value: &str) -> Self
```

**Parameters:**
- `name` - Header name
- `value` - Header value

**Example:**
```rust
let options = DownloadOptions::new()
    .with_header("Accept", "application/octet-stream")
    .with_header("Authorization", "Bearer token123");
```

### `with_adaptive_concurrency()`

Enable/disable adaptive concurrency.

```rust
pub fn with_adaptive_concurrency(self, enabled: bool) -> Self
```

**Parameters:**
- `enabled` - Whether to enable adaptive concurrency

**Default:** true

**Example:**
```rust
let options = DownloadOptions::new()
    .with_adaptive_concurrency(true);
```

## Usage Examples

### Large File Download

```rust
use turbo_cdn::*;
use std::time::Duration;

#[tokio::main]
async fn main() -> turbo_cdn::Result<()> {
    let downloader = TurboCdn::new().await?;
    
    let options = DownloadOptions::new()
        .with_max_concurrent_chunks(16)
        .with_chunk_size(4 * 1024 * 1024)  // 4 MB chunks
        .with_resume(true)
        .with_timeout(Duration::from_secs(300))  // 5 minute timeout
        .with_adaptive_concurrency(true);
    
    let result = downloader.download_with_options(
        "https://example.com/large-file.zip",
        "./downloads/large-file.zip",
        options
    ).await?;
    
    println!("Downloaded: {}", result.path.display());
    Ok(())
}
```

### Small File Download

```rust
use turbo_cdn::*;
use std::time::Duration;

#[tokio::main]
async fn main() -> turbo_cdn::Result<()> {
    let downloader = TurboCdn::new().await?;
    
    let options = DownloadOptions::new()
        .with_max_concurrent_chunks(4)
        .with_chunk_size(256 * 1024)  // 256 KB chunks
        .with_timeout(Duration::from_secs(30));
    
    let result = downloader.download_with_options(
        "https://example.com/small-file.txt",
        "./downloads/small-file.txt",
        options
    ).await?;
    
    Ok(())
}
```

### Authenticated Download

```rust
use turbo_cdn::*;

#[tokio::main]
async fn main() -> turbo_cdn::Result<()> {
    let downloader = TurboCdn::new().await?;
    
    let options = DownloadOptions::new()
        .with_header("Authorization", "Bearer your-token-here")
        .with_header("Accept", "application/octet-stream");
    
    let result = downloader.download_with_options(
        "https://api.example.com/private/file.zip",
        "./downloads/file.zip",
        options
    ).await?;
    
    Ok(())
}
```

### Unstable Network

```rust
use turbo_cdn::*;
use std::time::Duration;

#[tokio::main]
async fn main() -> turbo_cdn::Result<()> {
    let downloader = TurboCdn::new().await?;
    
    // Conservative settings for unstable networks
    let options = DownloadOptions::new()
        .with_max_concurrent_chunks(4)      // Fewer concurrent
        .with_chunk_size(256 * 1024)        // Smaller chunks
        .with_resume(true)                   // Enable resume
        .with_timeout(Duration::from_secs(60))
        .with_adaptive_concurrency(true);   // Let system adapt
    
    let result = downloader.download_with_options(
        "https://example.com/file.zip",
        "./downloads/file.zip",
        options
    ).await?;
    
    Ok(())
}
```

## Default Values

| Option | Default |
|--------|---------|
| `max_concurrent_chunks` | 8 |
| `chunk_size` | 1 MB |
| `resume` | true |
| `timeout` | 30 seconds |
| `integrity_verification` | false |
| `adaptive_concurrency` | true |
| `headers` | Empty |

## Best Practices

### For Large Files (> 100 MB)

```rust
let options = DownloadOptions::new()
    .with_max_concurrent_chunks(16)
    .with_chunk_size(4 * 1024 * 1024)
    .with_resume(true)
    .with_adaptive_concurrency(true);
```

### For Small Files (< 10 MB)

```rust
let options = DownloadOptions::new()
    .with_max_concurrent_chunks(4)
    .with_chunk_size(512 * 1024);
```

### For Slow Networks

```rust
let options = DownloadOptions::new()
    .with_max_concurrent_chunks(4)
    .with_chunk_size(256 * 1024)
    .with_resume(true)
    .with_timeout(Duration::from_secs(120));
```

### For Fast Networks

```rust
let options = DownloadOptions::new()
    .with_max_concurrent_chunks(32)
    .with_chunk_size(8 * 1024 * 1024)
    .with_adaptive_concurrency(true);
```

## See Also

- [TurboCdn](/api/turbo-cdn) - Main client documentation
- [Smart Chunking](/guide/smart-chunking) - Chunk size optimization
- [Adaptive Concurrency](/guide/adaptive-concurrency) - Dynamic parallelization
