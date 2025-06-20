// Licensed under the MIT License
// Copyright (c) 2025 Hal <hal.long@outlook.com>

use turbo_cdn::error::{ErrorContext, TurboCdnError};

#[test]
fn test_config_error_creation() {
    let error = TurboCdnError::config("Invalid configuration");

    assert_eq!(error.category(), "config");
    assert!(!error.is_retryable());
    assert_eq!(
        error.to_string(),
        "Configuration error: Invalid configuration"
    );
}

#[test]
fn test_download_error_creation() {
    let error = TurboCdnError::download("Download failed");

    assert_eq!(error.category(), "download");
    assert!(!error.is_retryable());
    assert_eq!(error.to_string(), "Download failed: Download failed");
}

#[test]
fn test_source_validation_error_creation() {
    let error = TurboCdnError::source_validation("Invalid source");

    assert_eq!(error.category(), "source_validation");
    assert!(!error.is_retryable());
    assert_eq!(
        error.to_string(),
        "Source validation failed: Invalid source"
    );
}

#[test]
fn test_compliance_error_creation() {
    let error = TurboCdnError::compliance("Compliance check failed");

    assert_eq!(error.category(), "compliance");
    assert!(!error.is_retryable());
    assert_eq!(
        error.to_string(),
        "Compliance check failed: Compliance check failed"
    );
}

#[test]
fn test_cache_error_creation() {
    let error = TurboCdnError::cache("Cache operation failed");

    assert_eq!(error.category(), "cache");
    assert!(!error.is_retryable());
    assert_eq!(error.to_string(), "Cache error: Cache operation failed");
}

#[test]
fn test_routing_error_creation() {
    let error = TurboCdnError::routing("Routing failed");

    assert_eq!(error.category(), "routing");
    assert!(!error.is_retryable());
    assert_eq!(error.to_string(), "Routing error: Routing failed");
}

#[test]
fn test_authentication_error_creation() {
    let error = TurboCdnError::authentication("Auth failed");

    assert_eq!(error.category(), "authentication");
    assert!(!error.is_retryable());
    assert_eq!(error.to_string(), "Authentication failed: Auth failed");
}

#[test]
fn test_rate_limit_error_creation() {
    let error = TurboCdnError::rate_limit("Rate limit exceeded");

    assert_eq!(error.category(), "rate_limit");
    assert!(error.is_retryable()); // Rate limit errors are retryable
    assert_eq!(
        error.to_string(),
        "Rate limit exceeded: Rate limit exceeded"
    );
}

#[test]
fn test_timeout_error_creation() {
    let error = TurboCdnError::timeout("Request timed out");

    assert_eq!(error.category(), "timeout");
    assert!(error.is_retryable()); // Timeout errors are retryable
    assert_eq!(error.to_string(), "Operation timed out: Request timed out");
}

#[test]
fn test_checksum_mismatch_error_creation() {
    let error = TurboCdnError::checksum_mismatch("abc123", "def456");

    assert_eq!(error.category(), "checksum");
    assert!(!error.is_retryable());
    assert_eq!(
        error.to_string(),
        "Checksum validation failed: expected abc123, got def456"
    );
}

#[test]
fn test_file_not_found_error_creation() {
    let error = TurboCdnError::file_not_found("/path/to/file.txt");

    assert_eq!(error.category(), "file_not_found");
    assert!(!error.is_retryable());
    assert_eq!(error.to_string(), "File not found: /path/to/file.txt");
}

#[test]
fn test_unsupported_error_creation() {
    let error = TurboCdnError::unsupported("Unsupported operation");

    assert_eq!(error.category(), "unsupported");
    assert!(!error.is_retryable());
    assert_eq!(
        error.to_string(),
        "Unsupported operation: Unsupported operation"
    );
}

#[test]
fn test_internal_error_creation() {
    let error = TurboCdnError::internal("Internal error occurred");

    assert_eq!(error.category(), "internal");
    assert!(!error.is_retryable());
    assert_eq!(error.to_string(), "Internal error: Internal error occurred");
}

#[test]
fn test_network_error_retryable() {
    // Test with IO error which is retryable
    let io_error = std::io::Error::new(std::io::ErrorKind::ConnectionRefused, "Connection refused");
    let error = TurboCdnError::Io(io_error);

    assert_eq!(error.category(), "io");
    assert!(error.is_retryable());
}

#[test]
fn test_io_error_retryable() {
    let io_error = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "Permission denied");
    let error = TurboCdnError::Io(io_error);

    assert_eq!(error.category(), "io");
    assert!(error.is_retryable());
}

#[test]
fn test_url_parse_error_not_retryable() {
    let url_error = url::ParseError::EmptyHost;
    let error = TurboCdnError::InvalidUrl(url_error);

    assert_eq!(error.category(), "url");
    assert!(!error.is_retryable());
}

#[test]
fn test_json_error_not_retryable() {
    // Create a JSON error by trying to parse invalid JSON
    let json_error = serde_json::from_str::<serde_json::Value>("invalid json").unwrap_err();
    let error = TurboCdnError::Json(json_error);

    assert_eq!(error.category(), "json");
    assert!(!error.is_retryable());
}

#[test]
fn test_error_context_creation() {
    let context = ErrorContext {
        operation: "download".to_string(),
        source: Some("github".to_string()),
        file_path: Some("/tmp/file.txt".to_string()),
        url: Some("https://github.com/owner/repo/releases/download/v1.0.0/file.txt".to_string()),
        timestamp: chrono::Utc::now(),
    };

    assert_eq!(context.operation, "download");
    assert_eq!(context.source, Some("github".to_string()));
    assert_eq!(context.file_path, Some("/tmp/file.txt".to_string()));
    assert!(context.url.is_some());
}

#[test]
fn test_error_categories_comprehensive() {
    let test_cases = vec![
        (TurboCdnError::config("test"), "config"),
        (TurboCdnError::download("test"), "download"),
        (
            TurboCdnError::source_validation("test"),
            "source_validation",
        ),
        (TurboCdnError::compliance("test"), "compliance"),
        (TurboCdnError::cache("test"), "cache"),
        (TurboCdnError::routing("test"), "routing"),
        (TurboCdnError::authentication("test"), "authentication"),
        (TurboCdnError::rate_limit("test"), "rate_limit"),
        (TurboCdnError::timeout("test"), "timeout"),
        (TurboCdnError::checksum_mismatch("a", "b"), "checksum"),
        (TurboCdnError::file_not_found("test"), "file_not_found"),
        (TurboCdnError::unsupported("test"), "unsupported"),
        (TurboCdnError::internal("test"), "internal"),
    ];

    for (error, expected_category) in test_cases {
        assert_eq!(error.category(), expected_category);
    }
}

#[test]
fn test_retryable_errors_comprehensive() {
    // Retryable errors
    let retryable_errors = vec![
        TurboCdnError::timeout("test"),
        TurboCdnError::rate_limit("test"),
    ];

    for error in retryable_errors {
        assert!(
            error.is_retryable(),
            "Error should be retryable: {:?}",
            error
        );
    }

    // Non-retryable errors
    let non_retryable_errors = vec![
        TurboCdnError::config("test"),
        TurboCdnError::download("test"),
        TurboCdnError::source_validation("test"),
        TurboCdnError::compliance("test"),
        TurboCdnError::cache("test"),
        TurboCdnError::routing("test"),
        TurboCdnError::authentication("test"),
        TurboCdnError::checksum_mismatch("a", "b"),
        TurboCdnError::file_not_found("test"),
        TurboCdnError::unsupported("test"),
        TurboCdnError::internal("test"),
    ];

    for error in non_retryable_errors {
        assert!(
            !error.is_retryable(),
            "Error should not be retryable: {:?}",
            error
        );
    }
}

#[test]
fn test_error_display_formatting() {
    let test_cases = vec![
        (
            TurboCdnError::config("test message"),
            "Configuration error: test message",
        ),
        (TurboCdnError::download("failed"), "Download failed: failed"),
        (
            TurboCdnError::timeout("timed out"),
            "Operation timed out: timed out",
        ),
        (
            TurboCdnError::checksum_mismatch("expected", "actual"),
            "Checksum validation failed: expected expected, got actual",
        ),
        (
            TurboCdnError::file_not_found("/path"),
            "File not found: /path",
        ),
    ];

    for (error, expected_message) in test_cases {
        assert_eq!(error.to_string(), expected_message);
    }
}

#[test]
fn test_error_debug_formatting() {
    let error = TurboCdnError::config("test");
    let debug_str = format!("{:?}", error);
    assert!(debug_str.contains("Config"));
    assert!(debug_str.contains("test"));
}

#[test]
fn test_result_type_alias() {
    // Test that our Result type alias works correctly
    fn test_function() -> turbo_cdn::error::Result<String> {
        Ok("success".to_string())
    }

    let result = test_function();
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "success");

    fn test_error_function() -> turbo_cdn::error::Result<String> {
        Err(TurboCdnError::config("test error"))
    }

    let result = test_error_function();
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().category(), "config");
}
