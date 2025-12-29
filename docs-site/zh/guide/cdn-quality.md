# CDN 质量评估

Turbo CDN 持续监控 CDN 性能以选择最佳镜像。

## 质量评分系统

### 评分算法

每个 CDN 镜像根据以下因素获得 0-100 的质量分数：

| 因素 | 权重 | 描述 |
|------|------|------|
| 延迟 | 40% | 首字节响应时间 |
| 带宽 | 35% | 下载速度能力 |
| 可用性 | 25% | 请求成功率 |

### 分数计算

```
质量分数 = (延迟分数 × 0.4) + (带宽分数 × 0.35) + (可用性分数 × 0.25)
```

其中：
- **延迟分数**：`100 - min(latency_ms / 10, 100)`
- **带宽分数**：`min(bandwidth_mbps × 10, 100)`
- **可用性分数**：`success_rate × 100`

## 实时监控

### 评估流程

```
┌─────────────────────────────────────────────────────────────┐
│                     CDN 质量评估                             │
├─────────────────────────────────────────────────────────────┤
│  1. 后台测试                                                 │
│     ├─ 定期健康检查                                          │
│     ├─ 延迟测量                                              │
│     └─ 带宽采样                                              │
│                                                              │
│  2. 分数计算                                                 │
│     ├─ 加权评分算法                                          │
│     └─ 历史平均                                              │
│                                                              │
│  3. 动态排名                                                 │
│     ├─ 按分数排序镜像                                        │
│     └─ 更新选择优先级                                        │
│                                                              │
│  4. 智能缓存                                                 │
│     ├─ 带 TTL 缓存分数                                       │
│     └─ 避免重复测试                                          │
└─────────────────────────────────────────────────────────────┘
```

## 镜像选择

### 选择过程

1. **获取可用镜像**：基于 URL 模式和区域
2. **获取分数**：从缓存或执行实时评估
3. **按分数排名**：按质量分数排序镜像
4. **选择最佳**：选择最高分镜像
5. **备用**：如果最佳失败，尝试下一个

### 选择示例

对于中国区域的 GitHub release 下载：

| 镜像 | 延迟 | 带宽 | 可用性 | 分数 |
|------|------|------|--------|------|
| ghfast.top | 50ms | 10 MB/s | 99% | 87.25 |
| gh.con.sh | 80ms | 8 MB/s | 98% | 79.50 |
| cors.isteed.cc | 100ms | 6 MB/s | 95% | 70.75 |

结果：`ghfast.top` 被选为主镜像。

## 配置

### 默认行为

```rust
use turbo_cdn::*;

#[tokio::main]
async fn main() -> turbo_cdn::Result<()> {
    // 质量评估默认启用
    let downloader = TurboCdn::new().await?;
    
    // 获取基于质量选择的最优 URL
    let optimal = downloader.get_optimal_url("https://github.com/user/repo/releases/download/v1.0/file.zip").await?;
    
    Ok(())
}
```

### 自定义评估

```rust
use turbo_cdn::*;

#[tokio::main]
async fn main() -> turbo_cdn::Result<()> {
    let downloader = TurboCdn::builder()
        .with_quality_assessment(true)      // 启用（默认）
        .with_assessment_timeout(5)         // 每次测试 5 秒超时
        .with_assessment_interval(300)      // 每 5 分钟重新评估
        .build()
        .await?;
    
    Ok(())
}
```

## 性能统计

### 访问统计

```rust
use turbo_cdn::*;

#[tokio::main]
async fn main() -> turbo_cdn::Result<()> {
    let downloader = TurboCdn::new().await?;
    
    // 获取性能摘要
    let summary = downloader.get_performance_summary();
    
    println!("跟踪的服务器总数: {}", summary.total_servers);
    println!("总体成功率: {:.1}%", summary.overall_success_rate * 100.0);
    
    if let Some((url, score)) = summary.best_server {
        println!("最佳服务器: {} (分数: {:.2})", url, score);
    }
    
    Ok(())
}
```

### CLI 统计

```bash
turbo-cdn stats
```

## 质量阈值

| 分数范围 | 质量 | 操作 |
|----------|------|------|
| 80-100 | 优秀 | 首选 |
| 60-79 | 良好 | 可接受的备用 |
| 40-59 | 一般 | 无更好选项时使用 |
| 0-39 | 差 | 尽量避免 |

## 缓存策略

### 缓存参数

| 参数 | 默认值 | 描述 |
|------|--------|------|
| TTL | 5 分钟 | 分数有效期 |
| 最大条目 | 100 | 最大缓存镜像数 |
| 刷新 | 后台 | 非阻塞更新 |

### 缓存行为

- **命中**：立即返回缓存分数
- **未命中**：执行实时评估
- **过期**：返回缓存，后台刷新
- **失效**：阻塞直到新评估完成

## 故障排除

### 性能差

如果尽管有质量评估但下载仍然很慢：

1. **检查网络**：本地网络可能是瓶颈
2. **强制刷新**：清除缓存分数
3. **手动选择**：使用已知良好的镜像覆盖

```rust
// 强制直接下载以绕过 CDN 选择
let downloader = TurboCdn::new().await?;
let result = downloader.download_direct_from_url("https://example.com/file.zip").await?;
```

### 调试日志

```bash
RUST_LOG=turbo_cdn::cdn_quality=debug turbo-cdn dl "https://example.com/file.zip"
```

## 下一步

- [智能下载](/zh/guide/smart-download) - 自动方式选择
- [自适应并发](/zh/guide/adaptive-concurrency) - 动态并行化
