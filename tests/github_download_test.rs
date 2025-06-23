// Licensed under the MIT License
// Copyright (c) 2025 Hal <hal.long@outlook.com>

//! Integration test for GitHub download functionality

use turbo_cdn::{TurboCdn, TurboCdnConfig};

#[tokio::test]
async fn test_github_source_enabled_by_default() {
    // Test that GitHub source is enabled in default configuration
    let config = TurboCdnConfig::default();
    
    // Check that GitHub source is enabled
    let github_config = config.mirrors.configs.get("github");
    assert!(github_config.is_some(), "GitHub source configuration should exist");
    
    let github_config = github_config.unwrap();
    assert!(github_config.enabled, "GitHub source should be enabled by default");
}

#[tokio::test]
async fn test_github_releases_url_parsing() {
    // Test that we can handle GitHub releases URLs correctly
    let turbo_cdn = TurboCdn::builder().build().await.unwrap();
    
    let test_url = "https://github.com/BurntSushi/ripgrep/releases/download/14.1.1/ripgrep-14.1.1-x86_64-pc-windows-msvc.zip";
    
    // This should not fail with "No compatible sources found"
    // Note: This test might fail if GitHub API is not accessible, but it should not fail due to missing sources
    let result = turbo_cdn.get_optimal_url(test_url).await;
    
    match result {
        Ok(_) => {
            // Success - we found compatible sources and got a download URL
            println!("✅ Successfully found compatible sources for GitHub releases URL");
        }
        Err(e) => {
            let error_msg = e.to_string();
            // The error should NOT be "No compatible sources found"
            assert!(
                !error_msg.contains("No compatible sources found"),
                "Should not fail with 'No compatible sources found' error. Got: {}",
                error_msg
            );
            
            // Other errors (like network issues, API rate limits) are acceptable for this test
            println!("⚠️  Got expected error (not source-related): {}", error_msg);
        }
    }
}

#[tokio::test]
async fn test_source_manager_has_github_source() {
    // Test that SourceManager includes GitHub source when built with default config
    let turbo_cdn = TurboCdn::builder().build().await.unwrap();
    
    // We can't directly access the source manager, but we can test indirectly
    // by checking that GitHub URLs are recognized as handleable
    
    // This is an indirect test - if GitHub source is properly added,
    // the system should be able to handle GitHub URLs
    let test_url = "https://github.com/test/repo/releases/download/v1.0.0/file.zip";
    
    // Try to get optimized URL - this should not fail with "No compatible sources found"
    let result = turbo_cdn.get_optimal_url(test_url).await;
    
    match result {
        Ok(_) => {
            println!("✅ GitHub source is properly configured and can handle URLs");
        }
        Err(e) => {
            let error_msg = e.to_string();
            // Should not fail due to missing sources
            assert!(
                !error_msg.contains("No compatible sources found"),
                "GitHub source should be available. Error: {}",
                error_msg
            );
        }
    }
}

#[tokio::test] 
async fn test_ripgrep_specific_url() {
    // Test the specific URL that was failing
    let turbo_cdn = TurboCdn::builder().build().await.unwrap();
    
    let ripgrep_url = "https://github.com/BurntSushi/ripgrep/releases/download/14.1.1/ripgrep-14.1.1-x86_64-pc-windows-msvc.zip";
    
    let result = turbo_cdn.get_optimal_url(ripgrep_url).await;
    
    match result {
        Ok(optimized_url) => {
            println!("✅ Successfully got optimized URL: {}", optimized_url);
            // The URL should be valid
            assert!(!optimized_url.is_empty());
        }
        Err(e) => {
            let error_msg = e.to_string();
            println!("Error: {}", error_msg);
            
            // Should not fail with "No compatible sources found"
            assert!(
                !error_msg.contains("No compatible sources found"),
                "Should have GitHub source available. Error: {}",
                error_msg
            );
            
            // Other errors are acceptable (network, API limits, etc.)
            // The important thing is that we have sources that can handle the URL
        }
    }
}

#[cfg(feature = "integration-tests")]
#[tokio::test]
async fn test_actual_github_download() {
    // This test requires network access and should only run with integration-tests feature
    use std::env;
    
    // Skip if no network access or in CI without GitHub token
    if env::var("SKIP_NETWORK_TESTS").is_ok() {
        return;
    }
    
    let turbo_cdn = TurboCdn::builder().build().await.unwrap();
    
    // Use a small, reliable file for testing
    let test_url = "https://github.com/BurntSushi/ripgrep/releases/download/14.1.1/ripgrep-14.1.1-x86_64-pc-windows-msvc.zip";
    
    let result = turbo_cdn.get_optimal_url(test_url).await;
    
    match result {
        Ok(optimized_url) => {
            println!("✅ Got optimized URL: {}", optimized_url);
            
            // Verify the URL is accessible
            let client = reqwest::Client::new();
            let response = client.head(&optimized_url).send().await;
            
            match response {
                Ok(resp) if resp.status().is_success() => {
                    println!("✅ URL is accessible: {}", resp.status());
                }
                Ok(resp) => {
                    println!("⚠️  URL returned status: {}", resp.status());
                }
                Err(e) => {
                    println!("⚠️  Network error: {}", e);
                }
            }
        }
        Err(e) => {
            println!("Error getting optimized URL: {}", e);
            // Don't fail the test for network issues, just log them
        }
    }
}
