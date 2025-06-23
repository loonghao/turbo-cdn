// Licensed under the MIT License
// Copyright (c) 2025 Hal <hal.long@outlook.com>

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use tokio::io::AsyncWriteExt;
use tracing::{debug, info, warn};
use uuid::Uuid;

use crate::config::ComplianceConfig;
use crate::domain_manager::DomainManager;
use crate::error::{Result, TurboCdnError};
use crate::utils::PathManager;

/// Compliance checker for ensuring legal and ethical downloads
#[derive(Debug)]
pub struct ComplianceChecker {
    config: ComplianceConfig,
    audit_logger: AuditLogger,
    license_validator: LicenseValidator,
    content_validator: ContentValidator,
    domain_manager: DomainManager,
}

/// Audit logger for compliance tracking
#[derive(Debug)]
pub struct AuditLogger {
    log_path: std::path::PathBuf,
    enabled: bool,
}

/// License validator for open source verification
#[derive(Debug)]
pub struct LicenseValidator {
    known_open_source_licenses: HashSet<String>,
}

/// Content validator for copyright and source verification
#[derive(Debug)]
pub struct ContentValidator {
    blocked_patterns: Vec<regex::Regex>,
    #[allow(dead_code)]
    allowed_domains: HashSet<String>,
}

/// Download request information for compliance checking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadRequest {
    pub id: Uuid,
    pub url: String,
    pub source: String,
    pub repository: Option<String>,
    pub file_name: String,
    pub user_agent: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub user_consent: bool,
}

/// Compliance check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceResult {
    pub request_id: Uuid,
    pub approved: bool,
    pub reasons: Vec<String>,
    pub license_info: Option<LicenseInfo>,
    pub risk_level: RiskLevel,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// License information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseInfo {
    pub license_type: String,
    pub is_open_source: bool,
    pub allows_redistribution: bool,
    pub requires_attribution: bool,
    pub source_url: Option<String>,
}

/// Risk level assessment
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// Audit log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogEntry {
    pub id: Uuid,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub event_type: AuditEventType,
    pub request: DownloadRequest,
    pub result: ComplianceResult,
    pub additional_data: serde_json::Value,
}

/// Types of audit events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditEventType {
    ComplianceCheck,
    DownloadStarted,
    DownloadCompleted,
    DownloadFailed,
    PolicyViolation,
    UserConsentGranted,
    UserConsentRevoked,
}

impl ComplianceChecker {
    /// Create a new compliance checker
    pub fn new(config: ComplianceConfig) -> Result<Self> {
        let audit_logger =
            AuditLogger::new(config.audit_log_path.as_deref(), config.audit_logging)?;
        let license_validator = LicenseValidator::new();
        let content_validator = ContentValidator::new()?;
        let domain_manager = DomainManager::new();

        Ok(Self {
            config,
            audit_logger,
            license_validator,
            content_validator,
            domain_manager,
        })
    }

    /// Create a new compliance checker with custom domain manager
    pub fn with_domain_manager(
        config: ComplianceConfig,
        domain_manager: DomainManager,
    ) -> Result<Self> {
        let audit_logger =
            AuditLogger::new(config.audit_log_path.as_deref(), config.audit_logging)?;
        let license_validator = LicenseValidator::new();
        let content_validator = ContentValidator::new()?;

        Ok(Self {
            config,
            audit_logger,
            license_validator,
            content_validator,
            domain_manager,
        })
    }

    /// Get a reference to the domain manager
    pub fn domain_manager(&self) -> &DomainManager {
        &self.domain_manager
    }

    /// Get a mutable reference to the domain manager
    pub fn domain_manager_mut(&mut self) -> &mut DomainManager {
        &mut self.domain_manager
    }

    /// Check if a download request is compliant
    pub async fn check_compliance(&self, request: &DownloadRequest) -> Result<ComplianceResult> {
        let mut result = ComplianceResult {
            request_id: request.id,
            approved: true,
            reasons: Vec::new(),
            license_info: None,
            risk_level: RiskLevel::Low,
            timestamp: chrono::Utc::now(),
        };

        // Check user consent if required
        if self.config.data_protection.user_consent_required && !request.user_consent {
            result.approved = false;
            result
                .reasons
                .push("User consent required but not provided".to_string());
            result.risk_level = RiskLevel::Critical;
        }

        // Validate source domain using domain manager
        if self.config.validate_source {
            if let Err(e) = self.domain_manager.validate_url(&request.url) {
                result.approved = false;
                result
                    .reasons
                    .push(format!("Domain validation failed: {}", e));
                result.risk_level = std::cmp::max(result.risk_level, RiskLevel::High);
            }
        }

        // Check for blocked content patterns
        if let Err(e) = self
            .content_validator
            .check_blocked_patterns(&request.url, &request.file_name)
        {
            result.approved = false;
            result
                .reasons
                .push(format!("Content validation failed: {}", e));
            result.risk_level = RiskLevel::Critical;
        }

        // Verify open source license if repository is provided
        if self.config.verify_open_source {
            if let Some(repo) = &request.repository {
                match self.license_validator.validate_repository(repo).await {
                    Ok(license_info) => {
                        if !license_info.is_open_source {
                            result.approved = false;
                            result.reasons.push(
                                "Repository does not have an open source license".to_string(),
                            );
                            result.risk_level = RiskLevel::Critical;
                        }
                        result.license_info = Some(license_info);
                    }
                    Err(e) => {
                        warn!("Failed to validate license for repository {}: {}", repo, e);
                        if self.config.strict_mode {
                            result.approved = false;
                            result
                                .reasons
                                .push(format!("License validation failed: {}", e));
                            result.risk_level = std::cmp::max(result.risk_level, RiskLevel::Medium);
                        }
                    }
                }
            }
        }

        // Log the compliance check
        if self.config.audit_logging {
            let audit_entry = AuditLogEntry {
                id: Uuid::new_v4(),
                timestamp: chrono::Utc::now(),
                event_type: AuditEventType::ComplianceCheck,
                request: request.clone(),
                result: result.clone(),
                additional_data: serde_json::Value::Null,
            };

            if let Err(e) = self.audit_logger.log_entry(&audit_entry).await {
                warn!("Failed to log audit entry: {}", e);
            }
        }

        if result.approved {
            info!("Compliance check passed for request {}", request.id);
        } else {
            warn!(
                "Compliance check failed for request {}: {:?}",
                request.id, result.reasons
            );
        }

        Ok(result)
    }

    /// Log a download event
    pub async fn log_download_event(
        &self,
        event_type: AuditEventType,
        request: &DownloadRequest,
        additional_data: Option<serde_json::Value>,
    ) -> Result<()> {
        if !self.config.audit_logging {
            return Ok(());
        }

        let result = ComplianceResult {
            request_id: request.id,
            approved: true,
            reasons: Vec::new(),
            license_info: None,
            risk_level: RiskLevel::Low,
            timestamp: chrono::Utc::now(),
        };

        let audit_entry = AuditLogEntry {
            id: Uuid::new_v4(),
            timestamp: chrono::Utc::now(),
            event_type,
            request: request.clone(),
            result,
            additional_data: additional_data.unwrap_or(serde_json::Value::Null),
        };

        self.audit_logger.log_entry(&audit_entry).await
    }
}

impl AuditLogger {
    fn new(log_path_str: Option<&str>, enabled: bool) -> Result<Self> {
        let path_manager = PathManager::default();

        // Get audit log path - use provided path or default to data directory
        let log_path = if let Some(path_str) = log_path_str {
            // Expand the provided path (handles ~ and environment variables)
            path_manager.expand_path(path_str)?
        } else {
            // Use default audit log path in data directory
            path_manager.data_file("audit.log")?
        };

        if enabled {
            // Ensure the parent directory exists using PathManager
            if let Some(parent) = log_path.parent() {
                // Use sync version since this is called from sync context
                std::fs::create_dir_all(parent).map_err(|e| {
                    TurboCdnError::compliance(format!(
                        "Failed to create audit log directory: {}",
                        e
                    ))
                })?;
            }
        }

        Ok(Self { log_path, enabled })
    }

    async fn log_entry(&self, entry: &AuditLogEntry) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }

        let json_entry = serde_json::to_string(entry).map_err(|e| {
            TurboCdnError::compliance(format!("Failed to serialize audit entry: {}", e))
        })?;

        let log_line = format!("{}\n", json_entry);

        tokio::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_path)
            .await
            .map_err(|e| {
                TurboCdnError::compliance(format!("Failed to open audit log file: {}", e))
            })?
            .write_all(log_line.as_bytes())
            .await
            .map_err(|e| {
                TurboCdnError::compliance(format!("Failed to write audit log entry: {}", e))
            })?;

        debug!("Logged audit entry: {}", entry.id);
        Ok(())
    }
}

impl LicenseValidator {
    fn new() -> Self {
        let mut known_open_source_licenses = HashSet::new();

        // Add common open source licenses
        let licenses = vec![
            "MIT",
            "Apache-2.0",
            "GPL-3.0",
            "GPL-2.0",
            "BSD-3-Clause",
            "BSD-2-Clause",
            "ISC",
            "MPL-2.0",
            "LGPL-3.0",
            "LGPL-2.1",
            "AGPL-3.0",
            "Unlicense",
            "CC0-1.0",
            "Zlib",
            "BSL-1.0",
            "WTFPL",
            "0BSD",
        ];

        for license in licenses {
            known_open_source_licenses.insert(license.to_string());
        }

        Self {
            known_open_source_licenses,
        }
    }

    async fn validate_repository(&self, repository: &str) -> Result<LicenseInfo> {
        // For GitHub repositories, we can check the license via API
        if repository.contains("github.com") {
            return self.validate_github_repository(repository).await;
        }

        // For other repositories, we'll do a basic check
        self.validate_generic_repository(repository).await
    }

    async fn validate_github_repository(&self, repository: &str) -> Result<LicenseInfo> {
        // Extract owner/repo from URL
        let repo_path = repository
            .trim_start_matches("https://github.com/")
            .trim_start_matches("http://github.com/")
            .trim_end_matches(".git")
            .trim_end_matches('/');

        let api_url = format!("https://api.github.com/repos/{}", repo_path);

        let client = reqwest::Client::new();
        let response = client
            .get(&api_url)
            .header("User-Agent", "turbo-cdn")
            .send()
            .await
            .map_err(|e| {
                TurboCdnError::compliance(format!("Failed to fetch repository info: {}", e))
            })?;

        if !response.status().is_success() {
            return Err(TurboCdnError::compliance(format!(
                "GitHub API returned status: {}",
                response.status()
            )));
        }

        let repo_info: serde_json::Value = response.json().await.map_err(|e| {
            TurboCdnError::compliance(format!("Failed to parse GitHub API response: {}", e))
        })?;

        let license_info = repo_info.get("license");

        if let Some(license) = license_info {
            if license.is_null() {
                return Ok(LicenseInfo {
                    license_type: "Unknown".to_string(),
                    is_open_source: false,
                    allows_redistribution: false,
                    requires_attribution: false,
                    source_url: Some(repository.to_string()),
                });
            }

            let license_key = license
                .get("key")
                .and_then(|k| k.as_str())
                .unwrap_or("unknown");

            let license_name = license
                .get("name")
                .and_then(|n| n.as_str())
                .unwrap_or("Unknown");

            let is_open_source = self.known_open_source_licenses.contains(license_key)
                || self.known_open_source_licenses.contains(license_name);

            Ok(LicenseInfo {
                license_type: license_name.to_string(),
                is_open_source,
                allows_redistribution: is_open_source,
                requires_attribution: is_open_source
                    && !matches!(license_key, "unlicense" | "cc0-1.0"),
                source_url: Some(repository.to_string()),
            })
        } else {
            Ok(LicenseInfo {
                license_type: "Unknown".to_string(),
                is_open_source: false,
                allows_redistribution: false,
                requires_attribution: false,
                source_url: Some(repository.to_string()),
            })
        }
    }

    async fn validate_generic_repository(&self, repository: &str) -> Result<LicenseInfo> {
        // For non-GitHub repositories, we'll be more conservative
        // and assume they're not open source unless we can verify otherwise
        Ok(LicenseInfo {
            license_type: "Unknown".to_string(),
            is_open_source: false,
            allows_redistribution: false,
            requires_attribution: false,
            source_url: Some(repository.to_string()),
        })
    }
}

impl ContentValidator {
    fn new() -> Result<Self> {
        let mut blocked_patterns = Vec::new();

        // Add patterns for potentially problematic content
        let patterns = vec![
            r"(?i)crack",
            r"(?i)keygen",
            r"(?i)serial",
            r"(?i)patch",
            r"(?i)pirate",
            r"(?i)warez",
            r"(?i)nulled",
            r"(?i)torrent",
        ];

        for pattern in patterns {
            let regex = regex::Regex::new(pattern)
                .map_err(|e| TurboCdnError::compliance(format!("Invalid regex pattern: {}", e)))?;
            blocked_patterns.push(regex);
        }

        let mut allowed_domains = HashSet::new();
        let domains = vec![
            "github.com",
            "githubusercontent.com",
            "cdn.jsdelivr.net",
            "fastly.jsdelivr.net",
            "cdnjs.cloudflare.com",
            "unpkg.com",
            "registry.npmjs.org",
            "pypi.org",
            "crates.io",
            "maven.org",
            "repo1.maven.org",
            "central.sonatype.com",
        ];

        for domain in domains {
            allowed_domains.insert(domain.to_string());
        }

        Ok(Self {
            blocked_patterns,
            allowed_domains,
        })
    }

    #[allow(dead_code)]
    fn validate_source(&self, url: &str) -> Result<()> {
        let parsed_url = url::Url::parse(url)
            .map_err(|e| TurboCdnError::compliance(format!("Invalid URL: {}", e)))?;

        if let Some(domain) = parsed_url.domain() {
            if !self
                .allowed_domains
                .iter()
                .any(|allowed| domain.ends_with(allowed))
            {
                return Err(TurboCdnError::compliance(format!(
                    "Domain '{}' is not in the allowed list",
                    domain
                )));
            }
        } else {
            return Err(TurboCdnError::compliance(
                "URL does not have a valid domain".to_string(),
            ));
        }

        Ok(())
    }

    fn check_blocked_patterns(&self, url: &str, file_name: &str) -> Result<()> {
        let combined_text = format!("{} {}", url, file_name);

        for pattern in &self.blocked_patterns {
            if pattern.is_match(&combined_text) {
                return Err(TurboCdnError::compliance(format!(
                    "Content matches blocked pattern: {}",
                    pattern.as_str()
                )));
            }
        }

        Ok(())
    }
}

impl std::cmp::PartialOrd for RiskLevel {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl std::cmp::Ord for RiskLevel {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let self_value = match self {
            RiskLevel::Low => 0,
            RiskLevel::Medium => 1,
            RiskLevel::High => 2,
            RiskLevel::Critical => 3,
        };

        let other_value = match other {
            RiskLevel::Low => 0,
            RiskLevel::Medium => 1,
            RiskLevel::High => 2,
            RiskLevel::Critical => 3,
        };

        self_value.cmp(&other_value)
    }
}
