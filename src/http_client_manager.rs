// Licensed under the MIT License
// Copyright (c) 2025 Hal <hal.long@outlook.com>

//! HTTP client manager using reqwest with rustls
//!
//! This module provides a simplified HTTP client manager using only reqwest
//! for better cross-platform compatibility and easier maintenance.

use crate::config::TurboCdnConfig;
use crate::error::Result;
use crate::http_client::HttpClient;
use std::sync::Arc;
use std::time::Duration;
use tracing::info;

/// HTTP client performance metrics
#[derive(Debug, Clone)]
pub struct ClientMetrics {
    pub avg_response_time: Duration,
    pub success_rate: f64,
    pub throughput_mbps: f64,
    pub last_updated: std::time::Instant,
}

/// HTTP request configuration
#[derive(Debug, Clone)]
pub struct RequestConfig {
    pub url: String,
    pub method: String,
    pub headers: Vec<(String, String)>,
    pub timeout: Duration,
    pub follow_redirects: bool,
    pub max_redirects: u32,
    pub enable_compression: bool,
    pub enable_http2: bool,
}

impl Default for RequestConfig {
    fn default() -> Self {
        Self {
            url: String::new(),
            method: "GET".to_string(),
            headers: Vec::new(),
            timeout: Duration::from_secs(30),
            follow_redirects: true,
            max_redirects: 10,
            enable_compression: true,
            enable_http2: true,
        }
    }
}

/// HTTP response wrapper
#[derive(Debug)]
pub struct HttpResponse {
    pub status: u16,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
    pub response_time: Duration,
    pub content_length: Option<u64>,
    pub supports_ranges: bool,
}

/// HTTP client manager using reqwest
#[derive(Debug)]
pub struct HttpClientManager {
    config: Arc<TurboCdnConfig>,
    client: HttpClient,
    metrics: Arc<std::sync::Mutex<ClientMetrics>>,
}

impl HttpClientManager {
    /// Create a new HTTP client manager
    pub fn new(config: Arc<TurboCdnConfig>) -> Result<Self> {
        let timeout = Duration::from_secs(config.performance.timeout);
        let client = HttpClient::new(timeout)?;

        let metrics = ClientMetrics {
            avg_response_time: Duration::from_millis(0),
            success_rate: 1.0,
            throughput_mbps: 0.0,
            last_updated: std::time::Instant::now(),
        };

        info!("HTTP client manager initialized with reqwest client");

        Ok(Self {
            config,
            client,
            metrics: Arc::new(std::sync::Mutex::new(metrics)),
        })
    }

    /// Perform a GET request
    pub async fn get(&self, url: &str) -> Result<HttpResponse> {
        let start_time = std::time::Instant::now();

        let response = self.client.get(url).await?;
        let response_time = start_time.elapsed();

        // Update metrics
        self.update_metrics(response_time, true);

        Ok(HttpResponse {
            status: response.status,
            headers: response.headers.into_iter().collect(),
            body: response.body,
            response_time,
            content_length: None,   // TODO: extract from headers
            supports_ranges: false, // TODO: check Accept-Ranges header
        })
    }

    /// Perform a GET request with custom headers
    pub async fn get_with_headers(
        &self,
        url: &str,
        headers: &std::collections::HashMap<String, String>,
    ) -> Result<HttpResponse> {
        let start_time = std::time::Instant::now();

        let response = self.client.get_with_headers(url, headers).await?;
        let response_time = start_time.elapsed();

        // Update metrics
        self.update_metrics(response_time, true);

        Ok(HttpResponse {
            status: response.status,
            headers: response.headers.into_iter().collect(),
            body: response.body,
            response_time,
            content_length: None,   // TODO: extract from headers
            supports_ranges: false, // TODO: check Accept-Ranges header
        })
    }

    /// Perform a HEAD request
    pub async fn head(&self, url: &str) -> Result<HttpResponse> {
        let start_time = std::time::Instant::now();

        let response = self.client.head(url).await?;
        let response_time = start_time.elapsed();

        // Update metrics
        self.update_metrics(response_time, true);

        Ok(HttpResponse {
            status: response.status,
            headers: response.headers.into_iter().collect(),
            body: response.body,
            response_time,
            content_length: None,   // TODO: extract from headers
            supports_ranges: false, // TODO: check Accept-Ranges header
        })
    }

    /// Update client metrics
    fn update_metrics(&self, response_time: Duration, success: bool) {
        if let Ok(mut metrics) = self.metrics.lock() {
            // Simple exponential moving average
            let alpha = 0.1;
            let new_avg = metrics.avg_response_time.as_millis() as f64 * (1.0 - alpha)
                + response_time.as_millis() as f64 * alpha;
            metrics.avg_response_time = Duration::from_millis(new_avg as u64);

            // Update success rate
            if success {
                metrics.success_rate = metrics.success_rate * 0.99 + 0.01;
            } else {
                metrics.success_rate = metrics.success_rate * 0.99;
            }

            metrics.last_updated = std::time::Instant::now();
        }
    }

    /// Get current metrics
    pub fn get_metrics(&self) -> Option<ClientMetrics> {
        self.metrics.lock().ok().map(|m| m.clone())
    }
}
