use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;

use crate::models::ClipboardHistory;

/// Trait for clipboard history persistence
pub trait HistoryStorage: Send + Sync {
    /// Load clipboard history from storage
    fn load(&self) -> Result<ClipboardHistory>;

    /// Save clipboard history to storage
    fn save(&self, history: &ClipboardHistory) -> Result<()>;

    /// Get the storage file path
    fn path(&self) -> &PathBuf;
}

/// Bincode-based implementation of HistoryStorage
/// Uses atomic write pattern with .tmp file for safety
pub struct BincodeHistoryStorage {
    path: PathBuf,
    default_max_entries: usize,
}

impl BincodeHistoryStorage {
    /// Create a new BincodeHistoryStorage with the given path and default max entries
    pub fn new(path: PathBuf, default_max_entries: usize) -> Self {
        BincodeHistoryStorage {
            path,
            default_max_entries,
        }
    }
}

impl HistoryStorage for BincodeHistoryStorage {
    fn load(&self) -> Result<ClipboardHistory> {
        // If file doesn't exist, return empty history
        if !self.path.exists() {
            log::info!(
                "History file not found at {:?}, creating new history with max {} entries",
                self.path,
                self.default_max_entries
            );
            return Ok(ClipboardHistory::new(self.default_max_entries));
        }

        // Read and deserialize
        let bytes = fs::read(&self.path)
            .with_context(|| format!("Failed to read history from {:?}", self.path))?;

        match bincode::decode_from_slice::<ClipboardHistory, _>(&bytes, bincode::config::standard())
        {
            Ok((mut history, _bytes_read)) => {
                // Rebuild hash_to_id index after deserialization
                history.rebuild_hash_map();
                log::info!("Loaded {} clips from {:?}", history.len(), self.path);
                Ok(history)
            }
            Err(e) => {
                // Corrupted file - backup and return empty history
                let backup_path = self.path.with_extension("bin.corrupted");
                log::warn!(
                    "History file corrupted, backing up to {:?}: {}",
                    backup_path,
                    e
                );

                if let Err(backup_err) = fs::rename(&self.path, &backup_path) {
                    log::error!("Failed to backup corrupted file: {}", backup_err);
                }

                Ok(ClipboardHistory::new(self.default_max_entries))
            }
        }
    }

    fn save(&self, history: &ClipboardHistory) -> Result<()> {
        // Serialize to bytes
        let bytes = bincode::encode_to_vec(history, bincode::config::standard())
            .with_context(|| "Failed to serialize clipboard history")?;

        // Atomic write pattern: write to .tmp, then rename
        let tmp_path = self.path.with_extension("bin.tmp");

        // Ensure parent directory exists
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory {:?}", parent))?;
        }

        // Write to temporary file
        fs::write(&tmp_path, &bytes)
            .with_context(|| format!("Failed to write to temporary file {:?}", tmp_path))?;

        // Atomic rename
        fs::rename(&tmp_path, &self.path)
            .with_context(|| format!("Failed to rename {:?} to {:?}", tmp_path, self.path))?;

        log::debug!("Saved {} clips to {:?}", history.len(), self.path);

        Ok(())
    }

    fn path(&self) -> &PathBuf {
        &self.path
    }
}
