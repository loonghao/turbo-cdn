# 自适应并发

Turbo CDN 根据网络条件动态调整下载并发度。

## 概述

传统下载工具使用固定并发度，这可能导致：
- **慢速网络过载** → 失败增加
- **快速网络利用不足** → 下载变慢

Turbo CDN 的自适应并发控制器通过以下方式解决这个问题：
- 实时监控网络条件
- 检测拥塞模式
- 动态调整并行度

## 工作原理

### 并发控制流程

```
┌─────────────────────────────────────────────────────────────┐
│                    自适应并发控制器                          │
├─────────────────────────────────────────────────────────────┤
│  1. 初始评估                                                 │
│     ├─ 测试网络带宽                                          │
│     └─ 设置初始并发级别                                      │
│                                                              │
│  2. 持续监控                                                 │
│     ├─ 跟踪分块完成时间                                      │
│     ├─ 监控错误率                                            │
│     └─ 检测拥塞模式                                          │
│                                                              │
│  3. 动态调整                                                 │
│     ├─ 网络快时增加并发                                      │
│     ├─ 检测到拥塞时减少                                      │
│     └─ 保持最优吞吐量                                        │
│                                                              │
│  4. 稳定化                                                   │
│     └─ 收敛到最优级别                                        │
└─────────────────────────────────────────────────────────────┘
```

## 拥塞检测

### 指标

| 指标 | 阈值 | 操作 |
|------|------|------|
| 分块超时 | > 2 次连续 | 减少并发 |
| 错误率 | > 10% | 减少并发 |
| 延迟峰值 | > 2x 基线 | 减少并发 |
| 快速完成 | < 50% 预期时间 | 增加并发 |

### 算法

```rust
// 简化的拥塞检测逻辑
if error_rate > 0.1 || consecutive_timeouts > 2 {
    concurrency = max(concurrency - 2, MIN_CONCURRENCY);
} else if avg_chunk_time < expected_time * 0.5 {
    concurrency = min(concurrency + 1, MAX_CONCURRENCY);
}
```

## 配置

### 默认设置

| 参数 | 默认值 | 描述 |
|------|--------|------|
| 最小并发 | 2 | 最小并行分块数 |
| 最大并发 | 32 | 最大并行分块数 |
| 初始并发 | 8 | 起始级别 |
| 调整间隔 | 1 秒 | 调整频率 |

### 自定义配置

```rust
use turbo_cdn::*;

#[tokio::main]
async fn main() -> turbo_cdn::Result<()> {
    let downloader = TurboCdn::builder()
        .with_max_concurrent_downloads(16)    // 最大并发
        .with_adaptive_concurrency(true)      // 启用自适应控制
        .build()
        .await?;
    
    Ok(())
}
```

### 下载选项

```rust
use turbo_cdn::*;

let options = DownloadOptions::new()
    .with_max_concurrent_chunks(16)    // 最大并发分块
    .with_adaptive_concurrency(true);  // 启用自适应控制

let result = downloader.download_with_options(
    "https://example.com/large-file.zip",
    "./file.zip",
    options
).await?;
```

## 性能影响

### 基准测试结果

| 网络类型 | 固定 (8) | 自适应 | 提升 |
|----------|----------|--------|------|
| 快速 (100 Mbps) | 8.5 MB/s | 11.2 MB/s | +32% |
| 中等 (20 Mbps) | 2.1 MB/s | 2.4 MB/s | +14% |
| 慢速 (5 Mbps) | 0.4 MB/s | 0.6 MB/s | +50% |
| 不稳定 | 0.2 MB/s | 0.5 MB/s | +150% |

### 主要优势

1. **快速网络**：通过增加并发最大化吞吐量
2. **慢速网络**：通过减少并发防止过载
3. **不稳定网络**：适应变化的条件
4. **拥塞网络**：检测并响应拥塞

## 监控

### 实时统计

```rust
use turbo_cdn::*;

#[tokio::main]
async fn main() -> turbo_cdn::Result<()> {
    let downloader = TurboCdn::new().await?;
    
    // 下载期间跟踪并发
    let result = downloader.download_from_url("https://example.com/file.zip").await?;
    
    // 获取性能摘要
    let summary = downloader.get_performance_summary();
    println!("平均并发: {}", summary.avg_concurrency);
    
    Ok(())
}
```

### CLI 详细输出

```bash
turbo-cdn dl "https://example.com/large-file.zip" --verbose
```

输出包括：
```
自适应并发: 8 → 12 (网络快)
自适应并发: 12 → 10 (轻微拥塞)
自适应并发: 10 → 10 (稳定)
```

## 最佳实践

### 何时启用

自适应并发**默认启用**，推荐用于：
- 大文件下载 (> 10 MB)
- 未知网络条件
- 网络质量不稳定

### 何时禁用

考虑使用固定并发当：
- 网络已知且稳定
- 服务器有严格的速率限制
- 调试下载问题

```rust
let downloader = TurboCdn::builder()
    .with_adaptive_concurrency(false)
    .with_max_concurrent_downloads(4)  // 固定为 4
    .build()
    .await?;
```

## 故障排除

### 过于激进

如果下载导致网络问题：

```rust
let downloader = TurboCdn::builder()
    .with_max_concurrent_downloads(8)  // 降低最大值
    .build()
    .await?;
```

### 过于保守

如果在快速网络上下载慢：

```rust
let downloader = TurboCdn::builder()
    .with_max_concurrent_downloads(32)  // 提高最大值
    .build()
    .await?;
```

### 调试日志

```bash
RUST_LOG=turbo_cdn::adaptive_concurrency=debug turbo-cdn dl "https://example.com/file.zip"
```

## 下一步

- [智能分块](/zh/guide/smart-chunking) - 智能分块大小
- [DNS 缓存](/zh/guide/dns-cache) - 高性能 DNS 解析
