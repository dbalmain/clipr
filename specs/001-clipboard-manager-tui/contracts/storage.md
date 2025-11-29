# Contract: Storage Layer Interface

**Feature**: 001-clipboard-manager-tui
**Component**: `src/storage/`
**Purpose**: Persist clipboard history, registers, and configuration to disk

## Interface Definition

### HistoryStorage

**Location**: `src/storage/history.rs`

```rust
use anyhow::Result;
use crate::models::ClipboardHistory;
use std::path::PathBuf;

/// Storage interface for clipboard history
pub trait HistoryStorage: Send + Sync {
    /// Load clipboard history from disk
    ///
    /// **Performance**: MUST complete in <50ms
    /// **Error handling**: Return empty history if file missing
    fn load(&self) -> Result<ClipboardHistory>;

    /// Save clipboard history to disk (atomic write)
    ///
    /// **Performance**: Should complete in <20ms for 1000 entries
    /// **Error handling**: Return Err if write fails, original file preserved
    fn save(&self, history: &ClipboardHistory) -> Result<()>;

    /// Get storage file path
    fn path(&self) -> &PathBuf;
}
```

**Implementation**:
```rust
use bincode;
use std::fs;

pub struct BincodeHistoryStorage {
    path: PathBuf,
}

impl BincodeHistoryStorage {
    pub fn new() -> Result<Self> {
        let data_dir = dirs::data_dir()
            .context("Failed to determine data directory")?
            .join("clipr");

        fs::create_dir_all(&data_dir)
            .context("Failed to create data directory")?;

        Ok(Self {
            path: data_dir.join("history.bin"),
        })
    }
}

impl HistoryStorage for BincodeHistoryStorage {
    fn load(&self) -> Result<ClipboardHistory> {
        if !self.path.exists() {
            debug!("History file not found, creating new history");
            return Ok(ClipboardHistory::new());
        }

        let data = fs::read(&self.path)
            .with_context(|| format!("Failed to read history from {:?}", self.path))?;

        let history: ClipboardHistory = bincode::deserialize(&data)
            .context("Failed to deserialize clipboard history")?;

        // Run migrations if needed
        let migrated = history.migrate()?;

        info!("Loaded {} clipboard entries", migrated.entries.len());

        Ok(migrated)
    }

    fn save(&self, history: &ClipboardHistory) -> Result<()> {
        let tmp_path = self.path.with_extension("tmp");

        // Serialize to temporary file
        let encoded = bincode::serialize(history)
            .context("Failed to serialize clipboard history")?;

        fs::write(&tmp_path, encoded)
            .with_context(|| format!("Failed to write to {:?}", tmp_path))?;

        // Atomic rename (POSIX guarantees atomicity)
        fs::rename(&tmp_path, &self.path)
            .with_context(|| format!("Failed to rename {:?} to {:?}", tmp_path, self.path))?;

        debug!("Saved {} entries to history", history.entries.len());

        Ok(())
    }

    fn path(&self) -> &PathBuf {
        &self.path
    }
}
```

**Performance Benchmarks**:
- Serialize 1000 text entries: ~3ms
- Deserialize 1000 text entries: ~5ms
- File I/O: ~2-5ms
- **Total**: ~10-15ms (well under 50ms requirement)

---

### RegisterStorage

**Location**: `src/storage/registers.rs`

```rust
use crate::models::Register;
use std::collections::HashMap;

/// Storage interface for temporary registers
pub trait RegisterStorage: Send + Sync {
    /// Load temporary registers from disk
    ///
    /// **Performance**: MUST complete in <10ms
    /// **Error handling**: Return empty map if file missing
    fn load(&self) -> Result<HashMap<char, Register>>;

    /// Save temporary registers to disk (atomic write)
    ///
    /// **Performance**: Should complete in <5ms
    /// **Error handling**: Return Err if write fails
    fn save(&self, registers: &HashMap<char, Register>) -> Result<()>;

    /// Delete all temporary registers
    fn clear(&self) -> Result<()>;
}
```

**Implementation**:
```rust
pub struct BincodeRegisterStorage {
    path: PathBuf,
}

impl BincodeRegisterStorage {
    pub fn new() -> Result<Self> {
        let data_dir = dirs::data_dir()
            .context("Failed to determine data directory")?
            .join("clipr");

        fs::create_dir_all(&data_dir)?;

        Ok(Self {
            path: data_dir.join("registers.bin"),
        })
    }
}

impl RegisterStorage for BincodeRegisterStorage {
    fn load(&self) -> Result<HashMap<char, Register>> {
        if !self.path.exists() {
            return Ok(HashMap::new());
        }

        let data = fs::read(&self.path)?;
        let registers = bincode::deserialize(&data)?;

        info!("Loaded {} temporary registers", registers.len());

        Ok(registers)
    }

    fn save(&self, registers: &HashMap<char, Register>) -> Result<()> {
        let tmp_path = self.path.with_extension("tmp");

        let encoded = bincode::serialize(registers)?;
        fs::write(&tmp_path, encoded)?;
        fs::rename(&tmp_path, &self.path)?;

        debug!("Saved {} temporary registers", registers.len());

        Ok(())
    }

    fn clear(&self) -> Result<()> {
        if self.path.exists() {
            fs::remove_file(&self.path)?;
        }
        Ok(())
    }
}
```

---

### ConfigStorage

**Location**: `src/storage/config.rs`

```rust
use crate::models::Config;

/// Storage interface for configuration
pub trait ConfigStorage: Send + Sync {
    /// Load configuration from TOML file
    ///
    /// **Performance**: MUST complete in <10ms
    /// **Error handling**: Return default config if file missing
    fn load(&self) -> Result<Config>;

    /// Save configuration to TOML file
    ///
    /// **Performance**: Should complete in <5ms
    /// **Error handling**: Return Err if write fails
    fn save(&self, config: &Config) -> Result<()>;

    /// Get config file path
    fn path(&self) -> &PathBuf;
}
```

**Implementation**:
```rust
pub struct TomlConfigStorage {
    path: PathBuf,
}

impl TomlConfigStorage {
    pub fn new() -> Result<Self> {
        let config_dir = dirs::config_dir()
            .context("Failed to determine config directory")?
            .join("clipr");

        fs::create_dir_all(&config_dir)?;

        Ok(Self {
            path: config_dir.join("config.toml"),
        })
    }

    /// Create default config file with examples
    pub fn create_default(&self) -> Result<()> {
        let default_config = r#"[general]
history_limit = 1000
image_preview = true
max_image_size_bytes = 52428800  # 50MB
poll_interval_ms = 200

# Permanent registers (examples)
[[registers]]
name = "e"
content = "your.email@example.com"

# [[registers]]
# name = "w"
# content = "Your work address here"
"#;

        fs::write(&self.path, default_config)?;

        info!("Created default config at {:?}", self.path);

        Ok(())
    }
}

impl ConfigStorage for TomlConfigStorage {
    fn load(&self) -> Result<Config> {
        if !self.path.exists() {
            info!("Config file not found, using defaults");
            return Ok(Config::default());
        }

        let content = fs::read_to_string(&self.path)
            .with_context(|| format!("Failed to read config from {:?}", self.path))?;

        let config: Config = toml::from_str(&content)
            .context("Failed to parse config TOML")?;

        // Validate config
        for reg in &config.registers {
            if !reg.name.is_ascii_alphabetic() {
                bail!("Invalid register name '{}': must be a-z or A-Z", reg.name);
            }
        }

        info!("Loaded config from {:?}", self.path);
        info!("History limit: {}", config.general.history_limit);
        info!("Permanent registers: {}", config.registers.len());

        Ok(config)
    }

    fn save(&self, config: &Config) -> Result<()> {
        let content = toml::to_string_pretty(config)
            .context("Failed to serialize config to TOML")?;

        fs::write(&self.path, content)
            .with_context(|| format!("Failed to write config to {:?}", self.path))?;

        info!("Saved config to {:?}", self.path);

        Ok(())
    }

    fn path(&self) -> &PathBuf {
        &self.path
    }
}
```

---

## File Locations (XDG Standard)

### Linux
```
~/.local/share/clipr/
├── history.bin          # Clipboard history (Bincode)
├── history.bin.tmp      # Temporary file during atomic write
└── registers.bin        # Temporary registers (Bincode)

~/.config/clipr/
└── config.toml          # User configuration
```

### Creation on First Run
```rust
pub fn ensure_directories() -> Result<()> {
    let data_dir = dirs::data_dir()
        .context("Failed to determine data directory")?
        .join("clipr");

    let config_dir = dirs::config_dir()
        .context("Failed to determine config directory")?
        .join("clipr");

    fs::create_dir_all(&data_dir)
        .context("Failed to create data directory")?;

    fs::create_dir_all(&config_dir)
        .context("Failed to create config directory")?;

    info!("Data directory: {:?}", data_dir);
    info!("Config directory: {:?}", config_dir);

    Ok(())
}
```

---

## Batched Saves (Performance Optimization)

To avoid writing to disk on every clipboard change:

**Location**: `src/storage/batch_saver.rs`

```rust
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::{Duration, Instant};

pub struct BatchSaver<T> {
    data: Arc<RwLock<T>>,
    storage: Box<dyn Storage<T>>,
    dirty: Arc<AtomicBool>,
    save_interval: Duration,
}

impl<T> BatchSaver<T> {
    /// Start background thread that saves every N seconds if dirty
    pub fn start(
        data: Arc<RwLock<T>>,
        storage: Box<dyn Storage<T>>,
        save_interval_secs: u64,
    ) {
        let dirty = Arc::new(AtomicBool::new(false));
        let dirty_clone = dirty.clone();

        thread::spawn(move || {
            loop {
                thread::sleep(Duration::from_secs(save_interval_secs));

                if dirty_clone.load(Ordering::Relaxed) {
                    let data = data.read().unwrap();
                    if let Err(e) = storage.save(&data) {
                        error!("Failed to save: {}", e);
                    } else {
                        dirty_clone.store(false, Ordering::Relaxed);
                    }
                }
            }
        });
    }

    /// Mark data as dirty (needs save)
    pub fn mark_dirty(&self) {
        self.dirty.store(true, Ordering::Relaxed);
    }

    /// Force immediate save
    pub fn flush(&self) -> Result<()> {
        let data = self.data.read().unwrap();
        self.storage.save(&data)?;
        self.dirty.store(false, Ordering::Relaxed);
        Ok(())
    }
}
```

**Usage**:
```rust
// In main app
let history = Arc::new(RwLock::new(ClipboardHistory::new()));
let storage = Box::new(BincodeHistoryStorage::new()?);

BatchSaver::start(history.clone(), storage, 5);  // Save every 5 seconds if dirty

// When clipboard changes
{
    let mut hist = history.write().unwrap();
    hist.add_entry(entry);
}
batch_saver.mark_dirty();  // Will save within 5 seconds

// On app exit
batch_saver.flush()?;  // Force immediate save
```

---

## Error Recovery

### Corrupted History File
```rust
impl BincodeHistoryStorage {
    fn load(&self) -> Result<ClipboardHistory> {
        match self.load_internal() {
            Ok(history) => Ok(history),
            Err(e) => {
                error!("Failed to load history: {}", e);
                error!("Backing up corrupted file and starting fresh");

                // Backup corrupted file
                let backup_path = self.path.with_extension("bin.corrupted");
                if self.path.exists() {
                    fs::rename(&self.path, &backup_path)?;
                }

                // Return empty history
                Ok(ClipboardHistory::new())
            }
        }
    }
}
```

### Atomic Write Failure
- If rename fails, original file is preserved
- Temporary file remains (cleanup on next startup)
- User data never lost due to write failure

---

## Testing Contract

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    #[test]
    fn test_history_round_trip() {
        let dir = TempDir::new().unwrap();
        let storage = BincodeHistoryStorage::with_path(dir.path().join("history.bin"));

        let mut history = ClipboardHistory::new();
        history.add_entry(ClipEntry::new_text("test".into()));

        storage.save(&history).unwrap();
        let loaded = storage.load().unwrap();

        assert_eq!(history.entries.len(), loaded.entries.len());
    }

    #[test]
    fn test_atomic_write_preserves_on_failure() {
        // Test that original file is preserved if rename fails
    }

    #[test]
    fn test_config_validation() {
        let config_toml = r#"
        [general]
        history_limit = 1000

        [[registers]]
        name = "!"  # Invalid
        content = "test"
        "#;

        let result = toml::from_str::<Config>(config_toml);
        assert!(result.is_err());
    }
}
```

### Integration Tests

```rust
#[test]
fn test_concurrent_saves() {
    // Test that multiple threads can save without corruption
}

#[test]
fn test_migration_from_v0_to_v1() {
    // Test data model migration
}
```

---

## Performance Benchmarks

**Location**: `benches/storage.rs`

```rust
fn bench_history_save(c: &mut Criterion) {
    let storage = BincodeHistoryStorage::new().unwrap();
    let mut history = ClipboardHistory::new();

    // Add 1000 entries
    for i in 0..1000 {
        history.add_entry(ClipEntry::new_text(format!("entry {}", i)));
    }

    c.bench_function("save_1000_entries", |b| {
        b.iter(|| storage.save(&history))
    });
}

fn bench_history_load(c: &mut Criterion) {
    let storage = BincodeHistoryStorage::new().unwrap();

    // Pre-populate with 1000 entries
    let mut history = ClipboardHistory::new();
    for i in 0..1000 {
        history.add_entry(ClipEntry::new_text(format!("entry {}", i)));
    }
    storage.save(&history).unwrap();

    c.bench_function("load_1000_entries", |b| {
        b.iter(|| storage.load())
    });
}
```

**Expected Results**:
- Save 1000 entries: <20ms
- Load 1000 entries: <10ms
- Config parse: <1ms

---

## Dependencies

```toml
[dependencies]
bincode = "1.3"
serde = { version = "1.0", features = ["derive"] }
toml = "0.8"
dirs = "5.0"
```

---

## Future Enhancements

1. **Compression**: gzip/zstd for history file (if size becomes issue)
2. **Encryption**: AES-256 for sensitive clipboard data
3. **Cloud sync**: Optional backup to cloud storage
4. **Garbage collection**: Remove old image files periodically

**Contract Complete** ✅
