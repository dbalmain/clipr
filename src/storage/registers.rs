use anyhow::Result;
use std::path::PathBuf;

use crate::models::Registry;

/// Trait for register persistence
pub trait RegisterStorage: Send + Sync {
    /// Load register assignments from storage
    fn load(&self) -> Result<Registry>;

    /// Save register assignments to storage
    fn save(&self, registry: &Registry) -> Result<()>;

    /// Clear all temporary registers
    fn clear(&self) -> Result<()>;

    /// Get the storage file path
    fn path(&self) -> &PathBuf;
}
