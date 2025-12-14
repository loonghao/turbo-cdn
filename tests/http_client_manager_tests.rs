// Licensed under the MIT License
// Copyright (c) 2025 Hal <hal.long@outlook.com>

//! Tests for the HTTP client manager module

use std::time::Duration;
use turbo_cdn::http_client_manager::{ClientMetrics, HttpResponse, RequestConfig};

// ============================================================================
// RequestConfig Tests
// ============================================================================

#[test]
fn test_request_config_default() {
    let config = RequestConfig::default();

    assert!(config.url.is_empty());
    assert_eq!(config.method, "GET");
    assert!(config.headers.is_empty());
    assert_eq!(config.timeout, Duration::from_secs(30));
    assert!(config.follow_redirects);
    assert_eq!(config.max_redirects, 10);
    assert!(config.enable_compression);
    assert!(config.enable_http2);
}

#[test]
fn test_request_config_custom() {
    let config = RequestConfig {
        url: "https://example.com/api".to_string(),
        method: "POST".to_string(),
        headers: vec![
            ("Content-Type".to_string(), "application/json".to_string()),
            ("Authorization".to_string(), "Bearer token".to_string()),
        ],
        timeout: Duration::from_secs(60),
        follow_redirects: false,
        max_redirects: 5,
        enable_compression: true,
        enable_http2: false,
    };

    assert_eq!(config.url, "https://example.com/api");
    assert_eq!(config.method, "POST");
    assert_eq!(config.headers.len(), 2);
    assert_eq!(config.timeout, Duration::from_secs(60));
    assert!(!config.follow_redirects);
    assert_eq!(config.max_redirects, 5);
    assert!(!config.enable_http2);
}

#[test]
fn test_request_config_clone() {
    let config = RequestConfig {
        url: "https://test.com".to_string(),
        method: "GET".to_string(),
        headers: vec![("Accept".to_string(), "text/html".to_string())],
        timeout: Duration::from_secs(15),
        follow_redirects: true,
        max_redirects: 3,
        enable_compression: false,
        enable_http2: true,
    };

    let cloned = config.clone();
    assert_eq!(cloned.url, config.url);
    assert_eq!(cloned.method, config.method);
    assert_eq!(cloned.headers, config.headers);
    assert_eq!(cloned.timeout, config.timeout);
}

#[test]
fn test_request_config_debug() {
    let config = RequestConfig::default();
    let debug_str = format!("{:?}", config);
    assert!(debug_str.contains("RequestConfig"));
    assert!(debug_str.contains("GET"));
}

// ============================================================================
// ClientMetrics Tests
// ============================================================================

#[test]
fn test_client_metrics_creation() {
    let metrics = ClientMetrics {
        avg_response_time: Duration::from_millis(150),
        success_rate: 0.95,
        throughput_mbps: 10.5,
        last_updated: std::time::Instant::now(),
    };

    assert_eq!(metrics.avg_response_time, Duration::from_millis(150));
    assert_eq!(metrics.success_rate, 0.95);
    assert_eq!(metrics.throughput_mbps, 10.5);
}

#[test]
fn test_client_metrics_clone() {
    let metrics = ClientMetrics {
        avg_response_time: Duration::from_millis(100),
        success_rate: 0.99,
        throughput_mbps: 25.0,
        last_updated: std::time::Instant::now(),
    };

    let cloned = metrics.clone();
    assert_eq!(cloned.avg_response_time, metrics.avg_response_time);
    assert_eq!(cloned.success_rate, metrics.success_rate);
    assert_eq!(cloned.throughput_mbps, metrics.throughput_mbps);
}

#[test]
fn test_client_metrics_debug() {
    let metrics = ClientMetrics {
        avg_response_time: Duration::from_millis(200),
        success_rate: 0.9,
        throughput_mbps: 5.0,
        last_updated: std::time::Instant::now(),
    };

    let debug_str = format!("{:?}", metrics);
    assert!(debug_str.contains("ClientMetrics"));
    assert!(debug_str.contains("avg_response_time"));
}

#[test]
fn test_client_metrics_edge_values() {
    // Test with zero values
    let zero_metrics = ClientMetrics {
        avg_response_time: Duration::ZERO,
        success_rate: 0.0,
        throughput_mbps: 0.0,
        last_updated: std::time::Instant::now(),
    };
    assert_eq!(zero_metrics.avg_response_time, Duration::ZERO);
    assert_eq!(zero_metrics.success_rate, 0.0);

    // Test with max values
    let max_metrics = ClientMetrics {
        avg_response_time: Duration::from_secs(3600), // 1 hour
        success_rate: 1.0,
        throughput_mbps: 10000.0, // 10 Gbps
        last_updated: std::time::Instant::now(),
    };
    assert_eq!(max_metrics.success_rate, 1.0);
    assert_eq!(max_metrics.throughput_mbps, 10000.0);
}

// ============================================================================
// HttpResponse Tests
// ============================================================================

#[test]
fn test_http_response_success() {
    let response = HttpResponse {
        status: 200,
        headers: vec![
            ("Content-Type".to_string(), "text/html".to_string()),
            ("Content-Length".to_string(), "1024".to_string()),
        ],
        body: vec![0u8; 1024],
        response_time: Duration::from_millis(100),
        content_length: Some(1024),
        supports_ranges: true,
    };

    assert_eq!(response.status, 200);
    assert_eq!(response.headers.len(), 2);
    assert_eq!(response.body.len(), 1024);
    assert_eq!(response.content_length, Some(1024));
    assert!(response.supports_ranges);
}

#[test]
fn test_http_response_redirect() {
    let response = HttpResponse {
        status: 302,
        headers: vec![(
            "Location".to_string(),
            "https://example.com/new-location".to_string(),
        )],
        body: vec![],
        response_time: Duration::from_millis(50),
        content_length: None,
        supports_ranges: false,
    };

    assert_eq!(response.status, 302);
    assert!(response.body.is_empty());
    assert!(response.content_length.is_none());
}

#[test]
fn test_http_response_error() {
    let response = HttpResponse {
        status: 500,
        headers: vec![("Content-Type".to_string(), "text/plain".to_string())],
        body: b"Internal Server Error".to_vec(),
        response_time: Duration::from_millis(200),
        content_length: Some(21),
        supports_ranges: false,
    };

    assert_eq!(response.status, 500);
    assert_eq!(response.body, b"Internal Server Error");
}

#[test]
fn test_http_response_debug() {
    let response = HttpResponse {
        status: 200,
        headers: vec![],
        body: vec![],
        response_time: Duration::from_millis(100),
        content_length: None,
        supports_ranges: false,
    };

    let debug_str = format!("{:?}", response);
    assert!(debug_str.contains("HttpResponse"));
    assert!(debug_str.contains("200"));
}

#[test]
fn test_http_response_large_body() {
    let large_body = vec![0u8; 10 * 1024 * 1024]; // 10 MB

    let response = HttpResponse {
        status: 200,
        headers: vec![],
        body: large_body.clone(),
        response_time: Duration::from_secs(5),
        content_length: Some(10 * 1024 * 1024),
        supports_ranges: true,
    };

    assert_eq!(response.body.len(), 10 * 1024 * 1024);
}

#[test]
fn test_http_response_various_status_codes() {
    let status_codes = vec![
        (200, "OK"),
        (201, "Created"),
        (204, "No Content"),
        (301, "Moved Permanently"),
        (304, "Not Modified"),
        (400, "Bad Request"),
        (401, "Unauthorized"),
        (403, "Forbidden"),
        (404, "Not Found"),
        (500, "Internal Server Error"),
        (502, "Bad Gateway"),
        (503, "Service Unavailable"),
    ];

    for (status, _description) in status_codes {
        let response = HttpResponse {
            status,
            headers: vec![],
            body: vec![],
            response_time: Duration::from_millis(100),
            content_length: None,
            supports_ranges: false,
        };

        assert_eq!(response.status, status);
    }
}

// ============================================================================
// Edge Case Tests
// ============================================================================

#[test]
fn test_request_config_empty_headers() {
    let config = RequestConfig {
        url: "https://example.com".to_string(),
        method: "GET".to_string(),
        headers: vec![],
        ..Default::default()
    };

    assert!(config.headers.is_empty());
}

#[test]
fn test_request_config_many_headers() {
    let mut headers = vec![];
    for i in 0..100 {
        headers.push((format!("X-Custom-Header-{}", i), format!("value-{}", i)));
    }

    let config = RequestConfig {
        url: "https://example.com".to_string(),
        method: "GET".to_string(),
        headers,
        ..Default::default()
    };

    assert_eq!(config.headers.len(), 100);
}

#[test]
fn test_request_config_special_characters_in_url() {
    let config = RequestConfig {
        url: "https://example.com/path?query=value&special=%20%21%40".to_string(),
        method: "GET".to_string(),
        headers: vec![],
        ..Default::default()
    };

    assert!(config.url.contains("special=%20%21%40"));
}

#[test]
fn test_http_response_binary_body() {
    // Test with binary data (not valid UTF-8)
    let binary_body: Vec<u8> = (0..=255).collect();

    let response = HttpResponse {
        status: 200,
        headers: vec![(
            "Content-Type".to_string(),
            "application/octet-stream".to_string(),
        )],
        body: binary_body.clone(),
        response_time: Duration::from_millis(50),
        content_length: Some(256),
        supports_ranges: true,
    };

    assert_eq!(response.body.len(), 256);
    assert_eq!(response.body, binary_body);
}

#[test]
fn test_http_response_unicode_headers() {
    let response = HttpResponse {
        status: 200,
        headers: vec![
            (
                "Content-Type".to_string(),
                "text/html; charset=utf-8".to_string(),
            ),
            ("X-Custom".to_string(), "值包含中文".to_string()),
        ],
        body: "Hello, 世界!".as_bytes().to_vec(),
        response_time: Duration::from_millis(100),
        content_length: None,
        supports_ranges: false,
    };

    assert_eq!(response.headers.len(), 2);
    assert!(response.headers[1].1.contains("中文"));
}

#[test]
fn test_client_metrics_precision() {
    let metrics = ClientMetrics {
        avg_response_time: Duration::from_nanos(123456789),
        success_rate: 0.999999,
        throughput_mbps: 0.001,
        last_updated: std::time::Instant::now(),
    };

    assert_eq!(metrics.avg_response_time.as_nanos(), 123456789);
    assert!((metrics.success_rate - 0.999999).abs() < 0.0000001);
    assert!((metrics.throughput_mbps - 0.001).abs() < 0.0001);
}

#[test]
fn test_request_config_all_http_methods() {
    let methods = vec!["GET", "POST", "PUT", "DELETE", "PATCH", "HEAD", "OPTIONS"];

    for method in methods {
        let config = RequestConfig {
            url: "https://example.com".to_string(),
            method: method.to_string(),
            ..Default::default()
        };

        assert_eq!(config.method, method);
    }
}

#[test]
fn test_http_response_empty_body_with_content_length() {
    let response = HttpResponse {
        status: 204, // No Content
        headers: vec![],
        body: vec![],
        response_time: Duration::from_millis(10),
        content_length: Some(0),
        supports_ranges: false,
    };

    assert!(response.body.is_empty());
    assert_eq!(response.content_length, Some(0));
}

#[test]
fn test_request_config_timeout_variations() {
    let timeouts = vec![
        Duration::from_millis(100),
        Duration::from_secs(1),
        Duration::from_secs(30),
        Duration::from_secs(300),
        Duration::from_secs(3600),
    ];

    for timeout in timeouts {
        let config = RequestConfig {
            url: "https://example.com".to_string(),
            timeout,
            ..Default::default()
        };

        assert_eq!(config.timeout, timeout);
    }
}
