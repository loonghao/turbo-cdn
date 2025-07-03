// Licensed under the MIT License
// Copyright (c) 2025 Hal <hal.long@outlook.com>

use std::fmt;
use thiserror::Error;

/// Main error type for turbo-cdn operations
#[derive(Error, Debug)]
pub enum TurboCdnError {
    /// Network-related errors
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    /// IO-related errors
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// URL parsing errors
    #[error("Invalid URL: {0}")]
    InvalidUrl(#[from] url::ParseError),

    /// JSON parsing errors
    #[error("JSON parsing error: {0}")]
    Json(#[from] serde_json::Error),

    /// Configuration errors
    #[error("Configuration error: {message}")]
    Config { message: String },

    /// Download errors
    #[error("Download failed: {message}")]
    Download { message: String },

    /// Source validation errors
    #[error("Source validation failed: {message}")]
    SourceValidation { message: String },

    /// Compliance errors
    #[error("Compliance check failed: {message}")]
    Compliance { message: String },

    /// Cache errors
    #[error("Cache error: {message}")]
    Cache { message: String },

    /// Routing errors
    #[error("Routing error: {message}")]
    Routing { message: String },

    /// Authentication errors
    #[error("Authentication failed: {message}")]
    Authentication { message: String },

    /// Rate limiting errors
    #[error("Rate limit exceeded: {message}")]
    RateLimit { message: String },

    /// Timeout errors
    #[error("Operation timed out: {message}")]
    Timeout { message: String },

    /// Checksum validation errors
    #[error("Checksum validation failed: expected {expected}, got {actual}")]
    ChecksumMismatch { expected: String, actual: String },

    /// File not found errors
    #[error("File not found: {path}")]
    FileNotFound { path: String },

    /// Unsupported operation errors
    #[error("Unsupported operation: {message}")]
    Unsupported { message: String },

    /// Generic errors
    #[error("Internal error: {message}")]
    Internal { message: String },
}

impl TurboCdnError {
    /// Create a new configuration error
    pub fn config<S: Into<String>>(message: S) -> Self {
        Self::Config {
            message: message.into(),
        }
    }

    /// Create a new download error
    pub fn download<S: Into<String>>(message: S) -> Self {
        Self::Download {
            message: message.into(),
        }
    }

    /// Create a new source validation error
    pub fn source_validation<S: Into<String>>(message: S) -> Self {
        Self::SourceValidation {
            message: message.into(),
        }
    }

    /// Create a new compliance error
    pub fn compliance<S: Into<String>>(message: S) -> Self {
        Self::Compliance {
            message: message.into(),
        }
    }

    /// Create a new cache error
    pub fn cache<S: Into<String>>(message: S) -> Self {
        Self::Cache {
            message: message.into(),
        }
    }

    /// Create a new routing error
    pub fn routing<S: Into<String>>(message: S) -> Self {
        Self::Routing {
            message: message.into(),
        }
    }

    /// Create a new authentication error
    pub fn authentication<S: Into<String>>(message: S) -> Self {
        Self::Authentication {
            message: message.into(),
        }
    }

    /// Create a new rate limit error
    pub fn rate_limit<S: Into<String>>(message: S) -> Self {
        Self::RateLimit {
            message: message.into(),
        }
    }

    /// Create a new timeout error
    pub fn timeout<S: Into<String>>(message: S) -> Self {
        Self::Timeout {
            message: message.into(),
        }
    }

    /// Create a new checksum mismatch error
    pub fn checksum_mismatch<S: Into<String>>(expected: S, actual: S) -> Self {
        Self::ChecksumMismatch {
            expected: expected.into(),
            actual: actual.into(),
        }
    }

    /// Create a new file not found error
    pub fn file_not_found<S: Into<String>>(path: S) -> Self {
        Self::FileNotFound { path: path.into() }
    }

    /// Create a new unsupported operation error
    pub fn unsupported<S: Into<String>>(message: S) -> Self {
        Self::Unsupported {
            message: message.into(),
        }
    }

    /// Create a new internal error
    pub fn internal<S: Into<String>>(message: S) -> Self {
        Self::Internal {
            message: message.into(),
        }
    }

    /// Create a new network error
    pub fn network<S: Into<String>>(message: S) -> Self {
        let msg = message.into();
        Self::Internal {
            message: format!("Network error: {msg}"),
        }
    }

    /// Create a new IO error
    pub fn io<S: Into<String>>(message: S) -> Self {
        let msg = message.into();
        Self::Internal {
            message: format!("IO error: {msg}"),
        }
    }

    /// Check if the error is retryable
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            TurboCdnError::Network(_)
                | TurboCdnError::Timeout { .. }
                | TurboCdnError::RateLimit { .. }
                | TurboCdnError::Io(_)
        )
    }

    /// Get the error category for metrics and logging
    pub fn category(&self) -> &'static str {
        match self {
            TurboCdnError::Network(_) => "network",
            TurboCdnError::Io(_) => "io",
            TurboCdnError::InvalidUrl(_) => "url",
            TurboCdnError::Json(_) => "json",
            TurboCdnError::Config { .. } => "config",
            TurboCdnError::Download { .. } => "download",
            TurboCdnError::SourceValidation { .. } => "source_validation",
            TurboCdnError::Compliance { .. } => "compliance",
            TurboCdnError::Cache { .. } => "cache",
            TurboCdnError::Routing { .. } => "routing",
            TurboCdnError::Authentication { .. } => "authentication",
            TurboCdnError::RateLimit { .. } => "rate_limit",
            TurboCdnError::Timeout { .. } => "timeout",
            TurboCdnError::ChecksumMismatch { .. } => "checksum",
            TurboCdnError::FileNotFound { .. } => "file_not_found",
            TurboCdnError::Unsupported { .. } => "unsupported",
            TurboCdnError::Internal { .. } => "internal",
        }
    }
}

/// Result type alias for turbo-cdn operations
pub type Result<T> = std::result::Result<T, TurboCdnError>;

/// Error context for better error reporting
#[derive(Debug, Clone)]
pub struct ErrorContext {
    pub operation: String,
    pub source: Option<String>,
    pub file_path: Option<String>,
    pub url: Option<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl ErrorContext {
    pub fn new(operation: impl Into<String>) -> Self {
        Self {
            operation: operation.into(),
            source: None,
            file_path: None,
            url: None,
            timestamp: chrono::Utc::now(),
        }
    }

    pub fn with_source(mut self, source: impl Into<String>) -> Self {
        self.source = Some(source.into());
        self
    }

    pub fn with_file_path(mut self, path: impl Into<String>) -> Self {
        self.file_path = Some(path.into());
        self
    }

    pub fn with_url(mut self, url: impl Into<String>) -> Self {
        self.url = Some(url.into());
        self
    }
}

impl fmt::Display for ErrorContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Operation: {}", self.operation)?;
        if let Some(source) = &self.source {
            write!(f, ", Source: {}", source)?;
        }
        if let Some(path) = &self.file_path {
            write!(f, ", File: {}", path)?;
        }
        if let Some(url) = &self.url {
            write!(f, ", URL: {}", url)?;
        }
        write!(
            f,
            ", Time: {}",
            self.timestamp.format("%Y-%m-%d %H:%M:%S UTC")
        )
    }
}
