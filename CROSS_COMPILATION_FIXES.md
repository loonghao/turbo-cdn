# Cross-Compilation Fixes for turbo-cdn

This document describes the cross-compilation issues we encountered and the solutions implemented.

## Problem

The project was failing to build on GNU targets, specifically with the error:
```
2025-06-29T07:00:47.934776Z error: failed to run custom build command for 'libmimalloc-sys v0.1.43'
```

This was happening because `mimalloc` has compatibility issues with certain targets, particularly:
- GNU toolchain targets (e.g., `x86_64-pc-windows-gnu`)
- musl libc targets (e.g., `x86_64-unknown-linux-musl`)
- ARM targets (e.g., `armv7-unknown-linux-gnueabihf`)
- 32-bit targets

## Solution Overview

We implemented a comprehensive solution that:

1. **Makes mimalloc optional** - Only enables it on supported platforms
2. **Automatic target detection** - Uses build script to detect problematic targets
3. **Conditional compilation** - Uses cfg attributes to conditionally include mimalloc
4. **Target-specific configurations** - Provides optimized settings per target

## Implementation Details

### 1. Cargo.toml Changes

```toml
# Made mimalloc optional
mimalloc = { version = "0.1", optional = true }

# Added feature flag
[features]
default = ["rustls-tls", "fast-hash", "high-performance", "mimalloc-allocator"]
mimalloc-allocator = ["mimalloc"]
```

### 2. Build Script (build.rs)

Created a build script that:
- Detects the target platform
- Automatically disables mimalloc for problematic targets
- Sets appropriate cfg flags
- Provides detailed build information

### 3. Conditional Compilation (src/main.rs)

```rust
#[cfg(all(
    feature = "mimalloc-allocator",
    not(disable_mimalloc),
    not(target_env = "musl"),
    not(target_arch = "arm"),
    any(target_os = "linux", target_os = "macos", target_os = "windows")
))]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;
```

### 4. Cargo Configuration (.cargo/config.toml)

Added target-specific configurations for:
- Static linking flags
- Cross-compilation optimizations
- Environment variables for better builds

## Supported Targets

### ✅ Targets with mimalloc (high performance)
- `x86_64-pc-windows-msvc`
- `x86_64-unknown-linux-gnu`
- `x86_64-apple-darwin`
- `aarch64-apple-darwin`

### ✅ Targets without mimalloc (compatible)
- `x86_64-pc-windows-gnu`
- `x86_64-unknown-linux-musl`
- `aarch64-unknown-linux-musl`
- `armv7-unknown-linux-gnueabihf`
- `aarch64-unknown-linux-gnu`

## Benefits

1. **Universal Compatibility** - Builds successfully on all target platforms
2. **Performance Optimization** - Uses mimalloc where supported for better performance
3. **Automatic Detection** - No manual configuration needed
4. **Clear Feedback** - Build script provides information about target decisions
5. **Future-Proof** - Easy to add new target configurations

## Testing

To test the solution:

```bash
# Test normal build (should use mimalloc on supported platforms)
cargo check

# Test with specific target
cargo check --target x86_64-pc-windows-gnu

# Test without mimalloc
cargo check --no-default-features --features rustls-tls,fast-hash,high-performance
```

## Workflow Integration

Updated all workflows to rust-actions-toolkit v2.5.0 which includes:
- Enhanced cross-compilation support
- Better error handling for problematic targets
- Optimized build matrices
- Improved caching strategies

This ensures that CI/CD pipelines can build for all supported targets without manual intervention.
