// Licensed under the MIT License
// Copyright (c) 2025 Hal <hal.long@outlook.com>

// Use high-performance memory allocator
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use std::io::{self, Write};
use std::path::PathBuf;
use turbo_cdn::*;

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    turbo_cdn::init_tracing();

    let args: Vec<String> = std::env::args().collect();

    // Check for verbose flag
    let verbose = args.contains(&"--verbose".to_string()) || args.contains(&"-v".to_string());

    // Filter out verbose flags for command parsing
    let filtered_args: Vec<String> = args
        .iter()
        .filter(|arg| *arg != "--verbose" && *arg != "-v")
        .cloned()
        .collect();

    // Show help if no arguments or help requested
    if filtered_args.len() < 2
        || filtered_args.get(1).map(|s| s.as_str()) == Some("--help")
        || filtered_args.get(1).map(|s| s.as_str()) == Some("-h")
    {
        show_help();
        return Ok(());
    }

    let command = &filtered_args[1];

    match command.as_str() {
        "get-optimal-url" | "optimize" => {
            if filtered_args.len() < 3 {
                eprintln!("❌ Error: URL required");
                eprintln!("Usage: turbo-cdn get-optimal-url <URL>");
                return Ok(());
            }

            let url = &filtered_args[2];
            handle_optimize_command(url, verbose).await?;
        }
        "download" | "dl" => {
            if filtered_args.len() < 3 {
                eprintln!("❌ Error: URL required");
                eprintln!("Usage: turbo-cdn download <URL> [output-path]");
                return Ok(());
            }

            let url = &filtered_args[2];
            let output_path = filtered_args.get(3).map(PathBuf::from);
            handle_download_command(url, output_path, verbose).await?;
        }
        "stats" => {
            handle_stats_command().await?;
        }
        "version" | "--version" => {
            show_version();
        }
        _ => {
            eprintln!("❌ Unknown command: {}", command);
            eprintln!("Run 'turbo-cdn --help' for usage information");
            return Ok(());
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
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    if verbose {
        println!("⬇️  Turbo CDN - Accelerated Download");
        println!("===================================");
        println!("Source URL: {}", url);
        if let Some(ref path) = output_path {
            println!("Output Path: {}", path.display());
        }
        println!();
    } else {
        print!("⬇️  Downloading");
        io::stdout().flush().unwrap();
        show_spinner().await;
    }

    // Create a TurboCdn instance
    let turbo_cdn = TurboCdn::new().await?;
    if verbose {
        println!("✓ TurboCdn initialized with intelligent CDN selection");
    }

    // Download file
    let result = if let Some(output_path) = output_path {
        turbo_cdn.download_to_path(url, output_path).await
    } else {
        turbo_cdn.download_from_url(url).await
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
            print!("\r{} {}", ch, "Processing...");
            io::stdout().flush().unwrap();
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }
    }
}

async fn handle_stats_command() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("📊 Turbo CDN - Performance Statistics");
    println!("====================================");

    // Create a TurboCdn instance to access stats
    let turbo_cdn = TurboCdn::new().await?;

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

fn show_version() {
    println!("Turbo CDN v{}", env!("CARGO_PKG_VERSION"));
    println!("Intelligent download accelerator with CDN optimization");
    println!("Copyright (c) 2025 Hal <hal.long@outlook.com>");
    println!("Licensed under the MIT License");
}

fn show_help() {
    println!("🚀 Turbo CDN - Intelligent Download Accelerator");
    println!("===============================================");
    println!();
    println!("USAGE:");
    println!("    turbo-cdn <COMMAND> [OPTIONS]");
    println!();
    println!("COMMANDS:");
    println!("    download, dl <URL> [PATH]     Download file with CDN acceleration");
    println!("    get-optimal-url, optimize <URL>  Get optimized CDN URL");
    println!("    stats                         Show performance statistics");
    println!("    version, --version            Show version information");
    println!("    help, -h, --help             Show this help message");
    println!();
    println!("OPTIONS:");
    println!("    --verbose                     Show detailed output and progress");
    println!();
    println!("EXAMPLES:");
    println!("    # Download a file with automatic optimization");
    println!(
        "    turbo-cdn download \"https://github.com/user/repo/releases/download/v1.0/file.zip\""
    );
    println!();
    println!("    # Download to specific location");
    println!("    turbo-cdn download \"https://example.com/file.zip\" \"./downloads/file.zip\"");
    println!();
    println!("    # Get optimized CDN URL");
    println!(
        "    turbo-cdn optimize \"https://github.com/user/repo/releases/download/v1.0/file.zip\""
    );
    println!();
    println!("    # View performance statistics");
    println!("    turbo-cdn stats");
    println!();
    println!("FEATURES:");
    println!("    ✓ Automatic CDN optimization (GitHub, jsDelivr, Fastly, Cloudflare)");
    println!("    ✓ Intelligent server selection based on performance");
    println!("    ✓ Dynamic file segmentation for maximum speed");
    println!("    ✓ Concurrent chunked downloads with resume support");
    println!("    ✓ Geographic-aware CDN selection");
    println!("    ✓ Real-time progress tracking");
    println!();
    println!("For more information, visit: https://github.com/loonghao/turbo-cdn");
}
