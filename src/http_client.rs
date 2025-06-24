// Licensed under the MIT License
// Copyright (c) 2025 Hal <hal.long@outlook.com>

//! High-performance HTTP client abstraction
//!
//! This module provides a unified interface for different HTTP clients,
//! allowing runtime selection of the best performing client.

use crate::error::{Result, TurboCdnError};
use futures_util::io::AsyncReadExt;
use isahc::config::Configurable;
use std::collections::HashMap;
use std::time::Duration;

/// HTTP response abstraction
#[derive(Debug)]
pub struct HttpResponse {
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
}

/// HTTP client trait for different implementations
#[async_trait::async_trait]
pub trait HttpClient: Send + Sync {
    /// Perform a GET request
    async fn get(&self, url: &str) -> Result<HttpResponse>;

    /// Perform a GET request with headers
    async fn get_with_headers(
        &self,
        url: &str,
        headers: &HashMap<String, String>,
    ) -> Result<HttpResponse>;

    /// Perform a HEAD request
    async fn head(&self, url: &str) -> Result<HttpResponse>;

    /// Get client name for debugging
    fn name(&self) -> &'static str;
}

/// Isahc-based HTTP client (high performance)
pub struct IsahcClient {
    client: isahc::HttpClient,
}

impl IsahcClient {
    pub fn new(timeout: Duration) -> Result<Self> {
        let client = isahc::HttpClient::builder()
            .timeout(timeout)
            .tcp_keepalive(Duration::from_secs(60))
            .tcp_nodelay()
            .max_connections_per_host(20)
            .build()
            .map_err(|e| TurboCdnError::network(format!("Failed to create isahc client: {}", e)))?;

        Ok(Self { client })
    }
}

#[async_trait::async_trait]
impl HttpClient for IsahcClient {
    async fn get(&self, url: &str) -> Result<HttpResponse> {
        let response = self
            .client
            .get_async(url)
            .await
            .map_err(|e| TurboCdnError::network(format!("GET request failed: {}", e)))?;

        let status = response.status().as_u16();
        let headers = response
            .headers()
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
            .collect();

        let mut body_reader = response.into_body();
        let mut body = Vec::new();
        body_reader
            .read_to_end(&mut body)
            .await
            .map_err(|e| TurboCdnError::network(format!("Failed to read response body: {}", e)))?;

        Ok(HttpResponse {
            status,
            headers,
            body,
        })
    }

    async fn get_with_headers(
        &self,
        url: &str,
        request_headers: &HashMap<String, String>,
    ) -> Result<HttpResponse> {
        let mut request = isahc::Request::get(url);

        for (key, value) in request_headers {
            request = request.header(key, value);
        }

        let response = self
            .client
            .send_async(request.body(()).unwrap())
            .await
            .map_err(|e| {
                TurboCdnError::network(format!("GET request with headers failed: {}", e))
            })?;

        let status = response.status().as_u16();
        let headers = response
            .headers()
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
            .collect();

        let mut body_reader = response.into_body();
        let mut body = Vec::new();
        body_reader
            .read_to_end(&mut body)
            .await
            .map_err(|e| TurboCdnError::network(format!("Failed to read response body: {}", e)))?;

        Ok(HttpResponse {
            status,
            headers,
            body,
        })
    }

    async fn head(&self, url: &str) -> Result<HttpResponse> {
        let response = self
            .client
            .head_async(url)
            .await
            .map_err(|e| TurboCdnError::network(format!("HEAD request failed: {}", e)))?;

        let status = response.status().as_u16();
        let headers = response
            .headers()
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
            .collect();

        Ok(HttpResponse {
            status,
            headers,
            body: Vec::new(),
        })
    }

    fn name(&self) -> &'static str {
        "isahc"
    }
}

/// Reqwest-based HTTP client (fallback)
pub struct ReqwestClient {
    client: reqwest::Client,
}

impl ReqwestClient {
    pub fn new(timeout: Duration) -> Result<Self> {
        let client = reqwest::Client::builder()
            .timeout(timeout)
            .tcp_keepalive(Duration::from_secs(60))
            .tcp_nodelay(true)
            .pool_max_idle_per_host(20)
            .http2_prior_knowledge()
            .build()
            .map_err(|e| {
                TurboCdnError::network(format!("Failed to create reqwest client: {}", e))
            })?;

        Ok(Self { client })
    }
}

#[async_trait::async_trait]
impl HttpClient for ReqwestClient {
    async fn get(&self, url: &str) -> Result<HttpResponse> {
        let response = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|e| TurboCdnError::network(format!("GET request failed: {}", e)))?;

        let status = response.status().as_u16();
        let headers = response
            .headers()
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
            .collect();

        let body = response
            .bytes()
            .await
            .map_err(|e| TurboCdnError::network(format!("Failed to read response body: {}", e)))?
            .to_vec();

        Ok(HttpResponse {
            status,
            headers,
            body,
        })
    }

    async fn get_with_headers(
        &self,
        url: &str,
        request_headers: &HashMap<String, String>,
    ) -> Result<HttpResponse> {
        let mut request = self.client.get(url);

        for (key, value) in request_headers {
            request = request.header(key, value);
        }

        let response = request.send().await.map_err(|e| {
            TurboCdnError::network(format!("GET request with headers failed: {}", e))
        })?;

        let status = response.status().as_u16();
        let headers = response
            .headers()
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
            .collect();

        let body = response
            .bytes()
            .await
            .map_err(|e| TurboCdnError::network(format!("Failed to read response body: {}", e)))?
            .to_vec();

        Ok(HttpResponse {
            status,
            headers,
            body,
        })
    }

    async fn head(&self, url: &str) -> Result<HttpResponse> {
        let response = self
            .client
            .head(url)
            .send()
            .await
            .map_err(|e| TurboCdnError::network(format!("HEAD request failed: {}", e)))?;

        let status = response.status().as_u16();
        let headers = response
            .headers()
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
            .collect();

        Ok(HttpResponse {
            status,
            headers,
            body: Vec::new(),
        })
    }

    fn name(&self) -> &'static str {
        "reqwest"
    }
}

/// Create the best available HTTP client
pub fn create_best_client(timeout: Duration) -> Result<Box<dyn HttpClient>> {
    // Try isahc first (highest performance)
    match IsahcClient::new(timeout) {
        Ok(client) => {
            tracing::info!("Using isahc HTTP client for optimal performance");
            Ok(Box::new(client))
        }
        Err(e) => {
            tracing::warn!(
                "Failed to create isahc client: {}, falling back to reqwest",
                e
            );
            let client = ReqwestClient::new(timeout)?;
            Ok(Box::new(client))
        }
    }
}
