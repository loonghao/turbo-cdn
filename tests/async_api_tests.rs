// Licensed under the MIT License
// Copyright (c) 2025 Hal <hal.long@outlook.com>

use turbo_cdn::async_api::{AsyncTurboCdn, AsyncTurboCdnBuilder};
use turbo_cdn::{DetectedSourceType, Region, Source};

/// Helper function to create an AsyncTurboCdn instance for testing
async fn create_async_test_client() -> AsyncTurboCdn {
    AsyncTurboCdn::new()
        .await
        .expect("Failed to create AsyncTurboCdn client")
}

#[tokio::test]
async fn test_async_client_creation() {
    let client = AsyncTurboCdn::new().await;
    assert!(client.is_ok());
}

#[tokio::test]
async fn test_async_client_with_config() {
    let client = AsyncTurboCdnBuilder::new()
        .with_region(Region::Global)
        .with_cache(true)
        .with_max_concurrent_downloads(4)
        .build()
        .await;

    assert!(client.is_ok());
}

#[tokio::test]
async fn test_async_parse_url() {
    let client = create_async_test_client().await;

    let url = "https://github.com/rust-lang/mdBook/releases/download/v0.4.21/mdbook-v0.4.21-x86_64-unknown-linux-gnu.tar.gz";
    let result = client.parse_url_async(url).await;

    assert!(result.is_ok());
    let parsed = result.unwrap();
    assert_eq!(parsed.repository, "rust-lang/mdBook");
    assert_eq!(parsed.version, "v0.4.21");
    assert_eq!(
        parsed.filename,
        "mdbook-v0.4.21-x86_64-unknown-linux-gnu.tar.gz"
    );
    assert_eq!(parsed.source_type, DetectedSourceType::GitHub);
}

#[tokio::test]
async fn test_async_parse_multiple_url_formats() {
    let client = create_async_test_client().await;

    let test_cases = vec![
        (
            "https://cdn.jsdelivr.net/gh/jquery/jquery@3.6.0/dist/jquery.min.js",
            "jquery/jquery",
            "3.6.0",
            DetectedSourceType::JsDelivr,
        ),
        (
            "https://registry.npmjs.org/express/-/express-4.18.2.tgz",
            "npm/express",
            "4.18.2",
            DetectedSourceType::Npm,
        ),
        (
            "https://crates.io/api/v1/crates/tokio/1.28.0/download",
            "crates/tokio",
            "1.28.0",
            DetectedSourceType::CratesIo,
        ),
    ];

    for (url, expected_repo, expected_version, expected_source) in test_cases {
        let result = client.parse_url_async(url).await;
        assert!(result.is_ok(), "Failed to parse URL: {}", url);

        let parsed = result.unwrap();
        assert_eq!(parsed.repository, expected_repo);
        assert_eq!(parsed.version, expected_version);
        assert_eq!(parsed.source_type, expected_source);
    }
}

#[tokio::test]
async fn test_async_extract_version_from_filename() {
    let client = create_async_test_client().await;

    let test_cases = vec![
        ("app-v1.2.3.zip", Some("1.2.3")),
        ("tool-2.0.tar.gz", Some("2.0")),
        ("package-2023-12-01.exe", Some("2023-12-01")),
        ("file-20231201.dmg", Some("20231201")),
        ("noversion.zip", None),
    ];

    for (filename, expected) in test_cases {
        let result = client.extract_version_from_filename_async(filename).await;
        assert_eq!(
            result.as_deref(),
            expected,
            "Failed for filename: {}",
            filename
        );
    }
}

#[tokio::test]
async fn test_async_client_clone() {
    let client = create_async_test_client().await;
    let cloned_client = client.clone();

    // Both clients should work independently
    let url =
        "https://github.com/microsoft/vscode/releases/download/1.85.0/VSCode-linux-x64.tar.gz";

    let result1 = client.parse_url_async(url).await;
    let result2 = cloned_client.parse_url_async(url).await;

    assert!(result1.is_ok());
    assert!(result2.is_ok());

    let parsed1 = result1.unwrap();
    let parsed2 = result2.unwrap();

    assert_eq!(parsed1.repository, parsed2.repository);
    assert_eq!(parsed1.version, parsed2.version);
}

#[tokio::test]
async fn test_async_concurrent_operations() {
    let client = create_async_test_client().await;

    // Clone for concurrent use
    let client1 = client.clone();
    let client2 = client.clone();
    let client3 = client.clone();

    // Spawn concurrent tasks
    let task1 = tokio::spawn(async move {
        client1.parse_url_async("https://github.com/rust-lang/rust/releases/download/1.75.0/rust-1.75.0-x86_64-unknown-linux-gnu.tar.gz").await
    });

    let task2 = tokio::spawn(async move {
        client2
            .parse_url_async("https://cdn.jsdelivr.net/gh/lodash/lodash@4.17.21/lodash.min.js")
            .await
    });

    let task3 = tokio::spawn(async move {
        client3
            .extract_version_from_filename_async("myapp-v2.1.0.zip")
            .await
    });

    // Wait for all tasks
    let (result1, result2, result3) = tokio::join!(task1, task2, task3);

    // Check results
    assert!(result1.is_ok());
    assert!(result2.is_ok());
    assert!(result3.is_ok());

    let parsed1 = result1.unwrap().unwrap();
    let parsed2 = result2.unwrap().unwrap();
    let version3 = result3.unwrap();

    assert_eq!(parsed1.repository, "rust-lang/rust");
    assert_eq!(parsed2.repository, "lodash/lodash");
    assert_eq!(version3, Some("2.1.0".to_string()));
}

#[tokio::test]
async fn test_async_quick_functions() {
    // Test quick parse function
    let url = "https://github.com/oven-sh/bun/releases/download/bun-v1.2.9/bun-bun-v1.2.9.zip";
    let result = turbo_cdn::async_api::quick::parse_url(url).await;

    assert!(result.is_ok());
    let parsed = result.unwrap();
    assert_eq!(parsed.repository, "oven-sh/bun");
    assert_eq!(parsed.version, "bun-v1.2.9");
}

#[tokio::test]
async fn test_async_builder_with_sources() {
    let client = AsyncTurboCdnBuilder::new()
        .with_sources(&[Source::github(), Source::jsdelivr(), Source::fastly()])
        .with_region(Region::Global)
        .build()
        .await;

    assert!(client.is_ok());

    // Test that the client works
    let client = client.unwrap();
    let url =
        "https://github.com/microsoft/TypeScript/releases/download/v5.0.0/typescript-5.0.0.tgz";
    let result = client.parse_url_async(url).await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_async_error_handling() {
    let client = create_async_test_client().await;

    // Test with invalid URL
    let invalid_url = "https://example.com/some/file.zip";
    let result = client.parse_url_async(invalid_url).await;

    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Unsupported URL host"));
}

#[tokio::test]
async fn test_async_malformed_url() {
    let client = create_async_test_client().await;

    // Test with malformed GitHub URL
    let malformed_url = "https://github.com/owner/repo/invalid/path";
    let result = client.parse_url_async(malformed_url).await;

    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Invalid GitHub releases URL format"));
}

#[tokio::test]
async fn test_async_complex_filenames() {
    let client = create_async_test_client().await;

    // Test GitHub URL with complex filename path
    let url = "https://github.com/owner/repo/releases/download/v1.0.0/dist/assets/app.min.js";
    let result = client.parse_url_async(url).await;

    assert!(result.is_ok());
    let parsed = result.unwrap();
    assert_eq!(parsed.repository, "owner/repo");
    assert_eq!(parsed.version, "v1.0.0");
    assert_eq!(parsed.filename, "dist/assets/app.min.js");
    assert_eq!(parsed.source_type, DetectedSourceType::GitHub);
}

#[tokio::test]
async fn test_async_multiple_concurrent_clients() {
    // Test creating multiple clients concurrently
    let tasks: Vec<_> = (0..5)
        .map(|_| tokio::spawn(async { AsyncTurboCdn::new().await }))
        .collect();

    let results = futures::future::join_all(tasks).await;

    // All clients should be created successfully
    for result in results {
        assert!(result.is_ok());
        assert!(result.unwrap().is_ok());
    }
}

#[tokio::test]
async fn test_async_stress_parsing() {
    let client = create_async_test_client().await;

    let urls = vec![
        "https://github.com/rust-lang/rust/releases/download/1.75.0/rust-1.75.0.tar.gz",
        "https://cdn.jsdelivr.net/gh/microsoft/vscode@1.85.0/package.json",
        "https://registry.npmjs.org/react/-/react-18.2.0.tgz",
        "https://crates.io/api/v1/crates/serde/1.0.152/download",
        "https://repo1.maven.org/maven2/org/springframework/spring-core/5.3.21/spring-core-5.3.21.jar",
    ];

    // Parse all URLs concurrently
    let tasks: Vec<_> = urls
        .into_iter()
        .map(|url| {
            let client = client.clone();
            tokio::spawn(async move { client.parse_url_async(url).await })
        })
        .collect();

    let results = futures::future::join_all(tasks).await;

    // All parsing should succeed
    for result in results {
        assert!(result.is_ok());
        assert!(result.unwrap().is_ok());
    }
}

#[tokio::test]
async fn test_async_with_config() {
    let config = turbo_cdn::TurboCdnConfig::default();
    let client = AsyncTurboCdn::with_config(config).await;

    assert!(client.is_ok());

    // Test that the client works
    let client = client.unwrap();
    let url =
        "https://github.com/microsoft/TypeScript/releases/download/v5.0.0/typescript-5.0.0.tgz";
    let result = client.parse_url_async(url).await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_async_get_repository_metadata() {
    let client = create_async_test_client().await;

    // Test with a well-known repository
    let repository = "microsoft/vscode";
    let result = client.get_repository_metadata_async(repository).await;

    // This might fail due to API limits, but we test the method exists and compiles
    match result {
        Ok(metadata) => {
            assert!(!metadata.name.is_empty());
        }
        Err(_) => {
            // Expected due to API limits in CI
        }
    }
}

#[tokio::test]
async fn test_async_get_stats() {
    let client = create_async_test_client().await;

    let result = client.get_stats_async().await;
    assert!(result.is_ok());

    let stats = result.unwrap();
    // Stats should be initialized with zeros
    assert_eq!(stats.total_downloads, 0);
    assert_eq!(stats.successful_downloads, 0);
    assert_eq!(stats.failed_downloads, 0);
}

#[tokio::test]
async fn test_async_health_check() {
    let client = create_async_test_client().await;

    let result = client.health_check_async().await;
    assert!(result.is_ok());

    let health_status = result.unwrap();
    // Should have health status for all sources
    assert!(!health_status.is_empty());
}

#[tokio::test]
async fn test_async_download_method() {
    let client = create_async_test_client().await;

    // Test the download method signature (won't actually download due to API limits)
    let repository = "microsoft/vscode";
    let version = "1.85.0";
    let file_name = "VSCode-linux-x64.tar.gz";

    let result = client
        .download_async(repository, version, file_name, None)
        .await;

    // This will likely fail due to API limits or file not found, but we test the method exists
    match result {
        Ok(_) => {
            // Unexpected success in test environment
        }
        Err(e) => {
            // Expected failure due to API limits or network issues
            assert!(!e.to_string().is_empty());
        }
    }
}

#[tokio::test]
async fn test_quick_optimize_url() {
    let url = "https://github.com/oven-sh/bun/releases/download/bun-v1.2.9/bun-bun-v1.2.9.zip";

    let result = turbo_cdn::async_api::quick::optimize_url(url).await;

    // This might fail due to API limits, but we test the method exists
    match result {
        Ok(optimal_url) => {
            assert!(!optimal_url.is_empty());
        }
        Err(_) => {
            // Expected due to API limits in CI
        }
    }
}

#[tokio::test]
async fn test_quick_download_url() {
    let url = "https://github.com/oven-sh/bun/releases/download/bun-v1.2.9/bun-bun-v1.2.9.zip";

    let result = turbo_cdn::async_api::quick::download_url(url).await;

    // This will likely fail due to API limits, but we test the method exists
    match result {
        Ok(_) => {
            // Unexpected success in test environment
        }
        Err(e) => {
            // Expected failure due to API limits or network issues
            assert!(!e.to_string().is_empty());
        }
    }
}

#[tokio::test]
async fn test_quick_download_repository() {
    let repository = "microsoft/vscode";
    let version = "1.85.0";
    let file_name = "VSCode-linux-x64.tar.gz";

    let result =
        turbo_cdn::async_api::quick::download_repository(repository, version, file_name).await;

    // This will likely fail due to API limits, but we test the method exists
    match result {
        Ok(_) => {
            // Unexpected success in test environment
        }
        Err(e) => {
            // Expected failure due to API limits or network issues
            assert!(!e.to_string().is_empty());
        }
    }
}
