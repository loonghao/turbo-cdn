// Licensed under the MIT License
// Copyright (c) 2025 Hal <hal.long@outlook.com>

//! # Geographic Location Detection
//!
//! Automatically detects user's geographic location for optimal CDN selection.
//! Uses multiple detection methods with fallback strategies.

use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::time::timeout;
use tracing::{debug, info, warn};

use crate::config::{Region, TurboCdnConfig};
use crate::error::{Result, TurboCdnError};

/// Geographic location detector with configuration support
#[derive(Debug)]
pub struct GeoDetector {
    client: reqwest::Client,
    cache: Option<DetectionResult>,
    cache_ttl: Duration,
    #[allow(dead_code)]
    config: TurboCdnConfig,
}

/// Detection result with confidence score
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectionResult {
    pub region: Region,
    pub country_code: String,
    pub confidence: f64,
    pub detected_at: chrono::DateTime<chrono::Utc>,
    pub method: DetectionMethod,
}

/// Detection method used
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DetectionMethod {
    IpApi,
    NetworkTest,
    Fallback,
    Manual,
}

/// IP geolocation response from ip-api.com
#[derive(Debug, Deserialize)]
struct IpApiResponse {
    status: String,
    country: String,
    #[serde(rename = "countryCode")]
    country_code: String,
    #[allow(dead_code)]
    region: String,
    #[serde(rename = "regionName")]
    #[allow(dead_code)]
    region_name: String,
    #[allow(dead_code)]
    city: String,
    timezone: String,
}

impl GeoDetector {
    /// Create a new geo detector with configuration
    pub fn new(config: TurboCdnConfig) -> Self {
        let timeout = Duration::from_secs(config.geo_detection.ip_detection_timeout);
        let client = reqwest::Client::builder()
            .timeout(timeout)
            .user_agent(&config.general.user_agent)
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        Self {
            client,
            cache: None,
            cache_ttl: Duration::from_secs(config.general.url_cache_ttl),
            config,
        }
    }

    /// Detect user's geographic region
    pub async fn detect_region(&mut self) -> Result<Region> {
        // Check cache first
        if let Some(cached) = &self.cache {
            let age = chrono::Utc::now().signed_duration_since(cached.detected_at);
            if age.to_std().unwrap_or(Duration::MAX) < self.cache_ttl {
                debug!("Using cached geo detection result: {:?}", cached.region);
                return Ok(cached.region.clone());
            }
        }

        info!("Detecting geographic location...");

        // Try multiple detection methods
        let result = self.detect_with_fallback().await?;

        // Cache the result
        self.cache = Some(result.clone());

        info!(
            "Detected region: {:?} (confidence: {:.1}%, method: {:?})",
            result.region,
            result.confidence * 100.0,
            result.method
        );

        Ok(result.region)
    }

    /// Detect with multiple fallback methods
    async fn detect_with_fallback(&self) -> Result<DetectionResult> {
        // Method 1: IP geolocation API
        if let Ok(result) = self.detect_via_ip_api().await {
            return Ok(result);
        }

        warn!("IP geolocation failed, trying network tests...");

        // Method 2: Network performance tests
        if let Ok(result) = self.detect_via_network_test().await {
            return Ok(result);
        }

        warn!("Network tests failed, using fallback...");

        // Method 3: Fallback to Global
        Ok(DetectionResult {
            region: Region::Global,
            country_code: "UNKNOWN".to_string(),
            confidence: 0.1,
            detected_at: chrono::Utc::now(),
            method: DetectionMethod::Fallback,
        })
    }

    /// Detect region using IP geolocation API
    async fn detect_via_ip_api(&self) -> Result<DetectionResult> {
        let url = "http://ip-api.com/json/?fields=status,country,countryCode,region,regionName,city,timezone";

        let response = timeout(Duration::from_secs(5), self.client.get(url).send())
            .await
            .map_err(|_| TurboCdnError::network("IP geolocation request timeout"))?
            .map_err(TurboCdnError::Network)?;

        let ip_info: IpApiResponse = response.json().await.map_err(TurboCdnError::Network)?;

        if ip_info.status != "success" {
            return Err(TurboCdnError::network("IP geolocation API failed"));
        }

        let region = self.country_to_region(&ip_info.country_code);
        let confidence = self.calculate_confidence(&ip_info);

        debug!(
            "IP geolocation: {} ({}) -> {:?}",
            ip_info.country, ip_info.country_code, region
        );

        Ok(DetectionResult {
            region,
            country_code: ip_info.country_code,
            confidence,
            detected_at: chrono::Utc::now(),
            method: DetectionMethod::IpApi,
        })
    }

    /// Detect region using network performance tests
    async fn detect_via_network_test(&self) -> Result<DetectionResult> {
        let test_urls = vec![
            ("github", "https://api.github.com"),
            ("jsdelivr", "https://cdn.jsdelivr.net"),
            ("fastly", "https://fastly.jsdelivr.net"),
            ("cloudflare", "https://cdnjs.cloudflare.com"),
        ];

        let mut results = Vec::new();

        for (name, url) in test_urls {
            if let Ok(latency) = self.test_latency(url).await {
                results.push((name, latency));
                debug!("Network test {}: {:.0}ms", name, latency);
            }
        }

        if results.is_empty() {
            return Err(TurboCdnError::network("All network tests failed"));
        }

        // Sort by latency
        results.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

        let best_cdn = results[0].0;
        let region = self.cdn_to_region(best_cdn);

        Ok(DetectionResult {
            region,
            country_code: "DETECTED".to_string(),
            confidence: 0.7,
            detected_at: chrono::Utc::now(),
            method: DetectionMethod::NetworkTest,
        })
    }

    /// Test latency to a URL
    async fn test_latency(&self, url: &str) -> Result<f64> {
        let start = std::time::Instant::now();

        let _response = timeout(Duration::from_secs(3), self.client.head(url).send())
            .await
            .map_err(|_| TurboCdnError::network("Latency test timeout"))?
            .map_err(TurboCdnError::Network)?;

        let latency = start.elapsed().as_millis() as f64;
        Ok(latency)
    }

    /// Map country code to region
    fn country_to_region(&self, country_code: &str) -> Region {
        match country_code {
            // China
            "CN" => Region::China,

            // North America
            "US" | "CA" | "MX" => Region::NorthAmerica,

            // Europe
            "GB" | "DE" | "FR" | "IT" | "ES" | "NL" | "SE" | "NO" | "DK" | "FI" | "PL" | "CZ"
            | "AT" | "CH" | "BE" | "IE" | "PT" | "GR" | "HU" | "RO" | "BG" | "HR" | "SI" | "SK"
            | "LT" | "LV" | "EE" | "LU" | "MT" | "CY" => Region::Europe,

            // Asia Pacific
            "JP" | "KR" | "SG" | "HK" | "TW" | "AU" | "NZ" | "IN" | "TH" | "MY" | "ID" | "PH"
            | "VN" | "BD" | "PK" | "LK" | "MM" | "KH" | "LA" | "BN" => Region::AsiaPacific,

            // Default to Global for other countries
            _ => Region::Global,
        }
    }

    /// Map best performing CDN to likely region
    fn cdn_to_region(&self, cdn: &str) -> Region {
        match cdn {
            "github" => Region::NorthAmerica, // GitHub is US-based
            "jsdelivr" => Region::Europe,     // jsDelivr has good EU presence
            "fastly" => Region::China,        // Fastly works well in China
            "cloudflare" => Region::Global,   // Cloudflare is global
            _ => Region::Global,
        }
    }

    /// Calculate confidence based on IP info quality
    fn calculate_confidence(&self, ip_info: &IpApiResponse) -> f64 {
        let mut confidence: f64 = 0.8; // Base confidence for IP geolocation

        // Higher confidence for specific countries
        if matches!(
            ip_info.country_code.as_str(),
            "CN" | "US" | "GB" | "DE" | "JP"
        ) {
            confidence += 0.1;
        }

        // Lower confidence if timezone doesn't match expected region
        if ip_info.timezone.is_empty() {
            confidence -= 0.1;
        }

        confidence.clamp(0.1, 0.95)
    }

    /// Get cached detection result
    pub fn get_cached_result(&self) -> Option<&DetectionResult> {
        self.cache.as_ref()
    }

    /// Clear cache to force re-detection
    pub fn clear_cache(&mut self) {
        self.cache = None;
    }
}

impl Default for GeoDetector {
    fn default() -> Self {
        Self::new(TurboCdnConfig::default())
    }
}
