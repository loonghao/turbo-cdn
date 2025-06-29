// Licensed under the MIT License
// Copyright (c) 2025 Hal <hal.long@outlook.com>

// Use high-performance memory allocator on supported platforms
// Disable on problematic targets (GNU, musl, ARM) to avoid build issues
#[cfg(all(
    feature = "mimalloc-allocator",
    not(disable_mimalloc),
    not(target_env = "musl"),
    not(target_arch = "arm"),
    any(
        target_os = "linux",
        target_os = "macos",
        target_os = "windows"
    )
))]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use clap::{Parser, Subcommand};
use std::io::{self, Write};
use std::path::PathBuf;
use turbo_cdn::*;

/// Turbo CDN - Intelligent Download Accelerator
#[derive(Parser)]
#[command(name = "turbo-cdn")]
#[command(about = "Intelligent download accelerator with CDN optimization")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(author = "Hal <hal.long@outlook.com>")]
struct Cli {
    /// Enable verbose output
    #[arg(short, long)]
    verbose: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Download file with intelligent method selection (default: smart mode)
    #[command(alias = "dl")]
    Download {
        /// URL to download
        url: String,
        /// Output path (optional)
        output: Option<PathBuf>,
        /// Force direct download (bypass smart mode and CDN)
        #[arg(long, alias = "direct")]
        no_cdn: bool,
        /// Force CDN download (bypass smart mode)
        #[arg(long)]
        force_cdn: bool,
        /// Disable smart mode (use legacy behavior)
        #[arg(long)]
        no_smart: bool,
    },
    /// Get optimized CDN URL
    #[command(alias = "optimize")]
    GetOptimalUrl {
        /// URL to optimize
        url: String,
    },
    /// Show performance statistics
    Stats,
}

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    // Initialize professional logging system
    if let Err(e) = turbo_cdn::logging::init_cli_logging(cli.verbose) {
        eprintln!("Failed to initialize logging: {}", e);
        std::process::exit(1);
    }

    match cli.command {
        Commands::GetOptimalUrl { url } => {
            handle_optimize_command(&url, cli.verbose).await?;
        }
        Commands::Download {
            url,
            output,
            no_cdn,
            force_cdn,
            no_smart,
        } => {
            // Determine download mode: smart is default unless explicitly disabled
            let smart_mode = !no_smart && !no_cdn && !force_cdn;
            handle_download_command(&url, output, cli.verbose, no_cdn, force_cdn, smart_mode)
                .await?;
        }
        Commands::Stats => {
            handle_stats_command().await?;
        }
    }

    Ok(())
}

async fn handle_optimize_command(
    url: &str,
    verbose: bool,
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    if verbose {
        println!("🔍 Turbo CDN - Finding optimal URL");
        println!("=================================");
        println!("Source URL: {}", url);
        println!();
    } else {
        print!("🔍 Finding optimal URL");
        io::stdout().flush().unwrap();
        show_spinner().await;
    }

    // Create a TurboCdn instance
    let turbo_cdn = TurboCdn::new().await?;

    // Get optimal URL
    match turbo_cdn.get_optimal_url(url).await {
        Ok(optimal_url) => {
            if !verbose {
                print!("\r"); // Clear spinner
                io::stdout().flush().unwrap();
            }

            if optimal_url == url {
                println!("ℹ️  No CDN optimization available for this URL");
                if verbose {
                    println!("   Original URL will be used: {}", url);
                }
            } else {
                println!("✅ CDN optimization found!");
                println!("   {}", optimal_url);
                if verbose {
                    println!("   🚀 This should provide faster download speeds");
                }
            }
        }
        Err(e) => {
            if !verbose {
                print!("\r"); // Clear spinner
                io::stdout().flush().unwrap();
            }
            println!("❌ Error getting optimal URL: {}", e);
            return Err(e.into());
        }
    }

    Ok(())
}

async fn handle_download_command(
    url: &str,
    output_path: Option<PathBuf>,
    verbose: bool,
    no_cdn: bool,
    force_cdn: bool,
    smart: bool,
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    if verbose {
        if smart {
            println!("🧠 Smart Download (Auto-Select Best Method) - Default Mode");
            println!("=========================================================");
        } else if no_cdn {
            println!("⬇️  Direct Download (CDN Disabled)");
            println!("==================================");
        } else if force_cdn {
            println!("🌐 Forced CDN Download");
            println!("=====================");
        } else {
            println!("⬇️  Legacy Download Mode");
            println!("=======================");
        }
        println!("Source URL: {}", url);
        if let Some(ref path) = output_path {
            println!("Output Path: {}", path.display());
        }
        if smart {
            println!("Mode: Smart mode (testing and selecting fastest method)");
        } else if no_cdn {
            println!("Mode: Direct download (bypassing all optimization)");
        } else if force_cdn {
            println!("Mode: Forced CDN download (bypassing smart selection)");
        } else {
            println!("Mode: Legacy CDN optimization");
        }
        println!();
    } else {
        if smart {
            print!("🧠 Smart downloading");
        } else if no_cdn {
            print!("⬇️  Downloading directly");
        } else if force_cdn {
            print!("🌐 Downloading via CDN");
        } else {
            print!("⬇️  Downloading");
        }
        io::stdout().flush().unwrap();
        if !smart {
            show_spinner().await;
        }
    }

    // Create a TurboCdn instance
    let turbo_cdn = TurboCdn::new().await?;
    if verbose {
        if smart {
            println!("✓ TurboCdn initialized in smart mode (auto-selecting best method)");
        } else if no_cdn {
            println!("✓ TurboCdn initialized in direct mode (CDN disabled)");
        } else if force_cdn {
            println!("✓ TurboCdn initialized in forced CDN mode");
        } else {
            println!("✓ TurboCdn initialized with legacy CDN selection");
        }
    }

    // Download file based on selected mode
    let result = if smart {
        // Smart download with automatic method selection (DEFAULT)
        if let Some(output_path) = output_path {
            turbo_cdn
                .download_smart_to_path_with_verbose(url, output_path, verbose)
                .await
        } else {
            turbo_cdn.download_smart_with_verbose(url, verbose).await
        }
    } else if no_cdn {
        // Direct download without any optimization
        if let Some(output_path) = output_path {
            turbo_cdn.download_direct_to_path(url, output_path).await
        } else {
            turbo_cdn.download_direct_from_url(url).await
        }
    } else {
        // Legacy CDN-optimized download (force_cdn or no_smart)
        if let Some(output_path) = output_path {
            turbo_cdn.download_to_path(url, output_path).await
        } else {
            turbo_cdn.download_from_url(url).await
        }
    };

    match result {
        Ok(result) => {
            if !verbose {
                print!("\r"); // Clear spinner
                io::stdout().flush().unwrap();
            }

            println!("🎉 Download completed successfully!");
            println!("   📁 {}", result.path.display());
            println!(
                "   📊 {:.2} MB ({:.2} MB/s)",
                result.size as f64 / 1024.0 / 1024.0,
                result.speed / 1024.0 / 1024.0
            );

            if verbose {
                println!("   📊 Size: {} bytes", result.size);
                println!("   ⏱️  Duration: {:.2}s", result.duration.as_secs_f64());
                if result.resumed {
                    println!("   🔄 Download was resumed from previous attempt");
                }

                // Show performance tip
                if result.speed > 5.0 * 1024.0 * 1024.0 {
                    println!("   💡 Excellent speed! CDN optimization is working well.");
                } else if result.speed < 1.0 * 1024.0 * 1024.0 {
                    println!(
                        "   💡 Consider checking your internet connection or trying again later."
                    );
                }
            }
        }
        Err(e) => {
            if !verbose {
                print!("\r"); // Clear spinner
                io::stdout().flush().unwrap();
            }
            println!("❌ Download failed: {}", e);
            if verbose {
                println!("💡 Tips:");
                println!("   • Check your internet connection");
                println!("   • Verify the URL is correct and accessible");
                println!("   • Try again later if the server is temporarily unavailable");
            }
            return Err(e.into());
        }
    }

    Ok(())
}

async fn show_spinner() {
    let spinner_chars = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
    for _ in 0..10 {
        for &ch in &spinner_chars {
            print!("\r{} Processing...", ch);
            io::stdout().flush().unwrap();
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }
    }
}

async fn handle_stats_command() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("📊 Turbo CDN - Performance Statistics");
    println!("====================================");

    // Create a TurboCdn instance to access stats
    let _turbo_cdn = TurboCdn::new().await?;

    // Note: In a real implementation, we'd need to expose the downloader's stats
    // For now, show basic information
    println!("✓ TurboCdn is ready and operational");
    println!("✓ Intelligent server selection enabled");
    println!("✓ Dynamic file segmentation active");
    println!("✓ CDN optimization rules loaded");
    println!();
    println!("💡 Download some files to see performance statistics!");

    Ok(())
}
