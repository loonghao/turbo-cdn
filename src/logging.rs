// Licensed under the MIT License
// Copyright (c) 2025 Hal <hal.long@outlook.com>

//! Professional logging configuration for turbo-cdn
//!
//! This module provides structured logging with different levels for CLI and API usage.

use std::io;
use tracing_subscriber::{
    fmt::{self, format::FmtSpan},
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter,
};

/// Logging mode for different usage contexts
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoggingMode {
    /// CLI mode - user-friendly output
    Cli,
    /// API mode - structured logging for applications
    Api,
    /// Debug mode - verbose logging for development
    Debug,
    /// Silent mode - minimal logging
    Silent,
}

/// Logging configuration
#[derive(Debug, Clone)]
pub struct LoggingConfig {
    pub mode: LoggingMode,
    pub level: String,
    pub show_target: bool,
    pub show_thread_ids: bool,
    pub show_file_line: bool,
    pub use_ansi_colors: bool,
    pub log_to_file: Option<String>,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            mode: LoggingMode::Api,
            level: "warn".to_string(),
            show_target: false,
            show_thread_ids: false,
            show_file_line: false,
            use_ansi_colors: true,
            log_to_file: None,
        }
    }
}

impl LoggingConfig {
    /// Create CLI logging configuration
    pub fn cli(verbose: bool) -> Self {
        Self {
            mode: LoggingMode::Cli,
            level: if verbose {
                "info".to_string()
            } else {
                "warn".to_string()
            },
            show_target: verbose,
            show_thread_ids: false,
            show_file_line: verbose,
            use_ansi_colors: true,
            log_to_file: None,
        }
    }

    /// Create API logging configuration
    pub fn api() -> Self {
        Self {
            mode: LoggingMode::Api,
            level: "warn".to_string(),
            show_target: false,
            show_thread_ids: false,
            show_file_line: false,
            use_ansi_colors: false,
            log_to_file: None,
        }
    }

    /// Create debug logging configuration
    pub fn debug() -> Self {
        Self {
            mode: LoggingMode::Debug,
            level: "debug".to_string(),
            show_target: true,
            show_thread_ids: true,
            show_file_line: true,
            use_ansi_colors: true,
            log_to_file: Some("turbo-cdn-debug.log".to_string()),
        }
    }

    /// Create silent logging configuration
    pub fn silent() -> Self {
        Self {
            mode: LoggingMode::Silent,
            level: "error".to_string(),
            show_target: false,
            show_thread_ids: false,
            show_file_line: false,
            use_ansi_colors: false,
            log_to_file: None,
        }
    }
}

/// Initialize logging with the given configuration
pub fn init_logging(config: LoggingConfig) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Create environment filter
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(format!("turbo_cdn={}", config.level)));

    let registry = tracing_subscriber::registry().with(env_filter);

    match config.mode {
        LoggingMode::Cli => {
            // CLI mode: clean, user-friendly output
            let fmt_layer = fmt::layer()
                .with_target(config.show_target)
                .with_thread_ids(config.show_thread_ids)
                .with_file(config.show_file_line)
                .with_line_number(config.show_file_line)
                .with_ansi(config.use_ansi_colors)
                .with_span_events(FmtSpan::NONE)
                .compact();

            registry.with(fmt_layer).init();
        }
        LoggingMode::Api => {
            // API mode: structured, machine-readable output
            let fmt_layer = fmt::layer()
                .with_target(false)
                .with_thread_ids(false)
                .with_file(false)
                .with_line_number(false)
                .with_ansi(false)
                .with_span_events(FmtSpan::NONE)
                .compact()
                .with_writer(io::stderr);

            registry.with(fmt_layer).init();
        }
        LoggingMode::Debug => {
            // Debug mode: verbose output with optional file logging
            let fmt_layer = fmt::layer()
                .with_target(true)
                .with_thread_ids(true)
                .with_file(true)
                .with_line_number(true)
                .with_ansi(config.use_ansi_colors)
                .with_span_events(FmtSpan::ENTER | FmtSpan::CLOSE)
                .pretty();

            if let Some(log_file) = config.log_to_file {
                let file_appender = tracing_appender::rolling::daily("./logs", log_file);
                let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

                let file_layer = fmt::layer()
                    .with_target(true)
                    .with_thread_ids(true)
                    .with_file(true)
                    .with_line_number(true)
                    .with_ansi(false)
                    .with_writer(non_blocking)
                    .json();

                registry.with(fmt_layer).with(file_layer).init();
            } else {
                registry.with(fmt_layer).init();
            }
        }
        LoggingMode::Silent => {
            // Silent mode: minimal output
            let fmt_layer = fmt::layer()
                .with_target(false)
                .with_thread_ids(false)
                .with_file(false)
                .with_line_number(false)
                .with_ansi(false)
                .with_span_events(FmtSpan::NONE)
                .compact()
                .with_writer(io::stderr);

            registry.with(fmt_layer).init();
        }
    }

    Ok(())
}

/// Initialize CLI logging (convenience function)
pub fn init_cli_logging(verbose: bool) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    init_logging(LoggingConfig::cli(verbose))
}

/// Initialize API logging (convenience function)
pub fn init_api_logging() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    init_logging(LoggingConfig::api())
}

/// Initialize debug logging (convenience function)
pub fn init_debug_logging() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    init_logging(LoggingConfig::debug())
}

/// Initialize silent logging (convenience function)
pub fn init_silent_logging() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    init_logging(LoggingConfig::silent())
}

/// Logging macros for different contexts
#[macro_export]
macro_rules! cli_info {
    ($($arg:tt)*) => {
        if tracing::enabled!(tracing::Level::INFO) {
            tracing::info!($($arg)*);
        }
    };
}

#[macro_export]
macro_rules! cli_warn {
    ($($arg:tt)*) => {
        tracing::warn!($($arg)*);
    };
}

#[macro_export]
macro_rules! cli_error {
    ($($arg:tt)*) => {
        tracing::error!($($arg)*);
    };
}

#[macro_export]
macro_rules! api_debug {
    ($($arg:tt)*) => {
        if tracing::enabled!(tracing::Level::DEBUG) {
            tracing::debug!($($arg)*);
        }
    };
}

#[macro_export]
macro_rules! api_info {
    ($($arg:tt)*) => {
        if tracing::enabled!(tracing::Level::INFO) {
            tracing::info!($($arg)*);
        }
    };
}

/// Check if we're in verbose mode
pub fn is_verbose() -> bool {
    tracing::enabled!(tracing::Level::INFO)
}

/// Check if we're in debug mode
pub fn is_debug() -> bool {
    tracing::enabled!(tracing::Level::DEBUG)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logging_config_creation() {
        let cli_config = LoggingConfig::cli(true);
        assert_eq!(cli_config.mode, LoggingMode::Cli);
        assert_eq!(cli_config.level, "info");
        assert!(cli_config.show_target);

        let api_config = LoggingConfig::api();
        assert_eq!(api_config.mode, LoggingMode::Api);
        assert_eq!(api_config.level, "warn");
        assert!(!api_config.show_target);
    }

    #[test]
    fn test_debug_config() {
        let debug_config = LoggingConfig::debug();
        assert_eq!(debug_config.mode, LoggingMode::Debug);
        assert_eq!(debug_config.level, "debug");
        assert!(debug_config.show_target);
        assert!(debug_config.show_thread_ids);
        assert!(debug_config.show_file_line);
    }
}
