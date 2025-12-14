// Licensed under the MIT License
// Copyright (c) 2025 Hal <hal.long@outlook.com>

//! URL mapping system for CDN optimization
//!
//! This module provides automatic URL mapping from original sources to optimized CDN URLs
//! based on regex patterns and geographic location.

use crate::cdn_quality::CdnQualityAssessor;
use crate::config::{Region, TurboCdnConfig, UrlMappingRuleConfig};
use crate::error::{Result, TurboCdnError};
use crate::server_tracker::ServerTracker;
use regex::Regex;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tracing::{debug, info, warn};

use dashmap::DashMap;

/// URL mapping rule configuration
#[derive(Debug, Clone)]
pub struct UrlMappingRule {
    /// Regex pattern to match URLs
    pub pattern: Regex,
    /// Replacement URL templates (in priority order)
    pub replacements: Vec<String>,
    /// Applicable regions for this rule
    pub regions: Vec<Region>,
    /// Priority (lower = higher priority)
    pub priority: u32,
    /// Whether this rule is enabled
    pub enabled: bool,
}

/// Cache entry for URL mappings
#[derive(Debug, Clone)]
struct CacheEntry {
    /// Mapped URLs
    urls: Vec<String>,
    /// Timestamp when cached
    cached_at: Instant,
    /// TTL for this entry
    ttl: Duration,
}

impl CacheEntry {
    fn new(urls: Vec<String>, ttl: Duration) -> Self {
        Self {
            urls,
            cached_at: Instant::now(),
            ttl,
        }
    }

    fn is_expired(&self) -> bool {
        self.cached_at.elapsed() > self.ttl
    }
}

/// High-performance concurrent cache
type CacheMap = DashMap<String, CacheEntry>;

/// URL mapper for CDN optimization with performance-based selection
#[derive(Debug)]
pub struct UrlMapper {
    rules: Vec<UrlMappingRule>,
    current_region: Region,
    cache: CacheMap,
    cache_enabled: bool,
    cache_ttl: Duration,
    max_cache_entries: usize,
    server_tracker: Arc<Mutex<ServerTracker>>,
    #[allow(dead_code)]
    quality_assessor: Option<Arc<CdnQualityAssessor>>,
}

impl UrlMapper {
    /// Create high-performance cache
    fn create_cache() -> CacheMap {
        DashMap::new()
    }

    /// Create a new URL mapper with configuration
    pub fn new(config: &TurboCdnConfig, region: Region) -> Result<Self> {
        let mut rules = Vec::new();

        // Load rules from configuration
        for rule_config in &config.url_mapping_rules {
            if let Ok(rule) = Self::create_rule_from_config(rule_config) {
                rules.push(rule);
            } else {
                warn!("Failed to create rule from config: {}", rule_config.name);
            }
        }

        // If no rules in config, use built-in rules as fallback
        if rules.is_empty() {
            warn!("No URL mapping rules found in config, using built-in rules");
            rules.extend(Self::create_builtin_rules()?);
        }

        // Sort rules by priority
        rules.sort_by_key(|rule| rule.priority);

        info!(
            "Initialized URL mapper with {} rules for region {:?}",
            rules.len(),
            region
        );

        // CDN quality assessor will be created separately to avoid async complexity
        let quality_assessor = None;

        Ok(Self {
            rules,
            current_region: region,
            cache: Self::create_cache(),
            cache_enabled: config.general.enable_url_cache,
            cache_ttl: Duration::from_secs(config.general.url_cache_ttl),
            max_cache_entries: config.general.max_cache_entries,
            server_tracker: Arc::new(Mutex::new(ServerTracker::new())),
            quality_assessor,
        })
    }

    /// Create a rule from configuration
    fn create_rule_from_config(config: &UrlMappingRuleConfig) -> Result<UrlMappingRule> {
        let pattern = Regex::new(&config.pattern).map_err(|e| {
            TurboCdnError::config(format!("Invalid regex pattern '{}': {e}", config.pattern))
        })?;

        Ok(UrlMappingRule {
            pattern,
            replacements: config.replacements.clone(),
            regions: config.regions.clone(),
            priority: config.priority,
            enabled: config.enabled,
        })
    }

    /// Map a URL to the best CDN alternative with performance-based ordering
    ///
    /// This is a non-mutable version that works with RwLock read guards.
    pub fn map_url(&self, original_url: &str) -> Result<Vec<String>> {
        debug!("Mapping URL: {}", original_url);

        // Check cache first
        if self.cache_enabled {
            if let Some(entry) = self.cache.get(original_url) {
                if !entry.is_expired() {
                    debug!("Cache hit for URL: {}", original_url);
                    return Ok(entry.urls.clone());
                }
            }
        }

        let mut mapped_urls = Vec::new();

        // Try to match against rules
        for rule in &self.rules {
            if !rule.enabled {
                continue;
            }

            // Check if rule applies to current region
            if !rule.regions.is_empty() && !rule.regions.contains(&self.current_region) {
                continue;
            }

            // Try to match the pattern
            if let Some(captures) = rule.pattern.captures(original_url) {
                debug!(
                    "Matched rule '{}' with pattern: {}",
                    rule.pattern.as_str(),
                    rule.pattern.as_str()
                );

                // Generate replacement URLs
                for template in &rule.replacements {
                    if let Ok(mapped_url) = Self::apply_template(template, &captures) {
                        if mapped_url != original_url {
                            mapped_urls.push(mapped_url);
                        }
                    }
                }

                // Only use the first matching rule
                break;
            }
        }

        // Always include the original URL as fallback
        mapped_urls.push(original_url.to_string());

        // Sort URLs by server performance if we have performance data
        if mapped_urls.len() > 1 {
            mapped_urls = self.sort_urls_by_performance(mapped_urls);
        }

        // Cache the result
        if self.cache_enabled {
            self.cache_result(original_url.to_string(), mapped_urls.clone());
        }

        debug!("Mapped to {} URLs: {:?}", mapped_urls.len(), mapped_urls);
        Ok(mapped_urls)
    }

    /// Map a URL (mutable version for backward compatibility)
    #[deprecated(note = "Use map_url instead, which doesn't require mutable access")]
    pub fn map_url_mut(&mut self, original_url: &str) -> Result<Vec<String>> {
        self.map_url(original_url)
    }

    /// Create built-in rules as fallback when no config rules are available
    fn create_builtin_rules() -> Result<Vec<UrlMappingRule>> {
        warn!("Using built-in fallback rules. Consider adding rules to your configuration file.");

        // Create minimal fallback rules from embedded config
        let fallback_configs = vec![UrlMappingRuleConfig {
            name: "GitHub Releases - Global Fallback".to_string(),
            pattern: r"^https://github\.com/([^/]+)/([^/]+)/releases/download/([^/]+)/(.+)$"
                .to_string(),
            replacements: vec![
                "https://ghproxy.net/https://github.com/$1/$2/releases/download/$3/$4".to_string(),
                "https://github.com/$1/$2/releases/download/$3/$4".to_string(),
            ],
            regions: vec![Region::Global],
            priority: 100, // Lower priority than config rules
            enabled: true,
        }];

        let mut rules = Vec::new();
        for config in fallback_configs {
            if let Ok(rule) = Self::create_rule_from_config(&config) {
                rules.push(rule);
            }
        }

        Ok(rules)
    }

    /// Apply template with regex captures
    fn apply_template(template: &str, captures: &regex::Captures) -> Result<String> {
        let mut result = template.to_string();

        // Replace $1, $2, etc. with capture groups
        for (i, capture) in captures.iter().enumerate() {
            if let Some(matched) = capture {
                let placeholder = format!("${i}");
                result = result.replace(&placeholder, matched.as_str());
            }
        }

        Ok(result)
    }

    /// Update current region
    pub fn set_region(&mut self, region: Region) {
        info!("Updated URL mapper region to: {:?}", region);
        self.current_region = region;
    }

    /// Get current region
    pub fn region(&self) -> &Region {
        &self.current_region
    }

    /// Sort URLs by server performance (best performing first)
    /// Note: CDN quality assessment is handled separately to avoid async complexity
    fn sort_urls_by_performance(&self, mut urls: Vec<String>) -> Vec<String> {
        // Use server tracker performance for sorting
        if let Ok(tracker) = self.server_tracker.lock() {
            urls.sort_by(|a, b| {
                let score_a = tracker
                    .get_stats(a)
                    .map(|s| s.performance_score())
                    .unwrap_or(0.0);
                let score_b = tracker
                    .get_stats(b)
                    .map(|s| s.performance_score())
                    .unwrap_or(0.0);
                // Higher score = better performance, so reverse order
                score_b
                    .partial_cmp(&score_a)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
        }
        urls
    }

    /// Get CDN quality assessor for external use
    pub fn get_quality_assessor(&self) -> Option<Arc<CdnQualityAssessor>> {
        self.quality_assessor.clone()
    }

    /// Cache the mapping result
    fn cache_result(&self, url: String, urls: Vec<String>) {
        // Clean up expired entries if cache is getting full
        if self.cache.len() >= self.max_cache_entries {
            self.cleanup_cache();
        }

        let entry = CacheEntry::new(urls, self.cache_ttl);
        self.cache.insert(url, entry);
    }

    /// Clean up expired cache entries
    fn cleanup_cache(&self) {
        // Remove expired entries
        self.cache.retain(|_, entry| !entry.is_expired());

        // If still too many entries, remove oldest ones
        if self.cache.len() >= self.max_cache_entries {
            let mut entries: Vec<_> = self
                .cache
                .iter()
                .map(|item| (item.key().clone(), item.value().cached_at))
                .collect();
            entries.sort_by_key(|(_, cached_at)| *cached_at);

            let to_remove = entries.len() - self.max_cache_entries / 2;
            let urls_to_remove: Vec<_> = entries
                .into_iter()
                .take(to_remove)
                .map(|(url, _)| url)
                .collect();

            for url in urls_to_remove {
                self.cache.remove(&url);
            }
        }
    }

    /// Get server tracker for external use
    pub fn get_server_tracker(&self) -> Arc<Mutex<ServerTracker>> {
        self.server_tracker.clone()
    }

    /// Get number of active rules
    pub fn rule_count(&self) -> usize {
        self.rules.iter().filter(|rule| rule.enabled).count()
    }

    /// Get cache statistics
    pub fn cache_stats(&self) -> (usize, usize) {
        let total = self.cache.len();
        let expired = self
            .cache
            .iter()
            .filter(|item| item.value().is_expired())
            .count();
        (total, expired)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_github_url_mapping() {
        let config = TurboCdnConfig::default();
        let mapper = UrlMapper::new(&config, Region::China).unwrap();

        let original_url = "https://github.com/BurntSushi/ripgrep/releases/download/14.1.1/ripgrep-14.1.1-x86_64-pc-windows-msvc.zip";
        let mapped_urls = mapper.map_url(original_url).unwrap();

        assert!(!mapped_urls.is_empty());
        assert!(mapped_urls.contains(&original_url.to_string()));
    }

    #[test]
    fn test_jsdelivr_url_mapping() {
        let config = TurboCdnConfig::default();
        let mapper = UrlMapper::new(&config, Region::Global).unwrap();

        let original_url = "https://cdn.jsdelivr.net/npm/package@1.0.0/dist/package.min.js";
        let mapped_urls = mapper.map_url(original_url).unwrap();

        assert!(!mapped_urls.is_empty());
        assert!(mapped_urls.contains(&original_url.to_string()));
    }

    #[test]
    fn test_unknown_url_passthrough() {
        let config = TurboCdnConfig::default();
        let mapper = UrlMapper::new(&config, Region::Global).unwrap();

        let original_url = "https://example.com/file.zip";
        let mapped_urls = mapper.map_url(original_url).unwrap();

        assert_eq!(mapped_urls.len(), 1);
        assert_eq!(mapped_urls[0], original_url);
    }
}
