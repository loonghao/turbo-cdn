use turbo_cdn::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    turbo_cdn::init_tracing();

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
    println!("  let result = turbo_cdn.download(\"owner/repo\", \"v1.0.0\", \"file.zip\").execute().await?;");

    Ok(())
}
