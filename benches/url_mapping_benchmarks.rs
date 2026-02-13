use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use std::collections::HashMap;
use std::hint::black_box;
use turbo_cdn::string_interner::StringInterner;
use turbo_cdn::url_mapper::UrlMapper;
use turbo_cdn::{Region, TurboCdnConfig};

/// Benchmark URL mapping string operations
fn benchmark_url_mapping(c: &mut Criterion) {
    let config = TurboCdnConfig::default();

    // Test URLs for benchmarking
    let test_urls = vec![
        "https://github.com/BurntSushi/ripgrep/releases/download/14.1.1/ripgrep-14.1.1-x86_64-pc-windows-msvc.zip",
        "https://github.com/sharkdp/fd/releases/download/v8.7.0/fd-v8.7.0-x86_64-pc-windows-msvc.zip",
        "https://github.com/sharkdp/bat/releases/download/v0.24.0/bat-v0.24.0-x86_64-pc-windows-msvc.zip",
        "https://cdn.jsdelivr.net/npm/typescript@5.3.3/lib/typescript.js",
        "https://registry.npmjs.org/typescript/-/typescript-5.3.3.tgz",
        "https://files.pythonhosted.org/packages/source/r/requests/requests-2.31.0.tar.gz",
        "https://crates.io/api/v1/crates/serde/1.0.193/download",
        "https://repo1.maven.org/maven2/org/apache/commons/commons-lang3/3.12.0/commons-lang3-3.12.0.jar",
    ];

    let mut group = c.benchmark_group("url_mapping");

    // Benchmark single URL mapping
    group.bench_function("single_url_mapping", |b| {
        let mut mapper = UrlMapper::new(&config, Region::Global).unwrap();
        b.iter(|| {
            let url = black_box(test_urls[0]);
            let _result = mapper.map_url(url).unwrap();
        });
    });

    // Benchmark repeated URL mapping (cache performance)
    group.bench_function("repeated_url_mapping", |b| {
        let mut mapper = UrlMapper::new(&config, Region::Global).unwrap();
        let url = test_urls[0];
        b.iter(|| {
            let url = black_box(url);
            let _result = mapper.map_url(url).unwrap();
        });
    });

    // Benchmark multiple URL mapping
    group.bench_function("multiple_url_mapping", |b| {
        let mut mapper = UrlMapper::new(&config, Region::Global).unwrap();
        b.iter(|| {
            for url in black_box(&test_urls) {
                let _result = mapper.map_url(url).unwrap();
            }
        });
    });

    // Benchmark URL mapping with different regions
    for region in [
        Region::Global,
        Region::China,
        Region::Europe,
        Region::NorthAmerica,
    ] {
        group.bench_with_input(
            BenchmarkId::new("region_mapping", format!("{:?}", region)),
            &region,
            |b, region| {
                let mut mapper = UrlMapper::new(&config, region.clone()).unwrap();
                b.iter(|| {
                    for url in black_box(&test_urls) {
                        let _result = mapper.map_url(url).unwrap();
                    }
                });
            },
        );
    }

    group.finish();
}

/// Benchmark string interning performance
fn benchmark_string_interning(c: &mut Criterion) {
    let mut group = c.benchmark_group("string_interning");

    // Test strings for interning
    let test_strings = vec![
        "https://github.com/owner/repo/releases/download/v1.0.0/file.zip",
        "https://cdn.jsdelivr.net/npm/package@1.0.0/dist/package.min.js",
        "https://registry.npmjs.org/package/-/package-1.0.0.tgz",
        "https://files.pythonhosted.org/packages/source/p/package/package-1.0.0.tar.gz",
        "https://crates.io/api/v1/crates/package/1.0.0/download",
    ];

    // Benchmark string interning
    group.bench_function("string_intern", |b| {
        let interner = StringInterner::new();
        b.iter(|| {
            for s in black_box(&test_strings) {
                let _interned = interner.intern(s);
            }
        });
    });

    // Benchmark repeated string interning (cache hits)
    group.bench_function("repeated_string_intern", |b| {
        let interner = StringInterner::new();
        let test_string = test_strings[0];
        b.iter(|| {
            let s = black_box(test_string);
            let _interned = interner.intern(s);
        });
    });

    // Benchmark string interning vs cloning
    group.bench_function("string_clone_vs_intern", |b| {
        let interner = StringInterner::new();
        b.iter(|| {
            for s in black_box(&test_strings) {
                // Compare interning vs cloning
                let _interned = interner.intern(s);
                let _cloned = s.to_string();
            }
        });
    });

    // Benchmark global interner
    group.bench_function("global_interner", |b| {
        b.iter(|| {
            for s in black_box(&test_strings) {
                let _interned = turbo_cdn::string_interner::intern_string(s);
            }
        });
    });

    group.finish();
}

/// Benchmark cache performance
fn benchmark_cache_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("cache_performance");
    let config = TurboCdnConfig::default();

    // Test cache hit rates
    group.bench_function("cache_hit_rate", |b| {
        let mut mapper = UrlMapper::new(&config, Region::Global).unwrap();
        let test_url = "https://github.com/BurntSushi/ripgrep/releases/download/14.1.1/ripgrep-14.1.1-x86_64-pc-windows-msvc.zip";
        // Prime the cache
        let _ = mapper.map_url(test_url).unwrap();
        b.iter(|| {
            let url = black_box(test_url);
            let _result = mapper.map_url(url).unwrap();
        });
    });

    // Test cache miss performance
    group.bench_function("cache_miss_rate", |b| {
        let mut mapper = UrlMapper::new(&config, Region::Global).unwrap();
        let mut counter = 0;

        b.iter(|| {
            let url = format!("https://example.com/file{}.zip", counter);
            counter += 1;
            let _result = mapper.map_url(black_box(&url)).unwrap();
        });
    });

    group.finish();
}

/// Benchmark memory usage patterns
fn benchmark_memory_usage(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_usage");
    let config = TurboCdnConfig::default();

    // Test memory usage with many URLs
    group.bench_function("memory_usage_many_urls", |b| {
        b.iter(|| {
            let mut mapper = UrlMapper::new(&config, Region::Global).unwrap();

            // Process many different URLs to test memory usage
            for i in 0..1000 {
                let url = format!(
                    "https://github.com/owner{}/repo{}/releases/download/v1.0.{}/file{}.zip",
                    i % 10,
                    i % 20,
                    i % 5,
                    i
                );
                let _result = mapper.map_url(black_box(&url)).unwrap();
            }
        });
    });

    // Test string interner memory efficiency
    group.bench_function("interner_memory_efficiency", |b| {
        b.iter(|| {
            let interner = StringInterner::new();

            // Intern many similar strings
            for i in 0..1000 {
                let base_url = "https://github.com/owner/repo/releases/download/v1.0.0/file";
                let url = format!("{}{}.zip", base_url, i % 10); // Only 10 unique endings
                let _interned = interner.intern(black_box(&url));
            }

            // Check interner stats
            let stats = interner.stats();
            assert!(stats.hit_rate > 80.0); // Should have high hit rate due to similar patterns
        });
    });

    group.finish();
}

/// Performance comparison benchmark
fn benchmark_performance_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("performance_comparison");
    let config = TurboCdnConfig::default();

    // Compare optimized vs unoptimized string operations
    let test_urls = vec![
        "https://github.com/BurntSushi/ripgrep/releases/download/14.1.1/ripgrep-14.1.1-x86_64-pc-windows-msvc.zip",
        "https://github.com/sharkdp/fd/releases/download/v8.7.0/fd-v8.7.0-x86_64-pc-windows-msvc.zip",
        "https://github.com/sharkdp/bat/releases/download/v0.24.0/bat-v0.24.0-x86_64-pc-windows-msvc.zip",
    ];

    // Benchmark optimized version (current implementation)
    group.bench_function("optimized_string_ops", |b| {
        let mut mapper = UrlMapper::new(&config, Region::Global).unwrap();
        b.iter(|| {
            for url in black_box(&test_urls) {
                let _result = mapper.map_url(url).unwrap();
            }
        });
    });

    // Simulate unoptimized version (using String clones)
    group.bench_function("unoptimized_string_ops", |b| {
        b.iter(|| {
            let mut url_cache: HashMap<String, Vec<String>> = HashMap::new();

            for url in black_box(&test_urls) {
                // Simulate the old approach with String clones
                let key = url.to_string(); // Clone for key
                if let Some(cached) = url_cache.get(&key) {
                    let _result = cached.clone(); // Clone the result
                } else {
                    let result = vec![url.to_string(), format!("optimized-{}", url)];
                    url_cache.insert(key, result.clone()); // Clone for storage
                    let _result = result;
                }
            }
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    benchmark_url_mapping,
    benchmark_string_interning,
    benchmark_cache_performance,
    benchmark_memory_usage,
    benchmark_performance_comparison
);
criterion_main!(benches);
