# 智能分块

Turbo CDN 使用智能分块大小以获得最佳下载性能。

## 概述

智能分块根据以下因素动态确定最优分块大小：
- 文件大小
- 网络带宽
- 历史性能
- 服务器能力

## 工作原理

### 分块大小选择

```
┌─────────────────────────────────────────────────────────────┐
│                     智能分块算法                             │
├─────────────────────────────────────────────────────────────┤
│  1. 文件分析                                                 │
│     ├─ 获取文件大小                                          │
│     └─ 检查服务器 Range 支持                                 │
│                                                              │
│  2. 初始分块大小                                             │
│     ├─ 小文件 (< 1MB): 单个分块                              │
│     ├─ 中等文件 (1-100MB): 1MB 分块                          │
│     └─ 大文件 (> 100MB): 2-4MB 分块                          │
│                                                              │
│  3. 动态调整                                                 │
│     ├─ 监控分块完成时间                                      │
│     ├─ 根据吞吐量调整                                        │
│     └─ 考虑网络条件                                          │
│                                                              │
│  4. 性能学习                                                 │
│     └─ 存储最优大小供未来下载使用                            │
└─────────────────────────────────────────────────────────────┘
```

## 默认分块大小

| 文件大小 | 默认分块 | 原因 |
|----------|----------|------|
| < 1 MB | 整个文件 | 分割的开销不值得 |
| 1-10 MB | 512 KB | 平衡并行度和开销 |
| 10-100 MB | 1 MB | 适合大多数连接 |
| 100 MB - 1 GB | 2 MB | 减少分块数量 |
| > 1 GB | 4 MB | 最小化大文件的开销 |

## 配置

### Builder 模式

```rust
use turbo_cdn::*;

#[tokio::main]
async fn main() -> turbo_cdn::Result<()> {
    let downloader = TurboCdn::builder()
        .with_chunk_size(2 * 1024 * 1024)    // 2MB 固定分块大小
        .with_adaptive_chunking(true)         // 启用智能调整
        .build()
        .await?;
    
    Ok(())
}
```

### 下载选项

```rust
use turbo_cdn::*;

let options = DownloadOptions::new()
    .with_chunk_size(4 * 1024 * 1024);  // 4MB 分块

let result = downloader.download_with_options(
    "https://example.com/large-file.zip",
    "./file.zip",
    options
).await?;
```

## 自适应分块

启用后，分块大小在下载过程中调整：

### 增加分块大小当

- 分块完成比预期快
- 网络带宽高
- 错误率低

### 减少分块大小当

- 分块频繁超时
- 网络看起来拥塞
- 错误率高

### 适应示例

```
下载: 500MB 文件
初始分块大小: 2MB (250 个分块)

进度:
  0-10%:  2MB 分块, 15 MB/s → 增加到 4MB
  10-50%: 4MB 分块, 18 MB/s → 稳定
  50-80%: 4MB 分块, 12 MB/s → 减少到 2MB (拥塞)
  80-100%: 2MB 分块, 14 MB/s → 完成
```

## 性能影响

### 分块大小 vs 性能

| 分块大小 | 开销 | 并行度 | 最适合 |
|----------|------|--------|--------|
| 256 KB | 高 | 高 | 慢速、不稳定网络 |
| 1 MB | 中等 | 中等 | 大多数场景 |
| 4 MB | 低 | 较低 | 快速、稳定网络 |
| 8 MB | 很低 | 低 | 非常快的连接 |

### 基准测试结果

对于 100MB 文件：

| 分块大小 | 时间（快速网络）| 时间（慢速网络）|
|----------|-----------------|-----------------|
| 256 KB | 12.5s | 85s |
| 1 MB | 10.2s | 82s |
| 4 MB | 9.8s | 95s |
| 自适应 | 9.5s | 78s |

## 内存考虑

### 内存使用

```
内存 = 分块大小 × 并发分块数 × 缓冲因子
```

示例：
- 4MB 分块 × 8 并发 × 1.5 缓冲 = 48MB

### 优化

对于内存受限环境：

```rust
let downloader = TurboCdn::builder()
    .with_chunk_size(1024 * 1024)         // 1MB 分块
    .with_max_concurrent_downloads(4)      // 较少并发
    .build()
    .await?;
```

## 服务器兼容性

### Range 请求支持

智能分块需要 HTTP Range 支持。Turbo CDN：

1. 检查 `Accept-Ranges` 头
2. 如果不支持则回退到单分块
3. 处理部分内容响应

### 检测

```rust
// 下载期间自动检测
let result = downloader.download_from_url("https://example.com/file.zip").await?;

// 检查是否使用了分块下载
if result.chunks_used > 1 {
    println!("使用 {} 个分块下载", result.chunks_used);
}
```

## 最佳实践

### 大文件（> 100MB）

```rust
let options = DownloadOptions::new()
    .with_chunk_size(4 * 1024 * 1024)      // 4MB 分块
    .with_max_concurrent_chunks(16)
    .with_adaptive_chunking(true);
```

### 小文件（< 10MB）

```rust
let options = DownloadOptions::new()
    .with_chunk_size(512 * 1024)           // 512KB 分块
    .with_max_concurrent_chunks(8);
```

### 不稳定网络

```rust
let options = DownloadOptions::new()
    .with_chunk_size(256 * 1024)           // 小分块
    .with_max_concurrent_chunks(4)
    .with_resume(true);                     // 启用续传
```

## 故障排除

### 下载慢

如果下载比预期慢：

1. **检查分块大小**：可能太小或太大
2. **检查并发**：可能需要调整
3. **启用自适应**：让系统优化

```rust
let downloader = TurboCdn::builder()
    .with_adaptive_chunking(true)
    .build()
    .await?;
```

### 内存问题

如果内存不足：

```rust
let downloader = TurboCdn::builder()
    .with_chunk_size(512 * 1024)           // 更小的分块
    .with_max_concurrent_downloads(4)       // 更少并发
    .build()
    .await?;
```

### 调试日志

```bash
RUST_LOG=turbo_cdn::smart_chunking=debug turbo-cdn dl "https://example.com/file.zip"
```

## 下一步

- [DNS 缓存](/zh/guide/dns-cache) - 高性能 DNS 解析
- [API 参考](/zh/api/) - 完整 API 文档
