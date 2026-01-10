# Installation

Multiple ways to install Turbo CDN based on your needs.

## Pre-built Binaries

### GitHub Releases

Download pre-built binaries from [GitHub Releases](https://github.com/loonghao/turbo-cdn/releases):

| Platform | Architecture | Download |
|----------|--------------|----------|
| Linux | x86_64 | `turbo-cdn-x86_64-unknown-linux-gnu.tar.gz` |
| Linux | x86_64 (musl) | `turbo-cdn-x86_64-unknown-linux-musl.tar.gz` |
| Linux | aarch64 | `turbo-cdn-aarch64-unknown-linux-gnu.tar.gz` |
| Linux | aarch64 (musl) | `turbo-cdn-aarch64-unknown-linux-musl.tar.gz` |
| macOS | x86_64 | `turbo-cdn-x86_64-apple-darwin.tar.gz` |
| macOS | aarch64 (Apple Silicon) | `turbo-cdn-aarch64-apple-darwin.tar.gz` |
| Windows | x86_64 | `turbo-cdn-x86_64-pc-windows-msvc.zip` |
| Windows | aarch64 | `turbo-cdn-aarch64-pc-windows-msvc.zip` |

### Installation Script (Coming Soon)

```bash
# Linux/macOS
curl -fsSL https://raw.githubusercontent.com/loonghao/turbo-cdn/main/scripts/install.sh | bash

# Windows (PowerShell)
irm https://raw.githubusercontent.com/loonghao/turbo-cdn/main/scripts/install.ps1 | iex
```

## From Crates.io

### Using Cargo

```bash
cargo install turbo-cdn
```

### With Specific Features

```bash
# Default (library-friendly: rustls ring backend, self-update off)
cargo install turbo-cdn

# Enable CLI self-update
cargo install turbo-cdn --features self-update

# Prefer native TLS (e.g., Windows SChannel) with self-update
cargo install turbo-cdn --no-default-features --features "native-tls,fast-hash,high-performance,self-update"
```



## From Source

### Prerequisites

- Rust 1.70+ (stable)
- Git

### Build Steps

```bash
# Clone the repository
git clone https://github.com/loonghao/turbo-cdn.git
cd turbo-cdn

# Build release binary
cargo build --release

# The binary is at target/release/turbo-cdn
./target/release/turbo-cdn --version
```

### Build with Optimizations

For maximum performance:

```bash
RUSTFLAGS="-C target-cpu=native" cargo build --release --profile dist
```

## As a Library

Add to your `Cargo.toml`:

```toml
[dependencies]
# Library-friendly defaults (self-update off, rustls ring backend - no cmake/NASM needed)
turbo-cdn = { version = "0.7", features = ["rustls", "fast-hash", "high-performance"] }
```

### Feature Flags

| Feature | Default | Description |
|---------|---------|-------------|
| `rustls` | ✅ | TLS via rustls (ring backend, no cmake/NASM toolchain required) |
| `native-tls` | ❌ | Use system/native TLS instead of rustls |
| `fast-hash` | ✅ | Use ahash for faster hashing |
| `high-performance` | ✅ | Enable all performance optimizations |
| `self-update` | ❌ | CLI self-update functionality (opt-in for binaries) |

::: tip rustls ring backend
Starting from v0.7, Turbo CDN uses rustls with the ring backend (via `rustls-no-provider`), removing the `aws-lc-sys` toolchain requirement.
:::

### Example Configurations

```toml
# Default (library-friendly, self-update off)
turbo-cdn = { version = "0.7", features = ["rustls", "fast-hash", "high-performance"] }

# CLI build with self-update
turbo-cdn = { version = "0.7", default-features = false, features = ["rustls", "fast-hash", "high-performance", "self-update"] }

# Native TLS with self-update
turbo-cdn = { version = "0.7", default-features = false, features = ["native-tls", "fast-hash", "high-performance", "self-update"] }
```


## Verification

After installation, verify it works:

```bash
# Check version
turbo-cdn --version

# Test with a download
turbo-cdn dl "https://github.com/BurntSushi/ripgrep/releases/download/14.1.1/ripgrep-14.1.1-x86_64-pc-windows-msvc.zip" --verbose
```

## Updating

### Cargo

```bash
cargo install turbo-cdn --force
```

### Self-Update (opt-in feature)

> Available when built with `--features self-update` or when using official CLI release binaries.

```bash
turbo-cdn self-update
```


## Troubleshooting

### Build Errors

If you encounter build errors:

1. **Update Rust**: `rustup update stable`
2. **Clean build**: `cargo clean && cargo build --release`
3. **Check dependencies**: Ensure you have a C compiler installed

### Runtime Issues

- **DNS errors**: Check your network connection
- **TLS errors**: Try with `--features native-tls` if using corporate proxy
- **Permission errors**: Ensure write access to download directory

## Next Steps

- [Quick Start](/guide/getting-started) - Get started with basic usage
- [CLI Reference](/api/) - Full CLI documentation
