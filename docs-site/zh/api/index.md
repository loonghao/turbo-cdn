# API 参考

Turbo CDN 完整 API 文档。

## 概览

Turbo CDN 提供 CLI 工具和 Rust 库，用于智能下载加速。

## CLI 命令

### `turbo-cdn download`（别名：`dl`）

使用智能优化下载文件。

```bash
turbo-cdn dl <URL> [OUTPUT] [OPTIONS]
```

**参数：**
- `URL` - 要下载的 URL（必需）
- `OUTPUT` - 输出路径（可选，默认当前目录）

**选项：**
| 选项 | 描述 |
|-----|------|
| `--verbose`, `-v` | 启用详细输出 |
| `--no-cdn` | 强制直接下载（绕过 CDN）|
| `--force-cdn` | 强制 CDN 下载 |
| `--no-smart` | 禁用智能模式 |

**示例：**
```bash
# 智能下载（默认）
turbo-cdn dl "https://github.com/user/repo/releases/download/v1.0/file.zip"

# 下载到指定路径
turbo-cdn dl "https://example.com/file.zip" "./downloads/file.zip"

# 详细输出
turbo-cdn dl "https://example.com/file.zip" --verbose

# 直接下载（无 CDN）
turbo-cdn dl "https://example.com/file.zip" --no-cdn
```

### `turbo-cdn optimize`（别名：`get-optimal-url`）

获取优化的 CDN URL 而不下载。

```bash
turbo-cdn optimize <URL>
```

### `turbo-cdn stats`

显示性能统计。

```bash
turbo-cdn stats
```

## 库 API

### 快速开始

```rust
use turbo_cdn::*;

#[tokio::main]
async fn main() -> turbo_cdn::Result<()> {
    let downloader = TurboCdn::new().await?;
    let result = downloader.download_from_url("https://example.com/file.zip").await?;
    println!("已下载: {}", result.path.display());
    Ok(())
}
```

### 主要类型

| 类型 | 描述 |
|-----|------|
| `TurboCdn` | 下载主客户端 |
| `TurboCdnBuilder` | 配置客户端的构建器 |
| `DownloadOptions` | 单次下载的选项 |
| `DownloadResult` | 下载操作的结果 |
| `Region` | 地理区域枚举 |

### 模块

| 模块 | 描述 |
|-----|------|
| `turbo_cdn` | 主模块，包含 `TurboCdn` 客户端 |
| `turbo_cdn::async_api` | 异步便捷函数 |
| `turbo_cdn::async_api::quick` | 快速一次性操作 |

## 错误处理

### 错误类型

```rust
use turbo_cdn::{Error, Result};

fn handle_download() -> Result<()> {
    // Result<T> 是 std::result::Result<T, turbo_cdn::Error> 的别名
    Ok(())
}
```

### 错误变体

| 错误 | 描述 |
|-----|------|
| `NetworkError` | 网络连接问题 |
| `HttpError` | HTTP 请求/响应错误 |
| `IoError` | 文件系统错误 |
| `ParseError` | URL 或响应解析错误 |
| `TimeoutError` | 操作超时 |
| `ConfigError` | 配置错误 |

### 错误处理示例

```rust
use turbo_cdn::*;

#[tokio::main]
async fn main() {
    let downloader = TurboCdn::new().await.unwrap();
    
    match downloader.download_from_url("https://example.com/file.zip").await {
        Ok(result) => {
            println!("成功: {}", result.path.display());
        }
        Err(e) => {
            eprintln!("下载失败: {}", e);
            match e {
                Error::NetworkError(_) => eprintln!("请检查网络连接"),
                Error::TimeoutError(_) => eprintln!("请重试或增加超时时间"),
                _ => eprintln!("意外错误"),
            }
        }
    }
}
```

## Feature Flags

| Feature | 默认 | 描述 |
|---------|-----|------|
| `rustls-tls` | ✅ | 使用 rustls 进行 TLS |
| `native-tls` | ❌ | 使用系统 TLS |
| `fast-hash` | ✅ | 使用 ahash 加速哈希 |
| `high-performance` | ✅ | 启用所有性能优化 |

### 使用 Features

```toml
# 默认（推荐）
[dependencies]
turbo-cdn = "0.5"

# 使用原生 TLS
[dependencies]
turbo-cdn = { version = "0.5", default-features = false, features = ["native-tls"] }
```

## 另请参阅

- [TurboCdn](/zh/api/turbo-cdn) - 主客户端文档
- [DownloadOptions](/zh/api/download-options) - 下载配置选项
- [指南](/zh/guide/) - 使用指南和教程
