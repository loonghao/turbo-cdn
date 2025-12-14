// Licensed under the MIT License
// Copyright (c) 2025 Hal <hal.long@outlook.com>

//! Performance benchmarks for turbo-cdn
//!
//! Run with: `cargo bench --bench download_benchmarks`
//!
//! Note: Some benchmarks require network access and are marked with `_network` suffix.
//! These will perform real downloads and measure actual throughput.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::time::Duration;
use turbo_cdn::adaptive_concurrency::{AdaptiveConcurrencyController, CongestionMetrics};
use turbo_cdn::config::{Region, TurboCdnConfig};
use turbo_cdn::server_tracker::ServerTracker;
use turbo_cdn::url_mapper::UrlMapper;

// ============================================================================
// URL Mapping Benchmarks
// ============================================================================

fn bench_url_mapping(c: &mut Criterion) {
    let config = TurboCdnConfig::load().unwrap_or_default();
    let mapper = UrlMapper::new(&config, Region::China).expect("Failed to create UrlMapper");

    let test_urls = [
        "https://github.com/BurntSushi/ripgrep/releases/download/14.1.1/ripgrep-14.1.1-x86_64-pc-windows-msvc.zip",
        "https://cdn.jsdelivr.net/npm/jquery@3.7.1/dist/jquery.min.js",
        "https://registry.npmjs.org/lodash",
        "https://pypi.org/simple/requests/",
        "https://example.com/unknown/file.zip",
    ];

    let mut group = c.benchmark_group("url_mapping");

    for url in test_urls.iter() {
        let url_short = if url.len() > 40 {
            format!("{}...", &url[..40])
        } else {
            url.to_string()
        };

        group.bench_with_input(BenchmarkId::new("map_url", &url_short), url, |b, url| {
            b.iter(|| {
                let _ = black_box(mapper.map_url(url));
            })
        });
    }

    group.finish();
}

fn bench_url_mapping_batch(c: &mut Criterion) {
    let config = TurboCdnConfig::load().unwrap_or_default();
    let mapper = UrlMapper::new(&config, Region::China).expect("Failed to create UrlMapper");

    let test_urls: Vec<&str> = vec![
        "https://github.com/user/repo1/releases/download/v1.0.0/file1.zip",
        "https://github.com/user/repo2/releases/download/v2.0.0/file2.zip",
        "https://github.com/user/repo3/releases/download/v3.0.0/file3.zip",
        "https://cdn.jsdelivr.net/npm/package1@1.0.0/dist/file.js",
        "https://cdn.jsdelivr.net/npm/package2@2.0.0/dist/file.js",
    ];

    c.bench_function("url_mapping_batch_5", |b| {
        b.iter(|| {
            for url in test_urls.iter() {
                let _ = black_box(mapper.map_url(url));
            }
        })
    });
}

fn bench_url_mapping_cache(c: &mut Criterion) {
    let config = TurboCdnConfig::load().unwrap_or_default();
    let mapper = UrlMapper::new(&config, Region::China).expect("Failed to create UrlMapper");

    let url = "https://github.com/BurntSushi/ripgrep/releases/download/14.1.1/ripgrep.zip";

    // First call to populate cache
    let _ = mapper.map_url(url);

    c.bench_function("url_mapping_cached", |b| {
        b.iter(|| {
            let _ = black_box(mapper.map_url(url));
        })
    });
}

// ============================================================================
// Server Tracker Benchmarks
// ============================================================================

fn bench_server_tracker(c: &mut Criterion) {
    let mut group = c.benchmark_group("server_tracker");

    // Benchmark recording success
    group.bench_function("record_success", |b| {
        let mut tracker = ServerTracker::new();
        let url = "http://test.example.com";

        b.iter(|| {
            tracker.record_success(
                black_box(url),
                black_box(1024.0 * 1024.0),
                black_box(Duration::from_millis(100)),
            );
        })
    });

    // Benchmark recording failure
    group.bench_function("record_failure", |b| {
        let mut tracker = ServerTracker::new();
        let url = "http://test.example.com";

        b.iter(|| {
            tracker.record_failure(black_box(url), black_box(Duration::from_millis(500)));
        })
    });

    // Benchmark server selection with different server counts
    for server_count in [10, 50, 100, 500].iter() {
        let mut tracker = ServerTracker::new();
        let urls: Vec<String> = (0..*server_count)
            .map(|i| format!("http://server{}.example.com", i))
            .collect();

        // Populate tracker with data
        for (i, url) in urls.iter().enumerate() {
            let speed = ((i + 1) as f64) * 1024.0 * 1024.0;
            tracker.record_success(url, speed, Duration::from_millis(100));
        }

        group.bench_with_input(
            BenchmarkId::new("select_best_servers", server_count),
            &urls,
            |b, urls| {
                b.iter(|| {
                    let _ = black_box(tracker.select_best_servers(urls, 5));
                })
            },
        );
    }

    group.finish();
}

// ============================================================================
// Adaptive Concurrency Benchmarks
// ============================================================================

fn bench_adaptive_concurrency(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let mut group = c.benchmark_group("adaptive_concurrency");

    // Benchmark controller creation
    group.bench_function("controller_new", |b| {
        let config = TurboCdnConfig::default();
        b.iter(|| {
            let _ = black_box(AdaptiveConcurrencyController::new(&config));
        })
    });

    // Benchmark record_success
    group.bench_function("record_success", |b| {
        let config = TurboCdnConfig::default();
        let controller = AdaptiveConcurrencyController::new(&config);

        b.to_async(&rt).iter(|| async {
            controller
                .record_success(
                    black_box(Duration::from_millis(100)),
                    black_box(1024 * 1024),
                )
                .await;
        })
    });

    // Benchmark record_failure
    group.bench_function("record_failure", |b| {
        let config = TurboCdnConfig::default();
        let controller = AdaptiveConcurrencyController::new(&config);

        b.to_async(&rt).iter(|| async {
            controller.record_failure(black_box("test_error")).await;
        })
    });

    // Benchmark get_stats
    group.bench_function("get_stats", |b| {
        let config = TurboCdnConfig::default();
        let controller = AdaptiveConcurrencyController::new(&config);

        b.to_async(&rt).iter(|| async {
            let _ = black_box(controller.get_stats().await);
        })
    });

    // Benchmark adaptive_backoff
    group.bench_function("adaptive_backoff", |b| {
        let config = TurboCdnConfig::default();
        let controller = AdaptiveConcurrencyController::new(&config);

        b.to_async(&rt).iter(|| async {
            let _ = black_box(controller.adaptive_backoff(3).await);
        })
    });

    group.finish();
}

// ============================================================================
// Configuration Benchmarks
// ============================================================================

fn bench_config(c: &mut Criterion) {
    let mut group = c.benchmark_group("config");

    group.bench_function("load_default", |b| {
        b.iter(|| {
            let _ = black_box(TurboCdnConfig::default());
        })
    });

    group.bench_function("load_from_embedded", |b| {
        b.iter(|| {
            let _ = black_box(TurboCdnConfig::load());
        })
    });

    group.finish();
}

// ============================================================================
// CongestionMetrics Benchmarks
// ============================================================================

fn bench_congestion_metrics(c: &mut Criterion) {
    c.bench_function("congestion_metrics_default", |b| {
        b.iter(|| {
            let _ = black_box(CongestionMetrics::default());
        })
    });

    c.bench_function("congestion_metrics_clone", |b| {
        let metrics = CongestionMetrics::default();
        b.iter(|| {
            let _ = black_box(metrics.clone());
        })
    });
}

// ============================================================================
// TurboCdn Client Benchmarks
// ============================================================================

fn bench_turbo_cdn_client(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let mut group = c.benchmark_group("turbo_cdn_client");

    // Benchmark client creation
    group.bench_function("new", |b| {
        b.to_async(&rt).iter(|| async {
            let _ = black_box(turbo_cdn::TurboCdn::new().await);
        })
    });

    // Benchmark builder pattern
    group.bench_function("builder", |b| {
        b.to_async(&rt).iter(|| async {
            let _ = black_box(
                turbo_cdn::TurboCdn::builder()
                    .with_auto_detect_region(false)
                    .with_region(Region::Global)
                    .with_max_concurrent_downloads(16)
                    .build()
                    .await,
            );
        })
    });

    // Benchmark get_optimal_url
    group.bench_function("get_optimal_url", |b| {
        let cdn = rt.block_on(async {
            turbo_cdn::TurboCdn::builder()
                .with_auto_detect_region(false)
                .with_region(Region::China)
                .build()
                .await
                .unwrap()
        });

        let url = "https://github.com/user/repo/releases/download/v1.0.0/file.zip";

        b.to_async(&rt).iter(|| async {
            let _ = black_box(cdn.get_optimal_url(url).await);
        })
    });

    // Benchmark get_all_cdn_urls
    group.bench_function("get_all_cdn_urls", |b| {
        let cdn = rt.block_on(async {
            turbo_cdn::TurboCdn::builder()
                .with_auto_detect_region(false)
                .with_region(Region::China)
                .build()
                .await
                .unwrap()
        });

        let url = "https://github.com/user/repo/releases/download/v1.0.0/file.zip";

        b.to_async(&rt).iter(|| async {
            let _ = black_box(cdn.get_all_cdn_urls(url).await);
        })
    });

    // Benchmark can_optimize_url
    group.bench_function("can_optimize_url", |b| {
        let cdn = rt.block_on(async {
            turbo_cdn::TurboCdn::builder()
                .with_auto_detect_region(false)
                .with_region(Region::China)
                .build()
                .await
                .unwrap()
        });

        let url = "https://github.com/user/repo/releases/download/v1.0.0/file.zip";

        b.to_async(&rt).iter(|| async {
            let _ = black_box(cdn.can_optimize_url(url).await);
        })
    });

    group.finish();
}

// ============================================================================
// Throughput Benchmarks (simulated)
// ============================================================================

fn bench_throughput_simulation(c: &mut Criterion) {
    let mut group = c.benchmark_group("throughput_simulation");

    // Simulate different data sizes
    for size in [1024, 1024 * 1024, 10 * 1024 * 1024].iter() {
        group.throughput(Throughput::Bytes(*size as u64));

        group.bench_with_input(
            BenchmarkId::new("simulated_transfer", size),
            size,
            |b, &size| {
                let data = vec![0u8; size];
                b.iter(|| {
                    // Simulate data processing
                    let sum: u64 = data.iter().map(|&x| x as u64).sum();
                    black_box(sum)
                })
            },
        );
    }

    group.finish();
}

// ============================================================================
// Criterion Groups
// ============================================================================

criterion_group!(
    name = url_benches;
    config = Criterion::default().sample_size(100);
    targets = bench_url_mapping, bench_url_mapping_batch, bench_url_mapping_cache
);

criterion_group!(
    name = tracker_benches;
    config = Criterion::default().sample_size(100);
    targets = bench_server_tracker
);

criterion_group!(
    name = concurrency_benches;
    config = Criterion::default().sample_size(50);
    targets = bench_adaptive_concurrency, bench_congestion_metrics
);

criterion_group!(
    name = config_benches;
    config = Criterion::default().sample_size(100);
    targets = bench_config
);

criterion_group!(
    name = client_benches;
    config = Criterion::default().sample_size(20);
    targets = bench_turbo_cdn_client
);

criterion_group!(
    name = throughput_benches;
    config = Criterion::default().sample_size(50);
    targets = bench_throughput_simulation
);

criterion_main!(
    url_benches,
    tracker_benches,
    concurrency_benches,
    config_benches,
    client_benches,
    throughput_benches
);
