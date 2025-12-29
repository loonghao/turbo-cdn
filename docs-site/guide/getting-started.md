# Quick Start

Get started with Turbo CDN in just a few minutes.

## Installation

### From Crates.io (Recommended)

```bash
cargo install turbo-cdn
```

### From Source

```bash
git clone https://github.com/loonghao/turbo-cdn.git
cd turbo-cdn
cargo build --release
```

The binary will be available at `target/release/turbo-cdn`.

## CLI Usage

### Smart Download (Default)

The default mode automatically selects the best download method:

```bash
turbo-cdn dl "https://github.com/BurntSushi/ripgrep/releases/download/14.1.1/ripgrep-14.1.1-x86_64-pc-windows-msvc.zip"
```

This will:
1. Detect your geographic region
2. Find available CDN mirrors
3. Test performance of each option
4. Select the fastest method
5. Download with progress tracking

### Get Optimized URL

Get the best CDN URL without downloading:

```bash
turbo-cdn optimize "https://github.com/user/repo/releases/download/v1.0/file.zip"
```

### Download Options

```bash
# Download to specific location
turbo-cdn dl "https://example.com/file.zip" "./downloads/file.zip"

# Verbose output with detailed information
turbo-cdn dl "https://example.com/file.zip" --verbose

# Force direct download (bypass CDN)
turbo-cdn dl "https://example.com/file.zip" --no-cdn

# Force CDN download
turbo-cdn dl "https://example.com/file.zip" --force-cdn
```

### View Statistics

```bash
turbo-cdn stats
```

## Library Usage

### Add Dependency

```toml
[dependencies]
turbo-cdn = "0.5"
```

### Basic Usage

```rust
use turbo_cdn::*;

#[tokio::main]
async fn main() -> turbo_cdn::Result<()> {
    // Create client with default settings
    let downloader = TurboCdn::new().await?;

    // Download with automatic CDN optimization
    let result = downloader.download_from_url(
        "https://github.com/BurntSushi/ripgrep/releases/download/14.1.1/ripgrep-14.1.1-x86_64-pc-windows-msvc.zip"
    ).await?;

    println!("Downloaded {} bytes to: {}", result.size, result.path.display());
    println!("Speed: {:.2} MB/s", result.speed / 1024.0 / 1024.0);

    Ok(())
}
```

### Builder Pattern

For full control over configuration:

```rust
use turbo_cdn::*;

#[tokio::main]
async fn main() -> turbo_cdn::Result<()> {
    let downloader = TurboCdn::builder()
        .with_region(Region::China)           // Set region explicitly
        .with_max_concurrent_downloads(16)    // Configure concurrency
        .with_chunk_size(2 * 1024 * 1024)     // 2MB chunks
        .with_timeout(60)                     // 60 second timeout
        .with_adaptive_chunking(true)         // Enable adaptive chunking
        .with_retry_attempts(5)               // Retry up to 5 times
        .with_user_agent("my-app/1.0")        // Custom user agent
        .build()
        .await?;

    let result = downloader.download_to_path(
        "https://github.com/user/repo/releases/download/v1.0.0/file.zip",
        "./downloads/file.zip"
    ).await?;

    println!("Downloaded to: {}", result.path.display());
    Ok(())
}
```

### Advanced Options

```rust
use turbo_cdn::*;
use std::time::Duration;

#[tokio::main]
async fn main() -> turbo_cdn::Result<()> {
    let downloader = TurboCdn::new().await?;

    let options = DownloadOptions::new()
        .with_max_concurrent_chunks(16)
        .with_chunk_size(2 * 1024 * 1024)     // 2MB chunks
        .with_resume(true)                     // Enable resume support
        .with_timeout(Duration::from_secs(120))
        .with_integrity_verification(true)
        .with_header("Accept", "application/octet-stream");

    let result = downloader.download_with_options(
        "https://example.com/large-file.zip",
        "./downloads/file.zip",
        options
    ).await?;

    println!("Downloaded to: {}", result.path.display());
    println!("Speed: {:.2} MB/s", result.speed / 1024.0 / 1024.0);
    
    if result.resumed {
        println!("Download was resumed from previous state");
    }

    Ok(())
}
```

### Quick API

For simple one-off operations:

```rust
use turbo_cdn::async_api;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Quick URL optimization
    let optimized_url = async_api::quick::optimize_url(
        "https://github.com/user/repo/releases/download/v1.0/file.zip"
    ).await?;
    println!("Optimized URL: {}", optimized_url);

    // Quick download
    let result = async_api::quick::download_url(
        "https://github.com/user/repo/releases/download/v1.0/file.zip"
    ).await?;
    println!("Downloaded: {}", result.path.display());

    Ok(())
}
```

## Next Steps

- [Installation Details](/guide/installation) - Platform-specific installation
- [Geographic Detection](/guide/geo-detection) - How region detection works
- [CDN Quality Assessment](/guide/cdn-quality) - Understanding quality scoring
- [API Reference](/api/) - Complete API documentation
