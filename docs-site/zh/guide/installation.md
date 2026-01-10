# 安装

根据您的需求，有多种方式安装 Turbo CDN。

## 预编译二进制文件

### GitHub Releases

从 [GitHub Releases](https://github.com/loonghao/turbo-cdn/releases) 下载预编译二进制文件：

| 平台 | 架构 | 下载 |
|------|------|------|
| Linux | x86_64 | `turbo-cdn-x86_64-unknown-linux-gnu.tar.gz` |
| Linux | x86_64 (musl) | `turbo-cdn-x86_64-unknown-linux-musl.tar.gz` |
| Linux | aarch64 | `turbo-cdn-aarch64-unknown-linux-gnu.tar.gz` |
| Linux | aarch64 (musl) | `turbo-cdn-aarch64-unknown-linux-musl.tar.gz` |
| macOS | x86_64 | `turbo-cdn-x86_64-apple-darwin.tar.gz` |
| macOS | aarch64 (Apple Silicon) | `turbo-cdn-aarch64-apple-darwin.tar.gz` |
| Windows | x86_64 | `turbo-cdn-x86_64-pc-windows-msvc.zip` |
| Windows | aarch64 | `turbo-cdn-aarch64-pc-windows-msvc.zip` |

### 安装脚本（即将推出）

```bash
# Linux/macOS
curl -fsSL https://raw.githubusercontent.com/loonghao/turbo-cdn/main/scripts/install.sh | bash

# Windows (PowerShell)
irm https://raw.githubusercontent.com/loonghao/turbo-cdn/main/scripts/install.ps1 | iex
```

## 从 Crates.io

### 使用 Cargo

```bash
cargo install turbo-cdn
```

### 指定 Features

```bash
# 默认（库友好：rustls 使用 ring 后端，关闭自更新）
cargo install turbo-cdn

# 启用 CLI 自更新
cargo install turbo-cdn --features self-update

# 偏好原生 TLS（如 Windows SChannel）并启用自更新
cargo install turbo-cdn --no-default-features --features "native-tls,fast-hash,high-performance,self-update"
```


## 从源码

### 前提条件

- Rust 1.70+（stable）
- Git

### 构建步骤

```bash
# 克隆仓库
git clone https://github.com/loonghao/turbo-cdn.git
cd turbo-cdn

# 构建发布版本
cargo build --release

# 二进制文件位于 target/release/turbo-cdn
./target/release/turbo-cdn --version
```

### 优化构建

获得最大性能：

```bash
RUSTFLAGS="-C target-cpu=native" cargo build --release --profile dist
```

## 作为库使用

添加到您的 `Cargo.toml`：

```toml
[dependencies]
# 库默认：rustls 使用 ring 后端，无需 cmake/NASM，自更新关闭
turbo-cdn = { version = "0.7", features = ["rustls", "fast-hash", "high-performance"] }
```

### Feature Flags

| Feature | 默认 | 描述 |
|---------|------|------|
| `rustls` | ✅ | 通过 rustls（ring 后端，无需 cmake/NASM）进行 TLS |
| `native-tls` | ❌ | 使用系统/原生 TLS 替代 rustls |
| `fast-hash` | ✅ | 使用 ahash 加速哈希 |
| `high-performance` | ✅ | 启用所有性能优化 |
| `self-update` | ❌ | CLI 自更新功能（按需开启） |

::: tip rustls ring 后端
从 v0.7 起，Turbo CDN 的 rustls 使用 ring 后端（`rustls-no-provider`），不再依赖 `aws-lc-sys` 构建链。
:::

### 配置示例

```toml
# 默认（库友好，关闭自更新）
turbo-cdn = { version = "0.7", features = ["rustls", "fast-hash", "high-performance"] }

# CLI 版本启用自更新
turbo-cdn = { version = "0.7", default-features = false, features = ["rustls", "fast-hash", "high-performance", "self-update"] }

# 原生 TLS + 自更新
turbo-cdn = { version = "0.7", default-features = false, features = ["native-tls", "fast-hash", "high-performance", "self-update"] }
```


## 验证

安装后，验证是否正常工作：

```bash
# 检查版本
turbo-cdn --version

# 测试下载
turbo-cdn dl "https://github.com/BurntSushi/ripgrep/releases/download/14.1.1/ripgrep-14.1.1-x86_64-pc-windows-msvc.zip" --verbose
```

## 更新

### Cargo

```bash
cargo install turbo-cdn --force
```

### 自更新（按需开启）

> 仅在构建时启用 `--features self-update` 或使用官方发布的 CLI 二进制时可用。

```bash
turbo-cdn self-update
```


## 故障排除

### 构建错误

如果遇到构建错误：

1. **更新 Rust**：`rustup update stable`
2. **清理构建**：`cargo clean && cargo build --release`
3. **检查依赖**：确保已安装 C 编译器

### 运行时问题

- **DNS 错误**：检查网络连接
- **TLS 错误**：如果使用企业代理，尝试 `--features native-tls`
- **权限错误**：确保对下载目录有写入权限

## 下一步

- [快速开始](/zh/guide/getting-started) - 开始基本使用
- [CLI 参考](/zh/api/) - 完整 CLI 文档
