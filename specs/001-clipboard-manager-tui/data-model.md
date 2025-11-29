# Data Model: Clipboard Manager TUI

**Feature**: 001-clipboard-manager-tui
**Date**: 2025-11-24
**Status**: Design Complete

## Overview

This document defines the core data structures for the clipboard history manager TUI. All models use Rust's type system for safety and Serde for serialization.

---

## Core Entities

### ClipEntry

Represents a single clipboard entry with content, metadata, and state.

```rust
use serde::{Serialize, Deserialize};
use std::time::SystemTime;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ClipEntry {
    /// Unique identifier (timestamp-based or UUID)
    pub id: u64,

    /// Clipboard content (text or image)
    pub content: ClipContent,

    /// When this entry was captured
    pub timestamp: SystemTime,

    /// Whether this entry is pinned (exempt from history rotation)
    pub pinned: bool,

    /// Optional register assignment ('a'-'z', 'A'-'Z')
    pub register: Option<char>,

    /// Hash of content for deduplication
    pub content_hash: u64,
}

impl ClipEntry {
    /// Create new text entry
    pub fn new_text(text: String) -> Self {
        Self {
            id: Self::generate_id(),
            content_hash: Self::hash_content(&text),
            content: ClipContent::Text(text),
            timestamp: SystemTime::now(),
            pinned: false,
            register: None,
        }
    }

    /// Create new image entry
    pub fn new_image(data: Vec<u8>) -> Self {
        Self {
            id: Self::generate_id(),
            content_hash: Self::hash_content(&data),
            content: ClipContent::Image(data),
            timestamp: SystemTime::now(),
            pinned: false,
            register: None,
        }
    }

    /// Get preview text (first N characters)
    pub fn preview(&self, max_len: usize) -> String {
        match &self.content {
            ClipContent::Text(text) => {
                let preview = text.chars().take(max_len).collect::<String>();
                if text.len() > max_len {
                    format!("{}...", preview)
                } else {
                    preview
                }
            }
            ClipContent::Image(data) => {
                format!("[Image: {} bytes]", data.len())
            }
        }
    }

    /// Check if this entry is a duplicate of another
    pub fn is_duplicate(&self, other: &ClipEntry) -> bool {
        self.content_hash == other.content_hash
    }

    fn generate_id() -> u64 {
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64
    }

    fn hash_content<T: AsRef<[u8]>>(content: T) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        content.as_ref().hash(&mut hasher);
        hasher.finish()
    }
}
```

**Validation Rules**:
- `id` must be unique across all entries
- `timestamp` must be valid SystemTime
- `register` must be None or single ASCII letter
- `content_hash` must match actual content

**State Transitions**:
```
Created → Normal
Normal → Pinned (via pin action)
Pinned → Normal (via unpin action)
Normal → Registered (via m<letter> command)
Registered → Normal (via unregister or overwrite)
```

---

### ClipContent

Enumeration of clipboard content types.

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ClipContent {
    /// UTF-8 text content
    Text(String),

    /// Image data (PNG-encoded for storage)
    Image(Vec<u8>),
}

impl ClipContent {
    /// Get content type as string
    pub fn content_type(&self) -> &'static str {
        match self {
            ClipContent::Text(_) => "text",
            ClipContent::Image(_) => "image",
        }
    }

    /// Get content size in bytes
    pub fn size_bytes(&self) -> usize {
        match self {
            ClipContent::Text(s) => s.len(),
            ClipContent::Image(data) => data.len(),
        }
    }

    /// Check if content exceeds size limit
    pub fn exceeds_limit(&self, max_bytes: usize) -> bool {
        self.size_bytes() > max_bytes
    }
}
```

**Constraints**:
- Text: UTF-8 valid, max 10MB
- Image: PNG format, max 50MB
- Future: Add binary content type for non-text/image

---

### Register

Named storage location for frequently-used clipboard content.

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Register {
    /// Register name (single letter: 'a'-'z', 'A'-'Z')
    pub name: char,

    /// Register type (permanent or temporary)
    pub register_type: RegisterType,

    /// Stored content
    pub content: ClipContent,

    /// When this register was created
    pub created_at: SystemTime,

    /// When content was last updated
    pub updated_at: SystemTime,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum RegisterType {
    /// Defined in config file, cannot be deleted from TUI
    Permanent,

    /// Created in TUI via m<letter>, can be deleted
    Temporary,
}

impl Register {
    /// Create new temporary register
    pub fn new_temporary(name: char, content: ClipContent) -> Self {
        let now = SystemTime::now();
        Self {
            name,
            register_type: RegisterType::Temporary,
            content,
            created_at: now,
            updated_at: now,
        }
    }

    /// Create permanent register from config
    pub fn new_permanent(name: char, content: String) -> Self {
        let now = SystemTime::now();
        Self {
            name,
            register_type: RegisterType::Permanent,
            content: ClipContent::Text(content),
            created_at: now,
            updated_at: now,
        }
    }

    /// Update register content
    pub fn update_content(&mut self, content: ClipContent) {
        self.content = content;
        self.updated_at = SystemTime::now();
    }

    /// Check if register can be deleted
    pub fn can_delete(&self) -> bool {
        matches!(self.register_type, RegisterType::Temporary)
    }

    /// Get display name with type indicator
    pub fn display_name(&self) -> String {
        match self.register_type {
            RegisterType::Permanent => format!("[P] {}", self.name),
            RegisterType::Temporary => format!("[T] {}", self.name),
        }
    }
}
```

**Validation Rules**:
- `name` must be single ASCII letter (a-z, A-Z)
- Permanent registers cannot be deleted via TUI
- Temporary registers persist across restarts
- Register names are case-sensitive ('a' ≠ 'A')

---

### ClipboardHistory

Container for all clipboard entries with rotation and deduplication.

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct ClipboardHistory {
    /// Format version for migration compatibility
    pub version: u32,

    /// All clipboard entries (newest first)
    pub entries: Vec<ClipEntry>,

    /// Maximum number of non-pinned entries
    pub max_entries: usize,
}

impl ClipboardHistory {
    pub const CURRENT_VERSION: u32 = 1;
    pub const DEFAULT_MAX_ENTRIES: usize = 1000;

    /// Create new empty history
    pub fn new() -> Self {
        Self {
            version: Self::CURRENT_VERSION,
            entries: Vec::new(),
            max_entries: Self::DEFAULT_MAX_ENTRIES,
        }
    }

    /// Add new entry (with deduplication and rotation)
    pub fn add_entry(&mut self, entry: ClipEntry) {
        // Skip if duplicate of most recent entry
        if let Some(last) = self.entries.first() {
            if last.is_duplicate(&entry) {
                return;
            }
        }

        // Insert at front (newest first)
        self.entries.insert(0, entry);

        // Rotate out oldest non-pinned entries if over limit
        self.enforce_limit();
    }

    /// Remove entry by ID
    pub fn remove_entry(&mut self, id: u64) -> Option<ClipEntry> {
        if let Some(idx) = self.entries.iter().position(|e| e.id == id) {
            Some(self.entries.remove(idx))
        } else {
            None
        }
    }

    /// Toggle pin status
    pub fn toggle_pin(&mut self, id: u64) -> Option<bool> {
        self.entries
            .iter_mut()
            .find(|e| e.id == id)
            .map(|entry| {
                entry.pinned = !entry.pinned;
                entry.pinned
            })
    }

    /// Assign entry to register
    pub fn assign_register(&mut self, id: u64, register: char) -> bool {
        self.entries
            .iter_mut()
            .find(|e| e.id == id)
            .map(|entry| {
                entry.register = Some(register);
                true
            })
            .unwrap_or(false)
    }

    /// Get entries by filter
    pub fn filter_entries(&self, filter: EntryFilter) -> Vec<&ClipEntry> {
        self.entries.iter().filter(|e| filter.matches(e)).collect()
    }

    /// Enforce max_entries limit (keep pinned entries)
    fn enforce_limit(&mut self) {
        let mut unpinned_count = 0;
        self.entries.retain(|entry| {
            if entry.pinned {
                true
            } else {
                unpinned_count += 1;
                unpinned_count <= self.max_entries
            }
        });
    }

    /// Count entries by type
    pub fn stats(&self) -> HistoryStats {
        let mut stats = HistoryStats::default();
        for entry in &self.entries {
            stats.total += 1;
            if entry.pinned {
                stats.pinned += 1;
            }
            if entry.register.is_some() {
                stats.registered += 1;
            }
            match entry.content {
                ClipContent::Text(_) => stats.text += 1,
                ClipContent::Image(_) => stats.images += 1,
            }
        }
        stats
    }
}

#[derive(Debug, Default)]
pub struct HistoryStats {
    pub total: usize,
    pub pinned: usize,
    pub registered: usize,
    pub text: usize,
    pub images: usize,
}
```

---

### EntryFilter

Filtering criteria for clipboard entries.

```rust
#[derive(Debug, Clone)]
pub enum EntryFilter {
    /// Show all entries
    All,

    /// Show only text entries
    TextOnly,

    /// Show only image entries
    ImagesOnly,

    /// Show only pinned entries
    PinnedOnly,

    /// Show only entries assigned to temporary registers
    TemporaryRegisters,

    /// Show only entries assigned to permanent registers
    PermanentRegisters,

    /// Fuzzy search by content
    Search(String),
}

impl EntryFilter {
    pub fn matches(&self, entry: &ClipEntry) -> bool {
        match self {
            EntryFilter::All => true,
            EntryFilter::TextOnly => matches!(entry.content, ClipContent::Text(_)),
            EntryFilter::ImagesOnly => matches!(entry.content, ClipContent::Image(_)),
            EntryFilter::PinnedOnly => entry.pinned,
            EntryFilter::TemporaryRegisters => entry.register.is_some(),
            EntryFilter::PermanentRegisters => false, // Handled separately via Register lookup
            EntryFilter::Search(query) => {
                // Basic substring search (fuzzy matching done separately)
                match &entry.content {
                    ClipContent::Text(text) => text.contains(query),
                    ClipContent::Image(_) => false,
                }
            }
        }
    }
}
```

---

### Configuration

User configuration loaded from TOML file.

```rust
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    /// General settings
    pub general: GeneralConfig,

    /// Keybinding overrides (optional)
    #[serde(default)]
    pub keybindings: KeybindingsConfig,

    /// Permanent registers
    #[serde(default)]
    pub registers: Vec<PermanentRegister>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GeneralConfig {
    /// Maximum clipboard history size
    #[serde(default = "default_history_limit")]
    pub history_limit: usize,

    /// Enable image preview
    #[serde(default = "default_image_preview")]
    pub image_preview: bool,

    /// Maximum image size in bytes
    #[serde(default = "default_max_image_size")]
    pub max_image_size_bytes: usize,

    /// Clipboard polling interval in milliseconds
    #[serde(default = "default_poll_interval")]
    pub poll_interval_ms: u64,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct KeybindingsConfig {
    // Future: Custom keybinding overrides
    // For MVP, use hardcoded vim-style bindings
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PermanentRegister {
    /// Register name (single letter)
    pub name: char,

    /// Register content (text only for permanent registers)
    pub content: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            general: GeneralConfig {
                history_limit: default_history_limit(),
                image_preview: default_image_preview(),
                max_image_size_bytes: default_max_image_size(),
                poll_interval_ms: default_poll_interval(),
            },
            keybindings: KeybindingsConfig::default(),
            registers: Vec::new(),
        }
    }
}

fn default_history_limit() -> usize { 1000 }
fn default_image_preview() -> bool { true }
fn default_max_image_size() -> usize { 50 * 1024 * 1024 } // 50MB
fn default_poll_interval() -> u64 { 200 } // 200ms
```

**Example TOML Configuration**:
```toml
[general]
history_limit = 1000
image_preview = true
max_image_size_bytes = 52428800  # 50MB
poll_interval_ms = 200

[[registers]]
name = "e"
content = "user@example.com"

[[registers]]
name = "w"
content = "123 Main Street, City, State 12345"

[[registers]]
name = "s"
content = "Best regards,\nJohn Doe"
```

---

### SearchIndex

Optimized index for fuzzy searching clipboard entries.

```rust
use nucleo::{Nucleo, Injector};
use std::sync::Arc;

pub struct SearchIndex {
    /// Nucleo matcher instance
    matcher: Nucleo<ClipEntry>,

    /// Injector for adding items
    injector: Injector<ClipEntry>,
}

impl SearchIndex {
    /// Create new search index
    pub fn new() -> Self {
        let matcher = Nucleo::new(Default::default(), Arc::new(|| {}));
        let injector = matcher.injector();

        Self { matcher, injector }
    }

    /// Add entry to index
    pub fn add_entry(&self, entry: ClipEntry) {
        let text = match &entry.content {
            ClipContent::Text(t) => t.clone(),
            ClipContent::Image(_) => format!("[Image: {}]", entry.id),
        };

        self.injector.push(entry, move |_| text.clone());
    }

    /// Update index with new entry list
    pub fn update(&self, entries: &[ClipEntry]) {
        // Clear existing
        self.injector.clear();

        // Re-add all entries
        for entry in entries {
            self.add_entry(entry.clone());
        }
    }

    /// Search and return matched entry IDs (sorted by score)
    pub fn search(&self, query: &str) -> Vec<u64> {
        let snapshot = self.matcher.snapshot();
        snapshot
            .matched_items(..100)  // Top 100 results
            .map(|item| item.data.id)
            .collect()
    }
}
```

---

## Relationships

```
ClipboardHistory
    └─ entries: Vec<ClipEntry>
           └─ content: ClipContent (Text | Image)
           └─ register: Option<char> → Register.name

RegisterStore
    └─ registers: HashMap<char, Register>
           └─ content: ClipContent

Configuration
    └─ registers: Vec<PermanentRegister>
           → Creates Register instances at startup

SearchIndex
    └─ Maintains reverse index: query → entry IDs
    └─ Synced with ClipboardHistory on changes
```

---

## Persistence

### Files

**Clipboard History** (`~/.local/share/clipr/history.bin`):
```rust
bincode::serialize(&ClipboardHistory) -> Vec<u8>
```
- Format: Bincode (fast binary serialization)
- Size: ~2MB for 1000 text entries
- Load time: <10ms

**Temporary Registers** (`~/.local/share/clipr/registers.bin`):
```rust
bincode::serialize(&HashMap<char, Register>) -> Vec<u8>
```
- Format: Bincode
- Size: <100KB typical
- Load time: <1ms

**Configuration** (`~/.config/clipr/config.toml`):
```rust
toml::from_str::<Config>(&content)
```
- Format: TOML (human-readable)
- Size: <10KB typical
- Parse time: <1ms

### Atomic Writes

```rust
fn save_atomically<T: Serialize>(path: &Path, data: &T) -> Result<()> {
    let tmp_path = path.with_extension("tmp");

    // Serialize to temp file
    let encoded = bincode::serialize(data)?;
    fs::write(&tmp_path, encoded)?;

    // Atomic rename (POSIX guarantees)
    fs::rename(&tmp_path, path)?;

    Ok(())
}
```

---

## Performance Considerations

### Memory Usage
- ClipEntry: ~8 bytes overhead + content size
- 1000 text entries (avg 200 chars): ~200KB
- 1000 entries with 10 images (avg 1MB): ~10MB
- SearchIndex: ~5MB for 1000 entries (nucleo internal state)
- **Total**: 15-20MB for typical usage

### Serialization Performance
- Bincode serialize (1000 entries): ~3ms
- Bincode deserialize (1000 entries): ~5ms
- TOML parse config: ~1ms
- **Total cold start**: ~10ms (well under 50ms budget)

### Deduplication
- Hash-based: O(1) duplicate detection
- Only checks against most recent entry (optimization)
- Full scan deduplication on startup (rare)

---

## Migration Strategy

### Version Tracking
```rust
impl ClipboardHistory {
    pub fn migrate(mut self) -> Result<Self> {
        match self.version {
            1 => Ok(self),  // Current version
            0 => {
                // Migrate from v0 to v1
                self.version = 1;
                // Add migration logic
                Ok(self)
            }
            _ => Err(anyhow!("Unsupported history version: {}", self.version)),
        }
    }
}
```

### Breaking Changes
If data model changes incompatibly:
1. Increment `ClipboardHistory::CURRENT_VERSION`
2. Add migration logic in `migrate()` method
3. Test migration with old data files
4. Document migration in CHANGELOG

---

## Next Steps

1. Implement these structs in `src/models/`
2. Add unit tests for validation rules
3. Test serialization round-trips
4. Benchmark memory usage with realistic data
5. Implement State transitions in application layer

**Data Model Complete** ✅
