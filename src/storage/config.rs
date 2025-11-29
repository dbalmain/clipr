use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Main configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub general: GeneralConfig,
    #[serde(rename = "permanent-registers", default)]
    pub permanent_registers: HashMap<char, PermanentRegisterValue>,
}

impl Config {
    /// Create default configuration
    pub fn default() -> Self {
        Config {
            general: GeneralConfig::default(),
            permanent_registers: HashMap::new(),
        }
    }
}

/// General configuration settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    /// Maximum number of clipboard entries to keep
    #[serde(default = "default_max_history")]
    pub max_history: usize,

    /// Maximum image size in bytes (reject larger images)
    #[serde(default = "default_max_image_size")]
    pub max_image_size_bytes: u64,

    /// Image size threshold: ≤ this size stored in memory, > saved to file
    #[serde(default = "default_max_image_memory_size")]
    pub max_image_memory_size_bytes: u64,

    /// Maximum image file size to show preview (don't preview larger files)
    #[serde(default = "default_max_image_preview_size")]
    pub max_image_preview_size_bytes: u64,

    /// Exit TUI after selecting a clip
    #[serde(default = "default_exit_on_select")]
    pub exit_on_select: bool,

    /// Enable debug logging
    #[serde(default)]
    pub debug_logging: bool,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        GeneralConfig {
            max_history: default_max_history(),
            max_image_size_bytes: default_max_image_size(),
            max_image_memory_size_bytes: default_max_image_memory_size(),
            max_image_preview_size_bytes: default_max_image_preview_size(),
            exit_on_select: default_exit_on_select(),
            debug_logging: false,
        }
    }
}

// Default value functions for serde
fn default_max_history() -> usize {
    1000
}

fn default_max_image_size() -> u64 {
    52_428_800 // 50MB
}

fn default_max_image_memory_size() -> u64 {
    5_242_880 // 5MB
}

fn default_max_image_preview_size() -> u64 {
    10_485_760 // 10MB
}

fn default_exit_on_select() -> bool {
    true
}

/// Value for a permanent register entry
/// Supports both inline content and file references
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PermanentRegisterValue {
    /// Inline content: key = { content = "text" }
    Inline {
        content: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        name: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        description: Option<String>,
    },
    /// File reference: key = { file = "path", mime_type = "..." }
    File {
        file: PathBuf,
        #[serde(skip_serializing_if = "Option::is_none")]
        mime_type: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        name: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        description: Option<String>,
    },
}

impl PermanentRegisterValue {
    /// Get the name field if present
    pub fn name(&self) -> Option<&str> {
        match self {
            PermanentRegisterValue::Inline { name, .. } => name.as_deref(),
            PermanentRegisterValue::File { name, .. } => name.as_deref(),
        }
    }

    /// Get the description field if present
    pub fn description(&self) -> Option<&str> {
        match self {
            PermanentRegisterValue::Inline { description, .. } => description.as_deref(),
            PermanentRegisterValue::File { description, .. } => description.as_deref(),
        }
    }

    /// Check if this is a file reference
    pub fn is_file(&self) -> bool {
        matches!(self, PermanentRegisterValue::File { .. })
    }

    /// Get file path if this is a file reference
    pub fn file_path(&self) -> Option<&PathBuf> {
        match self {
            PermanentRegisterValue::File { file, .. } => Some(file),
            _ => None,
        }
    }
}

/// Trait for configuration storage
pub trait ConfigStorage: Send + Sync {
    /// Load configuration from file
    fn load(&self) -> Result<Config>;

    /// Save configuration to file
    fn save(&self, config: &Config) -> Result<()>;

    /// Get the config file path
    fn path(&self) -> &PathBuf;

    /// Create default configuration file if it doesn't exist
    fn create_default(&self) -> Result<()>;
}

/// TOML-based implementation of ConfigStorage
pub struct TomlConfigStorage {
    path: PathBuf,
}

impl TomlConfigStorage {
    /// Create a new TomlConfigStorage with the given path
    pub fn new(path: PathBuf) -> Self {
        TomlConfigStorage { path }
    }
}

impl ConfigStorage for TomlConfigStorage {
    fn load(&self) -> Result<Config> {
        use anyhow::Context;
        use std::fs;

        // If file doesn't exist, create default and return it
        if !self.path.exists() {
            log::info!(
                "Config file not found at {:?}, creating default configuration",
                self.path
            );
            self.create_default()?;
            return Ok(Config::default());
        }

        // Read and parse TOML
        let contents = fs::read_to_string(&self.path)
            .with_context(|| format!("Failed to read config from {:?}", self.path))?;

        let config: Config = toml::from_str(&contents)
            .with_context(|| format!("Failed to parse config file {:?}", self.path))?;

        log::info!("Loaded configuration from {:?}", self.path);
        log::debug!(
            "Config: max_history={}, {} permanent registers",
            config.general.max_history,
            config.permanent_registers.len()
        );

        Ok(config)
    }

    fn save(&self, config: &Config) -> Result<()> {
        use anyhow::Context;
        use std::fs;

        // Serialize to TOML
        let toml_str = toml::to_string_pretty(config)
            .with_context(|| "Failed to serialize configuration")?;

        // Ensure parent directory exists
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory {:?}", parent))?;
        }

        // Write to file
        fs::write(&self.path, toml_str)
            .with_context(|| format!("Failed to write config to {:?}", self.path))?;

        log::debug!("Saved configuration to {:?}", self.path);

        Ok(())
    }

    fn path(&self) -> &PathBuf {
        &self.path
    }

    fn create_default(&self) -> Result<()> {
        use anyhow::Context;
        use std::fs;

        // Ensure parent directory exists
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory {:?}", parent))?;
        }

        // Use the example config compiled into the binary
        let example_config = include_str!("../../clipr.toml.example");

        fs::write(&self.path, example_config)
            .with_context(|| format!("Failed to create default config at {:?}", self.path))?;

        log::info!("Created default configuration at {:?}", self.path);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_defaults() {
        let config = GeneralConfig::default();
        assert_eq!(config.max_history, 1000);
        assert_eq!(config.max_image_size_bytes, 52_428_800);
        assert_eq!(config.max_image_memory_size_bytes, 5_242_880);
        assert_eq!(config.max_image_preview_size_bytes, 10_485_760);
        assert_eq!(config.exit_on_select, true);
    }

    #[test]
    fn test_permanent_register_inline() {
        let toml_str = r#"
        content = "test@example.com"
        name = "email"
        description = "Primary email"
        "#;

        let reg: PermanentRegisterValue = toml::from_str(toml_str).unwrap();
        assert!(!reg.is_file());
        assert_eq!(reg.name(), Some("email"));
        assert_eq!(reg.description(), Some("Primary email"));
    }

    #[test]
    fn test_permanent_register_file() {
        let toml_str = r#"
        file = "/tmp/sig.png"
        mime_type = "image/png"
        name = "signature"
        "#;

        let reg: PermanentRegisterValue = toml::from_str(toml_str).unwrap();
        assert!(reg.is_file());
        assert_eq!(reg.name(), Some("signature"));
        assert_eq!(reg.file_path(), Some(&PathBuf::from("/tmp/sig.png")));
    }
}
