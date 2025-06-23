// Licensed under the MIT License
// Copyright (c) 2025 Hal <hal.long@outlook.com>

//! # Path Management Module
//!
//! Provides cross-platform path management for turbo-cdn, ensuring proper
//! directory usage according to platform standards and avoiding creation
//! of directories in inappropriate locations.

use directories::{ProjectDirs, UserDirs};
use std::path::{Path, PathBuf};
use tracing::{debug, warn};

use crate::error::{Result, TurboCdnError};

/// Cross-platform path manager for turbo-cdn
///
/// This manager ensures that all paths follow platform conventions using the
/// `directories` crate which provides better cross-platform support similar
/// to Python's platform_dirs:
/// - Uses standard system directories (config, cache, data, runtime, etc.)
/// - Avoids creating directories in current working directory
/// - Follows platform-specific conventions (XDG on Linux, Library on macOS, AppData on Windows)
/// - Provides consistent fallback behavior
/// - Handles path expansion and validation
#[derive(Debug, Clone)]
pub struct PathManager {
    /// Project directories for application-specific paths
    project_dirs: Option<ProjectDirs>,
    /// User directories for standard user paths
    user_dirs: Option<UserDirs>,
}

impl Default for PathManager {
    fn default() -> Self {
        Self::new("turbo-cdn")
    }
}

impl PathManager {
    /// Create a new path manager with the specified application name
    ///
    /// Uses reverse domain notation for better cross-platform compatibility:
    /// - Qualifier: "com" (organization domain)
    /// - Organization: "turbo-cdn" (project organization)
    /// - Application: app_name (specific application name)
    pub fn new(app_name: &str) -> Self {
        // Create project directories using reverse domain notation
        // This follows platform conventions better than simple app names
        let project_dirs = ProjectDirs::from("com", "turbo-cdn", app_name);
        let user_dirs = UserDirs::new();

        debug!(
            "PathManager initialized for app '{}' - project_dirs: {:?}",
            app_name,
            project_dirs
                .as_ref()
                .map(|pd| (pd.config_dir(), pd.cache_dir(), pd.data_dir()))
        );

        Self {
            project_dirs,
            user_dirs,
        }
    }

    /// Get the application's configuration directory
    ///
    /// Returns the standard configuration directory for the current platform:
    /// - Linux: `$XDG_CONFIG_HOME/turbo-cdn/app` or `~/.config/turbo-cdn/app`
    /// - macOS: `~/Library/Application Support/com.turbo-cdn.app`
    /// - Windows: `%APPDATA%\turbo-cdn\app\config`
    ///
    /// If the standard directory is not available, returns an error rather
    /// than falling back to inappropriate locations.
    pub fn config_dir(&self) -> Result<PathBuf> {
        self.project_dirs
            .as_ref()
            .map(|pd| pd.config_dir().to_path_buf())
            .ok_or_else(|| {
                TurboCdnError::config(
                    "Unable to determine configuration directory for this platform".to_string(),
                )
            })
    }

    /// Get the application's cache directory
    ///
    /// Returns the standard cache directory for the current platform:
    /// - Linux: `$XDG_CACHE_HOME/turbo-cdn/app` or `~/.cache/turbo-cdn/app`
    /// - macOS: `~/Library/Caches/com.turbo-cdn.app`
    /// - Windows: `%LOCALAPPDATA%\turbo-cdn\app\cache`
    pub fn cache_dir(&self) -> Result<PathBuf> {
        self.project_dirs
            .as_ref()
            .map(|pd| pd.cache_dir().to_path_buf())
            .ok_or_else(|| {
                TurboCdnError::config(
                    "Unable to determine cache directory for this platform".to_string(),
                )
            })
    }

    /// Get the application's data directory
    ///
    /// Returns the standard data directory for the current platform:
    /// - Linux: `$XDG_DATA_HOME/turbo-cdn/app` or `~/.local/share/turbo-cdn/app`
    /// - macOS: `~/Library/Application Support/com.turbo-cdn.app`
    /// - Windows: `%APPDATA%\turbo-cdn\app\data`
    pub fn data_dir(&self) -> Result<PathBuf> {
        self.project_dirs
            .as_ref()
            .map(|pd| pd.data_dir().to_path_buf())
            .ok_or_else(|| {
                TurboCdnError::config(
                    "Unable to determine data directory for this platform".to_string(),
                )
            })
    }

    /// Get the application's local data directory
    ///
    /// Returns the local data directory (for machine-specific data):
    /// - Linux: `$XDG_DATA_HOME/turbo-cdn/app` or `~/.local/share/turbo-cdn/app`
    /// - macOS: `~/Library/Application Support/com.turbo-cdn.app`
    /// - Windows: `%LOCALAPPDATA%\turbo-cdn\app\data`
    pub fn data_local_dir(&self) -> Result<PathBuf> {
        self.project_dirs
            .as_ref()
            .map(|pd| pd.data_local_dir().to_path_buf())
            .ok_or_else(|| {
                TurboCdnError::config(
                    "Unable to determine local data directory for this platform".to_string(),
                )
            })
    }

    /// Get the application's runtime directory
    ///
    /// Returns the runtime directory for temporary files:
    /// - Linux: `$XDG_RUNTIME_DIR/turbo-cdn/app` or `/tmp/turbo-cdn/app`
    /// - macOS: Same as cache directory
    /// - Windows: Same as cache directory
    pub fn runtime_dir(&self) -> Result<PathBuf> {
        if let Some(pd) = &self.project_dirs {
            if let Some(runtime_dir) = pd.runtime_dir() {
                return Ok(runtime_dir.to_path_buf());
            }
        }

        // Fallback to cache directory if runtime directory is not available
        self.cache_dir()
    }

    /// Get the user's home directory
    pub fn home_dir(&self) -> Result<PathBuf> {
        self.user_dirs
            .as_ref()
            .map(|ud| ud.home_dir().to_path_buf())
            .ok_or_else(|| {
                TurboCdnError::config(
                    "Unable to determine home directory for this platform".to_string(),
                )
            })
    }

    /// Get configuration file path with the given filename
    ///
    /// # Arguments
    /// * `filename` - The configuration file name (e.g., "config.toml")
    ///
    /// # Returns
    /// Full path to the configuration file
    pub fn config_file(&self, filename: &str) -> Result<PathBuf> {
        Ok(self.config_dir()?.join(filename))
    }

    /// Get cache file path with the given filename
    ///
    /// # Arguments
    /// * `filename` - The cache file name (e.g., "metadata.db")
    ///
    /// # Returns
    /// Full path to the cache file
    pub fn cache_file(&self, filename: &str) -> Result<PathBuf> {
        Ok(self.cache_dir()?.join(filename))
    }

    /// Get data file path with the given filename
    ///
    /// # Arguments
    /// * `filename` - The data file name (e.g., "performance.json")
    ///
    /// # Returns
    /// Full path to the data file
    pub fn data_file(&self, filename: &str) -> Result<PathBuf> {
        Ok(self.data_dir()?.join(filename))
    }

    /// Ensure a directory exists, creating it if necessary
    ///
    /// # Arguments
    /// * `path` - The directory path to create
    ///
    /// # Returns
    /// Result indicating success or failure
    pub async fn ensure_dir_exists<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let path = path.as_ref();
        if !path.exists() {
            debug!("Creating directory: {}", path.display());
            tokio::fs::create_dir_all(path).await.map_err(|e| {
                TurboCdnError::config(format!(
                    "Failed to create directory {}: {}",
                    path.display(),
                    e
                ))
            })?;
        }
        Ok(())
    }

    /// Discover configuration files in standard locations
    ///
    /// Returns a list of configuration file paths in order of precedence:
    /// 1. Project-level configuration files (current directory)
    /// 2. User configuration files (user config directory)
    /// 3. System-wide configuration files (system config directories)
    ///
    /// Only returns paths to files that actually exist.
    pub fn discover_config_files(&self) -> Vec<PathBuf> {
        let mut files = Vec::new();

        // Project-level configuration files (highest precedence)
        let project_files = ["turbo-cdn.toml", ".turbo-cdn.toml", "config/default.toml"];
        for filename in &project_files {
            let path = PathBuf::from(filename);
            if path.exists() {
                debug!("Found project config: {}", path.display());
                files.push(path);
            }
        }

        // User configuration files
        if let Ok(config_dir) = self.config_dir() {
            let user_files = ["config.toml", "turbo-cdn.toml"];
            for filename in &user_files {
                let path = config_dir.join(filename);
                if path.exists() {
                    debug!("Found user config: {}", path.display());
                    files.push(path);
                }
            }
        }

        // System-wide configuration files (lowest precedence)
        let system_paths = [
            "/etc/turbo-cdn/config.toml",
            "/usr/local/etc/turbo-cdn/config.toml",
        ];
        for path_str in &system_paths {
            let path = PathBuf::from(path_str);
            if path.exists() {
                debug!("Found system config: {}", path.display());
                files.push(path);
            }
        }

        if files.is_empty() {
            warn!("No configuration files found in standard locations");
        }

        files
    }

    /// Expand a path string that may contain environment variables or tildes
    ///
    /// # Arguments
    /// * `path_str` - Path string that may contain ~ or environment variables
    ///
    /// # Returns
    /// Expanded PathBuf or error if expansion fails
    pub fn expand_path(&self, path_str: &str) -> Result<PathBuf> {
        let path_str = path_str.trim();

        // Handle tilde expansion
        if let Some(stripped) = path_str.strip_prefix("~/") {
            let home = self.home_dir()?;
            return Ok(home.join(stripped));
        }

        // For now, just return the path as-is
        // In the future, we could add environment variable expansion
        Ok(PathBuf::from(path_str))
    }

    /// Validate that a path is safe to use (not in inappropriate locations)
    ///
    /// # Arguments
    /// * `path` - The path to validate
    ///
    /// # Returns
    /// Result indicating if the path is safe to use
    pub fn validate_path<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let path = path.as_ref();

        // Check if path is in current directory (usually not desired for app data)
        if let Ok(current_dir) = std::env::current_dir() {
            if path.starts_with(&current_dir) && path != current_dir {
                warn!(
                    "Path {} is in current directory, this may not be desired",
                    path.display()
                );
            }
        }

        // Check if path is absolute (recommended for app data)
        if !path.is_absolute() {
            warn!(
                "Path {} is relative, consider using absolute paths for app data",
                path.display()
            );
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_path_manager_creation() {
        let manager = PathManager::new("test-app");
        // Should have project directories initialized
        assert!(manager.project_dirs.is_some());
    }

    #[test]
    fn test_config_dir() {
        let manager = PathManager::new("test-app");
        if let Ok(config_dir) = manager.config_dir() {
            assert!(config_dir.to_string_lossy().contains("test-app"));
            // Ensure it's an absolute path
            assert!(config_dir.is_absolute());
        }
    }

    #[test]
    fn test_cache_dir() {
        let manager = PathManager::new("test-app");
        if let Ok(cache_dir) = manager.cache_dir() {
            assert!(cache_dir.to_string_lossy().contains("test-app"));
            assert!(cache_dir.is_absolute());
        }
    }

    #[test]
    fn test_data_dir() {
        let manager = PathManager::new("test-app");
        if let Ok(data_dir) = manager.data_dir() {
            assert!(data_dir.to_string_lossy().contains("test-app"));
            assert!(data_dir.is_absolute());
        }
    }

    #[test]
    fn test_expand_path() {
        let manager = PathManager::new("test-app");

        // Test regular absolute path
        let result = manager.expand_path("/tmp/test").unwrap();
        assert_eq!(result, PathBuf::from("/tmp/test"));

        // Test tilde expansion (if home directory is available)
        if manager.home_dir().is_ok() {
            let result = manager.expand_path("~/test");
            assert!(result.is_ok());
            if let Ok(expanded) = result {
                assert!(expanded.is_absolute());
                assert!(!expanded.to_string_lossy().starts_with("~"));
            }
        }
    }

    #[test]
    fn test_config_file_paths() {
        let manager = PathManager::new("test-app");

        if let Ok(config_file) = manager.config_file("config.toml") {
            assert!(config_file.to_string_lossy().contains("test-app"));
            assert!(config_file.to_string_lossy().ends_with("config.toml"));
            assert!(config_file.is_absolute());
        }
    }

    #[test]
    fn test_cache_file_paths() {
        let manager = PathManager::new("test-app");

        if let Ok(cache_file) = manager.cache_file("metadata.db") {
            assert!(cache_file.to_string_lossy().contains("test-app"));
            assert!(cache_file.to_string_lossy().ends_with("metadata.db"));
            assert!(cache_file.is_absolute());
        }
    }

    #[test]
    fn test_data_file_paths() {
        let manager = PathManager::new("test-app");

        if let Ok(data_file) = manager.data_file("performance.json") {
            assert!(data_file.to_string_lossy().contains("test-app"));
            assert!(data_file.to_string_lossy().ends_with("performance.json"));
            assert!(data_file.is_absolute());
        }
    }

    #[test]
    fn test_discover_config_files() {
        let manager = PathManager::new("test-app");
        let files = manager.discover_config_files();

        // Should return a list (may be empty if no config files exist)
        // All returned paths should exist
        for file in files {
            assert!(file.exists());
        }
    }

    #[test]
    fn test_validate_path() {
        let manager = PathManager::new("test-app");

        // Test absolute path validation
        let abs_path = PathBuf::from("/tmp/test");
        assert!(manager.validate_path(&abs_path).is_ok());

        // Test relative path validation (should warn but not error)
        let rel_path = PathBuf::from("relative/path");
        assert!(manager.validate_path(&rel_path).is_ok());
    }

    #[tokio::test]
    async fn test_ensure_dir_exists() {
        let manager = PathManager::new("test-app");

        // Create a temporary directory for testing
        let temp_dir = env::temp_dir().join("turbo-cdn-test");

        // Ensure it doesn't exist first
        let _ = std::fs::remove_dir_all(&temp_dir);

        // Test directory creation
        assert!(manager.ensure_dir_exists(&temp_dir).await.is_ok());
        assert!(temp_dir.exists());
        assert!(temp_dir.is_dir());

        // Clean up
        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_cross_platform_paths() {
        let manager = PathManager::new("test-app");

        // Test that paths work on current platform
        if cfg!(windows) {
            // On Windows, paths should use backslashes and include drive letters
            if let Ok(config_dir) = manager.config_dir() {
                let path_str = config_dir.to_string_lossy();
                // Should be absolute path on Windows
                assert!(path_str.len() > 2);
            }
        } else {
            // On Unix-like systems, paths should start with /
            if let Ok(config_dir) = manager.config_dir() {
                let path_str = config_dir.to_string_lossy();
                assert!(path_str.starts_with('/'));
            }
        }
    }

    #[test]
    fn test_no_current_directory_pollution() {
        let manager = PathManager::new("test-app");
        let current_dir = env::current_dir().unwrap();

        // Test that our paths don't pollute current directory
        if let Ok(config_dir) = manager.config_dir() {
            assert!(!config_dir.starts_with(&current_dir));
        }

        if let Ok(cache_dir) = manager.cache_dir() {
            assert!(!cache_dir.starts_with(&current_dir));
        }

        if let Ok(data_dir) = manager.data_dir() {
            assert!(!data_dir.starts_with(&current_dir));
        }
    }

    #[test]
    fn test_additional_directories() {
        let manager = PathManager::new("test-app");

        // Test local data directory
        if let Ok(data_local_dir) = manager.data_local_dir() {
            assert!(data_local_dir.to_string_lossy().contains("test-app"));
            assert!(data_local_dir.is_absolute());
        }

        // Test runtime directory
        if let Ok(runtime_dir) = manager.runtime_dir() {
            assert!(runtime_dir.is_absolute());
        }

        // Test home directory
        if let Ok(home_dir) = manager.home_dir() {
            assert!(home_dir.is_absolute());
        }
    }

    #[test]
    fn test_platform_specific_paths() {
        let manager = PathManager::new("test-app");

        // Test that paths follow platform conventions
        if let Ok(config_dir) = manager.config_dir() {
            let path_str = config_dir.to_string_lossy();

            if cfg!(target_os = "windows") {
                // Windows should use AppData
                assert!(path_str.contains("AppData") || path_str.contains("turbo-cdn"));
            } else if cfg!(target_os = "macos") {
                // macOS should use Library/Application Support
                assert!(path_str.contains("Library") || path_str.contains("Application Support"));
            } else {
                // Linux/Unix should use .config or XDG
                assert!(path_str.contains(".config") || path_str.contains("turbo-cdn"));
            }
        }
    }

    #[test]
    fn test_reverse_domain_notation() {
        let manager = PathManager::new("test-app");

        // Test that paths include proper organization structure
        if let Ok(config_dir) = manager.config_dir() {
            let path_str = config_dir.to_string_lossy();
            // Should contain either the app name or organization structure
            assert!(path_str.contains("test-app") || path_str.contains("turbo-cdn"));
        }
    }
}
