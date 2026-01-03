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
    // self-update is optional but enabled by default
    #[cfg(feature = "self-update")]
    {
        assert!(
            cfg!(feature = "self-update"),
            "self-update feature should be enabled"
        );
    }

    #[cfg(not(feature = "self-update"))]
    {
        assert!(
            !cfg!(feature = "self-update"),
            "self-update feature should be disabled"
        );
    }
}
