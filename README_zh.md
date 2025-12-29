# Turbo CDN

[![Crates.io](https://img.shields.io/crates/v/turbo-cdn.svg)](https://crates.io/crates/turbo-cdn)
[![Documentation](https://docs.rs/turbo-cdn/badge.svg)](https://docs.rs/turbo-cdn)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Build Status](https://github.com/loonghao/turbo-cdn/workflows/CI/badge.svg)](https://github.com/loonghao/turbo-cdn/actions)
[![Security Audit](https://github.com/loonghao/turbo-cdn/workflows/Security%20Audit/badge.svg)](https://github.com/loonghao/turbo-cdn/actions)
[![codecov](https://codecov.io/gh/loonghao/turbo-cdn/branch/main/graph/badge.svg)](https://codecov.io/gh/loonghao/turbo-cdn)
[![Downloads](https://img.shields.io/crates/d/turbo-cdn.svg)](https://crates.io/crates/turbo-cdn)
[![MSRV](https://img.shields.io/badge/MSRV-1.70-blue.svg)](https://blog.rust-lang.org/2023/06/01/Rust-1.70.0.html)
[![GitHub release](https://img.shields.io/github/v/release/loonghao/turbo-cdn)](https://github.com/loonghao/turbo-cdn/releases)
[![GitHub stars](https://img.shields.io/github/stars/loonghao/turbo-cdn?style=social)](https://github.com/loonghao/turbo-cdn)

[中文文档](README_zh.md) | [English](README.md) | [📖 文档](https://loonghao.github.io/turbo-cdn/zh/)

**新一代智能下载加速器，具备自动地理检测、实时 CDN 质量评估和 6+ 包管理器的全面镜像优化。**

## ✨ 特性

- 🌐 **智能地理检测** - 自动区域检测，多 API 回退
- 📊 **实时 CDN 质量评估** - 持续监控，动态排名
- ⚡ **高性能架构** - mimalloc、reqwest + rustls、自适应并发
- 🔗 **16+ CDN 镜像源** - GitHub、PyPI、Crates.io、npm、Docker Hub、Maven 等
- 🧠 **智能下载模式** - 基于性能测试自动选择方式
- 🔄 **断点续传** - 强大的中断恢复能力

📖 **[阅读完整文档](https://loonghao.github.io/turbo-cdn/zh/)** 获取详细指南和 API 参考。

## 🚀 快速开始

### 安装

```bash
# 从 crates.io 安装
cargo install turbo-cdn

# 或从源码编译
git clone https://github.com/loonghao/turbo-cdn.git
cd turbo-cdn
cargo build --release
```

### CLI 使用

```bash
# 智能下载（默认 - 自动选择最佳方式）
turbo-cdn dl "https://github.com/BurntSushi/ripgrep/releases/download/14.1.1/ripgrep-14.1.1-x86_64-pc-windows-msvc.zip"

# 获取优化后的 CDN URL
turbo-cdn optimize "https://github.com/user/repo/releases/download/v1.0/file.zip"

# 详细输出下载
turbo-cdn dl "https://example.com/file.zip" --verbose

# 查看性能统计
turbo-cdn stats
```

### 库使用

```rust
use turbo_cdn::*;

#[tokio::main]
async fn main() -> turbo_cdn::Result<()> {
    let downloader = TurboCdn::new().await?;
    
    let result = downloader.download_from_url(
        "https://github.com/user/repo/releases/download/v1.0/file.zip"
    ).await?;
    
    println!("下载 {} 字节，速度 {:.2} MB/s", 
        result.size, result.speed / 1024.0 / 1024.0);
    Ok(())
}
```

## 📊 支持的包管理器

| 包管理器 | 镜像数 | 区域 |
|---------|-------|------|
| **GitHub** | 7 个镜像 | 中国、亚洲、全球 |
| **Python PyPI** | 清华、阿里云、豆瓣 | 中国 |
| **Rust Crates** | 清华、USTC | 中国 |
| **Go Modules** | goproxy.cn、阿里云 | 中国 |
| **Docker Hub** | USTC、网易、Docker China | 中国 |
| **Maven Central** | 阿里云、清华 | 中国 |
| **jsDelivr** | 5 个全球 CDN 节点 | 全球 |

## 📖 文档

- **[入门指南](https://loonghao.github.io/turbo-cdn/zh/guide/)** - 简介和快速开始
- **[安装](https://loonghao.github.io/turbo-cdn/zh/guide/installation)** - 详细安装选项
- **[地理检测](https://loonghao.github.io/turbo-cdn/zh/guide/geo-detection)** - 区域检测原理
- **[CDN 质量评估](https://loonghao.github.io/turbo-cdn/zh/guide/cdn-quality)** - 质量评分说明
- **[智能下载](https://loonghao.github.io/turbo-cdn/zh/guide/smart-download)** - 自动方式选择
- **[API 参考](https://loonghao.github.io/turbo-cdn/zh/api/)** - 完整 API 文档

## 🏗️ 架构

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   输入 URL      │───▶│ 地理检测         │───▶│ CDN 质量评估    │
│                 │    │                  │    │                 │
└─────────────────┘    └──────────────────┘    └─────────────────┘
                                │                        │
                                ▼                        ▼
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│ URL 映射器      │    │ 智能下载         │    │ 自适应并发      │
│ (16+ 规则)      │    │ 选择             │    │                 │
└─────────────────┘    └──────────────────┘    └─────────────────┘
```

## 🛡️ 合规性

- ✅ 开源软件（MIT、Apache、GPL、BSD 等）
- ✅ 公共领域（CC0、Unlicense 等）
- ❌ 不支持专有/商业软件
- 📋 符合 GDPR/CCPA，最小化数据收集

## 🤝 贡献

欢迎贡献！请查看 [贡献指南](CONTRIBUTING.md) 了解详情。

```bash
# 开发设置
git clone https://github.com/loonghao/turbo-cdn.git
cd turbo-cdn
cargo build
cargo test
cargo clippy
```

## 📄 许可证

本项目采用 MIT 许可证 - 详见 [LICENSE](LICENSE) 文件。

## 🙏 致谢

- [reqwest](https://github.com/seanmonstar/reqwest) - 高性能 HTTP 客户端
- [mimalloc](https://github.com/microsoft/mimalloc) - 高性能内存分配器
- [tokio](https://github.com/tokio-rs/tokio) - 异步运行时
- [clap](https://github.com/clap-rs/clap) - 命令行解析

## 📞 支持

- 📖 [文档](https://loonghao.github.io/turbo-cdn/zh/)
- 📚 [API 文档](https://docs.rs/turbo-cdn)
- 🐛 [问题追踪](https://github.com/loonghao/turbo-cdn/issues)
- 💬 [讨论](https://github.com/loonghao/turbo-cdn/discussions)

---

<div align="center">
  <strong>为开源社区用 ❤️ 制作</strong>
</div>
