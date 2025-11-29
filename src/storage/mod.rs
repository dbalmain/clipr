pub mod config;
pub mod history;
pub mod registers;

use anyhow::{Context, Result};
use std::env;
use std::fs;
use std::path::PathBuf;

pub use config::{Config, ConfigStorage, GeneralConfig, PermanentRegisterValue, TomlConfigStorage};
pub use history::{BincodeHistoryStorage, HistoryStorage};
pub use registers::RegisterStorage;

/// Ensure XDG data and config directories exist
/// Returns (data_dir, config_dir)
///
/// XDG Base Directory Specification:
/// - Data: $XDG_DATA_HOME/clipr (default: ~/.local/share/clipr)
/// - Config: $XDG_CONFIG_HOME/clipr (default: ~/.config/clipr)
pub fn ensure_directories() -> Result<(PathBuf, PathBuf)> {
    let home = env::var("HOME").context("HOME environment variable not set")?;
    let home_path = PathBuf::from(home);

    // Get XDG data directory
    let data_dir = if let Ok(xdg_data) = env::var("XDG_DATA_HOME") {
        PathBuf::from(xdg_data).join("clipr")
    } else {
        home_path.join(".local/share/clipr")
    };

    // Get XDG config directory
    let config_dir = if let Ok(xdg_config) = env::var("XDG_CONFIG_HOME") {
        PathBuf::from(xdg_config).join("clipr")
    } else {
        home_path.join(".config/clipr")
    };

    // Create data directory
    fs::create_dir_all(&data_dir)
        .with_context(|| format!("Failed to create data directory {:?}", data_dir))?;

    // Create config directory
    fs::create_dir_all(&config_dir)
        .with_context(|| format!("Failed to create config directory {:?}", config_dir))?;

    log::debug!("Data directory: {:?}", data_dir);
    log::debug!("Config directory: {:?}", config_dir);

    Ok((data_dir, config_dir))
}
