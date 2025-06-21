// Licensed under the MIT License
// Copyright (c) 2025 Hal <hal.long@outlook.com>

//! # Configuration System
//!
//! Robust, type-safe configuration management based on figment.

pub mod manager;
pub mod models;

use serde::{Deserialize, Serialize};

pub use manager::{ConfigBuilder, ConfigEvent, ConfigManager};
pub use models::*;

// Compatibility re-exports for existing code
pub use models::GlobalConfig as TurboCdnConfig;
pub use models::MirrorConfig as GitHubConfig;
pub use models::MirrorConfig as JsDelivrConfig;
pub use models::MirrorConfig as CloudflareConfig;
pub use models::MirrorConfig as FastlyConfig;
pub use models::SecurityConfig as ComplianceConfig;

/// Region enum for compatibility
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum Region {
    China,
    #[default]
    Global,
    AsiaPacific,
    Europe,
    NorthAmerica,
    Custom(String),
}

impl std::fmt::Display for Region {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Region::China => write!(f, "China"),
            Region::Global => write!(f, "Global"),
            Region::AsiaPacific => write!(f, "AsiaPacific"),
            Region::Europe => write!(f, "Europe"),
            Region::NorthAmerica => write!(f, "NorthAmerica"),
            Region::Custom(name) => write!(f, "{}", name),
        }
    }
}
