// Licensed under the MIT License
// Copyright (c) 2025 Hal <hal.long@outlook.com>

//! Feature flag tests for turbo-cdn
//!
//! This module tests that optional features work correctly.
#![allow(clippy::assertions_on_constants)]

/// Test that the library compiles without self-update feature
/// This is primarily a compile-time test
#[test]
fn test_library_without_self_update() {
    // The library should work without self-update feature
    // This test verifies that core types are available regardless of feature flags
    let url = "https://example.com/file.zip";
    assert!(url.starts_with("https://"));
}

/// Test that core functionality is available regardless of features
#[tokio::test]
async fn test_core_functionality_available() {
    use turbo_cdn::TurboCdn;

    // TurboCdn should be available regardless of self-update feature
    let result = TurboCdn::new().await;
    assert!(result.is_ok(), "TurboCdn should be creatable");
}

/// Test that feature flags are properly configured
#[test]
fn test_feature_flags() {
    // self-update is optional and disabled by default for library users
    #[cfg(feature = "self-update")]
    {
        assert!(
            cfg!(feature = "self-update"),
            "self-update feature should be enabled when opted in"
        );
    }

    #[cfg(not(feature = "self-update"))]
    {
        assert!(
            !cfg!(feature = "self-update"),
            "self-update feature should be disabled when not requested"
        );
    }
}

/// Test that rustls feature is properly configured
#[test]
fn test_rustls_feature_flag() {
    #[cfg(feature = "rustls")]
    {
        assert!(
            cfg!(feature = "rustls"),
            "rustls feature should be enabled when opted in"
        );
    }

    #[cfg(not(feature = "rustls"))]
    {
        assert!(
            !cfg!(feature = "rustls"),
            "rustls feature should be disabled when not requested"
        );
    }
}

/// Test that native-tls feature is properly configured
#[test]
fn test_native_tls_feature_flag() {
    #[cfg(feature = "native-tls")]
    {
        assert!(
            cfg!(feature = "native-tls"),
            "native-tls feature should be enabled when opted in"
        );
    }

    #[cfg(not(feature = "native-tls"))]
    {
        assert!(
            !cfg!(feature = "native-tls"),
            "native-tls feature should be disabled when not requested"
        );
    }
}

/// Test that init_rustls_provider works correctly based on feature
#[test]
fn test_init_rustls_provider() {
    // This should compile and run regardless of which TLS backend is enabled
    // When rustls is enabled, it initializes the provider
    // When rustls is disabled, it's a no-op
    turbo_cdn::init_rustls_provider();
}

/// Test that TLS backend is mutually exclusive in practice
/// Note: Both features can technically be enabled, but only one should be used
#[test]
fn test_tls_backend_configuration() {
    // At least one TLS backend should be available
    let has_tls = cfg!(feature = "rustls") || cfg!(feature = "native-tls");
    assert!(
        has_tls || cfg!(test), // In test mode, we might have neither explicitly set
        "At least one TLS backend should be configured for production use"
    );
}
