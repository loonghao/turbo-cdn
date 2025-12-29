# 快速开始

几分钟内开始使用 Turbo CDN。

## 安装

### 从 Crates.io（推荐）

```bash
cargo install turbo-cdn
```

### 从源码

```bash
git clone https://github.com/loonghao/turbo-cdn.git
cd turbo-cdn
cargo build --release
```

二进制文件将位于 `target/release/turbo-cdn`。

## CLI 使用

### 智能下载（默认）

默认模式自动选择最佳下载方式：

```bash
turbo-cdn dl "https://github.com/BurntSushi/ripgrep/releases/download/14.1.1/ripgrep-14.1.1-x86_64-pc-windows-msvc.zip"
```

这将：
1. 检测您的地理区域
2. 查找可用的 CDN 镜像
3. 测试每个选项的性能
4. 选择最快的方式
5. 带进度跟踪下载

### 获取优化 URL

获取最佳 CDN URL 而不下载：

```bash
turbo-cdn optimize "https://github.com/user/repo/releases/download/v1.0/file.zip"
```

### 下载选项

```bash
# 下载到指定位置
turbo-cdn dl "https://example.com/file.zip" "./downloads/file.zip"

# 详细输出
turbo-cdn dl "https://example.com/file.zip" --verbose

# 强制直接下载（绕过 CDN）
turbo-cdn dl "https://example.com/file.zip" --no-cdn

# 强制 CDN 下载
turbo-cdn dl "https://example.com/file.zip" --force-cdn
```

### 查看统计

```bash
turbo-cdn stats
```

## 库使用

### 添加依赖

```toml
[dependencies]
turbo-cdn = "0.5"
```

### 基本使用

```rust
use turbo_cdn::*;

#[tokio::main]
async fn main() -> turbo_cdn::Result<()> {
    // 使用默认设置创建客户端
    let downloader = TurboCdn::new().await?;

    // 自动 CDN 优化下载
    let result = downloader.download_from_url(
        "https://github.com/BurntSushi/ripgrep/releases/download/14.1.1/ripgrep-14.1.1-x86_64-pc-windows-msvc.zip"
    ).await?;

    println!("下载 {} 字节到: {}", result.size, result.path.display());
    println!("速度: {:.2} MB/s", result.speed / 1024.0 / 1024.0);

    Ok(())
}
```

### Builder 模式

完全控制配置：

```rust
use turbo_cdn::*;

#[tokio::main]
async fn main() -> turbo_cdn::Result<()> {
    let downloader = TurboCdn::builder()
        .with_region(Region::China)           // 显式设置区域
        .with_max_concurrent_downloads(16)    // 配置并发
        .with_chunk_size(2 * 1024 * 1024)     // 2MB 分块
        .with_timeout(60)                     // 60 秒超时
        .with_adaptive_chunking(true)         // 启用自适应分块
        .with_retry_attempts(5)               // 最多重试 5 次
        .with_user_agent("my-app/1.0")        // 自定义 User Agent
        .build()
        .await?;

    let result = downloader.download_to_path(
        "https://github.com/user/repo/releases/download/v1.0.0/file.zip",
        "./downloads/file.zip"
    ).await?;

    println!("下载到: {}", result.path.display());
    Ok(())
}
```

### 高级选项

```rust
use turbo_cdn::*;
use std::time::Duration;

#[tokio::main]
async fn main() -> turbo_cdn::Result<()> {
    let downloader = TurboCdn::new().await?;

    let options = DownloadOptions::new()
        .with_max_concurrent_chunks(16)
        .with_chunk_size(2 * 1024 * 1024)     // 2MB 分块
        .with_resume(true)                     // 启用断点续传
        .with_timeout(Duration::from_secs(120))
        .with_integrity_verification(true)
        .with_header("Accept", "application/octet-stream");

    let result = downloader.download_with_options(
        "https://example.com/large-file.zip",
        "./downloads/file.zip",
        options
    ).await?;

    println!("下载到: {}", result.path.display());
    println!("速度: {:.2} MB/s", result.speed / 1024.0 / 1024.0);
    
    if result.resumed {
        println!("下载从之前的状态恢复");
    }

    Ok(())
}
```

### 快捷 API

用于简单的一次性操作：

```rust
use turbo_cdn::async_api;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 快速 URL 优化
    let optimized_url = async_api::quick::optimize_url(
        "https://github.com/user/repo/releases/download/v1.0/file.zip"
    ).await?;
    println!("优化后的 URL: {}", optimized_url);

    // 快速下载
    let result = async_api::quick::download_url(
        "https://github.com/user/repo/releases/download/v1.0/file.zip"
    ).await?;
    println!("已下载: {}", result.path.display());

    Ok(())
}
```

## 下一步

- [安装详情](/zh/guide/installation) - 平台特定安装
- [地理检测](/zh/guide/geo-detection) - 区域检测原理
- [CDN 质量评估](/zh/guide/cdn-quality) - 质量评分说明
- [API 参考](/zh/api/) - 完整 API 文档
