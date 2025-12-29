# DownloadOptions

单次下载操作的配置选项。

## 概述

`DownloadOptions` 允许对特定下载操作进行细粒度控制，独立于全局 `TurboCdn` 配置。

## 创建选项

### 默认选项

```rust
use turbo_cdn::DownloadOptions;

let options = DownloadOptions::new();
```

### 链式配置

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

## 方法

### `new()`

创建带默认值的新选项。

```rust
pub fn new() -> Self
```

**返回：** `DownloadOptions` - 带默认值的选项

**示例：**
```rust
let options = DownloadOptions::new();
```

### `with_max_concurrent_chunks()`

设置最大并发分块数。

```rust
pub fn with_max_concurrent_chunks(self, max: usize) -> Self
```

**参数：**
- `max` - 最大并发分块下载数

**默认值：** 8

**示例：**
```rust
let options = DownloadOptions::new()
    .with_max_concurrent_chunks(16);
```

### `with_chunk_size()`

设置分块大小（字节）。

```rust
pub fn with_chunk_size(self, size: usize) -> Self
```

**参数：**
- `size` - 分块大小（字节）

**默认值：** 1 MB（1,048,576 字节）

**示例：**
```rust
let options = DownloadOptions::new()
    .with_chunk_size(4 * 1024 * 1024);  // 4 MB
```

### `with_resume()`

启用/禁用断点续传。

```rust
pub fn with_resume(self, enabled: bool) -> Self
```

**参数：**
- `enabled` - 是否启用断点续传

**默认值：** true

**示例：**
```rust
let options = DownloadOptions::new()
    .with_resume(true);
```

### `with_timeout()`

设置操作超时。

```rust
pub fn with_timeout(self, timeout: Duration) -> Self
```

**参数：**
- `timeout` - 超时时长

**默认值：** 30 秒

**示例：**
```rust
use std::time::Duration;

let options = DownloadOptions::new()
    .with_timeout(Duration::from_secs(120));
```

### `with_integrity_verification()`

启用/禁用完整性验证。

```rust
pub fn with_integrity_verification(self, enabled: bool) -> Self
```

**参数：**
- `enabled` - 是否验证文件完整性

**默认值：** false

**示例：**
```rust
let options = DownloadOptions::new()
    .with_integrity_verification(true);
```

### `with_header()`

添加自定义 HTTP 头。

```rust
pub fn with_header(self, name: &str, value: &str) -> Self
```

**参数：**
- `name` - 头名称
- `value` - 头值

**示例：**
```rust
let options = DownloadOptions::new()
    .with_header("Accept", "application/octet-stream")
    .with_header("Authorization", "Bearer token123");
```

### `with_adaptive_concurrency()`

启用/禁用自适应并发。

```rust
pub fn with_adaptive_concurrency(self, enabled: bool) -> Self
```

**参数：**
- `enabled` - 是否启用自适应并发

**默认值：** true

**示例：**
```rust
let options = DownloadOptions::new()
    .with_adaptive_concurrency(true);
```

## 使用示例

### 大文件下载

```rust
use turbo_cdn::*;
use std::time::Duration;

#[tokio::main]
async fn main() -> turbo_cdn::Result<()> {
    let downloader = TurboCdn::new().await?;
    
    let options = DownloadOptions::new()
        .with_max_concurrent_chunks(16)
        .with_chunk_size(4 * 1024 * 1024)  // 4 MB 分块
        .with_resume(true)
        .with_timeout(Duration::from_secs(300))  // 5 分钟超时
        .with_adaptive_concurrency(true);
    
    let result = downloader.download_with_options(
        "https://example.com/large-file.zip",
        "./downloads/large-file.zip",
        options
    ).await?;
    
    println!("已下载: {}", result.path.display());
    Ok(())
}
```

### 小文件下载

```rust
use turbo_cdn::*;
use std::time::Duration;

#[tokio::main]
async fn main() -> turbo_cdn::Result<()> {
    let downloader = TurboCdn::new().await?;
    
    let options = DownloadOptions::new()
        .with_max_concurrent_chunks(4)
        .with_chunk_size(256 * 1024)  // 256 KB 分块
        .with_timeout(Duration::from_secs(30));
    
    let result = downloader.download_with_options(
        "https://example.com/small-file.txt",
        "./downloads/small-file.txt",
        options
    ).await?;
    
    Ok(())
}
```

### 需要认证的下载

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

### 不稳定网络

```rust
use turbo_cdn::*;
use std::time::Duration;

#[tokio::main]
async fn main() -> turbo_cdn::Result<()> {
    let downloader = TurboCdn::new().await?;
    
    // 不稳定网络的保守设置
    let options = DownloadOptions::new()
        .with_max_concurrent_chunks(4)      // 较少并发
        .with_chunk_size(256 * 1024)        // 较小分块
        .with_resume(true)                   // 启用续传
        .with_timeout(Duration::from_secs(60))
        .with_adaptive_concurrency(true);   // 让系统自适应
    
    let result = downloader.download_with_options(
        "https://example.com/file.zip",
        "./downloads/file.zip",
        options
    ).await?;
    
    Ok(())
}
```

## 默认值

| 选项 | 默认值 |
|------|--------|
| `max_concurrent_chunks` | 8 |
| `chunk_size` | 1 MB |
| `resume` | true |
| `timeout` | 30 秒 |
| `integrity_verification` | false |
| `adaptive_concurrency` | true |
| `headers` | 空 |

## 最佳实践

### 大文件（> 100 MB）

```rust
let options = DownloadOptions::new()
    .with_max_concurrent_chunks(16)
    .with_chunk_size(4 * 1024 * 1024)
    .with_resume(true)
    .with_adaptive_concurrency(true);
```

### 小文件（< 10 MB）

```rust
let options = DownloadOptions::new()
    .with_max_concurrent_chunks(4)
    .with_chunk_size(512 * 1024);
```

### 慢速网络

```rust
let options = DownloadOptions::new()
    .with_max_concurrent_chunks(4)
    .with_chunk_size(256 * 1024)
    .with_resume(true)
    .with_timeout(Duration::from_secs(120));
```

### 快速网络

```rust
let options = DownloadOptions::new()
    .with_max_concurrent_chunks(32)
    .with_chunk_size(8 * 1024 * 1024)
    .with_adaptive_concurrency(true);
```

## 另请参阅

- [TurboCdn](/zh/api/turbo-cdn) - 主客户端文档
- [智能分块](/zh/guide/smart-chunking) - 分块大小优化
- [自适应并发](/zh/guide/adaptive-concurrency) - 动态并行化
