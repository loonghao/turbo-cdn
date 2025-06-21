// Licensed under the MIT License
// Copyright (c) 2025 Hal <hal.long@outlook.com>

//! # URL Optimization Example
//!
//! This example demonstrates how to use turbo-cdn to automatically optimize
//! download URLs from various sources for the user's geographic location.

use turbo_cdn::{DownloadOptions, TurboCdn};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    turbo_cdn::init_tracing();

    // Create a TurboCdn client
    let client = TurboCdn::new().await?;

    println!("🚀 Turbo CDN URL Optimization Demo");
    println!("==================================\n");

    // Example URLs from various sources (using smaller, more accessible files)
    let example_urls = vec![
        (
            "GitHub Releases",
            "https://github.com/rust-lang/mdBook/releases/download/v0.4.21/mdbook-v0.4.21-x86_64-unknown-linux-gnu.tar.gz"
        ),
        (
            "jsDelivr CDN",
            "https://cdn.jsdelivr.net/gh/jquery/jquery@3.6.0/dist/jquery.min.js"
        ),
        (
            "Cloudflare CDN",
            "https://cdnjs.cloudflare.com/ajax/libs/lodash.js/4.17.21/lodash.min.js"
        ),
        (
            "npm Registry",
            "https://registry.npmjs.org/express/-/express-4.18.2.tgz"
        ),
        (
            "PyPI",
            "https://files.pythonhosted.org/packages/source/c/click/click-8.1.3.tar.gz"
        ),
        (
            "Go Proxy",
            "https://proxy.golang.org/github.com/gorilla/mux/@v/v1.8.0.zip"
        ),
        (
            "Crates.io",
            "https://crates.io/api/v1/crates/tokio/1.28.0/download"
        ),
        (
            "Maven Central",
            "https://repo1.maven.org/maven2/com/fasterxml/jackson/core/jackson-core/2.15.2/jackson-core-2.15.2.jar"
        ),
    ];

    for (source_name, original_url) in example_urls {
        println!("📦 Source: {}", source_name);
        println!("🔗 Original URL: {}", original_url);

        // Parse the URL to show detected information
        match client.parse_url(original_url) {
            Ok(parsed) => {
                println!("   📋 Repository: {}", parsed.repository);
                println!("   🏷️  Version: {}", parsed.version);
                println!("   📄 Filename: {}", parsed.filename);
                println!("   🔍 Detected Source: {:?}", parsed.source_type);

                // Get the optimal URL for user's location
                match client.get_optimal_url(original_url).await {
                    Ok(optimal_url) => {
                        println!("   ⚡ Optimal URL: {}", optimal_url);

                        if optimal_url != original_url {
                            println!("   ✅ URL optimized for your location!");
                        } else {
                            println!("   ℹ️  Original URL is already optimal");
                        }
                    }
                    Err(e) => {
                        println!("   ❌ Failed to get optimal URL: {}", e);
                    }
                }
            }
            Err(e) => {
                println!("   ❌ Failed to parse URL: {}", e);
            }
        }

        println!(); // Empty line for readability
    }

    // Demonstrate actual download with optimization
    println!("🔄 Demonstrating Download with Automatic Optimization");
    println!("====================================================\n");

    let demo_url = "https://cdn.jsdelivr.net/gh/jquery/jquery@3.6.0/dist/jquery.min.js";
    println!("📥 Downloading: {}", demo_url);

    let _download_options = DownloadOptions {
        verify_checksum: true,
        use_cache: true,
        ..Default::default()
    };

    // This would perform the actual download in a real implementation
    // For demo purposes, we'll just show the optimization
    match client.get_optimal_url(demo_url).await {
        Ok(optimal_url) => {
            println!("   🎯 Optimized download URL: {}", optimal_url);
            println!("   💡 The download would use this optimized URL automatically");

            // In a real scenario, you would call:
            // let result = client.download_from_url(demo_url, Some(download_options)).await?;
            // println!("   ✅ Downloaded to: {}", result.path.display());
        }
        Err(e) => {
            println!("   ❌ Optimization failed: {}", e);
        }
    }

    println!("\n🎉 Demo completed!");
    println!("\n💡 Key Benefits:");
    println!("   • Automatic geographic optimization");
    println!("   • Support for 10+ major package sources");
    println!("   • Intelligent failover and load balancing");
    println!("   • Transparent URL parsing and conversion");
    println!("   • No manual CDN selection required");

    Ok(())
}

/// Example of version extraction from various filename patterns
#[allow(dead_code)]
async fn demonstrate_version_extraction() -> Result<(), Box<dyn std::error::Error>> {
    let client = TurboCdn::new().await?;

    println!("🔍 Version Extraction Examples");
    println!("==============================\n");

    let test_filenames = vec![
        "app-v1.2.3.zip",
        "tool-2.0.tar.gz",
        "package-2023-12-01.exe",
        "file-20231201.dmg",
        "library-1.0.0-beta.jar",
        "noversion.zip",
    ];

    for filename in test_filenames {
        match client.extract_version_from_filename(filename) {
            Some(version) => {
                println!("📄 {} → Version: {}", filename, version);
            }
            None => {
                println!("📄 {} → No version detected", filename);
            }
        }
    }

    Ok(())
}

/// Example of handling different URL formats
#[allow(dead_code)]
async fn demonstrate_url_formats() -> Result<(), Box<dyn std::error::Error>> {
    let client = TurboCdn::new().await?;

    println!("🌐 Supported URL Formats");
    println!("========================\n");

    let url_examples = vec![
        (
            "GitHub",
            "https://github.com/owner/repo/releases/download/v1.0.0/file.zip",
        ),
        (
            "GitLab",
            "https://gitlab.com/owner/repo/-/releases/v1.0.0/downloads/file.zip",
        ),
        (
            "Bitbucket",
            "https://bitbucket.org/owner/repo/downloads/file-v1.0.0.zip",
        ),
        (
            "jsDelivr",
            "https://cdn.jsdelivr.net/gh/owner/repo@v1.0.0/file.js",
        ),
        (
            "Fastly",
            "https://fastly.jsdelivr.net/gh/owner/repo@v1.0.0/file.js",
        ),
        (
            "Cloudflare",
            "https://cdnjs.cloudflare.com/ajax/libs/library/1.0.0/file.min.js",
        ),
        (
            "npm",
            "https://registry.npmjs.org/package/-/package-1.0.0.tgz",
        ),
        (
            "PyPI",
            "https://files.pythonhosted.org/packages/source/p/package/package-1.0.0.tar.gz",
        ),
        (
            "Go Proxy",
            "https://proxy.golang.org/example.com/module/@v/v1.0.0.zip",
        ),
        (
            "Crates.io",
            "https://crates.io/api/v1/crates/serde/1.0.0/download",
        ),
        (
            "Maven",
            "https://repo1.maven.org/maven2/org/example/artifact/1.0.0/artifact-1.0.0.jar",
        ),
        (
            "NuGet",
            "https://api.nuget.org/v3-flatcontainer/package/1.0.0/package.1.0.0.nupkg",
        ),
        (
            "Docker Hub",
            "https://registry-1.docker.io/v2/library/nginx/manifests/latest",
        ),
        (
            "SourceForge",
            "https://downloads.sourceforge.net/project/example/file-1.0.0.zip",
        ),
    ];

    for (platform, url) in url_examples {
        println!("🔗 {}: {}", platform, url);
        match client.parse_url(url) {
            Ok(parsed) => {
                println!("   ✅ Parsed successfully");
                println!("   📦 Repository: {}", parsed.repository);
                println!("   🏷️  Version: {}", parsed.version);
                println!("   📄 Filename: {}", parsed.filename);
            }
            Err(e) => {
                println!("   ❌ Parse error: {}", e);
            }
        }
        println!();
    }

    Ok(())
}
