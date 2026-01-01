# API Reference

Complete API documentation for Turbo CDN.

## Overview

Turbo CDN provides both a CLI tool and a Rust library for intelligent download acceleration.

## CLI Commands

### `turbo-cdn download` (alias: `dl`)

Download a file with intelligent optimization.

```bash
turbo-cdn dl <URL> [OUTPUT] [OPTIONS]
```

**Arguments:**
- `URL` - URL to download (required)
- `OUTPUT` - Output path (optional, defaults to current directory)

**Options:**
| Option | Description |
|--------|-------------|
| `--verbose`, `-v` | Enable verbose output |
| `--no-cdn` | Force direct download (bypass CDN) |
| `--force-cdn` | Force CDN download |
| `--no-smart` | Disable smart mode |

**Examples:**
```bash
# Smart download (default)
turbo-cdn dl "https://github.com/user/repo/releases/download/v1.0/file.zip"

# Download to specific path
turbo-cdn dl "https://example.com/file.zip" "./downloads/file.zip"

# Verbose output
turbo-cdn dl "https://example.com/file.zip" --verbose

# Direct download (no CDN)
turbo-cdn dl "https://example.com/file.zip" --no-cdn
```

### `turbo-cdn optimize` (alias: `get-optimal-url`)

Get the optimized CDN URL without downloading.

```bash
turbo-cdn optimize <URL>
```

**Arguments:**
- `URL` - URL to optimize (required)

**Examples:**
```bash
turbo-cdn optimize "https://github.com/user/repo/releases/download/v1.0/file.zip"
```

### `turbo-cdn stats`

Show performance statistics.

```bash
turbo-cdn stats
```

### Global Options

| Option | Description |
|--------|-------------|
| `--help`, `-h` | Show help |
| `--version`, `-V` | Show version |
| `--verbose`, `-v` | Enable verbose output |

## Library API

### Quick Start

```rust
use turbo_cdn::*;

#[tokio::main]
async fn main() -> turbo_cdn::Result<()> {
    let downloader = TurboCdn::new().await?;
    let result = downloader.download_from_url("https://example.com/file.zip").await?;
    println!("Downloaded: {}", result.path.display());
    Ok(())
}
```

### Main Types

| Type | Description |
|------|-------------|
| [`TurboCdn`](/api/turbo-cdn) | Main client for downloads |
| [`TurboCdnBuilder`](/api/turbo-cdn#builder) | Builder for configuring client |
| [`DownloadOptions`](/api/download-options) | Options for individual downloads |
| [`DownloadResult`](/api/turbo-cdn#downloadresult) | Result of a download operation |
| [`Region`](/api/turbo-cdn#region) | Geographic region enum |

### Modules

| Module | Description |
|--------|-------------|
| `turbo_cdn` | Main module with `TurboCdn` client |
| `turbo_cdn::async_api` | Async convenience functions |
| `turbo_cdn::async_api::quick` | Quick one-off operations |

## Error Handling

### Error Types

```rust
use turbo_cdn::{Error, Result};

fn handle_download() -> Result<()> {
    // Result<T> is an alias for std::result::Result<T, turbo_cdn::Error>
    Ok(())
}
```

### Error Variants

| Error | Description |
|-------|-------------|
| `NetworkError` | Network connectivity issues |
| `HttpError` | HTTP request/response errors |
| `IoError` | File system errors |
| `ParseError` | URL or response parsing errors |
| `TimeoutError` | Operation timed out |
| `ConfigError` | Configuration errors |

### Error Handling Example

```rust
use turbo_cdn::*;

#[tokio::main]
async fn main() {
    let downloader = TurboCdn::new().await.unwrap();
    
    match downloader.download_from_url("https://example.com/file.zip").await {
        Ok(result) => {
            println!("Success: {}", result.path.display());
        }
        Err(e) => {
            eprintln!("Download failed: {}", e);
            // Handle specific error types
            match e {
                Error::NetworkError(_) => eprintln!("Check your network connection"),
                Error::TimeoutError(_) => eprintln!("Try again or increase timeout"),
                _ => eprintln!("Unexpected error"),
            }
        }
    }
}
```

## Async API

### Quick Functions

```rust
use turbo_cdn::async_api;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Quick URL optimization
    let url = async_api::quick::optimize_url("https://github.com/...").await?;
    
    // Quick download
    let result = async_api::quick::download_url("https://github.com/...").await?;
    
    Ok(())
}
```

## Feature Flags

| Feature | Default | Description |
|---------|---------|-------------|
| `rustls` | ✅ | Use rustls for TLS (no OpenSSL needed) |
| `native-tls` | ❌ | Use system TLS |
| `fast-hash` | ✅ | Use ahash for faster hashing |
| `high-performance` | ✅ | Enable all optimizations |

### Using Features

```toml
# Default (recommended)
[dependencies]
turbo-cdn = "0.7"

# With native TLS
[dependencies]
turbo-cdn = { version = "0.7", default-features = false, features = ["native-tls"] }
```

## Version Compatibility

| Turbo CDN | Rust | MSRV | reqwest |
|-----------|------|------|---------|
| 0.7.x | 1.70+ | 1.70 | 0.13 |
| 0.5.x-0.6.x | 1.70+ | 1.70 | 0.12 |
| 0.4.x | 1.65+ | 1.65 | 0.11 |

## See Also

- [TurboCdn](/api/turbo-cdn) - Main client documentation
- [DownloadOptions](/api/download-options) - Download configuration
- [Guide](/guide/) - Usage guides and tutorials
