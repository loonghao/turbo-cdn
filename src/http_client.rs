// Licensed under the MIT License
// Copyright (c) 2025 Hal <hal.long@outlook.com>

//! HTTP client implementation using reqwest with rustls
//!
//! This module provides a simple, reliable HTTP client implementation
//! using reqwest with rustls for better cross-platform compatibility.

use crate::error::{Result, TurboCdnError};
use std::collections::HashMap;
use std::time::Duration;

/// HTTP response abstraction
#[derive(Debug)]
pub struct HttpResponse {
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
}

/// HTTP client implementation using reqwest
#[derive(Debug)]
pub struct HttpClient {
    client: reqwest::Client,
}

impl HttpClient {
    /// Create a new HTTP client with the specified timeout
    pub fn new(timeout: Duration) -> Result<Self> {
        // Initialize rustls provider before creating reqwest client
        crate::init_rustls_provider();

        let client = reqwest::Client::builder()
            .timeout(timeout)
            .tcp_keepalive(Duration::from_secs(60))
            .tcp_nodelay(true)
            .pool_max_idle_per_host(20)
            .http2_prior_knowledge()
            .build()
            .map_err(|e| TurboCdnError::network(format!("Failed to create HTTP client: {e}")))?;

        Ok(Self { client })
    }

    /// Perform a GET request
    pub async fn get(&self, url: &str) -> Result<HttpResponse> {
        let response = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|e| TurboCdnError::network(format!("GET request failed: {e}")))?;

        let status = response.status().as_u16();
        let headers = response
            .headers()
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
            .collect();

        let body = response
            .bytes()
            .await
            .map_err(|e| TurboCdnError::network(format!("Failed to read response body: {e}")))?
            .to_vec();

        Ok(HttpResponse {
            status,
            headers,
            body,
        })
    }

    /// Perform a GET request with custom headers
    pub async fn get_with_headers(
        &self,
        url: &str,
        request_headers: &HashMap<String, String>,
    ) -> Result<HttpResponse> {
        let mut request = self.client.get(url);

        for (key, value) in request_headers {
            request = request.header(key, value);
        }

        let response = request
            .send()
            .await
            .map_err(|e| TurboCdnError::network(format!("GET request with headers failed: {e}")))?;

        let status = response.status().as_u16();
        let headers = response
            .headers()
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
            .collect();

        let body = response
            .bytes()
            .await
            .map_err(|e| TurboCdnError::network(format!("Failed to read response body: {e}")))?
            .to_vec();

        Ok(HttpResponse {
            status,
            headers,
            body,
        })
    }

    /// Perform a HEAD request
    pub async fn head(&self, url: &str) -> Result<HttpResponse> {
        let response = self
            .client
            .head(url)
            .send()
            .await
            .map_err(|e| TurboCdnError::network(format!("HEAD request failed: {e}")))?;

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

    /// Get client name for debugging
    pub fn name(&self) -> &'static str {
        "reqwest"
    }
}
