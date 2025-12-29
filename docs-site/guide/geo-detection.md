# Geographic Detection

Turbo CDN automatically detects your geographic region to select optimal CDN mirrors.

## How It Works

### Detection Flow

```
┌─────────────────────────────────────────────────────────────┐
│                    Geographic Detection                      │
├─────────────────────────────────────────────────────────────┤
│  1. Check Cache                                              │
│     └─ If valid cached region exists, use it                │
│                                                              │
│  2. IP Geolocation (Primary)                                │
│     ├─ ip-api.com                                           │
│     ├─ ipinfo.io                                            │
│     └─ Multiple fallback APIs                               │
│                                                              │
│  3. Network Performance Testing (Fallback)                  │
│     ├─ Test latency to regional servers                     │
│     └─ Select region with lowest latency                    │
│                                                              │
│  4. Cache Result                                             │
│     └─ Store for future requests                            │
└─────────────────────────────────────────────────────────────┘
```

## Supported Regions

| Region | Code | Description |
|--------|------|-------------|
| China | `China` | Mainland China with specialized mirrors |
| Asia Pacific | `AsiaPacific` | Japan, Korea, Southeast Asia, etc. |
| Europe | `Europe` | European countries |
| North America | `NorthAmerica` | US, Canada, Mexico |
| Global | `Global` | Default for other regions |

## Region-Specific Optimizations

### China Region

Turbo CDN provides comprehensive mirror coverage for users in China:

| Service | Mirrors |
|---------|---------|
| GitHub | ghfast.top, gh.con.sh, cors.isteed.cc, github.moeyy.xyz, mirror.ghproxy.com, ghproxy.net |
| PyPI | Tsinghua, Aliyun, Douban |
| Crates.io | Tsinghua, USTC |
| Go Modules | goproxy.cn, Aliyun |
| Docker Hub | USTC, NetEase, Docker China |
| Maven | Aliyun, Tsinghua |

### Global Regions

For users outside China:

| Service | CDN Nodes |
|---------|-----------|
| jsDelivr | fastly, gcore, testingcf, jsdelivr.b-cdn |
| Cloudflare | Global edge network |
| Fastly | High-performance CDN |
| unpkg | npm package distribution |

## Configuration

### Automatic Detection (Default)

```rust
use turbo_cdn::*;

#[tokio::main]
async fn main() -> turbo_cdn::Result<()> {
    // Region is automatically detected
    let downloader = TurboCdn::new().await?;
    
    // Download uses optimal mirrors for detected region
    let result = downloader.download_from_url("https://example.com/file.zip").await?;
    Ok(())
}
```

### Manual Region Setting

```rust
use turbo_cdn::*;

#[tokio::main]
async fn main() -> turbo_cdn::Result<()> {
    let downloader = TurboCdn::builder()
        .with_region(Region::China)  // Explicitly set region
        .build()
        .await?;
    
    let result = downloader.download_from_url("https://example.com/file.zip").await?;
    Ok(())
}
```

### Available Regions

```rust
pub enum Region {
    China,
    AsiaPacific,
    Europe,
    NorthAmerica,
    Global,
}
```

## Caching

Geographic detection results are cached to avoid repeated API calls:

- **Cache Duration**: Configurable (default: 1 hour)
- **Cache Storage**: In-memory with optional persistence
- **Cache Invalidation**: Automatic on network change detection

## API Fallbacks

Turbo CDN uses multiple IP geolocation APIs with automatic fallback:

1. **Primary**: ip-api.com (free, no key required)
2. **Secondary**: ipinfo.io (free tier available)
3. **Tertiary**: Network latency testing

If all IP-based detection fails, the system falls back to network performance testing, measuring latency to known servers in each region.

## Performance Impact

| Scenario | Detection Time |
|----------|---------------|
| Cached result | < 1ms |
| IP geolocation | 50-200ms |
| Network testing | 500-2000ms |

The caching system ensures that geographic detection overhead is minimal for most operations.

## Troubleshooting

### Detection Issues

If region detection seems incorrect:

1. **Check network**: Ensure stable internet connection
2. **VPN/Proxy**: May affect IP-based detection
3. **Manual override**: Set region explicitly if needed

```rust
// Force a specific region
let downloader = TurboCdn::builder()
    .with_region(Region::China)
    .build()
    .await?;
```

### Logging

Enable debug logging to see detection details:

```bash
RUST_LOG=turbo_cdn=debug turbo-cdn dl "https://example.com/file.zip"
```

## Next Steps

- [CDN Quality Assessment](/guide/cdn-quality) - How mirrors are ranked
- [Smart Download](/guide/smart-download) - Automatic method selection
