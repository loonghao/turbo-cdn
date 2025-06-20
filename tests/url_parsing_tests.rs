// Licensed under the MIT License
// Copyright (c) 2025 Hal <hal.long@outlook.com>

use turbo_cdn::{DetectedSourceType, TurboCdn};

/// Helper function to create a TurboCdn instance for testing
async fn create_test_client() -> TurboCdn {
    TurboCdn::new()
        .await
        .expect("Failed to create TurboCdn client")
}

#[tokio::test]
async fn test_parse_github_url() {
    let client = create_test_client().await;

    let url = "https://github.com/oven-sh/bun/releases/download/bun-v1.2.9/bun-bun-v1.2.9.zip";
    let result = client.parse_url(url).unwrap();

    assert_eq!(result.repository, "oven-sh/bun");
    assert_eq!(result.version, "bun-v1.2.9");
    assert_eq!(result.filename, "bun-bun-v1.2.9.zip");
    assert_eq!(result.original_url, url);
    assert_eq!(result.source_type, DetectedSourceType::GitHub);
}

#[tokio::test]
async fn test_parse_gitlab_url() {
    let client = create_test_client().await;

    let url = "https://gitlab.com/owner/repo/-/releases/v1.0.0/downloads/file.zip";
    let result = client.parse_url(url).unwrap();

    assert_eq!(result.repository, "owner/repo");
    assert_eq!(result.version, "v1.0.0");
    assert_eq!(result.filename, "file.zip");
    assert_eq!(result.source_type, DetectedSourceType::GitLab);
}

#[tokio::test]
async fn test_parse_bitbucket_url() {
    let client = create_test_client().await;

    let url = "https://bitbucket.org/owner/repo/downloads/myapp-v2.1.0.tar.gz";
    let result = client.parse_url(url).unwrap();

    assert_eq!(result.repository, "owner/repo");
    assert_eq!(result.filename, "myapp-v2.1.0.tar.gz");
    assert_eq!(result.source_type, DetectedSourceType::Bitbucket);
    // Version should be extracted from filename
    assert_eq!(result.version, "2.1.0");
}

#[tokio::test]
async fn test_parse_jsdelivr_url() {
    let client = create_test_client().await;

    let url = "https://cdn.jsdelivr.net/gh/microsoft/vscode@1.74.0/package.json";
    let result = client.parse_url(url).unwrap();

    assert_eq!(result.repository, "microsoft/vscode");
    assert_eq!(result.version, "1.74.0");
    assert_eq!(result.filename, "package.json");
    assert_eq!(result.source_type, DetectedSourceType::JsDelivr);
}

#[tokio::test]
async fn test_parse_fastly_url() {
    let client = create_test_client().await;

    let url = "https://fastly.jsdelivr.net/gh/jquery/jquery@3.6.0/dist/jquery.min.js";
    let result = client.parse_url(url).unwrap();

    assert_eq!(result.repository, "jquery/jquery");
    assert_eq!(result.version, "3.6.0");
    assert_eq!(result.filename, "dist/jquery.min.js");
    assert_eq!(result.source_type, DetectedSourceType::Fastly);
}

#[tokio::test]
async fn test_parse_cloudflare_url() {
    let client = create_test_client().await;

    let url = "https://cdnjs.cloudflare.com/ajax/libs/react/18.2.0/umd/react.production.min.js";
    let result = client.parse_url(url).unwrap();

    assert_eq!(result.repository, "cdnjs/react");
    assert_eq!(result.version, "18.2.0");
    assert_eq!(result.filename, "umd/react.production.min.js");
    assert_eq!(result.source_type, DetectedSourceType::Cloudflare);
}

#[tokio::test]
async fn test_parse_npm_url() {
    let client = create_test_client().await;

    let url = "https://registry.npmjs.org/react/-/react-18.2.0.tgz";
    let result = client.parse_url(url).unwrap();

    assert_eq!(result.repository, "npm/react");
    assert_eq!(result.version, "18.2.0");
    assert_eq!(result.filename, "react-18.2.0.tgz");
    assert_eq!(result.source_type, DetectedSourceType::Npm);
}

#[tokio::test]
async fn test_parse_pypi_url() {
    let client = create_test_client().await;

    let url = "https://files.pythonhosted.org/packages/source/r/requests/requests-2.28.1.tar.gz";
    let result = client.parse_url(url).unwrap();

    assert_eq!(result.repository, "pypi/requests");
    assert_eq!(result.version, "2.28.1");
    assert_eq!(result.filename, "requests-2.28.1.tar.gz");
    assert_eq!(result.source_type, DetectedSourceType::PyPI);
}

#[tokio::test]
async fn test_parse_go_proxy_url() {
    let client = create_test_client().await;

    let url = "https://proxy.golang.org/github.com/gin-gonic/gin/@v/v1.9.1.zip";
    let result = client.parse_url(url).unwrap();

    assert_eq!(result.repository, "go/github.com/gin-gonic/gin");
    assert_eq!(result.version, "v1.9.1");
    assert_eq!(result.filename, "v1.9.1.zip");
    assert_eq!(result.source_type, DetectedSourceType::GoProxy);
}

#[tokio::test]
async fn test_parse_crates_io_url() {
    let client = create_test_client().await;

    let url = "https://crates.io/api/v1/crates/serde/1.0.152/download";
    let result = client.parse_url(url).unwrap();

    assert_eq!(result.repository, "crates/serde");
    assert_eq!(result.version, "1.0.152");
    assert_eq!(result.filename, "serde-1.0.152.crate");
    assert_eq!(result.source_type, DetectedSourceType::CratesIo);
}

#[tokio::test]
async fn test_parse_maven_url() {
    let client = create_test_client().await;

    let url = "https://repo1.maven.org/maven2/org/springframework/spring-core/5.3.21/spring-core-5.3.21.jar";
    let result = client.parse_url(url).unwrap();

    assert_eq!(result.repository, "maven/org.springframework.spring-core");
    assert_eq!(result.version, "5.3.21");
    assert_eq!(result.filename, "spring-core-5.3.21.jar");
    assert_eq!(result.source_type, DetectedSourceType::Maven);
}

#[tokio::test]
async fn test_parse_nuget_url() {
    let client = create_test_client().await;

    let url = "https://api.nuget.org/v3-flatcontainer/newtonsoft.json/13.0.3/newtonsoft.json.13.0.3.nupkg";
    let result = client.parse_url(url).unwrap();

    assert_eq!(result.repository, "nuget/newtonsoft.json");
    assert_eq!(result.version, "13.0.3");
    assert_eq!(result.filename, "newtonsoft.json.13.0.3.nupkg");
    assert_eq!(result.source_type, DetectedSourceType::NuGet);
}

#[tokio::test]
async fn test_parse_docker_hub_url() {
    let client = create_test_client().await;

    let url = "https://registry-1.docker.io/v2/library/nginx/manifests/latest";
    let result = client.parse_url(url).unwrap();

    assert_eq!(result.repository, "docker/nginx");
    assert_eq!(result.version, "latest");
    assert_eq!(result.filename, "nginx-latest.tar");
    assert_eq!(result.source_type, DetectedSourceType::DockerHub);
}

#[tokio::test]
async fn test_parse_sourceforge_url() {
    let client = create_test_client().await;

    let url = "https://downloads.sourceforge.net/project/sevenzip/7-Zip/22.01/7z2201-x64.exe";
    let result = client.parse_url(url).unwrap();

    assert_eq!(result.repository, "sourceforge/sevenzip");
    assert_eq!(result.filename, "7-Zip/22.01/7z2201-x64.exe");
    assert_eq!(result.source_type, DetectedSourceType::SourceForge);
    // Version should be extracted from filename
    assert_eq!(result.version, "22.01");
}

#[tokio::test]
async fn test_parse_invalid_url() {
    let client = create_test_client().await;

    let url = "https://example.com/some/file.zip";
    let result = client.parse_url(url);

    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Unsupported URL host"));
}

#[tokio::test]
async fn test_parse_malformed_github_url() {
    let client = create_test_client().await;

    let url = "https://github.com/owner/repo/invalid/path";
    let result = client.parse_url(url);

    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Invalid GitHub releases URL format"));
}

#[tokio::test]
async fn test_version_extraction_from_filename() {
    let client = create_test_client().await;

    // Test various version patterns
    let test_cases = vec![
        ("app-v1.2.3.zip", Some("1.2.3")),
        ("tool-2.0.zip", Some("2.0")),
        ("package-2023-12-01.tar.gz", Some("2023-12-01")),
        ("file-20231201.exe", Some("20231201")),
        ("noversion.zip", None),
    ];

    for (filename, expected) in test_cases {
        let result = client.extract_version_from_filename(filename);
        assert_eq!(
            result.as_deref(),
            expected,
            "Failed for filename: {}",
            filename
        );
    }
}

#[tokio::test]
async fn test_complex_repository_paths() {
    let client = create_test_client().await;

    // Test GitHub URL with complex filename path
    let url = "https://github.com/owner/repo/releases/download/v1.0.0/dist/assets/app.min.js";
    let result = client.parse_url(url).unwrap();

    assert_eq!(result.repository, "owner/repo");
    assert_eq!(result.version, "v1.0.0");
    assert_eq!(result.filename, "dist/assets/app.min.js");
    assert_eq!(result.source_type, DetectedSourceType::GitHub);
}

#[tokio::test]
async fn test_scoped_npm_package() {
    let client = create_test_client().await;

    // Note: This test shows the current limitation - scoped packages need special handling
    // For now, we test the basic npm format
    let url = "https://registry.npmjs.org/lodash/-/lodash-4.17.21.tgz";
    let result = client.parse_url(url).unwrap();

    assert_eq!(result.repository, "npm/lodash");
    assert_eq!(result.version, "4.17.21");
    assert_eq!(result.filename, "lodash-4.17.21.tgz");
    assert_eq!(result.source_type, DetectedSourceType::Npm);
}
