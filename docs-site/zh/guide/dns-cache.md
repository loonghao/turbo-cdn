# DNS 缓存

Turbo CDN 包含高性能 DNS 缓存系统，可减少延迟。

## 概述

DNS 解析可能会为下载增加显著的延迟：
- 每个新连接都需要 DNS 查询
- 公共 DNS 服务器可能很慢
- 重复查询浪费时间

Turbo CDN 的 DNS 缓存：
- 缓存已解析的地址
- 使用 hickory-dns 进行快速解析
- 实现基于 TTL 的过期机制
- 提供自动清理

## 工作原理

### DNS 解析流程

```
┌─────────────────────────────────────────────────────────────┐
│                    DNS 缓存系统                              │
├─────────────────────────────────────────────────────────────┤
│  1. 缓存查找                                                 │
│     ├─ 检查主机名是否已缓存                                  │
│     └─ 如果存在有效条目，立即返回                            │
│                                                              │
│  2. DNS 解析（缓存未命中时）                                 │
│     ├─ 使用 hickory-dns 解析器                              │
│     ├─ 查询配置的 DNS 服务器                                │
│     └─ 处理 A/AAAA 记录                                     │
│                                                              │
│  3. 缓存存储                                                 │
│     ├─ 存储已解析的地址                                      │
│     ├─ 从 DNS 响应设置 TTL                                  │
│     └─ 限制缓存大小（LRU 淘汰）                              │
│                                                              │
│  4. 后台维护                                                 │
│     ├─ 定期清理过期条目                                      │
│     └─ 预取常用主机                                          │
└─────────────────────────────────────────────────────────────┘
```

## 性能影响

### 延迟减少

| 场景 | 无缓存 | 有缓存 | 提升 |
|------|--------|--------|------|
| 首次请求 | 50-200ms | 50-200ms | - |
| 后续请求 | 50-200ms | < 1ms | 99%+ |
| 并行下载 | 每次 50-200ms | 每次 < 1ms | 99%+ |

### 实际收益

对于 16 个并行分块的下载：
- **无缓存**：16 × 100ms = 1.6s DNS 开销
- **有缓存**：100ms + 15 × 0ms = 100ms DNS 开销
- **节省**：每次下载节省 1.5 秒

## 配置

### 默认设置

| 参数 | 默认值 | 描述 |
|------|--------|------|
| 缓存大小 | 1000 条目 | 最大缓存主机数 |
| 默认 TTL | 300 秒 | 未指定时的 TTL |
| 最小 TTL | 60 秒 | 最小缓存时间 |
| 最大 TTL | 3600 秒 | 最大缓存时间 |
| 清理间隔 | 60 秒 | 清理过期条目的频率 |

### 自定义配置

```rust
use turbo_cdn::*;

#[tokio::main]
async fn main() -> turbo_cdn::Result<()> {
    let downloader = TurboCdn::builder()
        .with_dns_cache(true)              // 启用（默认）
        .with_dns_cache_size(2000)         // 增加缓存大小
        .with_dns_cache_ttl(600)           // 10 分钟默认 TTL
        .build()
        .await?;
    
    Ok(())
}
```

## DNS 解析器

### hickory-dns 集成

Turbo CDN 使用 [hickory-dns](https://github.com/hickory-dns/hickory-dns)（原 trust-dns）：

- 纯 Rust 实现
- 支持 async/await
- 支持 DNS-over-TLS
- DNSSEC 验证（可选）

### 解析器配置

```rust
// 默认解析器使用系统 DNS
let downloader = TurboCdn::new().await?;

// 自定义 DNS 服务器（未来功能）
let downloader = TurboCdn::builder()
    .with_dns_servers(vec!["8.8.8.8", "1.1.1.1"])
    .build()
    .await?;
```

## 缓存行为

### TTL 处理

1. **DNS 响应 TTL**：遵循服务器响应中的 TTL
2. **最小 TTL**：防止缓存时间过短
3. **最大 TTL**：防止条目过期

### 缓存淘汰

当缓存已满时：
1. 首先移除过期条目
2. LRU（最近最少使用）淘汰
3. 保留频繁访问的条目

### 预取

对于已知的 CDN 主机，Turbo CDN 可以预取 DNS：

```rust
// 预取已知镜像的 DNS
downloader.prefetch_dns(&[
    "ghfast.top",
    "gh.con.sh",
    "github.com",
]).await?;
```

## 监控

### 缓存统计

```rust
use turbo_cdn::*;

#[tokio::main]
async fn main() -> turbo_cdn::Result<()> {
    let downloader = TurboCdn::new().await?;
    
    // 执行一些下载...
    
    // 获取 DNS 缓存统计
    let stats = downloader.get_dns_cache_stats();
    println!("缓存命中: {}", stats.hits);
    println!("缓存未命中: {}", stats.misses);
    println!("命中率: {:.1}%", stats.hit_rate() * 100.0);
    println!("缓存条目: {}", stats.entries);
    
    Ok(())
}
```

### CLI 统计

```bash
turbo-cdn stats
```

包含 DNS 缓存指标。

## 最佳实践

### 生产环境启用

DNS 缓存默认启用，推荐用于：
- 所有生产部署
- 从相同主机重复下载
- 高并发场景

### 调试时禁用

排查 DNS 问题时：

```rust
let downloader = TurboCdn::builder()
    .with_dns_cache(false)
    .build()
    .await?;
```

### 大规模部署

对于有大量唯一主机的应用：

```rust
let downloader = TurboCdn::builder()
    .with_dns_cache_size(5000)     // 更大的缓存
    .with_dns_cache_ttl(900)       // 更长的 TTL
    .build()
    .await?;
```

## 故障排除

### DNS 解析失败

如果 DNS 解析失败：

1. **检查网络**：确保网络连接正常
2. **检查 DNS 服务器**：系统 DNS 可能配置错误
3. **尝试其他 DNS**：配置自定义 DNS 服务器

### 缓存条目过期

如果缓存条目变得陈旧：

```rust
// 清除 DNS 缓存
downloader.clear_dns_cache().await;
```

### 调试日志

```bash
RUST_LOG=turbo_cdn::dns_cache=debug turbo-cdn dl "https://example.com/file.zip"
```

## 安全考虑

### DNS 欺骗防护

- 使用 HTTPS 下载（TLS 验证服务器）
- 敏感环境考虑使用 DNS-over-TLS
- 可用 DNSSEC 验证（可选）

### 缓存投毒

- TTL 限制防止长期投毒
- 定期缓存清理
- 针对已知 CDN 主机的验证

## 下一步

- [API 参考](/zh/api/) - 完整 API 文档
- [性能指南](/zh/guide/smart-download) - 优化技巧
