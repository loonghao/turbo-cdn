use criterion::{criterion_group, criterion_main, Criterion};
use std::hint::black_box;
use turbo_cdn::config::{Region, TurboCdnConfig};
use turbo_cdn::url_mapper::UrlMapper;

fn benchmark_url_mapping_operations(c: &mut Criterion) {
    let config = TurboCdnConfig::default();
    let mut mapper = UrlMapper::new(&config, Region::Global).unwrap();

    // Test URLs for benchmarking
    let test_urls = vec![
        "https://github.com/BurntSushi/ripgrep/releases/download/14.1.1/ripgrep-14.1.1-x86_64-pc-windows-msvc.zip",
        "https://cdn.jsdelivr.net/npm/package@1.0.0/dist/package.min.js",
        "https://pypi.org/simple/requests/requests-2.28.1-py3-none-any.whl",
        "https://registry-1.docker.io/v2/library/nginx/manifests/latest",
        "https://crates.io/api/v1/crates/serde/1.0.152/download",
    ];

    c.bench_function("url_mapping_single", |b| {
        b.iter(|| {
            let url = black_box(test_urls[0]);
            mapper.map_url(url).unwrap()
        })
    });

    c.bench_function("url_mapping_multiple", |b| {
        b.iter(|| {
            for url in &test_urls {
                let _ = black_box(mapper.map_url(url).unwrap());
            }
        })
    });

    c.bench_function("url_mapping_with_cache", |b| {
        // Pre-populate cache
        for url in &test_urls {
            let _ = mapper.map_url(url).unwrap();
        }

        b.iter(|| {
            let url = black_box(test_urls[0]);
            mapper.map_url(url).unwrap()
        })
    });

    c.bench_function("string_interning_performance", |b| {
        use turbo_cdn::string_interner::{global_interner_stats, intern_string};

        let test_strings = vec![
            "https://github.com/user/repo/releases/download/v1.0.0/file.zip",
            "https://cdn.jsdelivr.net/npm/package@1.0.0/dist/file.js",
            "https://pypi.org/simple/package/package-1.0.0-py3-none-any.whl",
        ];

        b.iter(|| {
            for s in &test_strings {
                let _ = black_box(intern_string(s));
            }
        });

        // Print stats after benchmark
        let stats = global_interner_stats();
        println!(
            "String interner stats: hit_rate={:.2}%, cache_size={}",
            stats.hit_rate, stats.cache_size
        );
    });
}

criterion_group!(benches, benchmark_url_mapping_operations);
criterion_main!(benches);
