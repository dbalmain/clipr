use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};
use std::collections::{hash_map::DefaultHasher, HashMap};
use std::hash::{Hash, Hasher};
use std::path::PathBuf;
use std::time::SystemTime;

/// Content type for clipboard entries
#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode, PartialEq)]
pub enum ClipContent {
    /// Text content with UTF-8 string
    Text(String),
    /// Image content stored in memory (≤5MB from clipboard)
    Image { data: Vec<u8>, mime_type: String },
    /// File reference for large images or permanent register files
    File { path: PathBuf, mime_type: String },
}

impl ClipContent {
    /// Get a preview string (truncated for display)
    pub fn preview(&self, max_len: usize) -> String {
        match self {
            ClipContent::Text(text) => {
                let preview = text.lines().next().unwrap_or("");
                if preview.len() > max_len {
                    format!("{}...", &preview[..max_len])
                } else {
                    preview.to_string()
                }
            }
            ClipContent::Image { mime_type, data } => {
                format!("[Image: {} ({} bytes)]", mime_type, data.len())
            }
            ClipContent::File { path, mime_type } => {
                format!(
                    "[File: {} ({})]",
                    mime_type,
                    path.file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("unknown")
                )
            }
        }
    }

    /// Check if this is text content
    pub fn is_text(&self) -> bool {
        matches!(self, ClipContent::Text(_))
    }

    /// Check if this is image content
    pub fn is_image(&self) -> bool {
        matches!(self, ClipContent::Image { .. })
    }

    /// Check if this is a file reference
    pub fn is_file(&self) -> bool {
        matches!(self, ClipContent::File { .. })
    }

    /// Get content hash for deduplication
    /// Note: File hash is based on path + mime_type, NOT file contents
    pub fn content_hash(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        match self {
            ClipContent::Text(text) => text.hash(&mut hasher),
            ClipContent::Image { data, mime_type } => {
                data.hash(&mut hasher);
                mime_type.hash(&mut hasher);
            }
            ClipContent::File { path, mime_type } => {
                path.hash(&mut hasher);
                mime_type.hash(&mut hasher);
            }
        }
        hasher.finish()
    }
}

/// A single clipboard entry with metadata
#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode, PartialEq)]
pub struct ClipEntry {
    /// Unique identifier (monotonic counter)
    pub id: u64,
    /// The clipboard content (text, image, or file)
    pub content: ClipContent,
    /// When this entry was last seen (updated on duplicate)
    pub timestamp: SystemTime,
    /// Whether this entry is pinned (exempt from history rotation)
    pub pinned: bool,
    /// Optional human-readable name
    pub name: Option<String>,
    /// Optional description
    pub description: Option<String>,
    /// Temporary registers assigned to this clip
    pub temporary_registers: Vec<char>,
    /// Permanent registers assigned to this clip
    pub permanent_registers: Vec<char>,
    /// Content hash for deduplication
    pub content_hash: u64,
}

impl ClipEntry {
    /// Create a new text clipboard entry
    pub fn new_text(id: u64, text: String) -> Self {
        let content = ClipContent::Text(text);
        let content_hash = content.content_hash();
        ClipEntry {
            id,
            content,
            timestamp: SystemTime::now(),
            pinned: false,
            name: None,
            description: None,
            temporary_registers: Vec::new(),
            permanent_registers: Vec::new(),
            content_hash,
        }
    }

    /// Create a new image clipboard entry (in-memory)
    pub fn new_image(id: u64, data: Vec<u8>, mime_type: String) -> Self {
        let content = ClipContent::Image { data, mime_type };
        let content_hash = content.content_hash();
        ClipEntry {
            id,
            content,
            timestamp: SystemTime::now(),
            pinned: false,
            name: None,
            description: None,
            temporary_registers: Vec::new(),
            permanent_registers: Vec::new(),
            content_hash,
        }
    }

    /// Create a new file reference entry
    pub fn new_file(id: u64, path: PathBuf, mime_type: String) -> Self {
        let content = ClipContent::File { path, mime_type };
        let content_hash = content.content_hash();
        ClipEntry {
            id,
            content,
            timestamp: SystemTime::now(),
            pinned: false,
            name: None,
            description: None,
            temporary_registers: Vec::new(),
            permanent_registers: Vec::new(),
            content_hash,
        }
    }

    /// Create from existing content with optional metadata
    pub fn new_with_metadata(
        id: u64,
        content: ClipContent,
        name: Option<String>,
        description: Option<String>,
    ) -> Self {
        let content_hash = content.content_hash();
        ClipEntry {
            id,
            content,
            timestamp: SystemTime::now(),
            pinned: false,
            name,
            description,
            temporary_registers: Vec::new(),
            permanent_registers: Vec::new(),
            content_hash,
        }
    }

    /// Get a preview string for this entry
    pub fn preview(&self, max_len: usize) -> String {
        self.content.preview(max_len)
    }

    /// Check if this entry is a duplicate of another (same content hash)
    pub fn is_duplicate(&self, other: &ClipEntry) -> bool {
        self.content_hash == other.content_hash
    }

    /// Toggle pinned status
    pub fn toggle_pin(&mut self) {
        self.pinned = !self.pinned;
    }

    /// Update timestamp (used when duplicate is seen)
    pub fn bump_timestamp(&mut self) {
        self.timestamp = SystemTime::now();
    }

    /// Check if file exists (for File variant)
    pub fn file_exists(&self) -> bool {
        match &self.content {
            ClipContent::File { path, .. } => path.exists(),
            _ => true, // Text/Image always "exist"
        }
    }

    /// Check if this entry has a missing file reference
    pub fn has_missing_file(&self) -> bool {
        match &self.content {
            ClipContent::File { path, .. } => !path.exists(),
            _ => false,
        }
    }

    /// Check if this entry should be kept (has pins or registers)
    pub fn should_keep(&self) -> bool {
        self.pinned
            || !self.temporary_registers.is_empty()
            || !self.permanent_registers.is_empty()
    }

    /// Check if this entry can be deleted from TUI
    /// (Permanent register clips cannot be deleted)
    pub fn can_delete(&self) -> bool {
        self.permanent_registers.is_empty()
    }

    /// Add a temporary register to this clip
    pub fn add_temporary_register(&mut self, key: char) {
        if !self.temporary_registers.contains(&key) {
            self.temporary_registers.push(key);
        }
    }

    /// Remove a temporary register from this clip
    pub fn remove_temporary_register(&mut self, key: char) {
        self.temporary_registers.retain(|&k| k != key);
    }

    /// Add a permanent register to this clip
    pub fn add_permanent_register(&mut self, key: char) {
        if !self.permanent_registers.contains(&key) {
            self.permanent_registers.push(key);
        }
    }

    /// Remove a permanent register from this clip
    pub fn remove_permanent_register(&mut self, key: char) {
        self.permanent_registers.retain(|&k| k != key);
    }
}

/// Clipboard history manager
/// Entries are kept in timestamp order (most recent first)
#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub struct ClipboardHistory {
    /// All clipboard entries (sorted by timestamp, most recent first)
    pub entries: Vec<ClipEntry>,
    /// Maximum number of entries to keep (excludes pinned and registered)
    pub max_entries: usize,
    /// Next ID to assign (monotonic counter)
    next_id: u64,
    /// HashMap for fast duplicate detection: content_hash -> entry_id
    #[serde(skip)]
    hash_to_id: HashMap<u64, u64>,
}

impl ClipboardHistory {
    /// Create a new clipboard history with specified max entries
    pub fn new(max_entries: usize) -> Self {
        ClipboardHistory {
            entries: Vec::new(),
            max_entries,
            next_id: 1,
            hash_to_id: HashMap::new(),
        }
    }

    /// Rebuild the hash_to_id map (called after deserialization)
    pub fn rebuild_hash_map(&mut self) {
        self.hash_to_id.clear();
        for entry in &self.entries {
            self.hash_to_id.insert(entry.content_hash, entry.id);
        }
    }

    /// Add a new entry to the history
    /// If content is duplicate, updates timestamp and moves to front
    /// Returns the ID of the entry (existing or new)
    pub fn add_entry(&mut self, content: ClipContent) -> u64 {
        let content_hash = content.content_hash();

        // Check if this content already exists
        if let Some(&existing_id) = self.hash_to_id.get(&content_hash) {
            log::debug!(
                "Duplicate detected, bumping timestamp for entry {}",
                existing_id
            );

            // Find the existing entry, update timestamp, and move to front
            if let Some(pos) = self.entries.iter().position(|e| e.id == existing_id) {
                let mut entry = self.entries.remove(pos);
                entry.bump_timestamp();
                self.entries.insert(0, entry);
            }

            return existing_id;
        }

        // Create new entry
        let id = self.next_id;
        self.next_id += 1;

        let entry = match content {
            ClipContent::Text(text) => ClipEntry::new_text(id, text),
            ClipContent::Image { data, mime_type } => ClipEntry::new_image(id, data, mime_type),
            ClipContent::File { path, mime_type } => ClipEntry::new_file(id, path, mime_type),
        };

        // Add to hash map
        self.hash_to_id.insert(content_hash, id);

        // Add to front (most recent first)
        self.entries.insert(0, entry);

        // Rotate history if needed (keep pinned and registered entries)
        self.rotate_history();

        id
    }

    /// Add a new entry with metadata (for permanent registers)
    pub fn add_entry_with_metadata(
        &mut self,
        content: ClipContent,
        name: Option<String>,
        description: Option<String>,
    ) -> u64 {
        let content_hash = content.content_hash();

        // Check if this content already exists
        if let Some(&existing_id) = self.hash_to_id.get(&content_hash) {
            log::debug!(
                "Duplicate detected, bumping timestamp for entry {}",
                existing_id
            );

            // Update existing entry
            if let Some(pos) = self.entries.iter().position(|e| e.id == existing_id) {
                let mut entry = self.entries.remove(pos);
                entry.bump_timestamp();
                // Update metadata if provided
                if name.is_some() {
                    entry.name = name;
                }
                if description.is_some() {
                    entry.description = description;
                }
                self.entries.insert(0, entry);
            }

            return existing_id;
        }

        // Create new entry with metadata
        let id = self.next_id;
        self.next_id += 1;

        let entry = ClipEntry::new_with_metadata(id, content, name, description);

        // Add to hash map
        self.hash_to_id.insert(content_hash, id);

        // Add to front
        self.entries.insert(0, entry);

        // Rotate history
        self.rotate_history();

        id
    }

    /// Remove an entry by ID
    pub fn remove_entry(&mut self, id: u64) -> bool {
        if let Some(pos) = self.entries.iter().position(|e| e.id == id) {
            let entry = self.entries.remove(pos);
            self.hash_to_id.remove(&entry.content_hash);
            true
        } else {
            false
        }
    }

    /// Get all pinned entries
    pub fn get_pinned(&self) -> Vec<&ClipEntry> {
        self.entries.iter().filter(|e| e.pinned).collect()
    }

    /// Get entry by ID
    pub fn get_entry(&self, id: u64) -> Option<&ClipEntry> {
        self.entries.iter().find(|e| e.id == id)
    }

    /// Get mutable entry by ID
    pub fn get_entry_mut(&mut self, id: u64) -> Option<&mut ClipEntry> {
        self.entries.iter_mut().find(|e| e.id == id)
    }

    /// Clear all non-pinned, non-registered entries
    pub fn clear_unpinned(&mut self) {
        self.entries.retain(|e| {
            if e.should_keep() {
                true
            } else {
                self.hash_to_id.remove(&e.content_hash);
                false
            }
        });
    }

    /// Rotate history to enforce max_entries limit
    /// Pinned entries and entries with registers are exempt from rotation
    fn rotate_history(&mut self) {
        let protected_count = self.entries.iter().filter(|e| e.should_keep()).count();
        let unprotected_count = self.entries.len() - protected_count;

        if unprotected_count > self.max_entries {
            let to_remove = unprotected_count - self.max_entries;

            self.entries.retain(|e| {
                if e.should_keep() {
                    true
                } else {
                    static mut REMOVED: usize = 0;
                    unsafe {
                        if REMOVED < to_remove {
                            REMOVED += 1;
                            self.hash_to_id.remove(&e.content_hash);
                            false
                        } else {
                            true
                        }
                    }
                }
            });

            // Reset counter (unsafe but constrained to this function)
            unsafe {
                static mut REMOVED: usize = 0;
                REMOVED = 0;
            }
        }
    }

    /// Sort entries by timestamp (most recent first)
    pub fn sort_by_timestamp(&mut self) {
        self.entries
            .sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
    }

    /// Get next available ID
    pub fn next_id(&self) -> u64 {
        self.next_id
    }

    /// Get the number of entries in the history
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if history is empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Get a reference to all entries
    pub fn entries(&self) -> &[ClipEntry] {
        &self.entries
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clip_content_preview() {
        let text = ClipContent::Text("Hello, world!".to_string());
        assert_eq!(text.preview(10), "Hello, wor...");
        assert_eq!(text.preview(50), "Hello, world!");

        let image = ClipContent::Image {
            data: vec![0; 100],
            mime_type: "image/png".to_string(),
        };
        assert_eq!(image.preview(50), "[Image: image/png (100 bytes)]");

        let file = ClipContent::File {
            path: PathBuf::from("/tmp/test.png"),
            mime_type: "image/png".to_string(),
        };
        assert!(file.preview(50).contains("test.png"));
    }

    #[test]
    fn test_file_hash_stable() {
        let file1 = ClipContent::File {
            path: PathBuf::from("/tmp/test.png"),
            mime_type: "image/png".to_string(),
        };
        let file2 = ClipContent::File {
            path: PathBuf::from("/tmp/test.png"),
            mime_type: "image/png".to_string(),
        };

        // Same path + mime_type = same hash (even if file contents change)
        assert_eq!(file1.content_hash(), file2.content_hash());

        let file3 = ClipContent::File {
            path: PathBuf::from("/tmp/other.png"),
            mime_type: "image/png".to_string(),
        };

        // Different path = different hash
        assert_ne!(file1.content_hash(), file3.content_hash());
    }

    #[test]
    fn test_clip_entry_registers() {
        let mut entry = ClipEntry::new_text(1, "test".to_string());

        assert!(entry.temporary_registers.is_empty());
        assert!(entry.permanent_registers.is_empty());
        assert!(entry.can_delete());

        entry.add_temporary_register('a');
        assert_eq!(entry.temporary_registers, vec!['a']);
        assert!(entry.should_keep());

        entry.add_permanent_register('e');
        assert_eq!(entry.permanent_registers, vec!['e']);
        assert!(!entry.can_delete());

        entry.remove_temporary_register('a');
        assert!(entry.temporary_registers.is_empty());
    }

    #[test]
    fn test_duplicate_bumps_timestamp() {
        let mut history = ClipboardHistory::new(10);

        let id1 = history.add_entry(ClipContent::Text("test".to_string()));
        history.add_entry(ClipContent::Text("other".to_string()));

        assert_eq!(history.entries[1].id, id1);

        let id2 = history.add_entry(ClipContent::Text("test".to_string()));

        assert_eq!(id1, id2);
        assert_eq!(history.entries[0].id, id1);
        assert_eq!(history.entries.len(), 2);
    }

    #[test]
    fn test_registered_entries_exempt_from_rotation() {
        let mut history = ClipboardHistory::new(2);

        let id1 = history.add_entry(ClipContent::Text("Entry 1".to_string()));
        let id2 = history.add_entry(ClipContent::Text("Entry 2".to_string()));

        // Assign registers
        history.get_entry_mut(id1).unwrap().add_temporary_register('a');
        history
            .get_entry_mut(id2)
            .unwrap()
            .add_permanent_register('e');

        // Add more entries
        history.add_entry(ClipContent::Text("Entry 3".to_string()));
        history.add_entry(ClipContent::Text("Entry 4".to_string()));
        history.add_entry(ClipContent::Text("Entry 5".to_string()));

        // Should have 2 registered + 2 unregistered (max) = 4 total
        assert_eq!(history.entries.len(), 4);

        // Registered entries should still be present
        assert!(history.get_entry(id1).is_some());
        assert!(history.get_entry(id2).is_some());
    }
}
