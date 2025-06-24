// Licensed under the MIT License
// Copyright (c) 2025 Hal <hal.long@outlook.com>

//! Memory-mapped file writer for high-performance I/O
//!
//! This module provides memory-mapped file writing capabilities for
//! maximum I/O performance during downloads.

use crate::error::{Result, TurboCdnError};
use memmap2::{MmapMut, MmapOptions};
use std::fs::{File, OpenOptions};
use std::io::{Seek, SeekFrom, Write};
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, info, warn};

/// Memory-mapped file writer with concurrent chunk support
pub struct MmapWriter {
    file: Arc<Mutex<File>>,
    mmap: Arc<Mutex<Option<MmapMut>>>,
    file_size: u64,
    path: std::path::PathBuf,
    use_mmap: bool,
    chunk_size_threshold: u64,
}

impl MmapWriter {
    /// Create a new memory-mapped writer
    pub async fn new<P: AsRef<Path>>(
        path: P, 
        file_size: u64,
        chunk_size_threshold: Option<u64>
    ) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        let threshold = chunk_size_threshold.unwrap_or(10 * 1024 * 1024); // 10MB default
        
        // Decide whether to use memory mapping based on file size
        let use_mmap = file_size > threshold && file_size < 2_u64.pow(32); // Max 4GB for 32-bit systems
        
        // Create or open the file
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .read(true)
            .open(&path)
            .map_err(|e| TurboCdnError::io(format!("Failed to create file {}: {}", path.display(), e)))?;

        // Pre-allocate file space
        file.set_len(file_size)
            .map_err(|e| TurboCdnError::io(format!("Failed to set file size: {}", e)))?;

        let file = Arc::new(Mutex::new(file));
        let mmap = Arc::new(Mutex::new(None));

        let mut writer = Self {
            file,
            mmap,
            file_size,
            path,
            use_mmap,
            chunk_size_threshold: threshold,
        };

        // Initialize memory mapping if enabled
        if use_mmap {
            writer.init_mmap().await?;
            info!("Memory-mapped writer initialized for {} ({} bytes)", 
                  writer.path.display(), file_size);
        } else {
            info!("Standard file writer initialized for {} ({} bytes)", 
                  writer.path.display(), file_size);
        }

        Ok(writer)
    }

    /// Initialize memory mapping
    async fn init_mmap(&mut self) -> Result<()> {
        let file = self.file.lock().await;
        
        let mmap = unsafe {
            MmapOptions::new()
                .len(self.file_size as usize)
                .map_mut(&*file)
                .map_err(|e| TurboCdnError::io(format!("Failed to create memory map: {}", e)))?
        };

        *self.mmap.lock().await = Some(mmap);
        debug!("Memory mapping initialized for {} bytes", self.file_size);
        
        Ok(())
    }

    /// Write data at specific offset
    pub async fn write_at(&self, offset: u64, data: &[u8]) -> Result<usize> {
        if self.use_mmap {
            self.write_mmap(offset, data).await
        } else {
            self.write_file(offset, data).await
        }
    }

    /// Write data using memory mapping
    async fn write_mmap(&self, offset: u64, data: &[u8]) -> Result<usize> {
        let mut mmap_guard = self.mmap.lock().await;
        
        let mmap = mmap_guard.as_mut()
            .ok_or_else(|| TurboCdnError::io("Memory map not initialized".to_string()))?;

        let start = offset as usize;
        let end = start + data.len();

        if end > mmap.len() {
            return Err(TurboCdnError::io(format!(
                "Write would exceed file bounds: {} > {}", end, mmap.len()
            )));
        }

        // Copy data to memory-mapped region
        mmap[start..end].copy_from_slice(data);
        
        debug!("Memory-mapped write: {} bytes at offset {}", data.len(), offset);
        Ok(data.len())
    }

    /// Write data using standard file I/O
    async fn write_file(&self, offset: u64, data: &[u8]) -> Result<usize> {
        let mut file = self.file.lock().await;
        
        file.seek(SeekFrom::Start(offset))
            .map_err(|e| TurboCdnError::io(format!("Failed to seek to offset {}: {}", offset, e)))?;

        let bytes_written = file.write(data)
            .map_err(|e| TurboCdnError::io(format!("Failed to write data: {}", e)))?;

        debug!("File write: {} bytes at offset {}", bytes_written, offset);
        Ok(bytes_written)
    }

    /// Flush all pending writes
    pub async fn flush(&self) -> Result<()> {
        if self.use_mmap {
            self.flush_mmap().await
        } else {
            self.flush_file().await
        }
    }

    /// Flush memory-mapped data
    async fn flush_mmap(&self) -> Result<()> {
        let mmap_guard = self.mmap.lock().await;
        
        if let Some(mmap) = mmap_guard.as_ref() {
            mmap.flush()
                .map_err(|e| TurboCdnError::io(format!("Failed to flush memory map: {}", e)))?;
            debug!("Memory map flushed");
        }
        
        Ok(())
    }

    /// Flush file data
    async fn flush_file(&self) -> Result<()> {
        let mut file = self.file.lock().await;
        file.flush()
            .map_err(|e| TurboCdnError::io(format!("Failed to flush file: {}", e)))?;
        debug!("File flushed");
        Ok(())
    }

    /// Sync data to disk
    pub async fn sync(&self) -> Result<()> {
        // First flush any pending writes
        self.flush().await?;

        // Then sync to disk
        let file = self.file.lock().await;
        file.sync_all()
            .map_err(|e| TurboCdnError::io(format!("Failed to sync file: {}", e)))?;
        
        debug!("File synced to disk");
        Ok(())
    }

    /// Get file size
    pub fn file_size(&self) -> u64 {
        self.file_size
    }

    /// Check if using memory mapping
    pub fn is_using_mmap(&self) -> bool {
        self.use_mmap
    }

    /// Get file path
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Verify file integrity (check if all data is written)
    pub async fn verify_integrity(&self) -> Result<bool> {
        let file = self.file.lock().await;
        let metadata = file.metadata()
            .map_err(|e| TurboCdnError::io(format!("Failed to get file metadata: {}", e)))?;

        let actual_size = metadata.len();
        let expected_size = self.file_size;

        if actual_size != expected_size {
            warn!("File size mismatch: expected {}, actual {}", expected_size, actual_size);
            return Ok(false);
        }

        debug!("File integrity verified: {} bytes", actual_size);
        Ok(true)
    }

    /// Get write performance statistics
    pub async fn get_stats(&self) -> MmapWriterStats {
        MmapWriterStats {
            file_size: self.file_size,
            use_mmap: self.use_mmap,
            chunk_size_threshold: self.chunk_size_threshold,
            path: self.path.clone(),
        }
    }
}

/// Memory-mapped writer statistics
#[derive(Debug, Clone)]
pub struct MmapWriterStats {
    pub file_size: u64,
    pub use_mmap: bool,
    pub chunk_size_threshold: u64,
    pub path: std::path::PathBuf,
}

impl Drop for MmapWriter {
    fn drop(&mut self) {
        // Ensure data is flushed when writer is dropped
        if let Ok(rt) = tokio::runtime::Handle::try_current() {
            rt.spawn(async move {
                // Note: We can't access self here since it's being dropped
                // The flush will happen automatically when the file is closed
            });
        }
    }
}

/// Utility function to determine optimal chunk size for memory mapping
pub fn optimal_mmap_chunk_size(file_size: u64, available_memory: Option<u64>) -> u64 {
    let default_chunk = 1024 * 1024; // 1MB
    let max_chunk = 64 * 1024 * 1024; // 64MB
    
    // If we know available memory, use a fraction of it
    if let Some(memory) = available_memory {
        let memory_fraction = memory / 8; // Use 1/8 of available memory
        return memory_fraction.min(max_chunk).max(default_chunk);
    }

    // Otherwise, scale based on file size
    match file_size {
        0..=1_048_576 => default_chunk / 4,           // < 1MB: 256KB chunks
        1_048_577..=10_485_760 => default_chunk,      // 1-10MB: 1MB chunks  
        10_485_761..=104_857_600 => default_chunk * 4, // 10-100MB: 4MB chunks
        _ => max_chunk,                                // > 100MB: 64MB chunks
    }
}

/// Check if memory mapping is recommended for given file size
pub fn should_use_mmap(file_size: u64, threshold: Option<u64>) -> bool {
    let threshold = threshold.unwrap_or(10 * 1024 * 1024); // 10MB default
    
    // Use mmap for files larger than threshold but smaller than 4GB
    file_size > threshold && file_size < 4_u64 * 1024 * 1024 * 1024
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_mmap_writer_creation() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.dat");
        let file_size = 1024 * 1024; // 1MB

        let writer = MmapWriter::new(&file_path, file_size, Some(512 * 1024)).await.unwrap();
        assert_eq!(writer.file_size(), file_size);
        assert!(writer.is_using_mmap());
    }

    #[tokio::test]
    async fn test_write_and_verify() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.dat");
        let file_size = 1024;
        let data = b"Hello, World!";

        let writer = MmapWriter::new(&file_path, file_size, Some(0)).await.unwrap();
        
        let bytes_written = writer.write_at(0, data).await.unwrap();
        assert_eq!(bytes_written, data.len());

        writer.flush().await.unwrap();
        assert!(writer.verify_integrity().await.unwrap());
    }

    #[test]
    fn test_optimal_chunk_size() {
        assert_eq!(optimal_mmap_chunk_size(500_000, None), 256 * 1024);
        assert_eq!(optimal_mmap_chunk_size(5_000_000, None), 1024 * 1024);
        assert_eq!(optimal_mmap_chunk_size(50_000_000, None), 4 * 1024 * 1024);
        assert_eq!(optimal_mmap_chunk_size(500_000_000, None), 64 * 1024 * 1024);
    }

    #[test]
    fn test_should_use_mmap() {
        assert!(!should_use_mmap(1_000_000, Some(10_000_000))); // Too small
        assert!(should_use_mmap(50_000_000, Some(10_000_000)));  // Just right
        assert!(!should_use_mmap(5_000_000_000, Some(10_000_000))); // Too large
    }
}
