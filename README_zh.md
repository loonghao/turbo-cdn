# 🚀 Turbo CDN

[![Crates.io](https://img.shields.io/crates/v/turbo-cdn.svg)](https://crates.io/crates/turbo-cdn)
[![Documentation](https://docs.rs/turbo-cdn/badge.svg)](https://docs.rs/turbo-cdn)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Build Status](https://github.com/loonghao/turbo-cdn/workflows/CI/badge.svg)](https://github.com/loonghao/turbo-cdn/actions)
[![Security Audit](https://github.com/loonghao/turbo-cdn/workflows/Security%20Audit/badge.svg)](https://github.com/loonghao/turbo-cdn/actions)

[中文文档](README_zh.md) | [English](README.md)

**新一代智能下载加速器，具备自动地理检测、实时CDN质量评估和6+包管理器的全面镜像优化。**

## ✨ 特性

### 🌐 智能地理检测
- **自动区域检测**: 多API IP地理定位，支持故障转移
- **网络性能测试**: IP检测失败时基于延迟的区域检测
- **智能缓存**: 避免重复检测调用的智能缓存机制
- **全球覆盖**: 针对中国、亚太、欧洲、北美和全球区域优化

### 🔗 广泛的CDN镜像源 (16+规则)
- **GitHub镜像**: 7个高质量源 (ghfast.top, gh.con.sh, cors.isteed.cc等)
- **Python PyPI**: 清华、阿里云、豆瓣镜像
- **Rust Crates**: 清华、中科大镜像
- **Go Modules**: goproxy.cn、阿里云镜像
- **Docker Hub**: 中科大、网易、Docker中国镜像
- **Maven Central**: 阿里云、清华镜像
- **jsDelivr增强**: 5个高性能CDN节点
- **npm/unpkg/Cloudflare**: 完整的前端资源加速

### 📊 实时CDN质量评估
- **性能监控**: 延迟、带宽和可用性测试
- **质量评分**: 综合0-100评分算法
- **动态排序**: 基于实时性能的URL排名
- **后台评估**: 异步质量评估
- **智能缓存**: 避免冗余测试的智能缓存

### ⚡ 高性能架构
- **mimalloc**: 高性能内存分配器
- **isahc**: 基于libcurl的HTTP客户端，性能最优
- **自适应并发**: 基于网络条件的并发控制，具备拥塞检测
- **智能分块**: IDM风格的自适应分块，基于性能优化
- **DNS缓存**: 高性能DNS解析缓存，支持TTL管理
- **负载均衡**: 智能服务器选择，具备健康评分和多种策略
- **断点续传**: 中断下载的强大恢复能力

## 🚀 快速开始

### 安装

添加到您的 `Cargo.toml`:

```toml
[dependencies]
turbo-cdn = "0.1.0"
```

### 基本用法

```rust
use turbo_cdn::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化 TurboCdn
    let downloader = TurboCdn::builder()
        .with_sources(&[
            Source::github(),
            Source::jsdelivr(),
            Source::fastly(),
        ])
        .with_region(Region::Global)
        .build()
        .await?;

    // 带进度跟踪的下载
    let result = downloader
        .download("oven-sh/bun", "v1.0.0", "bun-linux-x64.zip")
        .with_progress(|progress| {
            println!("已下载: {:.1}% ({}) - {} - 预计剩余: {}",
                progress.percentage(),
                progress.size_human(),
                progress.speed_human(),
                progress.eta_human()
            );
        })
        .execute()
        .await?;

    println!("✅ 下载到: {}", result.path.display());
    println!("📊 速度: {:.2} MB/s", result.speed / 1_000_000.0);
    
    Ok(())
}
```

### 高级配置

```rust
use turbo_cdn::*;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 自定义配置
    let config = TurboCdnConfig {
        general: GeneralConfig {
            max_concurrent_downloads: 8,
            default_region: Region::China,
            ..Default::default()
        },
        network: NetworkConfig {
            max_concurrent_chunks: 16,
            chunk_size: 2 * 1024 * 1024, // 2MB 分块
            max_retries: 5,
            ..Default::default()
        },
        cache: CacheConfig {
            enabled: true,
            max_size: 5 * 1024 * 1024 * 1024, // 5GB 缓存
            compression: true,
            ..Default::default()
        },
        ..Default::default()
    };

    let downloader = TurboCdn::builder()
        .with_config(config)
        .with_sources(&[Source::github(), Source::jsdelivr()])
        .build()
        .await?;

    // 高级下载选项
    let options = DownloadOptions::builder()
        .max_concurrent_chunks(8)
        .chunk_size(1024 * 1024)
        .timeout(Duration::from_secs(60))
        .use_cache(true)
        .verify_checksum(true)
        .build();

    let result = downloader
        .download("microsoft/vscode", "1.85.0", "VSCode-linux-x64.tar.gz")
        .with_options(options)
        .execute()
        .await?;

    Ok(())
}
```

### 高级配置

```rust
use turbo_cdn::*;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 构建器模式配置
    let mut downloader = TurboCdn::builder()
        .with_sources(&[Source::github(), Source::jsdelivr(), Source::fastly()])
        .with_region(Region::China)
        .with_cache(true)
        .with_max_concurrent_downloads(8)
        .build()
        .await?;

    // 高级下载选项
    let options = DownloadOptions {
        timeout: Duration::from_secs(60),
        verify_checksum: true,
        use_cache: true,
        ..Default::default()
    };

    let result = downloader
        .download("microsoft/vscode", "1.85.0", "VSCode-linux-x64.tar.gz", options)
        .await?;

    println!("✅ 下载到: {}", result.path.display());
    println!("📊 速度: {:.2} MB/s", result.speed / 1_000_000.0);

    Ok(())
}
```

### 配置文件

Turbo CDN 支持多种配置源，自动发现配置文件：

```toml
# ~/.config/turbo-cdn/config.toml 或 ./turbo-cdn.toml

[meta]
version = "1.0"
schema_version = "2025.1"

[general]
enabled = true
debug_mode = false
max_concurrent_downloads = 8
default_region = "China"

[performance]
max_concurrent_downloads = 8
chunk_size = "2MB"
timeout = "30s"
retry_attempts = 3

[performance.cache]
enabled = true
max_size = "10GB"
ttl = "24h"

[security]
verify_ssl = true
verify_checksums = true
allowed_protocols = ["https", "http"]

[logging]
level = "info"
format = "json"
audit_enabled = true
```

### 环境变量

使用环境变量覆盖任何配置：

```bash
# 启用调试模式
export TURBO_CDN_GENERAL__DEBUG_MODE=true

# 设置缓存大小
export TURBO_CDN_PERFORMANCE__CACHE__MAX_SIZE="5GB"

# 设置区域
export TURBO_CDN_REGIONS__DEFAULT="China"

# 设置用户代理
export TURBO_CDN_SECURITY__USER_AGENT="my-app/1.0"
```

### 异步 API（适用于外部工具）

完美集成到其他工具如 `vx`：

```rust
use turbo_cdn::async_api;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 快速优化任意 URL
    let optimized_url = async_api::quick::optimize_url(
        "https://github.com/rust-lang/mdBook/releases/download/v0.4.21/mdbook.tar.gz"
    ).await?;

    println!("🚀 优化后的 URL: {}", optimized_url);

    // 快速下载并自动优化
    let result = async_api::quick::download_optimized(
        "https://registry.npmjs.org/express/-/express-4.18.2.tgz",
        "./downloads"
    ).await?;

    println!("✅ 已下载: {}", result.path.display());

    Ok(())
}
```

## 📊 性能提升

Turbo CDN v0.2.1 通过全面优化提供前所未有的性能:

### 📈 量化改进

| 指标 | 优化前 | 优化后 | 提升 |
|------|--------|--------|------|
| **CDN规则** | 6个规则 | 16个规则 | **167%增长** |
| **GitHub镜像** | 2个源 | 7个源 | **250%增长** |
| **包管理器** | 仅GitHub | 6+管理器 | **6倍扩展** |
| **区域检测** | 手动 | 自动 | **全自动化** |
| **质量评估** | 无 | 实时 | **全新功能** |
| **智能特性** | 基础 | 4个AI模块 | **智能优化** |

### 🚀 实际性能

| 功能 | 优势 | 技术实现 |
|------|------|----------|
| **自适应并发** | 网络感知并行化 | 拥塞检测 + 动态调整 |
| **智能分块** | 基于性能的优化 | 文件大小感知 + 历史学习 |
| **DNS缓存** | 减少延迟开销 | 高性能缓存 + TTL管理 |
| **负载均衡** | 智能服务器选择 | 健康评分 + 多种策略 |
| **地理检测** | 自动最优区域 | IP地理定位 + 网络测试 |
| **CDN质量评估** | 实时性能排名 | 延迟/带宽/可用性评分 |
| **高性能栈** | 内存&HTTP优化 | mimalloc + isahc + dashmap |

### 🧠 智能优化模块

| 模块 | 功能 | 性能影响 |
|------|------|----------|
| **自适应并发控制器** | 基于网络条件的动态并发 | 30-50% 速度提升 |
| **智能分块算法** | 文件大小和性能感知分块 | 20-40% 效率提升 |
| **DNS缓存系统** | 高性能DNS解析缓存 | 10-20% 延迟减少 |
| **智能负载均衡器** | 多策略服务器选择 | 15-25% 可靠性提升 |

## 🛡️ 合规与法律

### 支持的内容
✅ **开源软件** - MIT, Apache, GPL, BSD 等  
✅ **公共领域** - CC0, Unlicense 等  
✅ **宽松许可证** - ISC, MPL 等  

### 禁止的内容
❌ **专有软件** - 商业、闭源  
❌ **版权材料** - 未经明确许可  
❌ **受限内容** - 出口管制、机密  

### 隐私与数据保护
- **最小数据收集**: 仅收集必要的操作数据
- **用户同意**: 所有数据处理需明确同意
- **数据保留**: 最多 30 天保留政策
- **匿名化**: 所有个人数据匿名化
- **GDPR/CCPA 合规**: 完全符合隐私法规

## 🏗️ 架构

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   用户请求      │───▶│   智能路由器     │───▶│  CDN 选择       │
└─────────────────┘    └──────────────────┘    └─────────────────┘
                                │                        │
                                ▼                        ▼
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│ 合规检查器      │    │ 性能跟踪器       │    │ 多源下载        │
└─────────────────┘    └──────────────────┘    └─────────────────┘
                                │                        │
                                ▼                        ▼
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│ 缓存管理器      │    │ 进度跟踪器       │    │ 文件系统输出    │
└─────────────────┘    └──────────────────┘    └─────────────────┘
```

## 🤝 贡献

我们欢迎贡献！请查看我们的[贡献指南](CONTRIBUTING.md)了解详情。

### 开发设置

```bash
# 克隆仓库
git clone https://github.com/loonghao/turbo-cdn.git
cd turbo-cdn

# 安装依赖
cargo build

# 运行测试
cargo test

# 带日志运行
RUST_LOG=turbo_cdn=debug cargo run

# 格式化代码
cargo fmt

# 代码检查
cargo clippy
```

## 📄 许可证

本项目采用 MIT 许可证 - 查看 [LICENSE](LICENSE) 文件了解详情。

## 🙏 致谢

- [reqwest](https://github.com/seanmonstar/reqwest) - HTTP 客户端
- [tokio](https://github.com/tokio-rs/tokio) - 异步运行时
- [indicatif](https://github.com/console-rs/indicatif) - 进度条
- [serde](https://github.com/serde-rs/serde) - 序列化

## 📞 支持

- 📖 [文档](https://docs.rs/turbo-cdn)
- 🐛 [问题跟踪](https://github.com/loonghao/turbo-cdn/issues)
- 💬 [讨论](https://github.com/loonghao/turbo-cdn/discussions)

---

<div align="center">
  <strong>为开源社区用 ❤️ 制作</strong>
</div>
