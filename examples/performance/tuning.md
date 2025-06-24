# 🚀 Performance Tuning Guide

This guide provides comprehensive strategies for optimizing Turbo CDN performance across different scenarios and environments.

## 📊 Understanding Performance Factors

### Key Performance Metrics

1. **Download Speed (MB/s)** - Primary performance indicator
2. **Latency** - Time to first byte
3. **Throughput** - Sustained data transfer rate
4. **CPU Usage** - Resource consumption
5. **Memory Usage** - RAM consumption
6. **Network Efficiency** - Bandwidth utilization

### Performance Influencing Factors

```rust
// Factors that affect performance
let performance_factors = vec![
    "Network bandwidth and latency",
    "Server response time and load",
    "File size and type",
    "Geographic location",
    "CDN availability and quality",
    "Concurrent chunk count",
    "Chunk size configuration",
    "System resources (CPU/RAM)",
    "Network congestion",
    "DNS resolution time",
];
```

## ⚙️ Configuration Optimization

### 1. Chunk Configuration

#### Optimal Chunk Count by Connection Speed

```rust
use turbo_cdn::DownloadOptions;
use std::time::Duration;

// High-speed connections (100+ Mbps)
let high_speed_config = DownloadOptions {
    max_concurrent_chunks: 16,
    chunk_size: 4 * 1024 * 1024, // 4MB
    enable_resume: true,
    timeout_override: Some(Duration::from_secs(300)),
    verify_integrity: false, // Skip for maximum speed
    ..Default::default()
};

// Medium-speed connections (25-100 Mbps)
let medium_speed_config = DownloadOptions {
    max_concurrent_chunks: 8,
    chunk_size: 2 * 1024 * 1024, // 2MB
    enable_resume: true,
    timeout_override: Some(Duration::from_secs(180)),
    verify_integrity: true,
    ..Default::default()
};

// Low-speed connections (<25 Mbps)
let low_speed_config = DownloadOptions {
    max_concurrent_chunks: 4,
    chunk_size: 1024 * 1024, // 1MB
    enable_resume: true,
    timeout_override: Some(Duration::from_secs(120)),
    verify_integrity: true,
    ..Default::default()
};
```

#### Dynamic Chunk Size Calculation

```rust
fn calculate_optimal_chunk_size(bandwidth_mbps: f64, file_size: u64) -> usize {
    let base_chunk_size = match bandwidth_mbps {
        bw if bw > 100.0 => 8 * 1024 * 1024,  // 8MB for very fast
        bw if bw > 50.0  => 4 * 1024 * 1024,  // 4MB for fast
        bw if bw > 25.0  => 2 * 1024 * 1024,  // 2MB for medium
        bw if bw > 10.0  => 1024 * 1024,      // 1MB for slow
        _                => 512 * 1024,       // 512KB for very slow
    };
    
    // Adjust based on file size
    if file_size < 10 * 1024 * 1024 {  // Files < 10MB
        std::cmp::min(base_chunk_size, file_size as usize / 4)
    } else {
        base_chunk_size
    }
}

fn calculate_optimal_chunk_count(bandwidth_mbps: f64, cpu_cores: usize) -> usize {
    let base_count = match bandwidth_mbps {
        bw if bw > 100.0 => 16,
        bw if bw > 50.0  => 12,
        bw if bw > 25.0  => 8,
        bw if bw > 10.0  => 4,
        _                => 2,
    };
    
    // Don't exceed 2x CPU cores to avoid context switching overhead
    std::cmp::min(base_count, cpu_cores * 2)
}
```

### 2. Network Optimization

#### Connection Pooling and Reuse

```rust
use std::collections::HashMap;

// Custom headers for connection optimization
let mut optimized_headers = HashMap::new();
optimized_headers.insert("Connection".to_string(), "keep-alive".to_string());
optimized_headers.insert("Keep-Alive".to_string(), "timeout=30, max=100".to_string());
optimized_headers.insert("Accept-Encoding".to_string(), "gzip, deflate, br".to_string());

let network_optimized_config = DownloadOptions {
    custom_headers: Some(optimized_headers),
    timeout_override: Some(Duration::from_secs(60)),
    ..Default::default()
};
```

#### DNS Optimization

```bash
# System-level DNS optimization
# Add to /etc/systemd/resolved.conf (Linux)
[Resolve]
DNS=1.1.1.1 8.8.8.8
DNSOverTLS=yes
Cache=yes

# Or use environment variables
export TURBO_CDN_DNS_SERVERS="1.1.1.1,8.8.8.8"
```

### 3. File Type-Specific Optimizations

#### Large Binary Files (>100MB)

```rust
let large_file_config = DownloadOptions {
    max_concurrent_chunks: 20,
    chunk_size: 8 * 1024 * 1024, // 8MB chunks
    enable_resume: true,
    verify_integrity: false, // Skip for speed, verify after
    timeout_override: Some(Duration::from_secs(600)), // 10 minutes
    ..Default::default()
};
```

#### Small Files (<10MB)

```rust
let small_file_config = DownloadOptions {
    max_concurrent_chunks: 2, // Overhead not worth it
    chunk_size: 1024 * 1024, // 1MB
    enable_resume: false, // Not needed for small files
    verify_integrity: true,
    timeout_override: Some(Duration::from_secs(30)),
    ..Default::default()
};
```

#### Source Code Archives

```rust
let source_code_config = DownloadOptions {
    max_concurrent_chunks: 6,
    chunk_size: 2 * 1024 * 1024, // 2MB
    enable_resume: true,
    verify_integrity: true, // Always verify source code
    timeout_override: Some(Duration::from_secs(120)),
    ..Default::default()
};
```

## 🌐 Geographic and CDN Optimization

### Regional Configuration

```rust
// Configuration for different regions
fn get_regional_config(region: &str) -> DownloadOptions {
    match region {
        "china" => DownloadOptions {
            max_concurrent_chunks: 6, // Conservative due to GFW
            chunk_size: 1024 * 1024,
            timeout_override: Some(Duration::from_secs(180)),
            ..Default::default()
        },
        "europe" => DownloadOptions {
            max_concurrent_chunks: 12,
            chunk_size: 3 * 1024 * 1024,
            timeout_override: Some(Duration::from_secs(120)),
            ..Default::default()
        },
        "us" => DownloadOptions {
            max_concurrent_chunks: 16,
            chunk_size: 4 * 1024 * 1024,
            timeout_override: Some(Duration::from_secs(90)),
            ..Default::default()
        },
        _ => DownloadOptions::default(),
    }
}
```

### CDN Selection Strategy

```bash
# Environment variables for CDN preferences
export TURBO_CDN_PREFERRED_CDNS="fastly,cloudflare,jsdelivr"
export TURBO_CDN_FALLBACK_ENABLED=true
export TURBO_CDN_CDN_TIMEOUT=10
```

## 🖥️ System-Level Optimizations

### 1. Operating System Tuning

#### Linux Optimizations

```bash
# Increase network buffer sizes
echo 'net.core.rmem_max = 134217728' >> /etc/sysctl.conf
echo 'net.core.wmem_max = 134217728' >> /etc/sysctl.conf
echo 'net.ipv4.tcp_rmem = 4096 87380 134217728' >> /etc/sysctl.conf
echo 'net.ipv4.tcp_wmem = 4096 65536 134217728' >> /etc/sysctl.conf

# Increase connection tracking
echo 'net.netfilter.nf_conntrack_max = 1048576' >> /etc/sysctl.conf

# Apply changes
sysctl -p
```

#### Windows Optimizations

```powershell
# Increase TCP window size
netsh int tcp set global autotuninglevel=normal

# Enable TCP Chimney Offload
netsh int tcp set global chimney=enabled

# Optimize for throughput
netsh int tcp set global rss=enabled
```

### 2. Application-Level Tuning

#### Memory Management

```rust
// Use custom allocator for better performance
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

// Configure Tokio runtime for optimal performance
#[tokio::main(flavor = "multi_thread", worker_threads = 8)]
async fn main() {
    // Your application code
}
```

#### Async Runtime Optimization

```rust
use tokio::runtime::Builder;

let rt = Builder::new_multi_thread()
    .worker_threads(num_cpus::get())
    .thread_stack_size(3 * 1024 * 1024) // 3MB stack
    .enable_all()
    .build()
    .unwrap();
```

## 📈 Performance Monitoring and Adaptive Tuning

### Real-time Performance Adjustment

```rust
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

pub struct AdaptiveConfig {
    current_chunks: Arc<AtomicUsize>,
    current_chunk_size: Arc<AtomicUsize>,
    performance_history: Arc<Mutex<VecDeque<f64>>>,
}

impl AdaptiveConfig {
    pub fn adjust_based_on_performance(&self, current_speed: f64, target_speed: f64) {
        let speed_ratio = current_speed / target_speed;
        
        if speed_ratio < 0.8 {
            // Performance below target, try increasing chunks
            let current = self.current_chunks.load(Ordering::Relaxed);
            if current < 20 {
                self.current_chunks.store(current + 2, Ordering::Relaxed);
            }
        } else if speed_ratio > 1.2 {
            // Performance above target, can reduce chunks to save resources
            let current = self.current_chunks.load(Ordering::Relaxed);
            if current > 4 {
                self.current_chunks.store(current - 1, Ordering::Relaxed);
            }
        }
    }
}
```

### Bandwidth Detection

```rust
async fn detect_bandwidth() -> Result<f64> {
    let test_url = "https://speed.cloudflare.com/__down?bytes=10000000"; // 10MB test
    let start = Instant::now();
    
    match async_api::quick::download_url(test_url).await {
        Ok(result) => {
            let duration = start.elapsed();
            let speed_mbps = (result.size as f64 * 8.0) / (duration.as_secs_f64() * 1_000_000.0);
            Ok(speed_mbps)
        }
        Err(e) => Err(e),
    }
}
```

## 🔧 Environment-Specific Configurations

### Development Environment

```toml
# turbo-cdn-dev.toml
[performance]
max_concurrent_downloads = 4
chunk_size = "1MB"
timeout = "60s"
verify_checksums = true

[logging]
level = "debug"
performance_metrics = true
```

### Production Environment

```toml
# turbo-cdn-prod.toml
[performance]
max_concurrent_downloads = 12
chunk_size = "4MB"
timeout = "300s"
verify_checksums = false

[logging]
level = "info"
performance_metrics = true

[cache]
enabled = true
max_size = "10GB"
```

### CI/CD Environment

```toml
# turbo-cdn-ci.toml
[performance]
max_concurrent_downloads = 8
chunk_size = "2MB"
timeout = "180s"
verify_checksums = true

[cache]
enabled = true
max_size = "5GB"
```

## 📊 Performance Benchmarking

### Benchmark Script

```bash
#!/bin/bash
# performance-test.sh

echo "🚀 Turbo CDN Performance Benchmark"
echo "=================================="

# Test URLs of different sizes
SMALL_FILE="https://github.com/sharkdp/bat/releases/download/v0.24.0/bat-v0.24.0-x86_64-pc-windows-msvc.zip"
MEDIUM_FILE="https://github.com/microsoft/vscode/releases/download/1.85.0/VSCode-linux-x64.tar.gz"
LARGE_FILE="https://nodejs.org/dist/v20.10.0/node-v20.10.0-linux-x64.tar.xz"

# Test different configurations
echo "Testing small file performance..."
time turbo-cdn dl "$SMALL_FILE" --verbose

echo "Testing medium file performance..."
time turbo-cdn dl "$MEDIUM_FILE" --verbose

echo "Testing large file performance..."
time turbo-cdn dl "$LARGE_FILE" --verbose

# Compare with traditional tools
echo "Comparing with curl..."
time curl -L -o test-curl.zip "$SMALL_FILE"

echo "Comparing with wget..."
time wget -O test-wget.zip "$SMALL_FILE"
```

## 🎯 Best Practices Summary

### Do's ✅

1. **Profile before optimizing** - Measure current performance
2. **Use appropriate chunk sizes** - Match to bandwidth and file size
3. **Enable resume for large files** - Recover from interruptions
4. **Monitor performance metrics** - Track improvements
5. **Adjust timeouts appropriately** - Balance speed vs reliability
6. **Use CDN optimization** - Leverage geographic advantages
7. **Cache frequently downloaded files** - Avoid redundant downloads

### Don'ts ❌

1. **Don't over-chunk small files** - Overhead exceeds benefits
2. **Don't ignore network conditions** - Adapt to current environment
3. **Don't skip integrity verification** - For critical files
4. **Don't use excessive concurrency** - Can overwhelm servers
5. **Don't ignore error rates** - High errors indicate problems
6. **Don't use fixed configurations** - Adapt to different scenarios

## 🔗 Next Steps

- [Benchmarks Example](benchmarks.rs) - Run performance benchmarks
- [Monitoring Example](monitoring.rs) - Monitor real-time performance
- [API Examples](../api/) - Integrate optimizations into your code
