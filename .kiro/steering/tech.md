# Technology Stack & Build System

## Core Technology Stack

- **Language**: Rust 2021 Edition
- **Async Runtime**: Tokio with full feature set
- **HTTP Client**: reqwest with rustls-tls (cross-platform compatibility)
- **Memory Allocator**: mimalloc for high-performance memory management
- **Configuration**: figment with TOML and environment variable support
- **CLI Framework**: clap v4 with derive features
- **Logging**: tracing ecosystem (tracing, tracing-subscriber, tracing-appender)
- **Serialization**: serde with JSON support
- **DNS Resolution**: hickory-dns (formerly trust-dns) for performance
- **Concurrent Data Structures**: dashmap, once_cell
- **Error Handling**: anyhow and thiserror

## Key Dependencies

### Performance-Critical
- `reqwest` with `rustls-tls`, `http2`, `hickory-dns` features
- `mimalloc` - High-performance memory allocator
- `dashmap` - Concurrent hash maps
- `tokio-util` with io features for async I/O
- `memmap2` - Memory-mapped file I/O

### Development & Testing
- `tokio-test`, `tempfile`, `wiremock` for testing
- `criterion` for benchmarking
- `proptest` for property-based testing
- `pretty_assertions`, `serial_test` for test utilities

## Build System

### Cargo Configuration
- **Release Profile**: Optimized with LTO, single codegen unit, stripped binaries
- **Development Profile**: Fast compilation with debug info
- **Distribution Profile**: Inherits from release with maximum optimization

### Common Commands

```bash
# Development
cargo build                    # Debug build
cargo test                     # Run tests
cargo clippy                   # Linting
cargo fmt                      # Code formatting

# Release
cargo build --release          # Optimized build
cargo build --profile dist     # Distribution build

# Examples
cargo run --example basic_usage
cargo run --example async_api
cargo run --example vx_integration

# Benchmarks
cargo bench                    # Run benchmarks
cargo run --example benchmarks

# Cross-platform testing
cargo test --all-targets --all-features
```

### Scripts
- `scripts/fix-clippy-format.ps1` (Windows) / `scripts/fix-clippy-format.sh` (Unix) - Auto-fix clippy format warnings
- `scripts/test-build.ps1` / `scripts/test-build.sh` - Comprehensive build testing

## Features & Compilation

### Default Features
- `rustls-tls` - Use rustls instead of OpenSSL
- `fast-hash` - Enable ahash for performance
- `high-performance` - Enable performance optimizations

### Optional Features
- `native-tls` - Use system TLS (alternative to rustls)

## Architecture Patterns

### Module Organization
- **Core Logic**: `src/lib.rs` - Main API and client
- **CLI**: `src/main.rs` - Command-line interface
- **Configuration**: `src/config/` - Type-safe config system
- **Performance Modules**: Adaptive concurrency, smart chunking, DNS caching
- **Network**: HTTP client management, load balancing, CDN quality assessment
- **Utilities**: Progress tracking, logging, error handling

### Error Handling
- Custom error types with `thiserror`
- Result types throughout the codebase
- Comprehensive error context with `anyhow`

### Async Patterns
- Tokio-based async/await throughout
- Concurrent operations with proper synchronization
- Stream processing for downloads and progress tracking

### Configuration System
- TOML-based configuration with `figment`
- Environment variable overrides
- Type-safe deserialization with `serde`
- Default configurations embedded in binary