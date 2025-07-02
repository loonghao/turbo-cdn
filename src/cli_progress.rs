// Licensed under the MIT License
// Copyright (c) 2025 Hal <hal.long@outlook.com>

//! CLI progress display utilities using indicatif
//!
//! This module provides professional progress indicators for CLI usage.

use indicatif::{ProgressBar, ProgressStyle};
use std::io::{self, Write};
use std::time::Duration;

/// Create a spinner with message
pub fn create_spinner(message: &str) -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏")
            .template("{spinner:.blue} {msg}")
            .unwrap(),
    );
    pb.set_message(message.to_string());
    pb.enable_steady_tick(Duration::from_millis(100));
    pb
}

/// Create a progress bar for downloads
pub fn create_download_progress(total_size: u64, message: &str) -> ProgressBar {
    let pb = ProgressBar::new(total_size);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{msg} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})")
            .unwrap()
            .progress_chars("#>-")
    );
    pb.set_message(message.to_string());
    pb
}

/// Create a simple progress bar
pub fn create_progress_bar(total: u64, message: &str) -> ProgressBar {
    let pb = ProgressBar::new(total);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{msg} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})")
            .unwrap()
            .progress_chars("#>-"),
    );
    pb.set_message(message.to_string());
    pb
}

/// Show a quick spinner for async operations
pub async fn show_quick_spinner(message: &str) {
    let spinner = create_spinner(message);
    tokio::time::sleep(Duration::from_secs(1)).await;
    spinner.finish_and_clear();
}

/// Display a clean status message
pub fn status_message(icon: &str, message: &str) {
    println!("{} {}", icon, message);
}

/// Display an info message (only in verbose mode)
pub fn info_message(message: &str, verbose: bool) {
    if verbose {
        println!("ℹ️  {}", message);
    }
}

/// Display a success message
pub fn success_message(message: &str) {
    println!("✅ {message}");
}

/// Display an error message
pub fn error_message(message: &str) {
    println!("❌ {message}");
}

/// Display a warning message
pub fn warning_message(message: &str) {
    println!("⚠️  {message}");
}

/// Clear current line
pub fn clear_line() {
    print!("\r{:width$}\r", "", width = 80);
    io::stdout().flush().unwrap();
}

/// Display download result in a clean format
pub fn display_download_result(
    path: &std::path::Path,
    size_mb: f64,
    speed_mbps: f64,
    duration_secs: f64,
    verbose: bool,
) {
    success_message(&format!(
        "Downloaded {:.1} MB in {:.1}s ({:.1} MB/s)",
        size_mb, duration_secs, speed_mbps
    ));

    if verbose {
        info_message(&format!("Saved to: {}", path.display()), true);
    } else {
        println!("📁 {}", path.display());
    }
}

/// Display speed test results in a clean format
pub fn display_speed_test_results(
    results: &[(String, f64, bool)], // (method, speed_mbps, selected)
    verbose: bool,
) {
    if verbose {
        println!("🚀 Speed test results:");
        for (method, speed, selected) in results {
            let status = if *selected { " ✓ (selected)" } else { "" };
            if *speed > 0.0 {
                println!("   ├─ {method}: {speed:.2} MB/s{status}");
            } else {
                println!("   ├─ {method}: failed{status}");
            }
        }

        if let Some((selected_method, _, _)) = results.iter().find(|(_, _, selected)| *selected) {
            println!("   └─ Using {selected_method} method");
        }
        println!();
    } else {
        // Compact display for non-verbose mode
        if let Some((selected_method, speed, _)) = results.iter().find(|(_, _, selected)| *selected)
        {
            if *speed > 0.0 {
                status_message("⚡", &format!("Using {selected_method} ({speed:.1} MB/s)"));
            } else {
                status_message("⚡", &format!("Using {}", selected_method));
            }
        }
    }
}

/// Create a multi-progress container for multiple progress bars
pub fn create_multi_progress() -> indicatif::MultiProgress {
    indicatif::MultiProgress::new()
}

/// Finish a progress bar with success message
pub fn finish_progress_with_message(pb: &ProgressBar, message: &str) {
    pb.finish_with_message(format!("✅ {}", message));
}

/// Finish a progress bar with error message
pub fn finish_progress_with_error(pb: &ProgressBar, message: &str) {
    pb.finish_with_message(format!("❌ {}", message));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spinner_creation() {
        let spinner = create_spinner("Loading...");
        assert!(!spinner.is_finished());
    }

    #[test]
    fn test_progress_bar_creation() {
        let pb = create_progress_bar(100, "Testing");
        assert_eq!(pb.length(), Some(100));
    }

    #[test]
    fn test_download_progress_creation() {
        let pb = create_download_progress(1024 * 1024, "Downloading");
        assert_eq!(pb.length(), Some(1024 * 1024));
    }
}
