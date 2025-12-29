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

    /// HTTP status code errors (non-retryable client errors like 404)
    #[error("HTTP {status_code}: {message}")]
    HttpStatus {
        status_code: u16,
        message: String,
        url: String,
    },

    /// Server errors (5xx, potentially retryable)
    #[error("Server error {status_code}: {message}")]
    ServerError {
        status_code: u16,
        message: String,
        url: String,
    },

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

    /// Create a new HTTP status error (for 4xx client errors)
    pub fn http_status<S: Into<String>>(status_code: u16, message: S, url: S) -> Self {
        Self::HttpStatus {
            status_code,
            message: message.into(),
            url: url.into(),
        }
    }

    /// Create a new server error (for 5xx errors)
    pub fn server_error<S: Into<String>>(status_code: u16, message: S, url: S) -> Self {
        Self::ServerError {
            status_code,
            message: message.into(),
            url: url.into(),
        }
    }

    /// Create an error from HTTP status code
    pub fn from_status_code<S: Into<String>>(status_code: u16, url: S) -> Self {
        let url_str = url.into();
        let message = match status_code {
            400 => "Bad Request",
            401 => "Unauthorized",
            403 => "Forbidden",
            404 => "Not Found",
            405 => "Method Not Allowed",
            408 => "Request Timeout",
            429 => "Too Many Requests",
            500 => "Internal Server Error",
            502 => "Bad Gateway",
            503 => "Service Unavailable",
            504 => "Gateway Timeout",
            _ => "Unknown Error",
        };

        if status_code >= 500 {
            Self::ServerError {
                status_code,
                message: message.to_string(),
                url: url_str,
            }
        } else if status_code == 429 {
            Self::RateLimit {
                message: format!("Rate limited by {url_str}"),
            }
        } else {
            Self::HttpStatus {
                status_code,
                message: message.to_string(),
                url: url_str,
            }
        }
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
        match self {
            // Network errors are generally retryable
            TurboCdnError::Network(_) => true,
            // Timeouts are retryable
            TurboCdnError::Timeout { .. } => true,
            // Rate limits should be retried after delay
            TurboCdnError::RateLimit { .. } => true,
            // IO errors might be transient
            TurboCdnError::Io(_) => true,
            // Server errors (5xx) are retryable
            TurboCdnError::ServerError { .. } => true,
            // HTTP client errors (4xx) are NOT retryable - the resource doesn't exist
            TurboCdnError::HttpStatus { .. } => false,
            // All other errors are not retryable
            _ => false,
        }
    }

    /// Check if this error indicates the resource doesn't exist on this server
    /// and we should immediately try the next mirror
    pub fn should_try_next_mirror(&self) -> bool {
        match self {
            // 404 Not Found - definitely try next mirror
            TurboCdnError::HttpStatus { status_code, .. } => *status_code == 404,
            // Server errors might be temporary, but try next mirror anyway
            TurboCdnError::ServerError { .. } => true,
            // Network errors - try next mirror
            TurboCdnError::Network(_) => true,
            // Timeout - try next mirror
            TurboCdnError::Timeout { .. } => true,
            _ => false,
        }
    }

    /// Get HTTP status code if this is an HTTP error
    pub fn status_code(&self) -> Option<u16> {
        match self {
            TurboCdnError::HttpStatus { status_code, .. } => Some(*status_code),
            TurboCdnError::ServerError { status_code, .. } => Some(*status_code),
            _ => None,
        }
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
            TurboCdnError::HttpStatus { .. } => "http_status",
            TurboCdnError::ServerError { .. } => "server_error",
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
        let operation = &self.operation;
        write!(f, "Operation: {operation}")?;
        if let Some(source) = &self.source {
            write!(f, ", Source: {source}")?;
        }
        if let Some(path) = &self.file_path {
            write!(f, ", File: {path}")?;
        }
        if let Some(url) = &self.url {
            write!(f, ", URL: {url}")?;
        }
        let timestamp = self.timestamp.format("%Y-%m-%d %H:%M:%S UTC");
        write!(f, ", Time: {timestamp}")
    }
}
