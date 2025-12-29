# 地理检测

Turbo CDN 自动检测您的地理区域以选择最优的 CDN 镜像。

## 工作原理

### 检测流程

```
┌─────────────────────────────────────────────────────────────┐
│                       地理检测                               │
├─────────────────────────────────────────────────────────────┤
│  1. 检查缓存                                                 │
│     └─ 如果存在有效的缓存区域，直接使用                        │
│                                                              │
│  2. IP 地理定位（主要）                                       │
│     ├─ ip-api.com                                           │
│     ├─ ipinfo.io                                            │
│     └─ 多个备用 API                                          │
│                                                              │
│  3. 网络性能测试（备用）                                      │
│     ├─ 测试到各区域服务器的延迟                               │
│     └─ 选择延迟最低的区域                                     │
│                                                              │
│  4. 缓存结果                                                 │
│     └─ 存储以供后续请求使用                                   │
└─────────────────────────────────────────────────────────────┘
```

## 支持的区域

| 区域 | 代码 | 描述 |
|------|------|------|
| 中国 | `China` | 中国大陆，提供专用镜像 |
| 亚太 | `AsiaPacific` | 日本、韩国、东南亚等 |
| 欧洲 | `Europe` | 欧洲国家 |
| 北美 | `NorthAmerica` | 美国、加拿大、墨西哥 |
| 全球 | `Global` | 其他区域的默认值 |

## 区域特定优化

### 中国区域

Turbo CDN 为中国用户提供全面的镜像覆盖：

| 服务 | 镜像 |
|------|------|
| GitHub | ghfast.top, gh.con.sh, cors.isteed.cc, github.moeyy.xyz, mirror.ghproxy.com, ghproxy.net |
| PyPI | 清华、阿里云、豆瓣 |
| Crates.io | 清华、中科大 |
| Go Modules | goproxy.cn、阿里云 |
| Docker Hub | 中科大、网易、Docker 中国 |
| Maven | 阿里云、清华 |

### 全球区域

对于中国以外的用户：

| 服务 | CDN 节点 |
|------|----------|
| jsDelivr | fastly, gcore, testingcf, jsdelivr.b-cdn |
| Cloudflare | 全球边缘网络 |
| Fastly | 高性能 CDN |
| unpkg | npm 包分发 |

## 配置

### 自动检测（默认）

```rust
use turbo_cdn::*;

#[tokio::main]
async fn main() -> turbo_cdn::Result<()> {
    // 区域自动检测
    let downloader = TurboCdn::new().await?;
    
    // 下载使用检测到区域的最优镜像
    let result = downloader.download_from_url("https://example.com/file.zip").await?;
    Ok(())
}
```

### 手动设置区域

```rust
use turbo_cdn::*;

#[tokio::main]
async fn main() -> turbo_cdn::Result<()> {
    let downloader = TurboCdn::builder()
        .with_region(Region::China)  // 显式设置区域
        .build()
        .await?;
    
    let result = downloader.download_from_url("https://example.com/file.zip").await?;
    Ok(())
}
```

### 可用区域

```rust
pub enum Region {
    China,
    AsiaPacific,
    Europe,
    NorthAmerica,
    Global,
}
```

## 缓存

地理检测结果会被缓存以避免重复 API 调用：

- **缓存时长**：可配置（默认：1 小时）
- **缓存存储**：内存，可选持久化
- **缓存失效**：检测到网络变化时自动失效

## API 备用

Turbo CDN 使用多个 IP 地理定位 API 并自动切换：

1. **主要**：ip-api.com（免费，无需密钥）
2. **次要**：ipinfo.io（有免费额度）
3. **第三**：网络延迟测试

如果所有基于 IP 的检测都失败，系统会回退到网络性能测试，测量到各区域已知服务器的延迟。

## 性能影响

| 场景 | 检测时间 |
|------|----------|
| 缓存命中 | < 1ms |
| IP 地理定位 | 50-200ms |
| 网络测试 | 500-2000ms |

缓存系统确保大多数操作的地理检测开销最小。

## 故障排除

### 检测问题

如果区域检测似乎不正确：

1. **检查网络**：确保网络连接稳定
2. **VPN/代理**：可能影响基于 IP 的检测
3. **手动覆盖**：如需要可显式设置区域

```rust
// 强制使用特定区域
let downloader = TurboCdn::builder()
    .with_region(Region::China)
    .build()
    .await?;
```

### 日志

启用调试日志查看检测详情：

```bash
RUST_LOG=turbo_cdn=debug turbo-cdn dl "https://example.com/file.zip"
```

## 下一步

- [CDN 质量评估](/zh/guide/cdn-quality) - 镜像排名原理
- [智能下载](/zh/guide/smart-download) - 自动方式选择
