// Licensed under the MIT License
// Copyright (c) 2025 Hal <hal.long@outlook.com>

//! # Domain Manager
//!
//! Extensible domain management system for turbo-cdn.
//! Provides flexible domain validation with support for:
//! - Built-in trusted domains
//! - Custom domain lists
//! - Domain categories and tags
//! - Runtime domain management
//! - Validation bypass options

use crate::error::{Result, TurboCdnError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Domain category for organizing trusted domains
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DomainCategory {
    /// Official package repositories
    PackageRegistry,
    /// Content delivery networks
    CDN,
    /// Version control systems
    VersionControl,
    /// Official language/framework sites
    Official,
    /// Archive and mirror sites
    Archive,
    /// Custom user-defined category
    Custom(String),
}

/// Domain entry with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainEntry {
    /// Domain name (e.g., "github.com")
    pub domain: String,
    /// Category classification
    pub category: DomainCategory,
    /// Optional description
    pub description: Option<String>,
    /// Tags for flexible filtering
    pub tags: Vec<String>,
    /// Whether this domain is enabled
    pub enabled: bool,
    /// Trust level (0-100, higher = more trusted)
    pub trust_level: u8,
}

/// Domain validation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainValidationConfig {
    /// Enable domain validation
    pub enabled: bool,
    /// Strict mode - only allow explicitly listed domains
    pub strict_mode: bool,
    /// Allow subdomains of trusted domains
    pub allow_subdomains: bool,
    /// Minimum trust level required (0-100)
    pub min_trust_level: u8,
    /// Categories to allow (empty = allow all)
    pub allowed_categories: Vec<DomainCategory>,
    /// Tags to allow (empty = allow all)
    pub allowed_tags: Vec<String>,
}

/// Domain manager for extensible domain validation
#[derive(Debug, Clone)]
pub struct DomainManager {
    /// Domain entries by domain name
    domains: HashMap<String, DomainEntry>,
    /// Validation configuration
    config: DomainValidationConfig,
}

impl DomainEntry {
    /// Create a new domain entry
    pub fn new(domain: impl Into<String>, category: DomainCategory) -> Self {
        Self {
            domain: domain.into(),
            category,
            description: None,
            tags: Vec::new(),
            enabled: true,
            trust_level: 80, // Default trust level
        }
    }

    /// Set description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Add tags
    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    /// Set trust level
    pub fn with_trust_level(mut self, trust_level: u8) -> Self {
        self.trust_level = trust_level.min(100);
        self
    }

    /// Enable or disable domain
    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}

impl DomainValidationConfig {
    /// Create a new validation config with defaults
    pub fn new() -> Self {
        Self {
            enabled: true,
            strict_mode: true,
            allow_subdomains: true,
            min_trust_level: 50,
            allowed_categories: Vec::new(), // Empty = allow all
            allowed_tags: Vec::new(),       // Empty = allow all
        }
    }

    /// Disable domain validation entirely
    pub fn disabled() -> Self {
        Self {
            enabled: false,
            strict_mode: false,
            allow_subdomains: true,
            min_trust_level: 0,
            allowed_categories: Vec::new(),
            allowed_tags: Vec::new(),
        }
    }

    /// Permissive mode - allow most domains
    pub fn permissive() -> Self {
        Self {
            enabled: true,
            strict_mode: false,
            allow_subdomains: true,
            min_trust_level: 30,
            allowed_categories: Vec::new(),
            allowed_tags: Vec::new(),
        }
    }

    /// Strict mode - only explicitly allowed domains
    pub fn strict() -> Self {
        Self {
            enabled: true,
            strict_mode: true,
            allow_subdomains: false,
            min_trust_level: 70,
            allowed_categories: Vec::new(),
            allowed_tags: Vec::new(),
        }
    }
}

impl Default for DomainValidationConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl DomainManager {
    /// Create a new domain manager with built-in trusted domains
    pub fn new() -> Self {
        let mut manager = Self {
            domains: HashMap::new(),
            config: DomainValidationConfig::default(),
        };

        // Add built-in trusted domains
        manager.add_builtin_domains();
        manager
    }

    /// Create domain manager with custom config
    pub fn with_config(config: DomainValidationConfig) -> Self {
        let mut manager = Self {
            domains: HashMap::new(),
            config,
        };

        manager.add_builtin_domains();
        manager
    }

    /// Add built-in trusted domains
    fn add_builtin_domains(&mut self) {
        let builtin_domains = vec![
            // Version Control & Code Hosting
            DomainEntry::new("github.com", DomainCategory::VersionControl)
                .with_description("GitHub - Git repository hosting")
                .with_tags(vec!["git".to_string(), "opensource".to_string()])
                .with_trust_level(95),
            DomainEntry::new("githubusercontent.com", DomainCategory::VersionControl)
                .with_description("GitHub raw content")
                .with_tags(vec!["git".to_string(), "raw".to_string()])
                .with_trust_level(95),
            DomainEntry::new("gitlab.com", DomainCategory::VersionControl)
                .with_description("GitLab - Git repository hosting")
                .with_tags(vec!["git".to_string(), "opensource".to_string()])
                .with_trust_level(90),
            DomainEntry::new("bitbucket.org", DomainCategory::VersionControl)
                .with_description("Bitbucket - Git repository hosting")
                .with_tags(vec!["git".to_string()])
                .with_trust_level(85),
            // CDN Networks
            DomainEntry::new("cdn.jsdelivr.net", DomainCategory::CDN)
                .with_description("jsDelivr CDN")
                .with_tags(vec!["cdn".to_string(), "fast".to_string()])
                .with_trust_level(90),
            DomainEntry::new("fastly.jsdelivr.net", DomainCategory::CDN)
                .with_description("jsDelivr CDN via Fastly")
                .with_tags(vec!["cdn".to_string(), "fast".to_string()])
                .with_trust_level(90),
            DomainEntry::new("cdnjs.cloudflare.com", DomainCategory::CDN)
                .with_description("Cloudflare CDN")
                .with_tags(vec!["cdn".to_string(), "cloudflare".to_string()])
                .with_trust_level(90),
            DomainEntry::new("unpkg.com", DomainCategory::CDN)
                .with_description("unpkg CDN for npm packages")
                .with_tags(vec!["cdn".to_string(), "npm".to_string()])
                .with_trust_level(85),
            // Package Registries
            DomainEntry::new("registry.npmjs.org", DomainCategory::PackageRegistry)
                .with_description("npm package registry")
                .with_tags(vec!["npm".to_string(), "javascript".to_string()])
                .with_trust_level(95),
            DomainEntry::new("pypi.org", DomainCategory::PackageRegistry)
                .with_description("Python Package Index")
                .with_tags(vec!["python".to_string(), "pip".to_string()])
                .with_trust_level(95),
            DomainEntry::new("files.pythonhosted.org", DomainCategory::PackageRegistry)
                .with_description("PyPI file hosting")
                .with_tags(vec!["python".to_string(), "pip".to_string()])
                .with_trust_level(95),
            DomainEntry::new("crates.io", DomainCategory::PackageRegistry)
                .with_description("Rust package registry")
                .with_tags(vec!["rust".to_string(), "cargo".to_string()])
                .with_trust_level(95),
            DomainEntry::new("repo1.maven.org", DomainCategory::PackageRegistry)
                .with_description("Maven Central Repository")
                .with_tags(vec!["java".to_string(), "maven".to_string()])
                .with_trust_level(95),
            DomainEntry::new("central.sonatype.com", DomainCategory::PackageRegistry)
                .with_description("Sonatype Maven Central")
                .with_tags(vec!["java".to_string(), "maven".to_string()])
                .with_trust_level(90),
            // Official Language/Framework Sites
            DomainEntry::new("nodejs.org", DomainCategory::Official)
                .with_description("Node.js official website")
                .with_tags(vec![
                    "nodejs".to_string(),
                    "javascript".to_string(),
                    "official".to_string(),
                ])
                .with_trust_level(100),
            DomainEntry::new("golang.org", DomainCategory::Official)
                .with_description("Go programming language official site")
                .with_tags(vec![
                    "go".to_string(),
                    "golang".to_string(),
                    "official".to_string(),
                ])
                .with_trust_level(100),
            DomainEntry::new("go.dev", DomainCategory::Official)
                .with_description("Go development hub")
                .with_tags(vec![
                    "go".to_string(),
                    "golang".to_string(),
                    "official".to_string(),
                ])
                .with_trust_level(100),
            DomainEntry::new("forge.rust-lang.org", DomainCategory::Official)
                .with_description("Rust language forge")
                .with_tags(vec!["rust".to_string(), "official".to_string()])
                .with_trust_level(100),
            DomainEntry::new("static.rust-lang.org", DomainCategory::Official)
                .with_description("Rust static resources")
                .with_tags(vec!["rust".to_string(), "official".to_string()])
                .with_trust_level(100),
            DomainEntry::new("download.qt.io", DomainCategory::Official)
                .with_description("Qt framework downloads")
                .with_tags(vec![
                    "qt".to_string(),
                    "cpp".to_string(),
                    "official".to_string(),
                ])
                .with_trust_level(95),
            // Archives and Mirrors
            DomainEntry::new("archive.apache.org", DomainCategory::Archive)
                .with_description("Apache Software Foundation archive")
                .with_tags(vec!["apache".to_string(), "archive".to_string()])
                .with_trust_level(95),
            DomainEntry::new("releases.hashicorp.com", DomainCategory::Archive)
                .with_description("HashiCorp releases")
                .with_tags(vec!["hashicorp".to_string(), "terraform".to_string()])
                .with_trust_level(90),
        ];

        for domain in builtin_domains {
            self.domains.insert(domain.domain.clone(), domain);
        }
    }

    /// Add a custom domain
    pub fn add_domain(&mut self, domain: DomainEntry) -> Result<()> {
        if domain.domain.is_empty() {
            return Err(TurboCdnError::config(
                "Domain name cannot be empty".to_string(),
            ));
        }

        self.domains.insert(domain.domain.clone(), domain);
        Ok(())
    }

    /// Add multiple domains
    pub fn add_domains(&mut self, domains: Vec<DomainEntry>) -> Result<()> {
        for domain in domains {
            self.add_domain(domain)?;
        }
        Ok(())
    }

    /// Remove a domain
    pub fn remove_domain(&mut self, domain: &str) -> bool {
        self.domains.remove(domain).is_some()
    }

    /// Enable or disable a domain
    pub fn set_domain_enabled(&mut self, domain: &str, enabled: bool) -> Result<()> {
        if let Some(entry) = self.domains.get_mut(domain) {
            entry.enabled = enabled;
            Ok(())
        } else {
            Err(TurboCdnError::config(format!(
                "Domain '{}' not found",
                domain
            )))
        }
    }

    /// Get domain entry
    pub fn get_domain(&self, domain: &str) -> Option<&DomainEntry> {
        self.domains.get(domain)
    }

    /// List all domains
    pub fn list_domains(&self) -> Vec<&DomainEntry> {
        self.domains.values().collect()
    }

    /// List domains by category
    pub fn list_domains_by_category(&self, category: &DomainCategory) -> Vec<&DomainEntry> {
        self.domains
            .values()
            .filter(|entry| &entry.category == category)
            .collect()
    }

    /// List domains by tag
    pub fn list_domains_by_tag(&self, tag: &str) -> Vec<&DomainEntry> {
        self.domains
            .values()
            .filter(|entry| entry.tags.contains(&tag.to_string()))
            .collect()
    }

    /// Update validation configuration
    pub fn set_config(&mut self, config: DomainValidationConfig) {
        self.config = config;
    }

    /// Get current configuration
    pub fn get_config(&self) -> &DomainValidationConfig {
        &self.config
    }

    /// Validate a URL against domain rules
    pub fn validate_url(&self, url: &str) -> Result<()> {
        // If validation is disabled, allow everything
        if !self.config.enabled {
            return Ok(());
        }

        let parsed_url = url::Url::parse(url)
            .map_err(|e| TurboCdnError::compliance(format!("Invalid URL: {}", e)))?;

        let domain = parsed_url.domain().ok_or_else(|| {
            TurboCdnError::compliance("URL does not have a valid domain".to_string())
        })?;

        self.validate_domain(domain)
    }

    /// Validate a domain against rules
    pub fn validate_domain(&self, domain: &str) -> Result<()> {
        // If validation is disabled, allow everything
        if !self.config.enabled {
            return Ok(());
        }

        // Try exact match first
        if let Some(entry) = self.domains.get(domain) {
            return self.validate_domain_entry(entry);
        }

        // If subdomain checking is enabled, check parent domains
        if self.config.allow_subdomains {
            for (trusted_domain, entry) in &self.domains {
                if domain.ends_with(&format!(".{}", trusted_domain)) {
                    return self.validate_domain_entry(entry);
                }
            }
        }

        // In strict mode, reject unknown domains
        if self.config.strict_mode {
            return Err(TurboCdnError::compliance(format!(
                "Domain '{}' is not in the allowed list",
                domain
            )));
        }

        // In non-strict mode, allow unknown domains
        Ok(())
    }

    /// Validate a domain entry against configuration rules
    fn validate_domain_entry(&self, entry: &DomainEntry) -> Result<()> {
        // Check if domain is enabled
        if !entry.enabled {
            return Err(TurboCdnError::compliance(format!(
                "Domain '{}' is disabled",
                entry.domain
            )));
        }

        // Check trust level
        if entry.trust_level < self.config.min_trust_level {
            return Err(TurboCdnError::compliance(format!(
                "Domain '{}' trust level ({}) below minimum required ({})",
                entry.domain, entry.trust_level, self.config.min_trust_level
            )));
        }

        // Check allowed categories (if specified)
        if !self.config.allowed_categories.is_empty()
            && !self.config.allowed_categories.contains(&entry.category)
        {
            return Err(TurboCdnError::compliance(format!(
                "Domain '{}' category '{:?}' not in allowed categories",
                entry.domain, entry.category
            )));
        }

        // Check allowed tags (if specified)
        if !self.config.allowed_tags.is_empty() {
            let has_allowed_tag = entry
                .tags
                .iter()
                .any(|tag| self.config.allowed_tags.contains(tag));

            if !has_allowed_tag {
                return Err(TurboCdnError::compliance(format!(
                    "Domain '{}' does not have any allowed tags",
                    entry.domain
                )));
            }
        }

        Ok(())
    }

    /// Get statistics about managed domains
    pub fn get_stats(&self) -> DomainStats {
        let total = self.domains.len();
        let enabled = self.domains.values().filter(|d| d.enabled).count();
        let disabled = total - enabled;

        let mut by_category = HashMap::new();
        let mut by_trust_level = HashMap::new();

        for entry in self.domains.values() {
            *by_category.entry(entry.category.clone()).or_insert(0) += 1;

            let trust_range = match entry.trust_level {
                90..=100 => "High (90-100)".to_string(),
                70..=89 => "Medium (70-89)".to_string(),
                50..=69 => "Low (50-69)".to_string(),
                _ => "Very Low (0-49)".to_string(),
            };
            *by_trust_level.entry(trust_range).or_insert(0) += 1;
        }

        DomainStats {
            total,
            enabled,
            disabled,
            by_category,
            by_trust_level,
        }
    }
}

/// Domain statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainStats {
    pub total: usize,
    pub enabled: usize,
    pub disabled: usize,
    pub by_category: HashMap<DomainCategory, usize>,
    pub by_trust_level: HashMap<String, usize>,
}

impl Default for DomainManager {
    fn default() -> Self {
        Self::new()
    }
}
