---
layout: home

hero:
  name: "Turbo CDN"
  text: "智能下载加速器"
  tagline: 新一代下载工具，自动 CDN 优化、地理检测和实时质量评估
  image:
    src: /logo.svg
    alt: Turbo CDN
  actions:
    - theme: brand
      text: 快速开始
      link: /zh/guide/
    - theme: alt
      text: GitHub
      link: https://github.com/loonghao/turbo-cdn
    - theme: alt
      text: API 参考
      link: /zh/api/

features:
  - icon: 🌐
    title: 智能地理检测
    details: 自动 IP 地理定位，多 API 回退，网络性能测试，自动选择最优区域。
  - icon: ⚡
    title: 高性能架构
    details: 基于 mimalloc、reqwest + rustls、自适应并发和智能分块，实现最大下载速度。
  - icon: 📊
    title: 实时 CDN 质量评估
    details: 持续监控延迟、带宽和可用性，动态排名和智能缓存。
  - icon: 🔗
    title: 16+ CDN 镜像源
    details: 全面覆盖 GitHub、PyPI、Crates.io、npm、Docker Hub、Maven 等，支持全球区域。
  - icon: 🧠
    title: 智能下载模式
    details: 基于实时性能测试，自动选择直连或 CDN 下载方式。
  - icon: 🔄
    title: 断点续传支持
    details: 强大的断点续传能力，智能重试机制和故障转移。
---

## 快速开始

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
turbo-cdn dl "https://github.com/user/repo/releases/download/v1.0/file.zip"

# 获取优化后的 CDN URL
turbo-cdn optimize "https://github.com/user/repo/releases/download/v1.0/file.zip"

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

## 支持的包管理器

| 包管理器 | 可用镜像 | 地理优化 |
|---------|---------|---------|
| **GitHub** | 7 个镜像 | 中国、亚洲、全球 |
| **Python PyPI** | 清华、阿里云、豆瓣 | 中国优化 |
| **Rust Crates** | 清华、USTC | 中国优化 |
| **Go Modules** | goproxy.cn、阿里云 | 中国优化 |
| **Docker Hub** | USTC、网易、Docker China | 中国优化 |
| **Maven Central** | 阿里云、清华 | 中国优化 |
| **jsDelivr** | 5 个全球 CDN 节点 | 所有区域 |
| **npm/unpkg** | 多个 CDN 替代 | 全球 |

## 性能亮点

| 指标 | 数值 |
|-----|------|
| **CDN 规则** | 16+ 优化规则 |
| **GitHub 镜像** | 7 个高质量源 |
| **包管理器** | 6+ 支持 |
| **质量评估** | 实时监控 |
| **并发控制** | 自适应 |
| **断点续传** | 完整支持 |
