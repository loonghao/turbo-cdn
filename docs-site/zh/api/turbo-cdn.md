# TurboCdn

智能下载加速的主客户端。

## 概述

`TurboCdn` 是所有下载操作的主要接口。它处理：
- 地理检测
- CDN 质量评估
- 智能下载方式选择
- 并发分块下载

## 创建客户端

### 默认配置

```rust
use turbo_cdn::TurboCdn;

#[tokio::main]
async fn main() -> turbo_cdn::Result<()> {
    let downloader = TurboCdn::new().await?;
    Ok(())
}
```

### Builder 模式

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

## 方法

### `new()`

使用默认设置创建新客户端。

```rust
pub async fn new() -> Result<Self>
```

**返回：** `Result<TurboCdn>` - 配置好的客户端

**示例：**
```rust
let downloader = TurboCdn::new().await?;
```

### `builder()`

创建用于自定义配置的构建器。

```rust
pub fn builder() -> TurboCdnBuilder
```

**返回：** `TurboCdnBuilder` - 构建器实例

**示例：**
```rust
let downloader = TurboCdn::builder()
    .with_timeout(120)
    .build()
    .await?;
```

### `download_from_url()`

使用自动 CDN 优化下载文件。

```rust
pub async fn download_from_url(&self, url: &str) -> Result<DownloadResult>
```

**参数：**
- `url` - 要下载的 URL

**返回：** `Result<DownloadResult>` - 包含路径、大小、速度等的下载结果

**示例：**
```rust
let result = downloader.download_from_url(
    "https://github.com/user/repo/releases/download/v1.0/file.zip"
).await?;
println!("下载到: {}", result.path.display());
```

### `download_to_path()`

下载文件到指定路径。

```rust
pub async fn download_to_path(&self, url: &str, path: impl AsRef<Path>) -> Result<DownloadResult>
```

**参数：**
- `url` - 要下载的 URL
- `path` - 目标文件路径

**返回：** `Result<DownloadResult>` - 下载结果

**示例：**
```rust
let result = downloader.download_to_path(
    "https://example.com/file.zip",
    "./downloads/file.zip"
).await?;
```

### `download_with_options()`

使用自定义选项下载。

```rust
pub async fn download_with_options(
    &self,
    url: &str,
    path: impl AsRef<Path>,
    options: DownloadOptions
) -> Result<DownloadResult>
```

**参数：**
- `url` - 要下载的 URL
- `path` - 目标文件路径
- `options` - 自定义下载选项

**返回：** `Result<DownloadResult>` - 下载结果

**示例：**
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

使用智能模式下载（自动方式选择）。

```rust
pub async fn download_smart(&self, url: &str) -> Result<DownloadResult>
```

**参数：**
- `url` - 要下载的 URL

**返回：** `Result<DownloadResult>` - 下载结果

**示例：**
```rust
let result = downloader.download_smart("https://example.com/file.zip").await?;
```

### `download_direct_from_url()`

直接下载，不使用 CDN 优化。

```rust
pub async fn download_direct_from_url(&self, url: &str) -> Result<DownloadResult>
```

**参数：**
- `url` - 要下载的 URL

**返回：** `Result<DownloadResult>` - 下载结果

**示例：**
```rust
let result = downloader.download_direct_from_url("https://example.com/file.zip").await?;
```

### `get_optimal_url()`

获取最优 CDN URL 而不下载。

```rust
pub async fn get_optimal_url(&self, url: &str) -> Result<String>
```

**参数：**
- `url` - 要优化的 URL

**返回：** `Result<String>` - 优化后的 URL

**示例：**
```rust
let optimal = downloader.get_optimal_url(
    "https://github.com/user/repo/releases/download/v1.0/file.zip"
).await?;
println!("最优 URL: {}", optimal);
```

### `get_stats()`

获取下载统计。

```rust
pub async fn get_stats(&self) -> DownloadStats
```

**返回：** `DownloadStats` - 下载相关统计

**示例：**
```rust
let stats = downloader.get_stats().await;
println!("总下载次数: {}", stats.total_downloads);
println!("成功率: {:.1}%", stats.success_rate());
```

### `get_performance_summary()`

获取性能摘要。

```rust
pub fn get_performance_summary(&self) -> PerformanceSummary
```

**返回：** `PerformanceSummary` - 性能指标

**示例：**
```rust
let summary = downloader.get_performance_summary();
println!("服务器总数: {}", summary.total_servers);
if let Some((url, score)) = summary.best_server {
    println!("最佳服务器: {} (分数: {:.2})", url, score);
}
```

## Builder 方法

### `with_region()`

设置地理区域。

```rust
pub fn with_region(self, region: Region) -> Self
```

### `with_max_concurrent_downloads()`

设置最大并发下载数。

```rust
pub fn with_max_concurrent_downloads(self, max: usize) -> Self
```

### `with_chunk_size()`

设置分块大小（字节）。

```rust
pub fn with_chunk_size(self, size: usize) -> Self
```

### `with_timeout()`

设置超时时间（秒）。

```rust
pub fn with_timeout(self, seconds: u64) -> Self
```

### `with_adaptive_chunking()`

启用/禁用自适应分块。

```rust
pub fn with_adaptive_chunking(self, enabled: bool) -> Self
```

### `with_retry_attempts()`

设置重试次数。

```rust
pub fn with_retry_attempts(self, attempts: u32) -> Self
```

### `with_user_agent()`

设置自定义 User Agent。

```rust
pub fn with_user_agent(self, user_agent: &str) -> Self
```

### `build()`

构建客户端。

```rust
pub async fn build(self) -> Result<TurboCdn>
```

## 类型

### `Region`

地理区域枚举。

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

下载操作的结果。

```rust
pub struct DownloadResult {
    pub path: PathBuf,      // 下载的文件路径
    pub size: u64,          // 文件大小（字节）
    pub speed: f64,         // 平均速度（字节/秒）
    pub duration: Duration, // 总下载时间
    pub resumed: bool,      // 是否为续传
}
```

### `DownloadStats`

下载统计。

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

性能摘要。

```rust
pub struct PerformanceSummary {
    pub total_servers: usize,
    pub overall_success_rate: f64,
    pub best_server: Option<(String, f64)>,
}
```

## 另请参阅

- [DownloadOptions](/zh/api/download-options) - 下载配置选项
- [API 概览](/zh/api/) - 完整 API 参考
