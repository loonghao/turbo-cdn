// Licensed under the MIT License
// Copyright (c) 2025 Hal <hal.long@outlook.com>

use std::path::PathBuf;
use tempfile::TempDir;
use turbo_cdn::compliance::{ComplianceChecker, DownloadRequest, RiskLevel};
use turbo_cdn::config::{ComplianceConfig, DataProtectionConfig};
use uuid::Uuid;

/// Helper function to create a test compliance config
fn create_test_compliance_config(temp_dir: &TempDir) -> ComplianceConfig {
    ComplianceConfig {
        strict_mode: true,
        verify_open_source: true,
        check_copyright: true,
        validate_source: true,
        audit_logging: true,
        audit_log_path: temp_dir.path().join("audit.log"),
        data_protection: DataProtectionConfig::default(),
    }
}

/// Helper function to create a test download request
fn create_test_download_request(url: &str, user_consent: bool) -> DownloadRequest {
    DownloadRequest {
        id: Uuid::new_v4(),
        url: url.to_string(),
        source: "github".to_string(),
        repository: Some("owner/repo".to_string()),
        file_name: "test-file.zip".to_string(),
        user_agent: "turbo-cdn/1.0.0".to_string(),
        timestamp: chrono::Utc::now(),
        user_consent,
    }
}

#[test]
fn test_compliance_checker_creation() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let config = create_test_compliance_config(&temp_dir);

    let result = ComplianceChecker::new(config);
    assert!(result.is_ok(), "ComplianceChecker creation should succeed");
}

#[test]
fn test_compliance_checker_creation_with_disabled_audit() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let mut config = create_test_compliance_config(&temp_dir);
    config.audit_logging = false;

    let result = ComplianceChecker::new(config);
    assert!(
        result.is_ok(),
        "ComplianceChecker creation should succeed with disabled audit"
    );
}

#[tokio::test]
async fn test_compliance_check_with_user_consent() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let mut config = create_test_compliance_config(&temp_dir);
    // Disable open source verification to avoid GitHub API calls in tests
    config.verify_open_source = false;
    let checker = ComplianceChecker::new(config).unwrap();

    let request = create_test_download_request(
        "https://github.com/owner/repo/releases/download/v1.0.0/file.zip",
        true,
    );

    let result = checker.check_compliance(&request).await;
    assert!(result.is_ok(), "Compliance check should succeed");

    let compliance_result = result.unwrap();
    assert!(
        compliance_result.approved,
        "Request should be approved with user consent"
    );
    assert_eq!(compliance_result.request_id, request.id);
}

#[tokio::test]
async fn test_compliance_check_without_user_consent() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let config = create_test_compliance_config(&temp_dir);
    let checker = ComplianceChecker::new(config).unwrap();

    let request = create_test_download_request(
        "https://github.com/owner/repo/releases/download/v1.0.0/file.zip",
        false,
    );

    let result = checker.check_compliance(&request).await;
    assert!(result.is_ok(), "Compliance check should succeed");

    let compliance_result = result.unwrap();
    assert!(
        !compliance_result.approved,
        "Request should be rejected without user consent"
    );
    assert_eq!(compliance_result.risk_level, RiskLevel::Critical);
    assert!(!compliance_result.reasons.is_empty());
    assert!(compliance_result.reasons[0].contains("User consent required"));
}

#[tokio::test]
async fn test_compliance_check_with_disabled_consent_requirement() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let mut config = create_test_compliance_config(&temp_dir);
    config.data_protection.user_consent_required = false;
    config.verify_open_source = false; // Disable to avoid GitHub API calls
    let checker = ComplianceChecker::new(config).unwrap();

    let request = create_test_download_request(
        "https://github.com/owner/repo/releases/download/v1.0.0/file.zip",
        false,
    );

    let result = checker.check_compliance(&request).await;
    assert!(result.is_ok(), "Compliance check should succeed");

    let compliance_result = result.unwrap();
    assert!(
        compliance_result.approved,
        "Request should be approved when consent not required"
    );
}

#[tokio::test]
async fn test_compliance_check_with_suspicious_url() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let config = create_test_compliance_config(&temp_dir);
    let checker = ComplianceChecker::new(config).unwrap();

    // Test with a URL containing suspicious patterns
    let request = create_test_download_request("https://example.com/crack-tool.zip", true);

    let result = checker.check_compliance(&request).await;
    assert!(result.is_ok(), "Compliance check should succeed");

    let compliance_result = result.unwrap();
    // The request might be flagged due to suspicious content
    if !compliance_result.approved {
        assert!(
            compliance_result.risk_level == RiskLevel::High
                || compliance_result.risk_level == RiskLevel::Critical
        );
    }
}

#[tokio::test]
async fn test_compliance_check_with_allowed_domain() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let mut config = create_test_compliance_config(&temp_dir);
    config.verify_open_source = false; // Disable to avoid GitHub API calls
    let checker = ComplianceChecker::new(config).unwrap();

    // Test with allowed domains
    let allowed_urls = vec![
        "https://github.com/owner/repo/releases/download/v1.0.0/file.zip",
        "https://cdn.jsdelivr.net/gh/owner/repo@v1.0.0/file.zip",
        "https://fastly.jsdelivr.net/gh/owner/repo@v1.0.0/file.zip",
        "https://cdnjs.cloudflare.com/ajax/libs/library/1.0.0/file.js",
    ];

    for url in allowed_urls {
        let mut request = create_test_download_request(url, true);
        request.repository = None; // Remove repository to avoid license validation
        let result = checker.check_compliance(&request).await.unwrap();
        assert!(
            result.approved,
            "Request should be approved for allowed domain: {}",
            url
        );
    }
}

#[tokio::test]
async fn test_compliance_check_with_unknown_domain() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let config = create_test_compliance_config(&temp_dir);
    let checker = ComplianceChecker::new(config).unwrap();

    let mut request = create_test_download_request("https://unknown-domain.com/file.zip", true);
    request.repository = None; // Remove repository to avoid license validation

    let result = checker.check_compliance(&request).await;
    assert!(result.is_ok(), "Compliance check should succeed");

    let compliance_result = result.unwrap();
    // Unknown domains should be rejected due to source validation
    assert!(
        !compliance_result.approved,
        "Unknown domain should be rejected"
    );
    assert!(compliance_result
        .reasons
        .iter()
        .any(|r| r.contains("Source validation failed")));
}

#[tokio::test]
async fn test_compliance_check_with_disabled_strict_mode() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let mut config = create_test_compliance_config(&temp_dir);
    config.strict_mode = false;
    config.validate_source = false; // Disable source validation to test strict mode effect
    config.verify_open_source = false; // Disable to avoid GitHub API calls
    let checker = ComplianceChecker::new(config).unwrap();

    let mut request = create_test_download_request("https://unknown-domain.com/file.zip", true);
    request.repository = None; // Remove repository to avoid license validation

    let result = checker.check_compliance(&request).await;
    assert!(result.is_ok(), "Compliance check should succeed");

    let compliance_result = result.unwrap();
    // With strict mode disabled and source validation disabled, request should be approved
    assert!(
        compliance_result.approved,
        "Request should be approved with strict mode disabled"
    );
}

#[test]
fn test_download_request_creation() {
    let request = create_test_download_request(
        "https://github.com/owner/repo/releases/download/v1.0.0/file.zip",
        true,
    );

    assert!(!request.id.is_nil());
    assert_eq!(
        request.url,
        "https://github.com/owner/repo/releases/download/v1.0.0/file.zip"
    );
    assert_eq!(request.source, "github");
    assert_eq!(request.repository, Some("owner/repo".to_string()));
    assert_eq!(request.file_name, "test-file.zip");
    assert_eq!(request.user_agent, "turbo-cdn/1.0.0");
    assert!(request.user_consent);
}

#[test]
fn test_download_request_serialization() {
    let request = create_test_download_request(
        "https://github.com/owner/repo/releases/download/v1.0.0/file.zip",
        true,
    );

    // Test serialization
    let serialized = serde_json::to_string(&request).unwrap();
    assert!(serialized.contains("github.com"));
    assert!(serialized.contains("github"));
    assert!(serialized.contains("test-file.zip"));

    // Test deserialization
    let deserialized: DownloadRequest = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized.id, request.id);
    assert_eq!(deserialized.url, request.url);
    assert_eq!(deserialized.source, request.source);
    assert_eq!(deserialized.repository, request.repository);
    assert_eq!(deserialized.file_name, request.file_name);
    assert_eq!(deserialized.user_agent, request.user_agent);
    assert_eq!(deserialized.user_consent, request.user_consent);
}

#[test]
fn test_compliance_config_default() {
    let config = ComplianceConfig::default();

    assert!(config.strict_mode);
    assert!(config.verify_open_source);
    assert!(config.check_copyright);
    assert!(config.validate_source);
    assert!(config.audit_logging);
    assert_eq!(config.audit_log_path, PathBuf::from("./audit.log"));

    // Test data protection defaults
    assert!(config.data_protection.minimal_data_collection);
    assert!(config.data_protection.user_consent_required);
    assert_eq!(config.data_protection.data_retention_days, 30);
    assert!(config.data_protection.anonymize_data);
}

#[test]
fn test_data_protection_config_default() {
    let config = DataProtectionConfig::default();

    assert!(config.minimal_data_collection);
    assert!(config.user_consent_required);
    assert_eq!(config.data_retention_days, 30);
    assert!(config.anonymize_data);
}

#[tokio::test]
async fn test_compliance_check_with_different_sources() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let mut config = create_test_compliance_config(&temp_dir);
    config.verify_open_source = false; // Disable to avoid GitHub API calls
    let checker = ComplianceChecker::new(config).unwrap();

    let sources = vec!["github", "jsdelivr", "fastly", "cloudflare", "npm"];

    for source in sources {
        let mut request = create_test_download_request(
            "https://github.com/owner/repo/releases/download/v1.0.0/file.zip",
            true,
        );
        request.source = source.to_string();
        request.repository = None; // Remove repository to avoid license validation

        let result = checker.check_compliance(&request).await;
        assert!(
            result.is_ok(),
            "Compliance check should succeed for source: {}",
            source
        );

        let compliance_result = result.unwrap();
        assert!(
            compliance_result.approved,
            "Request should be approved for known source: {}",
            source
        );
    }
}

#[tokio::test]
async fn test_compliance_check_risk_levels() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let mut config = create_test_compliance_config(&temp_dir);
    config.verify_open_source = false; // Disable to avoid GitHub API calls
    let checker = ComplianceChecker::new(config).unwrap();

    // Test low risk (known good domain with consent)
    let mut low_risk_request = create_test_download_request(
        "https://github.com/owner/repo/releases/download/v1.0.0/file.zip",
        true,
    );
    low_risk_request.repository = None; // Remove repository to avoid license validation
    let result = checker.check_compliance(&low_risk_request).await.unwrap();
    assert_eq!(result.risk_level, RiskLevel::Low);

    // Test critical risk (no user consent)
    let critical_risk_request = create_test_download_request(
        "https://github.com/owner/repo/releases/download/v1.0.0/file.zip",
        false,
    );
    let result = checker
        .check_compliance(&critical_risk_request)
        .await
        .unwrap();
    assert_eq!(result.risk_level, RiskLevel::Critical);
}

#[tokio::test]
async fn test_compliance_check_with_disabled_features() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let mut config = create_test_compliance_config(&temp_dir);

    // Disable all checks
    config.verify_open_source = false;
    config.check_copyright = false;
    config.validate_source = false;
    config.data_protection.user_consent_required = false;

    let checker = ComplianceChecker::new(config).unwrap();
    let request =
        create_test_download_request("https://unknown-domain.com/suspicious-file.zip", false);

    let result = checker.check_compliance(&request).await;
    assert!(
        result.is_ok(),
        "Compliance check should succeed with all checks disabled"
    );

    let compliance_result = result.unwrap();
    assert!(
        compliance_result.approved,
        "Request should be approved with all checks disabled"
    );
}
