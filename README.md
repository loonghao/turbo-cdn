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

[中文文档](README_zh.md) | [English](README.md) | [📖 Documentation](https://loonghao.github.io/turbo-cdn/)

**Next-generation intelligent download accelerator with automatic geographic detection, real-time CDN quality assessment, and comprehensive mirror optimization for 6+ package managers.**

## ✨ Features

- 🌐 **Intelligent Geographic Detection** - Auto-region detection with multiple API fallbacks
- 📊 **Real-time CDN Quality Assessment** - Continuous monitoring with dynamic ranking
- ⚡ **High-Performance Architecture** - mimalloc, reqwest + rustls, adaptive concurrency
- 🔗 **16+ CDN Mirror Sources** - GitHub, PyPI, Crates.io, npm, Docker Hub, Maven, and more
- 🧠 **Smart Download Mode** - Automatic method selection based on performance testing
- 🔄 **Resume Support** - Robust resume capability for interrupted downloads

📖 **[Read the full documentation](https://loonghao.github.io/turbo-cdn/)** for detailed guides and API reference.

## 🚀 Quick Start

### Installation

```bash
# From crates.io
cargo install turbo-cdn

# Or from source
git clone https://github.com/loonghao/turbo-cdn.git
cd turbo-cdn
cargo build --release
```

### CLI Usage

```bash
# Smart download (default - auto-selects best method)
turbo-cdn dl "https://github.com/BurntSushi/ripgrep/releases/download/14.1.1/ripgrep-14.1.1-x86_64-pc-windows-msvc.zip"

# Get optimized CDN URL
turbo-cdn optimize "https://github.com/user/repo/releases/download/v1.0/file.zip"

# Download with verbose output
turbo-cdn dl "https://example.com/file.zip" --verbose

# View performance statistics
turbo-cdn stats
```

### Library Usage

```rust
use turbo_cdn::*;

#[tokio::main]
async fn main() -> turbo_cdn::Result<()> {
    let downloader = TurboCdn::new().await?;
    
    let result = downloader.download_from_url(
        "https://github.com/user/repo/releases/download/v1.0/file.zip"
    ).await?;
    
    println!("Downloaded {} bytes at {:.2} MB/s", 
        result.size, result.speed / 1024.0 / 1024.0);
    Ok(())
}
```

### Feature Flags

```toml
[dependencies]
# Library-friendly defaults (self-update off, rustls ring backend - no cmake/NASM needed)
turbo-cdn = { version = "0.7", features = ["rustls", "fast-hash", "high-performance"] }

# CLI build with self-update enabled
turbo-cdn = { version = "0.7", default-features = false, features = ["rustls", "fast-hash", "high-performance", "self-update"] }

# Windows-friendly native TLS (SChannel) if you prefer not to use rustls
turbo-cdn = { version = "0.7", default-features = false, features = ["native-tls", "fast-hash", "high-performance"] }
```

| Feature | Default | Description |
|---------|---------|-------------|
| `rustls` | Yes | TLS via rustls (ring backend, no cmake/NASM toolchain required) |
| `native-tls` | No | Use native TLS (SChannel/Secure Transport) instead of rustls |
| `fast-hash` | Yes | Use ahash for faster hashing |
| `high-performance` | Yes | Enable high-performance optimizations |
| `self-update` | No | CLI self-update functionality (opt-in for binaries) |

## 📊 Supported Package Managers


| Package Manager | Mirrors | Regions |
|----------------|---------|---------|
| **GitHub** | 7 mirrors | China, Asia, Global |
| **Microsoft Visual Studio downloads** | Direct + configurable mirrors | Global |
| **Python PyPI** | Tsinghua, Aliyun, Douban | China |
| **Rust Crates** | Tsinghua, USTC | China |
| **Go Modules** | goproxy.cn, Aliyun | China |
| **Docker Hub** | USTC, NetEase, Docker China | China |
| **Maven Central** | Aliyun, Tsinghua | China |
| **jsDelivr** | 5 global CDN nodes | Global |


## 📖 Documentation

- **[Getting Started](https://loonghao.github.io/turbo-cdn/guide/)** - Introduction and quick start
- **[Installation](https://loonghao.github.io/turbo-cdn/guide/installation)** - Detailed installation options
- **[Geographic Detection](https://loonghao.github.io/turbo-cdn/guide/geo-detection)** - How region detection works
- **[CDN Quality Assessment](https://loonghao.github.io/turbo-cdn/guide/cdn-quality)** - Understanding quality scoring
- **[Smart Download](https://loonghao.github.io/turbo-cdn/guide/smart-download)** - Automatic method selection
- **[API Reference](https://loonghao.github.io/turbo-cdn/api/)** - Complete API documentation

## 🏗️ Architecture

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   Input URL     │───▶│ Geographic       │───▶│ CDN Quality     │
│                 │    │ Detection        │    │ Assessment      │
└─────────────────┘    └──────────────────┘    └─────────────────┘
                                │                        │
                                ▼                        ▼
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│ URL Mapper      │    │ Smart Download   │    │ Adaptive        │
│ (16+ Rules)     │    │ Selection        │    │ Concurrency     │
└─────────────────┘    └──────────────────┘    └─────────────────┘
```

## 🛡️ Compliance

- ✅ Open Source Software (MIT, Apache, GPL, BSD, etc.)
- ✅ Public Domain (CC0, Unlicense, etc.)
- ❌ Proprietary/Commercial software not supported
- 📋 GDPR/CCPA compliant with minimal data collection

## 🤝 Contributing

We welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for details.

```bash
# Development setup
git clone https://github.com/loonghao/turbo-cdn.git
cd turbo-cdn
cargo build
cargo test
cargo clippy
```

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- [reqwest](https://github.com/seanmonstar/reqwest) - High-performance HTTP client
- [mimalloc](https://github.com/microsoft/mimalloc) - High-performance memory allocator
- [tokio](https://github.com/tokio-rs/tokio) - Async runtime
- [clap](https://github.com/clap-rs/clap) - Command line parsing

## 📞 Support

- 📖 [Documentation](https://loonghao.github.io/turbo-cdn/)
- 📚 [API Docs](https://docs.rs/turbo-cdn)
- 🐛 [Issue Tracker](https://github.com/loonghao/turbo-cdn/issues)
- 💬 [Discussions](https://github.com/loonghao/turbo-cdn/discussions)

---

<div align="center">
  <strong>Made with ❤️ for the open source community</strong>
</div>
