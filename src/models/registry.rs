use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::clip::ClipboardHistory;

/// Registry for managing register assignments
/// Maps register keys (a-z, A-Z, 0-9) to clip IDs
/// Total of 62 possible registers: 10 digits + 26 lowercase + 26 uppercase
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Registry {
    /// Temporary register assignments: key -> clip_id
    /// User creates these with m<key> command
    temporary: HashMap<char, u64>,
    /// Permanent register assignments: key -> clip_id
    /// Loaded from config file
    permanent: HashMap<char, u64>,
}

impl Registry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Registry {
            temporary: HashMap::new(),
            permanent: HashMap::new(),
        }
    }

    /// Assign a temporary register to a clip
    /// If register already assigned, removes it from the old clip
    pub fn assign_temporary(
        &mut self,
        key: char,
        clip_id: u64,
        history: &mut ClipboardHistory,
    ) -> Result<()> {
        if !is_valid_register_key(key) {
            return Err(anyhow!(
                "Invalid register key '{}': must be 0-9, a-z, or A-Z",
                key
            ));
        }

        // If key already assigned, remove from old clip
        if let Some(&old_clip_id) = self.temporary.get(&key)
            && let Some(clip) = history.get_entry_mut(old_clip_id)
        {
            clip.remove_temporary_register(key);
            log::debug!(
                "Removed temporary register '{}' from clip {}",
                key,
                old_clip_id
            );
        }

        // Assign to new clip
        self.temporary.insert(key, clip_id);
        if let Some(clip) = history.get_entry_mut(clip_id) {
            clip.add_temporary_register(key);
            log::debug!("Assigned temporary register '{}' to clip {}", key, clip_id);
        }

        Ok(())
    }

    /// Remove a temporary register assignment
    pub fn remove_temporary(&mut self, key: char, history: &mut ClipboardHistory) -> Result<()> {
        if !is_valid_register_key(key) {
            return Err(anyhow!("Invalid register key '{}'", key));
        }

        if let Some(clip_id) = self.temporary.remove(&key) {
            if let Some(clip) = history.get_entry_mut(clip_id) {
                clip.remove_temporary_register(key);
            }
            log::debug!("Removed temporary register '{}'", key);
        }

        Ok(())
    }

    /// Assign a permanent register to a clip
    pub fn assign_permanent(
        &mut self,
        key: char,
        clip_id: u64,
        history: &mut ClipboardHistory,
    ) -> Result<()> {
        if !is_valid_register_key(key) {
            return Err(anyhow!("Invalid register key '{}'", key));
        }

        // Remove from old clip if exists
        if let Some(&old_clip_id) = self.permanent.get(&key)
            && let Some(clip) = history.get_entry_mut(old_clip_id)
        {
            clip.remove_permanent_register(key);
        }

        // Assign to new clip
        self.permanent.insert(key, clip_id);
        if let Some(clip) = history.get_entry_mut(clip_id) {
            clip.add_permanent_register(key);
            log::debug!("Assigned permanent register '{}' to clip {}", key, clip_id);
        }

        Ok(())
    }

    /// Get the clip ID assigned to a temporary register
    pub fn get_temporary(&self, key: char) -> Option<u64> {
        self.temporary.get(&key).copied()
    }

    /// Get the clip ID assigned to a permanent register
    pub fn get_permanent(&self, key: char) -> Option<u64> {
        self.permanent.get(&key).copied()
    }

    /// Check if a temporary register is assigned
    pub fn has_temporary(&self, key: char) -> bool {
        self.temporary.contains_key(&key)
    }

    /// Check if a permanent register is assigned
    pub fn has_permanent(&self, key: char) -> bool {
        self.permanent.contains_key(&key)
    }

    /// Get all assigned temporary registers as (key, clip_id) pairs
    pub fn temporary_registers(&self) -> Vec<(char, u64)> {
        self.temporary.iter().map(|(&k, &v)| (k, v)).collect()
    }

    /// Get all assigned permanent registers as (key, clip_id) pairs
    pub fn permanent_registers(&self) -> Vec<(char, u64)> {
        self.permanent.iter().map(|(&k, &v)| (k, v)).collect()
    }

    /// Clear all temporary registers
    pub fn clear_temporary(&mut self, history: &mut ClipboardHistory) {
        for (&key, &clip_id) in &self.temporary {
            if let Some(clip) = history.get_entry_mut(clip_id) {
                clip.remove_temporary_register(key);
            }
        }
        self.temporary.clear();
    }

    /// Rebuild the registry from clipboard history
    /// This should be called after loading history from storage to sync with deserialized register assignments
    pub fn rebuild_from_history(&mut self, history: &ClipboardHistory) {
        self.temporary.clear();
        self.permanent.clear();

        for entry in history.entries() {
            for &key in &entry.temporary_registers {
                self.temporary.insert(key, entry.id);
            }
            for &key in &entry.permanent_registers {
                self.permanent.insert(key, entry.id);
            }
        }
    }

    /// Load permanent registers from config into history
    /// Creates clips for register content if they don't exist
    /// If content changes, moves register to new clip
    pub fn load_permanent_from_config(
        &mut self,
        config: &crate::storage::Config,
        history: &mut ClipboardHistory,
    ) -> Result<()> {
        use crate::models::clip::ClipContent;
        use crate::storage::PermanentRegisterValue;

        for (&key, register_value) in &config.permanent_registers {
            // Create ClipContent based on register type
            let (content, name, description) = match register_value {
                PermanentRegisterValue::Inline {
                    content,
                    name,
                    description,
                } => (
                    ClipContent::Text(content.clone()),
                    name.clone(),
                    description.clone(),
                ),
                PermanentRegisterValue::File {
                    file,
                    mime_type,
                    name,
                    description,
                } => {
                    let mime = mime_type.as_deref().unwrap_or("application/octet-stream");
                    (
                        ClipContent::File {
                            path: file.clone(),
                            mime_type: mime.to_string(),
                        },
                        name.clone(),
                        description.clone(),
                    )
                }
            };

            // Calculate content hash
            let content_hash = content.content_hash();

            // Check if clip with this hash already exists
            if let Some(existing_clip_id) = history.find_by_hash(content_hash) {
                // Clip exists - just assign register to it
                self.assign_permanent(key, existing_clip_id, history)?;
                log::debug!(
                    "Permanent register '{}' assigned to existing clip {}",
                    key,
                    existing_clip_id
                );
            } else {
                // Clip doesn't exist - create new one with metadata
                let clip_id = history.add_entry_with_metadata(content, name, description);

                // Assign permanent register
                self.assign_permanent(key, clip_id, history)?;
                log::debug!(
                    "Permanent register '{}' assigned to new clip {}",
                    key,
                    clip_id
                );
            }
        }

        Ok(())
    }

    /// Get count of assigned temporary registers
    pub fn temporary_count(&self) -> usize {
        self.temporary.len()
    }

    /// Get count of assigned permanent registers
    pub fn permanent_count(&self) -> usize {
        self.permanent.len()
    }
}

impl Default for Registry {
    fn default() -> Self {
        Self::new()
    }
}

/// Validate that a character is a valid register key
/// Valid keys: 0-9, a-z, A-Z (62 total)
pub fn is_valid_register_key(key: char) -> bool {
    matches!(key, '0'..='9' | 'a'..='z' | 'A'..='Z')
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::clip::{ClipContent, ClipboardHistory};

    #[test]
    fn test_valid_register_keys() {
        assert!(is_valid_register_key('a'));
        assert!(is_valid_register_key('Z'));
        assert!(is_valid_register_key('0'));
        assert!(!is_valid_register_key('!'));
        assert!(!is_valid_register_key(' '));
        assert!(!is_valid_register_key('_'));
    }

    #[test]
    fn test_registry_assign_temporary() {
        let mut registry = Registry::new();
        let mut history = ClipboardHistory::new(10);

        let id1 = history.add_entry(ClipContent::Text("test1".to_string()));
        let id2 = history.add_entry(ClipContent::Text("test2".to_string()));

        registry.assign_temporary('a', id1, &mut history).unwrap();
        assert_eq!(registry.get_temporary('a'), Some(id1));
        assert!(
            history
                .get_entry(id1)
                .unwrap()
                .temporary_registers
                .contains(&'a')
        );

        // Reassign should remove from old clip
        registry.assign_temporary('a', id2, &mut history).unwrap();
        assert_eq!(registry.get_temporary('a'), Some(id2));
        assert!(
            !history
                .get_entry(id1)
                .unwrap()
                .temporary_registers
                .contains(&'a')
        );
        assert!(
            history
                .get_entry(id2)
                .unwrap()
                .temporary_registers
                .contains(&'a')
        );
    }

    #[test]
    fn test_registry_list_registers() {
        let mut registry = Registry::new();
        let mut history = ClipboardHistory::new(10);

        let id1 = history.add_entry(ClipContent::Text("test1".to_string()));
        let id2 = history.add_entry(ClipContent::Text("test2".to_string()));

        registry.assign_temporary('a', id1, &mut history).unwrap();
        registry.assign_temporary('Z', id2, &mut history).unwrap();

        let temp_regs = registry.temporary_registers();
        assert_eq!(temp_regs.len(), 2);
        assert!(temp_regs.contains(&('a', id1)));
        assert!(temp_regs.contains(&('Z', id2)));
    }

    #[test]
    fn test_clear_temporary() {
        let mut registry = Registry::new();
        let mut history = ClipboardHistory::new(10);

        let id1 = history.add_entry(ClipContent::Text("test1".to_string()));
        registry.assign_temporary('a', id1, &mut history).unwrap();

        assert_eq!(registry.temporary_count(), 1);

        registry.clear_temporary(&mut history);

        assert_eq!(registry.temporary_count(), 0);
        assert_eq!(registry.get_temporary('a'), None);
        assert!(
            !history
                .get_entry(id1)
                .unwrap()
                .temporary_registers
                .contains(&'a')
        );
    }

    #[test]
    fn test_invalid_key_rejected() {
        let mut registry = Registry::new();
        let mut history = ClipboardHistory::new(10);

        let id1 = history.add_entry(ClipContent::Text("test".to_string()));

        assert!(registry.assign_temporary('!', id1, &mut history).is_err());
        assert!(registry.assign_temporary('_', id1, &mut history).is_err());
        assert!(registry.assign_temporary(' ', id1, &mut history).is_err());
    }

    #[test]
    fn test_multiple_registers_per_clip() {
        let mut registry = Registry::new();
        let mut history = ClipboardHistory::new(10);

        let id1 = history.add_entry(ClipContent::Text("test".to_string()));

        // Assign multiple registers to same clip
        registry.assign_temporary('a', id1, &mut history).unwrap();
        registry.assign_temporary('b', id1, &mut history).unwrap();
        registry.assign_permanent('e', id1, &mut history).unwrap();

        let clip = history.get_entry(id1).unwrap();
        assert_eq!(clip.temporary_registers.len(), 2);
        assert_eq!(clip.permanent_registers.len(), 1);
        assert!(clip.temporary_registers.contains(&'a'));
        assert!(clip.temporary_registers.contains(&'b'));
        assert!(clip.permanent_registers.contains(&'e'));
    }
}
