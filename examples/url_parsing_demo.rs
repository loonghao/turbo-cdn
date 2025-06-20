// Licensed under the MIT License
// Copyright (c) 2025 Hal <hal.long@outlook.com>

//! # URL Parsing Demo
//!
//! This example demonstrates the URL parsing capabilities of turbo-cdn
//! without relying on external API calls.

use turbo_cdn::TurboCdn;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    turbo_cdn::init_tracing();

    // Create a TurboCdn client
    let client = TurboCdn::new().await?;

    println!("🔍 Turbo CDN URL Parsing Demo");
    println!("============================\n");

    // Example URLs from various sources
    let example_urls = vec![
        (
            "GitHub Releases",
            "https://github.com/rust-lang/mdBook/releases/download/v0.4.21/mdbook-v0.4.21-x86_64-unknown-linux-gnu.tar.gz"
        ),
        (
            "GitLab Releases", 
            "https://gitlab.com/gitlab-org/gitlab/-/releases/v15.8.0/downloads/gitlab-v15.8.0.tar.gz"
        ),
        (
            "Bitbucket Downloads",
            "https://bitbucket.org/atlassian/atlassian-connect-express/downloads/atlassian-connect-express-v7.0.1.zip"
        ),
        (
            "jsDelivr CDN", 
            "https://cdn.jsdelivr.net/gh/jquery/jquery@3.6.0/dist/jquery.min.js"
        ),
        (
            "Fastly CDN",
            "https://fastly.jsdelivr.net/gh/lodash/lodash@4.17.21/lodash.min.js"
        ),
        (
            "Cloudflare CDN",
            "https://cdnjs.cloudflare.com/ajax/libs/react/18.2.0/umd/react.production.min.js"
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
        (
            "NuGet",
            "https://api.nuget.org/v3-flatcontainer/newtonsoft.json/13.0.3/newtonsoft.json.13.0.3.nupkg"
        ),
        (
            "Docker Hub",
            "https://registry-1.docker.io/v2/library/nginx/manifests/latest"
        ),
        (
            "SourceForge",
            "https://downloads.sourceforge.net/project/sevenzip/7-Zip/22.01/7z2201-x64.exe"
        ),
    ];

    for (source_name, url) in example_urls {
        println!("📦 Source: {}", source_name);
        println!("🔗 URL: {}", url);

        match client.parse_url(url) {
            Ok(parsed) => {
                println!("   ✅ Parsing successful!");
                println!("   📋 Repository: {}", parsed.repository);
                println!("   🏷️  Version: {}", parsed.version);
                println!("   📄 Filename: {}", parsed.filename);
                println!("   🔍 Detected Source: {:?}", parsed.source_type);
            }
            Err(e) => {
                println!("   ❌ Parsing failed: {}", e);
            }
        }

        println!(); // Empty line for readability
    }

    // Demonstrate version extraction from various filename patterns
    println!("🔍 Version Extraction Examples");
    println!("==============================\n");

    let test_filenames = vec![
        "app-v1.2.3.zip",
        "tool-2.0.tar.gz",
        "package-2023-12-01.exe",
        "file-20231201.dmg",
        "library-1.0.0-beta.jar",
        "simple-v0.1.0-alpha.1.tgz",
        "complex-name-v2.5.10.zip",
        "noversion.zip",
        "another-file.tar.gz",
        "version-in-middle-v3.1.4-final.exe",
    ];

    for filename in test_filenames {
        match client.extract_version_from_filename(filename) {
            Some(version) => {
                println!("📄 {} → ✅ Version: {}", filename, version);
            }
            None => {
                println!("📄 {} → ❌ No version detected", filename);
            }
        }
    }

    println!("\n🎯 URL Format Support Summary");
    println!("=============================\n");

    let supported_formats = vec![
        (
            "GitHub",
            "github.com/{owner}/{repo}/releases/download/{tag}/{filename}",
        ),
        (
            "GitLab",
            "gitlab.com/{owner}/{repo}/-/releases/{tag}/downloads/{filename}",
        ),
        (
            "Bitbucket",
            "bitbucket.org/{owner}/{repo}/downloads/{filename}",
        ),
        (
            "jsDelivr",
            "cdn.jsdelivr.net/gh/{owner}/{repo}@{tag}/{filename}",
        ),
        (
            "Fastly",
            "fastly.jsdelivr.net/gh/{owner}/{repo}@{tag}/{filename}",
        ),
        (
            "Cloudflare",
            "cdnjs.cloudflare.com/ajax/libs/{library}/{version}/{filename}",
        ),
        (
            "npm",
            "registry.npmjs.org/{package}/-/{package}-{version}.tgz",
        ),
        (
            "PyPI",
            "files.pythonhosted.org/packages/source/{letter}/{package}/{package}-{version}.tar.gz",
        ),
        ("Go Proxy", "proxy.golang.org/{module}/@v/{version}.zip"),
        (
            "Crates.io",
            "crates.io/api/v1/crates/{crate}/{version}/download",
        ),
        (
            "Maven",
            "repo1.maven.org/maven2/{group}/{artifact}/{version}/{artifact}-{version}.jar",
        ),
        (
            "NuGet",
            "api.nuget.org/v3-flatcontainer/{package}/{version}/{package}.{version}.nupkg",
        ),
        (
            "Docker Hub",
            "registry-1.docker.io/v2/library/{image}/manifests/{tag}",
        ),
        (
            "SourceForge",
            "downloads.sourceforge.net/project/{project}/{filename}",
        ),
    ];

    for (platform, format) in supported_formats {
        println!("🌐 {}: {}", platform, format);
    }

    println!("\n🚀 Key Features");
    println!("===============\n");
    println!("✅ Universal URL parsing for 14+ major package sources");
    println!("✅ Automatic version extraction from filenames");
    println!("✅ Strong type safety with DetectedSourceType enum");
    println!("✅ Comprehensive error handling and validation");
    println!("✅ Support for complex repository paths and nested files");
    println!("✅ Intelligent handling of different URL patterns");
    println!("✅ Ready for geographic optimization and CDN selection");

    println!("\n💡 Next Steps");
    println!("=============\n");
    println!("1. Run 'cargo run --example url_optimization' for full optimization demo");
    println!("2. Add GitHub token to config for better API rate limits");
    println!("3. Try downloading files using the parsed URLs");
    println!("4. Explore the intelligent CDN selection features");

    Ok(())
}
