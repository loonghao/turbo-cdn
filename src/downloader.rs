// Licensed under the MIT License
// Copyright (c) 2025 Hal <hal.long@outlook.com>

use futures::stream::{self, StreamExt};
use reqwest::header::{HeaderMap, HeaderValue, RANGE};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::fs::{File, OpenOptions};
use tokio::io::{AsyncSeekExt, AsyncWriteExt, SeekFrom};
use tokio::sync::Semaphore;
use tracing::{debug, info, warn};
use uuid::Uuid;

use crate::cache::{CacheLookup, CacheManager};
use crate::compliance::{ComplianceChecker, DownloadRequest};
use crate::config::TurboCdnConfig;
use crate::error::{Result, TurboCdnError};
use crate::progress::{ProgressCallback, ProgressTracker};
use crate::router::{DownloadPerformance, SmartRouter};
use crate::sources::DownloadUrl;

/// Core download engine
#[derive(Debug)]
pub struct Downloader {
    config: TurboCdnConfig,
    client: reqwest::Client,
    router: SmartRouter,
    cache_manager: CacheManager,
    compliance_checker: ComplianceChecker,
    semaphore: Arc<Semaphore>,
}

/// Download options
pub struct DownloadOptions {
    /// Maximum number of concurrent chunks
    pub max_concurrent_chunks: usize,

    /// Chunk size in bytes
    pub chunk_size: usize,

    /// Maximum retry attempts
    pub max_retries: usize,

    /// Retry delay
    pub retry_delay: Duration,

    /// Connection timeout
    pub timeout: Duration,

    /// Whether to use cache
    pub use_cache: bool,

    /// Whether to verify checksums
    pub verify_checksum: bool,

    /// Output directory
    pub output_dir: Option<PathBuf>,

    /// Custom headers
    pub headers: HeaderMap,

    /// Progress callback
    pub progress_callback: Option<ProgressCallback>,
}

impl std::fmt::Debug for DownloadOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DownloadOptions")
            .field("max_concurrent_chunks", &self.max_concurrent_chunks)
            .field("chunk_size", &self.chunk_size)
            .field("max_retries", &self.max_retries)
            .field("retry_delay", &self.retry_delay)
            .field("timeout", &self.timeout)
            .field("use_cache", &self.use_cache)
            .field("verify_checksum", &self.verify_checksum)
            .field("output_dir", &self.output_dir)
            .field("headers", &self.headers)
            .field("progress_callback", &"<callback>")
            .finish()
    }
}

/// Download result
#[derive(Debug, Clone)]
pub struct DownloadResult {
    /// Path to the downloaded file
    pub path: PathBuf,

    /// File size in bytes
    pub size: u64,

    /// Download duration
    pub duration: Duration,

    /// Average download speed in bytes per second
    pub speed: f64,

    /// Source that was used for download
    pub source: String,

    /// URL that was used for download
    pub url: String,

    /// Whether the file was served from cache
    pub from_cache: bool,

    /// Checksum of the downloaded file
    pub checksum: Option<String>,
}

/// Chunk download task
#[derive(Debug)]
struct ChunkTask {
    chunk_id: usize,
    url: String,
    start_byte: u64,
    end_byte: u64,
    output_file: PathBuf,
    progress_tracker: Arc<ProgressTracker>,
}

impl Downloader {
    /// Create a new downloader
    pub async fn new(
        config: TurboCdnConfig,
        router: SmartRouter,
        cache_manager: CacheManager,
        compliance_checker: ComplianceChecker,
    ) -> Result<Self> {
        let mut headers = HeaderMap::new();
        headers.insert(
            reqwest::header::USER_AGENT,
            HeaderValue::from_str(&config.general.user_agent)
                .map_err(|e| TurboCdnError::config(format!("Invalid user agent: {}", e)))?,
        );

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .timeout(config.connect_timeout())
            .connect_timeout(config.connect_timeout())
            .pool_idle_timeout(Duration::from_secs(30))
            .http2_prior_knowledge()
            .build()
            .map_err(|e| TurboCdnError::config(format!("Failed to create HTTP client: {}", e)))?;

        let semaphore = Arc::new(Semaphore::new(config.general.max_concurrent_downloads));

        Ok(Self {
            config,
            client,
            router,
            cache_manager,
            compliance_checker,
            semaphore,
        })
    }

    /// Download a file
    pub async fn download(
        &mut self,
        repository: &str,
        version: &str,
        file_name: &str,
        options: DownloadOptions,
    ) -> Result<DownloadResult> {
        let _permit =
            self.semaphore.clone().acquire_owned().await.map_err(|e| {
                TurboCdnError::internal(format!("Failed to acquire semaphore: {}", e))
            })?;

        let start_time = Instant::now();

        // Create download request for compliance checking
        let download_request = DownloadRequest {
            id: Uuid::new_v4(),
            url: format!("{}@{}/{}", repository, version, file_name),
            source: "multiple".to_string(),
            repository: Some(repository.to_string()),
            file_name: file_name.to_string(),
            user_agent: self.config.general.user_agent.clone(),
            timestamp: chrono::Utc::now(),
            user_consent: true, // User consent is implied by calling the download method
        };

        // Check compliance
        let compliance_result = self
            .compliance_checker
            .check_compliance(&download_request)
            .await?;
        if !compliance_result.approved {
            return Err(TurboCdnError::compliance(format!(
                "Download not approved: {:?}",
                compliance_result.reasons
            )));
        }

        info!(
            "Starting download: {} {} {}",
            repository, version, file_name
        );

        // Check cache first
        if options.use_cache {
            let cache_key = self
                .cache_manager
                .generate_key(repository, version, file_name);
            match self.cache_manager.lookup(&cache_key).await? {
                CacheLookup::Hit(entry) => {
                    info!("Cache hit for {}", cache_key);
                    let data = self.cache_manager.read(&entry).await?;
                    let output_path = self.write_cached_file(&data, file_name, &options).await?;

                    return Ok(DownloadResult {
                        path: output_path,
                        size: data.len() as u64,
                        duration: start_time.elapsed(),
                        speed: 0.0, // Instant from cache
                        source: "cache".to_string(),
                        url: entry.original_url,
                        from_cache: true,
                        checksum: entry.checksum,
                    });
                }
                CacheLookup::Expired(_entry) => {
                    info!("Cache entry expired for {}, will re-download", cache_key);
                    self.cache_manager.remove(&cache_key).await?;
                }
                CacheLookup::Miss => {
                    debug!("Cache miss for {}", cache_key);
                }
            }
        }

        // Get routing decision
        let routing_decision = self
            .router
            .route_download(repository, version, file_name)
            .await?;

        info!(
            "Routing decision: {} primary URLs, {} fallback URLs",
            routing_decision.selected_urls.len(),
            routing_decision.fallback_urls.len()
        );

        // Try downloading from selected URLs
        let mut last_error = None;

        for url in &routing_decision.selected_urls {
            match self
                .download_from_url(url, &options, &download_request)
                .await
            {
                Ok(result) => {
                    // Record successful performance
                    let performance = DownloadPerformance {
                        url: url.url.clone(),
                        source: url.source.clone(),
                        success: true,
                        response_time: result.duration,
                        download_speed: result.speed,
                        bytes_downloaded: result.size,
                        error: None,
                    };
                    self.router.record_performance(performance);

                    info!("Download completed successfully from {}", url.source);
                    return Ok(result);
                }
                Err(e) => {
                    warn!("Download failed from {}: {}", url.source, e);

                    // Record failed performance with elapsed time since start
                    let performance = DownloadPerformance {
                        url: url.url.clone(),
                        source: url.source.clone(),
                        success: false,
                        response_time: start_time.elapsed(),
                        download_speed: 0.0,
                        bytes_downloaded: 0,
                        error: Some(e.to_string()),
                    };
                    self.router.record_performance(performance);

                    last_error = Some(e);
                }
            }
        }

        // Try fallback URLs if primary URLs failed
        for url in &routing_decision.fallback_urls {
            match self
                .download_from_url(url, &options, &download_request)
                .await
            {
                Ok(result) => {
                    warn!("Download succeeded using fallback URL from {}", url.source);
                    return Ok(result);
                }
                Err(e) => {
                    warn!("Fallback download failed from {}: {}", url.source, e);
                    last_error = Some(e);
                }
            }
        }

        Err(last_error.unwrap_or_else(|| TurboCdnError::download("All download attempts failed")))
    }

    /// Download from a specific URL
    async fn download_from_url(
        &mut self,
        download_url: &DownloadUrl,
        options: &DownloadOptions,
        download_request: &DownloadRequest,
    ) -> Result<DownloadResult> {
        let start_time = Instant::now();

        // Get file size
        let file_size = self.get_file_size(&download_url.url).await?;

        // Create progress tracker
        let progress_tracker = Arc::new(ProgressTracker::new(file_size));
        // Note: Progress callback will be handled by the caller if needed

        // Determine output path
        let output_path = self.get_output_path(&download_request.file_name, options)?;

        // Choose download strategy
        if file_size > options.chunk_size as u64 && download_url.supports_ranges {
            // Multi-chunk download
            self.download_chunked(
                &download_url.url,
                file_size,
                &output_path,
                options,
                progress_tracker.clone(),
            )
            .await?
        } else {
            // Single-chunk download
            self.download_single(&download_url.url, &output_path, progress_tracker.clone())
                .await?
        };

        progress_tracker.complete().await;

        let duration = start_time.elapsed();
        let speed = if duration.as_secs_f64() > 0.0 {
            file_size as f64 / duration.as_secs_f64()
        } else {
            0.0
        };

        // Cache the file if enabled
        if options.use_cache {
            let cache_key = self.cache_manager.generate_key(
                download_request
                    .repository
                    .as_ref()
                    .unwrap_or(&"unknown".to_string()),
                download_request
                    .url
                    .split('@')
                    .nth(1)
                    .and_then(|part| part.split('/').next())
                    .unwrap_or("unknown"), // Extract version from URL format "repo@version/file"
                &download_request.file_name,
            );

            if let Ok(data) = tokio::fs::read(&output_path).await {
                if let Err(e) = self
                    .cache_manager
                    .store(&cache_key, &data, &download_url.url, None)
                    .await
                {
                    warn!("Failed to cache file: {}", e);
                }
            }
        }

        Ok(DownloadResult {
            path: output_path,
            size: file_size,
            duration,
            speed,
            source: download_url.source.clone(),
            url: download_url.url.clone(),
            from_cache: false,
            checksum: None, // Checksum calculation can be added as an optional feature
        })
    }

    /// Get file size from URL
    async fn get_file_size(&self, url: &str) -> Result<u64> {
        let response = self
            .client
            .head(url)
            .send()
            .await
            .map_err(TurboCdnError::Network)?;

        if !response.status().is_success() {
            return Err(TurboCdnError::download(format!(
                "HEAD request failed with status: {}",
                response.status()
            )));
        }

        let size = response
            .headers()
            .get(reqwest::header::CONTENT_LENGTH)
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);

        if size == 0 {
            return Err(TurboCdnError::download("Could not determine file size"));
        }

        Ok(size)
    }

    /// Download file in a single request
    async fn download_single(
        &self,
        url: &str,
        output_path: &Path,
        progress_tracker: Arc<ProgressTracker>,
    ) -> Result<()> {
        let mut response = self
            .client
            .get(url)
            .send()
            .await
            .map_err(TurboCdnError::Network)?;

        if !response.status().is_success() {
            return Err(TurboCdnError::download(format!(
                "GET request failed with status: {}",
                response.status()
            )));
        }

        let mut file = File::create(output_path).await.map_err(TurboCdnError::Io)?;

        let mut downloaded = 0u64;

        while let Some(chunk) = response.chunk().await.map_err(TurboCdnError::Network)? {
            file.write_all(&chunk).await.map_err(TurboCdnError::Io)?;

            downloaded += chunk.len() as u64;
            progress_tracker.update(downloaded).await;
        }

        file.sync_all().await.map_err(TurboCdnError::Io)?;
        Ok(())
    }

    /// Download file using multiple chunks
    async fn download_chunked(
        &self,
        url: &str,
        file_size: u64,
        output_path: &Path,
        options: &DownloadOptions,
        progress_tracker: Arc<ProgressTracker>,
    ) -> Result<()> {
        // Create output file
        let file = File::create(output_path).await.map_err(TurboCdnError::Io)?;

        // Set file size
        file.set_len(file_size).await.map_err(TurboCdnError::Io)?;

        drop(file); // Close the file so chunks can open it

        // Calculate chunk ranges
        let chunk_size = options.chunk_size as u64;
        let num_chunks = file_size.div_ceil(chunk_size);
        let max_chunks = options.max_concurrent_chunks.min(num_chunks as usize);

        let chunk_ranges: Vec<(u64, u64)> = (0..num_chunks)
            .map(|i| {
                let start = i * chunk_size;
                let end = ((i + 1) * chunk_size - 1).min(file_size - 1);
                (start, end)
            })
            .collect();

        progress_tracker.init_chunks(chunk_ranges.clone()).await;

        // Create chunk tasks
        let tasks: Vec<ChunkTask> = chunk_ranges
            .into_iter()
            .enumerate()
            .map(|(id, (start, end))| ChunkTask {
                chunk_id: id,
                url: url.to_string(),
                start_byte: start,
                end_byte: end,
                output_file: output_path.to_path_buf(),
                progress_tracker: progress_tracker.clone(),
            })
            .collect();

        // Execute chunks concurrently
        let semaphore = Arc::new(Semaphore::new(max_chunks));
        let client = self.client.clone();

        let results: Vec<Result<()>> = stream::iter(tasks)
            .map(|task| {
                let client = client.clone();
                let semaphore = semaphore.clone();

                async move {
                    let _permit = semaphore
                        .acquire()
                        .await
                        .map_err(|e| TurboCdnError::internal(format!("Semaphore error: {}", e)))?;

                    Self::download_chunk(client, task).await
                }
            })
            .buffer_unordered(max_chunks)
            .collect()
            .await;

        // Check for errors
        for result in results {
            result?;
        }

        Ok(())
    }

    /// Download a single chunk
    async fn download_chunk(client: reqwest::Client, task: ChunkTask) -> Result<()> {
        let range_header = format!("bytes={}-{}", task.start_byte, task.end_byte);

        let mut response = client
            .get(&task.url)
            .header(RANGE, range_header)
            .send()
            .await
            .map_err(TurboCdnError::Network)?;

        if !response.status().is_success()
            && response.status() != reqwest::StatusCode::PARTIAL_CONTENT
        {
            return Err(TurboCdnError::download(format!(
                "Chunk request failed with status: {}",
                response.status()
            )));
        }

        let mut file = OpenOptions::new()
            .write(true)
            .open(&task.output_file)
            .await
            .map_err(TurboCdnError::Io)?;

        file.seek(SeekFrom::Start(task.start_byte))
            .await
            .map_err(TurboCdnError::Io)?;

        let mut downloaded = 0u64;

        while let Some(chunk) = response.chunk().await.map_err(TurboCdnError::Network)? {
            file.write_all(&chunk).await.map_err(TurboCdnError::Io)?;

            downloaded += chunk.len() as u64;
            task.progress_tracker
                .update_chunk(task.chunk_id, downloaded)
                .await;
        }

        task.progress_tracker.complete_chunk(task.chunk_id).await;
        Ok(())
    }

    /// Write cached file to output directory
    async fn write_cached_file(
        &self,
        data: &[u8],
        file_name: &str,
        options: &DownloadOptions,
    ) -> Result<PathBuf> {
        let output_path = self.get_output_path(file_name, options)?;

        tokio::fs::write(&output_path, data)
            .await
            .map_err(TurboCdnError::Io)?;

        Ok(output_path)
    }

    /// Get output path for downloaded file
    fn get_output_path(&self, file_name: &str, options: &DownloadOptions) -> Result<PathBuf> {
        let output_dir = options
            .output_dir
            .as_ref()
            .unwrap_or(&self.config.general.download_dir);

        std::fs::create_dir_all(output_dir).map_err(TurboCdnError::Io)?;

        Ok(output_dir.join(file_name))
    }

    /// Get repository metadata from the first available source
    pub async fn get_repository_metadata(
        &self,
        repository: &str,
    ) -> Result<crate::sources::RepositoryMetadata> {
        let sources = self.router.get_source_manager().enabled_sources();

        if sources.is_empty() {
            return Err(TurboCdnError::source_validation("No sources available"));
        }

        let mut last_error = None;

        for source in sources {
            match source.get_repository_metadata(repository).await {
                Ok(metadata) => {
                    info!(
                        "Retrieved metadata for {} from {}",
                        repository,
                        source.name()
                    );
                    return Ok(metadata);
                }
                Err(e) => {
                    warn!("Failed to get metadata from {}: {}", source.name(), e);
                    last_error = Some(e);
                }
            }
        }

        Err(last_error.unwrap_or_else(|| {
            TurboCdnError::source_validation("No sources could provide repository metadata")
        }))
    }

    /// Get download statistics
    pub async fn get_stats(&self) -> Result<crate::TurboCdnStats> {
        let cache_stats = self.cache_manager.get_stats();
        let performance_stats = self.router.get_performance_stats();

        // Calculate statistics from router performance data
        let mut total_downloads = 0u64;
        let mut successful_downloads = 0u64;
        let mut failed_downloads = 0u64;
        let mut total_bytes = 0u64;
        let mut total_speed = 0f64;
        let mut speed_count = 0u64;

        for source_metrics in performance_stats.get_source_metrics().values() {
            total_downloads += source_metrics.total_requests;
            successful_downloads += source_metrics.successful_requests;
            failed_downloads += source_metrics.failed_requests;

            if source_metrics.average_download_speed > 0.0 {
                total_speed += source_metrics.average_download_speed;
                speed_count += 1;
            }
        }

        for url_metrics in performance_stats.get_url_metrics().values() {
            // URL metrics might have overlapping data with source metrics
            // We'll use them for additional insights but avoid double counting
            if url_metrics.average_download_speed > 0.0 {
                total_bytes += url_metrics.successful_requests * 1024 * 1024; // Estimate
            }
        }

        let average_speed = if speed_count > 0 {
            total_speed / speed_count as f64
        } else {
            0.0
        };

        let cache_hit_rate = if cache_stats.hit_count + cache_stats.miss_count > 0 {
            cache_stats.hit_count as f64 / (cache_stats.hit_count + cache_stats.miss_count) as f64
        } else {
            0.0
        };

        Ok(crate::TurboCdnStats {
            total_downloads,
            successful_downloads,
            failed_downloads,
            total_bytes,
            cache_hit_rate,
            average_speed,
        })
    }

    /// Perform health check on all sources
    pub async fn health_check(
        &self,
    ) -> Result<std::collections::HashMap<String, crate::sources::HealthStatus>> {
        Ok(self.router.get_source_manager().health_check_all().await)
    }
}

impl Default for DownloadOptions {
    fn default() -> Self {
        Self {
            max_concurrent_chunks: 8,
            chunk_size: 1024 * 1024, // 1MB
            max_retries: 3,
            retry_delay: Duration::from_millis(1000),
            timeout: Duration::from_secs(30),
            use_cache: true,
            verify_checksum: true,
            output_dir: None,
            headers: HeaderMap::new(),
            progress_callback: None,
        }
    }
}

impl DownloadOptions {
    /// Create a new builder for download options
    pub fn builder() -> DownloadOptionsBuilder {
        DownloadOptionsBuilder::default()
    }
}

/// Builder for download options
#[derive(Debug, Default)]
pub struct DownloadOptionsBuilder {
    options: DownloadOptions,
}

impl DownloadOptionsBuilder {
    pub fn max_concurrent_chunks(mut self, chunks: usize) -> Self {
        self.options.max_concurrent_chunks = chunks;
        self
    }

    pub fn chunk_size(mut self, size: usize) -> Self {
        self.options.chunk_size = size;
        self
    }

    pub fn max_retries(mut self, retries: usize) -> Self {
        self.options.max_retries = retries;
        self
    }

    pub fn retry_delay(mut self, delay: Duration) -> Self {
        self.options.retry_delay = delay;
        self
    }

    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.options.timeout = timeout;
        self
    }

    pub fn use_cache(mut self, use_cache: bool) -> Self {
        self.options.use_cache = use_cache;
        self
    }

    pub fn verify_checksum(mut self, verify: bool) -> Self {
        self.options.verify_checksum = verify;
        self
    }

    pub fn output_dir<P: Into<PathBuf>>(mut self, dir: P) -> Self {
        self.options.output_dir = Some(dir.into());
        self
    }

    pub fn progress_callback(mut self, callback: ProgressCallback) -> Self {
        self.options.progress_callback = Some(callback);
        self
    }

    pub fn build(self) -> DownloadOptions {
        self.options
    }
}
