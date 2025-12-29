---
layout: home

hero:
  name: "Turbo CDN"
  text: "Intelligent Download Accelerator"
  tagline: Next-generation download tool with automatic CDN optimization, geographic detection, and real-time quality assessment
  image:
    src: /logo.svg
    alt: Turbo CDN
  actions:
    - theme: brand
      text: Get Started
      link: /guide/
    - theme: alt
      text: View on GitHub
      link: https://github.com/loonghao/turbo-cdn
    - theme: alt
      text: API Reference
      link: /api/

features:
  - icon: 🌐
    title: Intelligent Geographic Detection
    details: Automatic IP geolocation with multiple API fallbacks and network performance testing for optimal region selection.
  - icon: ⚡
    title: High-Performance Architecture
    details: Built with mimalloc, reqwest + rustls, adaptive concurrency, and smart chunking for maximum download speed.
  - icon: 📊
    title: Real-time CDN Quality Assessment
    details: Continuous monitoring of latency, bandwidth, and availability with dynamic ranking and smart caching.
  - icon: 🔗
    title: 16+ CDN Mirror Sources
    details: Comprehensive coverage for GitHub, PyPI, Crates.io, npm, Docker Hub, Maven, and more across global regions.
  - icon: 🧠
    title: Smart Download Mode
    details: Automatic method selection between direct and CDN downloads based on real-time performance testing.
  - icon: 🔄
    title: Resume & Retry Support
    details: Robust resume capability for interrupted downloads with intelligent retry mechanisms and failover.
---

## Quick Start

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
turbo-cdn dl "https://github.com/user/repo/releases/download/v1.0/file.zip"

# Get optimized CDN URL
turbo-cdn optimize "https://github.com/user/repo/releases/download/v1.0/file.zip"

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

## Supported Package Managers

| Package Manager | Mirrors Available | Geographic Optimization |
|----------------|-------------------|------------------------|
| **GitHub** | 7 mirrors | China, Asia, Global |
| **Python PyPI** | Tsinghua, Aliyun, Douban | China optimized |
| **Rust Crates** | Tsinghua, USTC | China optimized |
| **Go Modules** | goproxy.cn, Aliyun | China optimized |
| **Docker Hub** | USTC, NetEase, Docker China | China optimized |
| **Maven Central** | Aliyun, Tsinghua | China optimized |
| **jsDelivr** | 5 global CDN nodes | All regions |
| **npm/unpkg** | Multiple CDN alternatives | Global |

## Performance Highlights

| Metric | Value |
|--------|-------|
| **CDN Rules** | 16+ optimization rules |
| **GitHub Mirrors** | 7 high-quality sources |
| **Package Managers** | 6+ supported |
| **Quality Assessment** | Real-time monitoring |
| **Concurrency** | Adaptive control |
| **Resume Support** | Full capability |
