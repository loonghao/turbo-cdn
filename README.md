# 🚀 Turbo CDN

[![Crates.io](https://img.shields.io/crates/v/turbo-cdn.svg)](https://crates.io/crates/turbo-cdn)
[![Documentation](https://docs.rs/turbo-cdn/badge.svg)](https://docs.rs/turbo-cdn)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Build Status](https://github.com/loonghao/turbo-cdn/workflows/CI/badge.svg)](https://github.com/loonghao/turbo-cdn/actions)
[![Security Audit](https://github.com/loonghao/turbo-cdn/workflows/Security%20Audit/badge.svg)](https://github.com/loonghao/turbo-cdn/actions)

[中文文档](README_zh.md) | [English](README.md)

**Next-generation intelligent download accelerator with automatic geographic detection, real-time CDN quality assessment, and comprehensive mirror optimization for 6+ package managers.**

## ✨ Features

### 🌐 Intelligent Geographic Detection
- **Auto-Region Detection**: Automatic IP geolocation with multiple API fallbacks
- **Network Performance Testing**: Latency-based region detection when IP fails
- **Smart Caching**: Intelligent caching to avoid repeated detection calls
- **Global Coverage**: Optimized for China, Asia-Pacific, Europe, North America, and Global regions

### 🔗 Extensive CDN Mirror Sources (16+ Rules)
- **GitHub Mirrors**: 7 high-quality sources (ghfast.top, gh.con.sh, cors.isteed.cc, etc.)
- **Python PyPI**: Tsinghua, Aliyun, Douban mirrors
- **Rust Crates**: Tsinghua, USTC mirrors
- **Go Modules**: goproxy.cn, Aliyun mirrors
- **Docker Hub**: USTC, NetEase, Docker China mirrors
- **Maven Central**: Aliyun, Tsinghua mirrors
- **jsDelivr Enhanced**: 5 high-performance CDN nodes
- **npm/unpkg/Cloudflare**: Complete frontend resource acceleration

### 📊 Real-time CDN Quality Assessment
- **Performance Monitoring**: Latency, bandwidth, and availability testing
- **Quality Scoring**: Comprehensive 0-100 scoring algorithm
- **Dynamic Sorting**: URL ranking based on real-time performance
- **Background Assessment**: Asynchronous quality evaluation
- **Smart Caching**: Avoid redundant testing with intelligent cache

### ⚡ High-Performance Architecture
- **mimalloc**: High-performance memory allocator
- **isahc**: libcurl-based HTTP client for optimal performance
- **Adaptive Concurrency**: Network condition-based concurrency control
- **Dynamic Chunking**: IDM-style adaptive chunk size adjustment
- **Resume Support**: Robust resume capability for interrupted downloads

## 🚀 Quick Start

### Installation

#### From Crates.io
```bash
cargo install turbo-cdn
```

#### From Source
```bash
git clone https://github.com/loonghao/turbo-cdn.git
cd turbo-cdn
cargo build --release
```

### CLI Usage

Experience next-generation download acceleration:

```bash
# Download with intelligent CDN optimization and geographic detection
turbo-cdn dl "https://github.com/BurntSushi/ripgrep/releases/download/14.1.1/ripgrep-14.1.1-x86_64-pc-windows-msvc.zip"

# Get optimized CDN URL with real-time quality assessment
turbo-cdn optimize "https://github.com/user/repo/releases/download/v1.0/file.zip"

# Download with verbose output showing geographic detection and CDN selection
turbo-cdn dl "https://example.com/file.zip" --verbose

# Download to specific location with progress tracking
turbo-cdn download "https://example.com/file.zip" "./downloads/file.zip"

# View comprehensive performance statistics
turbo-cdn stats

# Show help with all available options
turbo-cdn --help
```

### Library Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
turbo-cdn = "0.2.1"
```

```rust
use turbo_cdn::*;

#[tokio::main]
async fn main() -> turbo_cdn::Result<()> {
    // Create a TurboCdn client
    let downloader = TurboCdn::new().await?;

    // Download with automatic CDN optimization
    let result = downloader.download_from_url(
        "https://github.com/BurntSushi/ripgrep/releases/download/14.1.1/ripgrep-14.1.1-x86_64-pc-windows-msvc.zip"
    ).await?;

    println!("✅ Downloaded {} bytes to: {}", result.size, result.path.display());
    println!("🚀 Speed: {:.2} MB/s", result.speed / 1024.0 / 1024.0);

    // Get optimal CDN URL without downloading
    let optimal_url = downloader.get_optimal_url(
        "https://github.com/user/repo/releases/download/v1.0/file.zip"
    ).await?;

    println!("🌐 Optimal URL: {}", optimal_url);

    Ok(())
}
```

### Comprehensive CDN Optimizations

Turbo CDN now supports 16+ optimization rules across 6+ package managers:

| Package Manager | Mirrors Available | Geographic Optimization |
|----------------|-------------------|------------------------|
| **GitHub** | 7 mirrors (ghfast.top, gh.con.sh, cors.isteed.cc, etc.) | China, Asia, Global |
| **Python PyPI** | Tsinghua, Aliyun, Douban | China region optimized |
| **Rust Crates** | Tsinghua, USTC | China region optimized |
| **Go Modules** | goproxy.cn, Aliyun | China region optimized |
| **Docker Hub** | USTC, NetEase, Docker China | China region optimized |
| **Maven Central** | Aliyun, Tsinghua | China region optimized |
| **jsDelivr** | 5 global CDN nodes | All regions |
| **npm/unpkg** | Multiple CDN alternatives | Global optimization |

**Real-time Quality Assessment**: All mirrors are continuously monitored for latency, bandwidth, and availability with dynamic ranking.

### Advanced Usage

```rust
use turbo_cdn::*;

#[tokio::main]
async fn main() -> turbo_cdn::Result<()> {
    // Create downloader with custom configuration
    let downloader = TurboCdn::new().await?;

    // Download to specific path
    let result = downloader.download_to_path(
        "https://github.com/BurntSushi/ripgrep/releases/download/14.1.1/ripgrep-14.1.1-x86_64-pc-windows-msvc.zip",
        "./downloads/ripgrep.zip"
    ).await?;

    println!("✅ Downloaded to: {}", result.path.display());
    println!("📊 Speed: {:.2} MB/s", result.speed / 1024.0 / 1024.0);
    println!("⏱️  Duration: {:.2}s", result.duration.as_secs_f64());

    if result.resumed {
        println!("🔄 Download was resumed");
    }

    Ok(())
}
```

### Async API for External Tools

Perfect for integration with other tools like `vx`:

```rust
use turbo_cdn::async_api;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Quick URL optimization
    let optimized_url = async_api::quick::optimize_url(
        "https://github.com/BurntSushi/ripgrep/releases/download/14.1.1/ripgrep-14.1.1-x86_64-pc-windows-msvc.zip"
    ).await?;

    println!("🚀 Optimized URL: {}", optimized_url);

    // Quick download with automatic optimization
    let result = async_api::quick::download_url(
        "https://github.com/BurntSushi/ripgrep/releases/download/14.1.1/ripgrep-14.1.1-x86_64-pc-windows-msvc.zip"
    ).await?;

    println!("✅ Downloaded: {}", result.path.display());

    Ok(())
}
```

## 📊 Performance Improvements

Turbo CDN v0.2.1 delivers unprecedented performance through comprehensive optimizations:

### 📈 Quantified Improvements

| Metric | Before | After | Improvement |
|--------|--------|-------|-------------|
| **CDN Rules** | 6 rules | 16 rules | **167% increase** |
| **GitHub Mirrors** | 2 sources | 7 sources | **250% increase** |
| **Package Managers** | GitHub only | 6+ managers | **6x expansion** |
| **Region Detection** | Manual | Automatic | **Full automation** |
| **Quality Assessment** | None | Real-time | **New feature** |
| **Configuration** | Mixed languages | English only | **Internationalized** |

### 🚀 Real-World Performance

| Feature | Benefit | Technical Implementation |
|---------|---------|-------------------------|
| **Geographic Detection** | Auto-optimal region | IP geolocation + network testing |
| **CDN Quality Assessment** | Real-time performance ranking | Latency/bandwidth/availability scoring |
| **Intelligent Concurrency** | Adaptive parallelization | Network condition-based adjustment |
| **High-Performance Stack** | Memory & HTTP optimization | mimalloc + isahc + dashmap |
| **Smart Caching** | Reduced redundant operations | DNS cache + connection pooling |

## 🛡️ Compliance & Legal

### Supported Content
✅ **Open Source Software** - MIT, Apache, GPL, BSD, etc.  
✅ **Public Domain** - CC0, Unlicense, etc.  
✅ **Permissive Licenses** - ISC, MPL, etc.  

### Prohibited Content
❌ **Proprietary Software** - Commercial, closed-source  
❌ **Copyrighted Material** - Without explicit permission  
❌ **Restricted Content** - Export-controlled, classified  

### Privacy & Data Protection
- **Minimal Data Collection**: Only essential operational data
- **User Consent**: Explicit consent for all data processing
- **Data Retention**: 30-day maximum retention policy
- **Anonymization**: All personal data anonymized
- **GDPR/CCPA Compliant**: Full compliance with privacy regulations

## 🌟 Key Features

### 🔍 Universal URL Parsing
- **14+ Package Sources**: GitHub, GitLab, npm, PyPI, Crates.io, Maven, Docker Hub, and more
- **Intelligent Detection**: Automatic source type and format recognition
- **Version Extraction**: Smart version parsing from URLs and filenames
- **Error Handling**: Comprehensive validation and error reporting

### 🌍 Intelligent Geographic Optimization

#### 🇨🇳 China Region (Comprehensive Coverage)
- **GitHub**: 7 high-speed mirrors (ghfast.top, gh.con.sh, cors.isteed.cc, github.moeyy.xyz, mirror.ghproxy.com, ghproxy.net)
- **Python**: Tsinghua, Aliyun, Douban PyPI mirrors
- **Rust**: Tsinghua, USTC Crates mirrors
- **Go**: goproxy.cn, Aliyun Go modules
- **Docker**: USTC, NetEase, Docker China
- **Maven**: Aliyun, Tsinghua Central mirrors

#### 🌐 Global Regions (High-Performance CDN)
- **jsDelivr**: 5 global nodes (fastly, gcore, testingcf, jsdelivr.b-cdn)
- **Cloudflare**: Global edge network optimization
- **Fastly**: High-performance global CDN
- **unpkg**: npm package global distribution
- **Auto-Detection**: IP geolocation + network performance testing

### ⚡ Performance Features
- **Automatic Failover**: Seamless switching when primary CDN fails
- **Load Balancing**: Distributes load across multiple CDN sources
- **Smart Caching**: Cross-CDN caching with compression
- **Parallel Downloads**: Multi-threaded chunked downloading
- **Resume Support**: Robust resume capability for interrupted downloads

## 🏗️ Next-Generation Architecture

Turbo CDN v0.2.1 features an intelligent, high-performance architecture:

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   Input URL     │───▶│ Geographic       │───▶│ CDN Quality     │
│                 │    │ Detection        │    │ Assessment      │
└─────────────────┘    └──────────────────┘    └─────────────────┘
                                │                        │
                                ▼                        ▼
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│ URL Mapper      │    │ Real-time        │    │ Dynamic         │
│ (16+ Rules)     │    │ Performance      │    │ Ranking         │
└─────────────────┘    └──────────────────┘    └─────────────────┘
                                │                        │
                                ▼                        ▼
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│ Intelligent     │    │ High-Performance │    │ Downloaded      │
│ Concurrency     │    │ HTTP Client      │    │ File            │
└─────────────────┘    └──────────────────┘    └─────────────────┘
```

### 🔧 Advanced Components

- **Geographic Detection**: Multi-API IP geolocation with network performance fallback
- **CDN Quality Assessment**: Real-time latency/bandwidth/availability monitoring
- **URL Mapper**: 16+ regex rules covering 6+ package managers
- **Intelligent Concurrency**: Adaptive parallelization based on network conditions
- **High-Performance Stack**: mimalloc + isahc + dashmap for optimal performance

## 🤝 Contributing

We welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for details.

### Development Setup

```bash
# Clone the repository
git clone https://github.com/loonghao/turbo-cdn.git
cd turbo-cdn

# Install dependencies
cargo build

# Run tests
cargo test

# Run URL parsing demo
cargo run --example url_parsing_demo

# Run URL optimization demo
cargo run --example url_optimization

# Run with logging
RUST_LOG=turbo_cdn=debug cargo run

# Format code
cargo fmt

# Lint code
cargo clippy
```

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

### High-Performance Stack
- [isahc](https://github.com/sagebind/isahc) - libcurl-based HTTP client for optimal performance
- [mimalloc](https://github.com/microsoft/mimalloc) - High-performance memory allocator
- [dashmap](https://github.com/xacrimon/dashmap) - Concurrent hash maps
- [tokio](https://github.com/tokio-rs/tokio) - Async runtime

### Core Dependencies
- [figment](https://github.com/SergioBenitez/Figment) - Configuration management
- [tracing](https://github.com/tokio-rs/tracing) - Structured logging
- [serde](https://github.com/serde-rs/serde) - Serialization
- [clap](https://github.com/clap-rs/clap) - Command line parsing

## 📞 Support

- 📖 [Documentation](https://docs.rs/turbo-cdn)
- 🌐 [URL Optimization Guide](docs/URL_OPTIMIZATION.md)
- 🐛 [Issue Tracker](https://github.com/loonghao/turbo-cdn/issues)
- 💬 [Discussions](https://github.com/loonghao/turbo-cdn/discussions)
- 🚀 [Examples](examples/) - URL parsing and optimization demos

---

<div align="center">
  <strong>Made with ❤️ for the open source community</strong>
</div>
