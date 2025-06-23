// Licensed under the MIT License
// Copyright (c) 2025 Hal <hal.long@outlook.com>

use turbo_cdn::*;

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    turbo_cdn::init_tracing();

    let args: Vec<String> = std::env::args().collect();

    if args.len() > 2 && args[1] == "get-optimal-url" {
        // Handle get-optimal-url command
        let url = &args[2];

        println!("Turbo CDN - Getting optimal URL for: {}", url);
        println!("==============================================");

        // Create a TurboCdn instance
        let turbo_cdn = TurboCdn::builder().build().await?;

        println!("✓ TurboCdn initialized successfully");

        // Get optimal URL
        match turbo_cdn.get_optimal_url(url).await {
            Ok(optimal_url) => {
                println!("✅ Optimal URL found: {}", optimal_url);
            }
            Err(e) => {
                println!("❌ Error getting optimal URL: {}", e);
                return Err(e.into());
            }
        }

        return Ok(());
    }

    println!("Turbo CDN - Revolutionary Download Accelerator");
    println!("==============================================");

    // Create a TurboCdn instance
    let turbo_cdn = TurboCdn::builder()
        .with_sources(&[
            Source::github(),
            Source::jsdelivr(),
            Source::fastly(),
            Source::cloudflare(),
        ])
        .with_region(Region::Global)
        .with_cache(true)
        .build()
        .await?;

    println!("✓ TurboCdn initialized successfully");

    // Perform health check
    let health_status = turbo_cdn.health_check().await?;
    println!(
        "✓ Health check completed: {} sources checked",
        health_status.len()
    );

    // Get statistics
    let stats = turbo_cdn.get_stats().await?;
    println!(
        "✓ Statistics retrieved: {} total downloads",
        stats.total_downloads
    );

    println!("\nTurboCdn is ready for downloads!");
    println!("Example usage:");
    println!("  turbo-cdn get-optimal-url \"https://github.com/owner/repo/releases/download/v1.0.0/file.zip\"");

    Ok(())
}
