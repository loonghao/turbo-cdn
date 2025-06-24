// Licensed under the MIT License
// Copyright (c) 2025 Hal <hal.long@outlook.com>

//! High-performance HTTP client manager with multiple backend support
//!
//! This module provides a unified interface for different HTTP clients,
//! automatically selecting the best client based on performance characteristics.

use crate::config::TurboCdnConfig;
use crate::error::{Result, TurboCdnError};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, info, warn};

/// HTTP client types available
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HttpClientType {
    /// Reqwest - async HTTP client with good ecosystem support
    Reqwest,
    /// Isahc - high-performance HTTP client based on libcurl
    Isahc,
    /// Curl - direct libcurl bindings for maximum performance
    Curl,
}

/// HTTP client performance metrics
#[derive(Debug, Clone)]
pub struct ClientMetrics {
    pub client_type: HttpClientType,
    pub avg_response_time: Duration,
    pub success_rate: f64,
    pub throughput_mbps: f64,
    pub connection_reuse_rate: f64,
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
    pub client_type: HttpClientType,
    pub response_time: Duration,
    pub content_length: Option<u64>,
    pub supports_ranges: bool,
}

/// High-performance HTTP client manager
#[derive(Debug)]
pub struct HttpClientManager {
    config: Arc<TurboCdnConfig>,
    reqwest_client: Option<reqwest::Client>,
    isahc_client: Option<isahc::HttpClient>,
    client_metrics: Arc<std::sync::Mutex<Vec<ClientMetrics>>>,
    preferred_client: HttpClientType,
}

impl HttpClientManager {
    /// Create a new HTTP client manager
    pub fn new(config: Arc<TurboCdnConfig>) -> Result<Self> {
        let mut manager = Self {
            config: config.clone(),
            reqwest_client: None,
            isahc_client: None,
            client_metrics: Arc::new(std::sync::Mutex::new(Vec::new())),
            preferred_client: HttpClientType::Isahc, // Default to high-performance client
        };

        // Initialize clients
        manager.init_reqwest_client()?;
        manager.init_isahc_client()?;
        manager.init_metrics();

        info!(
            "HTTP client manager initialized with {} clients",
            manager.available_clients().len()
        );

        Ok(manager)
    }

    /// Initialize Reqwest client
    fn init_reqwest_client(&mut self) -> Result<()> {
        let mut builder = reqwest::Client::builder()
            .timeout(Duration::from_secs(self.config.performance.timeout))
            .user_agent(&self.config.general.user_agent)
            .pool_max_idle_per_host(self.config.performance.pool_max_idle_per_host)
            .pool_idle_timeout(Duration::from_secs(
                self.config.performance.pool_idle_timeout,
            ))
            .tcp_keepalive(Duration::from_secs(self.config.performance.tcp_keepalive))
            .tcp_nodelay(true)
            .redirect(reqwest::redirect::Policy::limited(10))
            .gzip(true)
            .brotli(true)
            .deflate(true);

        if self.config.performance.http2_prior_knowledge {
            builder = builder.http2_prior_knowledge();
        }

        // Enable HTTP/2 adaptive window and optimize frame size
        builder = builder
            .http2_adaptive_window(true)
            .http2_max_frame_size(Some(32768)); // 32KB frame size

        let client = builder.build().map_err(|e| {
            TurboCdnError::network(format!("Failed to create Reqwest client: {}", e))
        })?;

        self.reqwest_client = Some(client);
        debug!("Reqwest client initialized");
        Ok(())
    }

    /// Initialize Isahc client
    fn init_isahc_client(&mut self) -> Result<()> {
        use isahc::config::Configurable;

        let mut builder = isahc::HttpClient::builder()
            .timeout(Duration::from_secs(self.config.performance.timeout))
            .tcp_keepalive(Duration::from_secs(self.config.performance.tcp_keepalive))
            .tcp_nodelay()
            .redirect_policy(isahc::config::RedirectPolicy::Limit(10))
            .max_connections(self.config.performance.pool_max_idle_per_host)
            .max_connections_per_host(self.config.performance.pool_max_idle_per_host / 4);

        // Enable HTTP/2
        if self.config.performance.http2_prior_knowledge {
            builder = builder.version_negotiation(isahc::config::VersionNegotiation::http2());
        }

        // Enable compression
        builder = builder
            .automatic_decompression(true)
            .default_header("Accept-Encoding", "gzip, deflate, br");

        let client = builder
            .build()
            .map_err(|e| TurboCdnError::network(format!("Failed to create Isahc client: {}", e)))?;

        self.isahc_client = Some(client);
        debug!("Isahc client initialized");
        Ok(())
    }

    /// Initialize client metrics
    fn init_metrics(&mut self) {
        let mut metrics = self.client_metrics.lock().unwrap();

        // Initialize with default metrics for available clients
        for client_type in self.available_clients() {
            metrics.push(ClientMetrics {
                client_type,
                avg_response_time: Duration::from_millis(100),
                success_rate: 1.0,
                throughput_mbps: 10.0,
                connection_reuse_rate: 0.8,
                last_updated: std::time::Instant::now(),
            });
        }
    }

    /// Get list of available clients
    pub fn available_clients(&self) -> Vec<HttpClientType> {
        let mut clients = Vec::new();

        if self.reqwest_client.is_some() {
            clients.push(HttpClientType::Reqwest);
        }
        if self.isahc_client.is_some() {
            clients.push(HttpClientType::Isahc);
        }

        clients
    }

    /// Select the best client based on current metrics
    pub fn select_best_client(&self) -> HttpClientType {
        let metrics = self.client_metrics.lock().unwrap();

        if metrics.is_empty() {
            return self.preferred_client;
        }

        // Score clients based on multiple factors
        let mut best_client = self.preferred_client;
        let mut best_score = 0.0;

        for metric in metrics.iter() {
            let score = self.calculate_client_score(metric);
            if score > best_score {
                best_score = score;
                best_client = metric.client_type;
            }
        }

        debug!(
            "Selected {} client (score: {:.2})",
            format!("{:?}", best_client),
            best_score
        );

        best_client
    }

    /// Calculate client performance score
    fn calculate_client_score(&self, metrics: &ClientMetrics) -> f64 {
        // Weighted scoring: throughput (40%), success rate (30%), response time (20%), connection reuse (10%)
        let throughput_score = (metrics.throughput_mbps / 100.0).min(1.0) * 0.4;
        let success_score = metrics.success_rate * 0.3;
        let response_time_score =
            (1.0 - (metrics.avg_response_time.as_millis() as f64 / 1000.0).min(1.0)) * 0.2;
        let connection_score = metrics.connection_reuse_rate * 0.1;

        throughput_score + success_score + response_time_score + connection_score
    }

    /// Make an HTTP request using the best available client
    pub async fn request(&self, config: RequestConfig) -> Result<HttpResponse> {
        let client_type = self.select_best_client();
        let start_time = std::time::Instant::now();

        let result = match client_type {
            HttpClientType::Reqwest => self.request_with_reqwest(config).await,
            HttpClientType::Isahc => self.request_with_isahc(config).await,
            HttpClientType::Curl => {
                warn!("Curl client not yet implemented, falling back to Isahc");
                self.request_with_isahc(config).await
            }
        };

        let response_time = start_time.elapsed();

        // Update metrics based on result
        self.update_client_metrics(client_type, &result, response_time);

        result
    }

    /// Make request with Reqwest client
    async fn request_with_reqwest(&self, config: RequestConfig) -> Result<HttpResponse> {
        let client = self
            .reqwest_client
            .as_ref()
            .ok_or_else(|| TurboCdnError::network("Reqwest client not initialized".to_string()))?;

        let mut request = match config.method.as_str() {
            "GET" => client.get(&config.url),
            "HEAD" => client.head(&config.url),
            "POST" => client.post(&config.url),
            _ => {
                return Err(TurboCdnError::network(format!(
                    "Unsupported method: {}",
                    config.method
                )))
            }
        };

        // Add headers
        for (key, value) in config.headers {
            request = request.header(&key, &value);
        }

        // Set timeout
        request = request.timeout(config.timeout);

        let response = request
            .send()
            .await
            .map_err(|e| TurboCdnError::network(format!("Reqwest request failed: {}", e)))?;

        let status = response.status().as_u16();
        let headers: Vec<(String, String)> = response
            .headers()
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
            .collect();

        let content_length = response.content_length();
        let supports_ranges = response
            .headers()
            .get("accept-ranges")
            .map(|v| v.to_str().unwrap_or("").contains("bytes"))
            .unwrap_or(false);

        let body = response
            .bytes()
            .await
            .map_err(|e| TurboCdnError::network(format!("Failed to read response body: {}", e)))?
            .to_vec();

        Ok(HttpResponse {
            status,
            headers,
            body,
            client_type: HttpClientType::Reqwest,
            response_time: Duration::from_secs(0), // Will be set by caller
            content_length,
            supports_ranges,
        })
    }

    /// Make request with Isahc client
    async fn request_with_isahc(&self, config: RequestConfig) -> Result<HttpResponse> {
        let client = self
            .isahc_client
            .as_ref()
            .ok_or_else(|| TurboCdnError::network("Isahc client not initialized".to_string()))?;

        let mut request = isahc::Request::builder()
            .method(config.method.as_str())
            .uri(&config.url);

        // Add headers
        for (key, value) in config.headers {
            request = request.header(&key, &value);
        }

        let request = request
            .body(())
            .map_err(|e| TurboCdnError::network(format!("Failed to build Isahc request: {}", e)))?;

        let mut response = client
            .send_async(request)
            .await
            .map_err(|e| TurboCdnError::network(format!("Isahc request failed: {}", e)))?;

        let status = response.status().as_u16();
        let headers: Vec<(String, String)> = response
            .headers()
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
            .collect();

        let content_length = response
            .headers()
            .get("content-length")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse().ok());

        let supports_ranges = response
            .headers()
            .get("accept-ranges")
            .map(|v| v.to_str().unwrap_or("").contains("bytes"))
            .unwrap_or(false);

        let body = isahc::AsyncReadResponseExt::bytes(&mut response)
            .await
            .map_err(|e| {
                TurboCdnError::network(format!("Failed to read Isahc response body: {}", e))
            })?;

        Ok(HttpResponse {
            status,
            headers,
            body,
            client_type: HttpClientType::Isahc,
            response_time: Duration::from_secs(0), // Will be set by caller
            content_length,
            supports_ranges,
        })
    }

    /// Update client metrics based on request result
    fn update_client_metrics(
        &self,
        client_type: HttpClientType,
        result: &Result<HttpResponse>,
        response_time: Duration,
    ) {
        let mut metrics = self.client_metrics.lock().unwrap();

        if let Some(metric) = metrics.iter_mut().find(|m| m.client_type == client_type) {
            // Update response time (exponential moving average)
            let alpha = 0.3; // Smoothing factor
            metric.avg_response_time = Duration::from_millis(
                ((1.0 - alpha) * metric.avg_response_time.as_millis() as f64
                    + alpha * response_time.as_millis() as f64) as u64,
            );

            // Update success rate
            let success = result.is_ok();
            metric.success_rate = metric.success_rate * 0.9 + if success { 0.1 } else { 0.0 };

            // Update throughput if we have response data
            if let Ok(response) = result {
                if let Some(content_length) = response.content_length {
                    let throughput =
                        (content_length as f64 / 1024.0 / 1024.0) / response_time.as_secs_f64();
                    metric.throughput_mbps = metric.throughput_mbps * 0.8 + throughput * 0.2;
                }
            }

            metric.last_updated = std::time::Instant::now();
        }
    }

    /// Get current client metrics
    pub fn get_metrics(&self) -> Vec<ClientMetrics> {
        self.client_metrics.lock().unwrap().clone()
    }
}
