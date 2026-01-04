use anyhow::{Context, Result};
use lru::LruCache;
use notify::{RecommendedWatcher, Watcher};
use ratatui::Frame;
use ratatui::crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use ratatui_image::protocol::StatefulProtocol;
use std::num::NonZeroUsize;
use std::sync::mpsc::{self, Receiver, Sender};
use std::time::Instant;
use tui_input::Input;
use tui_input::backend::crossterm::EventHandler;

use crate::clipboard::ClipboardBackend;
use crate::image::ImageProtocol;
use crate::logging::FlashMessage;
use crate::models::{ClipboardHistory, Registry, SearchIndex};
use crate::storage::Config;
use crate::ui;
use crate::ui::Theme;

/// Request to load an image in the background
struct ImageLoadRequest {
    clip_id: u64,
    image_data: Vec<u8>,
}

/// Result of loading an image in the background
struct ImageLoadResult {
    clip_id: u64,
    protocol_image: Option<StatefulProtocol>,
}

/// Application mode determines which keybindings are active
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum AppMode {
    /// Normal browsing mode with vim-style navigation
    #[default]
    Normal,
    /// Search/filter mode (activated with '/')
    Search,
    /// Register assignment mode (activated with 'm' in normal mode, like vim marks)
    RegisterAssign,
    /// Confirmation dialog (for clear all operation)
    Confirm,
    /// Help overlay (activated with '?')
    Help,
    /// Numeric prefix mode with command palette (activated by typing digits)
    Numeric,
    /// Theme picker modal (activated with 'T')
    ThemePicker,
}

/// Register filter state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegisterFilter {
    /// No filter active - show all clips
    None,
    /// Show only clips with temporary registers (activated with ')
    Temporary,
    /// Show only clips with permanent registers (activated with ")
    Permanent,
    /// Show only pinned clips (activated with P)
    Pinned,
}

/// View mode for clip list display
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewMode {
    /// Compact: Single line per clip
    Compact,
    /// Comfortable: Two lines per clip with metadata
    Comfortable,
}

/// Paste request - content to paste after TUI exits
#[derive(Debug, Clone)]
pub enum PasteRequest {
    /// No paste requested
    None,
    /// Write to clipboard and simulate Ctrl-V (works for all content types)
    PasteFromClipboard(crate::models::ClipContent),
}

/// Main application state
pub struct App {
    /// Current interaction mode
    pub mode: AppMode,

    /// Clipboard history (loaded from storage)
    pub history: ClipboardHistory,

    /// Permanent registers (loaded from config)
    pub registers: Registry,

    /// Application configuration
    pub config: Config,

    /// Theme (loaded from config)
    theme: Theme,

    /// Fuzzy search index
    pub search_index: SearchIndex,

    /// Clipboard backend for copying selected entries
    clipboard_backend: Box<dyn ClipboardBackend>,

    /// LRU cache of decoded images (clip_id -> protocol_image)
    /// Caches recently viewed images to avoid re-decoding
    image_cache: LruCache<u64, StatefulProtocol>,

    /// Channel for requesting background image loads
    image_load_tx: Sender<ImageLoadRequest>,

    /// Channel for receiving completed image loads
    image_load_rx: Receiver<ImageLoadResult>,

    /// File watcher for theme development mode (only present if theme_dev_mode enabled)
    /// Kept alive to maintain the watch
    _theme_watcher: Option<RecommendedWatcher>,

    /// Channel for receiving theme file change notifications
    theme_watch_rx: Option<Receiver<notify::Result<notify::Event>>>,

    /// Currently selected index in the visible list
    pub selected_index: usize,

    /// Search input with cursor support
    pub search_input: Input,

    /// Filtered and sorted clip IDs from search
    /// Empty means show all clips in chronological order
    pub search_results: Vec<u64>,

    /// Register key being assigned (when in RegisterAssign mode)
    pub register_key: Option<char>,

    /// Numeric prefix for vim-style commands (e.g., 5j, 10G, 3Ctrl-d)
    pub numeric_prefix: String,

    /// Active register filter (None, Temporary, or Permanent)
    pub register_filter: RegisterFilter,

    /// Current view mode (Compact or Comfortable)
    pub view_mode: ViewMode,

    /// Startup error message (shown in modal, dismissible with ESC)
    pub startup_error: Option<String>,

    /// List area height in terminal rows (updated each frame)
    /// Used to calculate half-page and full-page movements
    list_height: u16,

    /// Scroll offset for clip list (top visible row index)
    /// Used to implement scroll padding (keep context lines visible)
    list_scroll_offset: usize,

    /// Theme picker state
    pub theme_picker_themes: Vec<String>,
    pub theme_picker_selected: usize,
    pub current_theme_name: String,

    /// Flag to request application exit
    pub should_quit: bool,

    /// Content to paste after TUI exits (allows terminal to close first)
    pub paste_request: PasteRequest,

    /// Flash messages displayed at bottom of preview
    pub flash_messages: Vec<FlashMessage>,

    /// Receiver for flash messages from logger
    flash_rx: Option<Receiver<FlashMessage>>,

    /// Scroll offset for help modal (0 = top)
    help_scroll: usize,

    /// Maximum scroll value for help modal (updated each frame)
    help_max_scroll: usize,
}

impl App {
    /// Create a new App instance by loading state from storage
    pub fn new(
        history: ClipboardHistory,
        registers: Registry,
        config: Config,
        clipboard_backend: Box<dyn ClipboardBackend>,
        mut image_protocol: ImageProtocol,
        flash_rx: Option<Receiver<FlashMessage>>,
    ) -> Result<Self> {
        // Create channels for async image loading
        let (load_tx, load_rx) = mpsc::channel::<ImageLoadRequest>();
        let (result_tx, result_rx) = mpsc::channel::<ImageLoadResult>();

        // Spawn background thread for image loading
        std::thread::spawn(move || {
            log::debug!("Image loader thread started");
            while let Ok(request) = load_rx.recv() {
                log::debug!("Loading image for clip {}", request.clip_id);

                // Decode image and create protocol state
                let protocol_image = match image::load_from_memory(&request.image_data) {
                    Ok(img) => {
                        let protocol = image_protocol.picker.new_resize_protocol(img);
                        Some(protocol)
                    }
                    Err(e) => {
                        log::warn!("Failed to decode image for clip {}: {}", request.clip_id, e);
                        None
                    }
                };

                // Send result back to main thread
                if result_tx
                    .send(ImageLoadResult {
                        clip_id: request.clip_id,
                        protocol_image,
                    })
                    .is_err()
                {
                    log::debug!("Image loader: main thread disconnected, exiting");
                    break;
                }
            }
            log::debug!("Image loader thread exiting");
        });

        // Load theme from config
        let (theme, startup_error) = match Theme::load(&config.general.theme) {
            Ok(t) => (t, None),
            Err(e) => {
                log::error!("Failed to load theme '{}': {}", config.general.theme, e);
                (Theme::default(), Some(e.to_string()))
            }
        };

        // Set up file watcher for theme development mode
        let (theme_watcher, theme_watch_rx) = if config.general.theme_dev_mode {
            log::info!("Theme development mode enabled - watching for theme file changes");

            let (tx, rx) = mpsc::channel();

            // Create watcher
            let mut watcher =
                notify::recommended_watcher(move |res: notify::Result<notify::Event>| {
                    let _ = tx.send(res);
                })
                .context("Failed to create theme file watcher")?;

            // Watch the themes directory instead of specific file
            // This handles editors that do atomic writes (create temp, rename)
            if let Ok(theme_path) = Theme::get_theme_path(&config.general.theme) {
                use notify::RecursiveMode;
                if let Some(parent_dir) = theme_path.parent() {
                    if let Err(e) = watcher.watch(parent_dir, RecursiveMode::NonRecursive) {
                        log::warn!("Failed to watch themes directory {:?}: {}", parent_dir, e);
                    } else {
                        log::info!("Watching themes directory: {:?}", parent_dir);
                    }
                }
            } else {
                log::warn!(
                    "Could not determine theme file path for '{}'",
                    config.general.theme
                );
            }

            (Some(watcher), Some(rx))
        } else {
            (None, None)
        };

        // Create LRU cache with configured size
        let cache_size = NonZeroUsize::new(config.general.image_cache_size)
            .unwrap_or_else(|| NonZeroUsize::new(20).unwrap());
        let image_cache = LruCache::new(cache_size);

        // Parse view mode from config
        let view_mode = match config.general.view_mode.to_lowercase().as_str() {
            "comfortable" => ViewMode::Comfortable,
            _ => ViewMode::Compact, // Default to compact for invalid values
        };

        // Store current theme name before moving config
        let current_theme_name = config.general.theme.clone();

        let mut app = App {
            mode: AppMode::default(),
            history,
            registers,
            theme,
            config,
            search_index: SearchIndex::new(),
            clipboard_backend,
            image_cache,
            image_load_tx: load_tx,
            image_load_rx: result_rx,
            _theme_watcher: theme_watcher,
            theme_watch_rx,
            selected_index: 0,
            search_input: Input::default(),
            search_results: Vec::new(),
            register_key: None,
            numeric_prefix: String::new(),
            register_filter: RegisterFilter::None,
            view_mode,
            startup_error,
            list_height: 20, // Default, will be updated each frame
            list_scroll_offset: 0,
            theme_picker_themes: Vec::new(),
            theme_picker_selected: 0,
            current_theme_name,
            should_quit: false,
            paste_request: PasteRequest::None,
            flash_messages: Vec::new(),
            flash_rx,
            help_scroll: 0,
            help_max_scroll: 0,
        };

        // Request initial image load for the first selected clip (if it's an image)
        app.request_image_load();

        Ok(app)
    }

    /// Poll flash message receiver and add to queue
    pub fn poll_flash_messages(&mut self) {
        if let Some(rx) = &self.flash_rx {
            while let Ok(msg) = rx.try_recv() {
                self.flash_messages.push(msg);
            }
        }
    }

    /// Remove expired flash messages (based on config duration)
    pub fn prune_flash_messages(&mut self) {
        if self.flash_messages.is_empty() {
            return;
        }

        let now = Instant::now();
        let duration_ms = self.config.general.flash_message_duration_ms;
        self.flash_messages
            .retain(|msg| now.duration_since(msg.timestamp).as_millis() < duration_ms as u128);
    }

    /// Clear all flash messages
    pub fn clear_flash_messages(&mut self) {
        self.flash_messages.clear();
    }

    /// Get the currently visible clip IDs (either search results or all history)
    /// Applies both search filtering and register filtering
    pub fn visible_clips(&self) -> Vec<u64> {
        let base_clips: Vec<u64> =
            if self.search_results.is_empty() && self.search_input.value().is_empty() {
                // Show all clips in chronological order (newest first)
                self.history.entries().iter().map(|e| e.id).collect()
            } else {
                // Show search results
                self.search_results.clone()
            };

        // Apply register filter if active
        match self.register_filter {
            RegisterFilter::None => base_clips,
            RegisterFilter::Temporary => base_clips
                .into_iter()
                .filter(|&id| {
                    if let Some(entry) = self.history.get_entry(id) {
                        !entry.temporary_registers.is_empty()
                    } else {
                        false
                    }
                })
                .collect(),
            RegisterFilter::Permanent => base_clips
                .into_iter()
                .filter(|&id| {
                    if let Some(entry) = self.history.get_entry(id) {
                        !entry.permanent_registers.is_empty()
                    } else {
                        false
                    }
                })
                .collect(),
            RegisterFilter::Pinned => base_clips
                .into_iter()
                .filter(|&id| {
                    if let Some(entry) = self.history.get_entry(id) {
                        entry.pinned
                    } else {
                        false
                    }
                })
                .collect(),
        }
    }

    /// Get the clip ID at the current selected index
    pub fn selected_clip_id(&self) -> Option<u64> {
        let visible = self.visible_clips();
        visible.get(self.selected_index).copied()
    }

    /// Calculate the number of entries for a half-page movement
    /// Takes into account the view mode (Comfortable uses 3 rows per entry)
    fn half_page_size(&self) -> usize {
        let rows_per_entry = match self.view_mode {
            ViewMode::Compact => 1,
            ViewMode::Comfortable => 3,
        };
        let visible_entries = (self.list_height as usize) / rows_per_entry;
        (visible_entries / 2).max(1) // At least 1
    }

    /// Calculate the number of entries for a full-page movement
    /// Takes into account the view mode (Comfortable uses 3 rows per entry)
    fn full_page_size(&self) -> usize {
        let rows_per_entry = match self.view_mode {
            ViewMode::Compact => 1,
            ViewMode::Comfortable => 3,
        };
        let visible_entries = (self.list_height as usize) / rows_per_entry;
        visible_entries.max(1) // At least 1
    }

    /// Update scroll offset to keep selected item visible with scroll padding
    /// Maintains 3 rows of context above/below selection (vim-style)
    fn update_scroll_offset(&mut self, total_visible_rows: usize) {
        const SCROLL_PADDING: usize = 3;

        let rows_per_entry = match self.view_mode {
            ViewMode::Compact => 1,
            ViewMode::Comfortable => 3, // Each entry takes 3 rows
        };

        // Calculate the row index of the selected item
        let selected_row = self.selected_index * rows_per_entry;

        // Calculate visible height in rows
        let visible_rows = self.list_height as usize;
        if visible_rows == 0 {
            return;
        }

        // Calculate the bottom visible row (exclusive)
        let scroll_bottom = self.list_scroll_offset + visible_rows;

        // Scroll down if selected item is too close to bottom. Add 1 to account for 2-row comfortable entries
        if selected_row + SCROLL_PADDING + 1 >= scroll_bottom {
            // Position the selected row SCROLL_PADDING rows from the bottom
            let target_offset = selected_row + SCROLL_PADDING + 2;
            if target_offset > visible_rows {
                self.list_scroll_offset = target_offset - visible_rows;
            }
        }
        // Scroll up if selected item is too close to top
        else if selected_row < self.list_scroll_offset + SCROLL_PADDING {
            // Position the selected row SCROLL_PADDING rows from the top
            self.list_scroll_offset = selected_row.saturating_sub(SCROLL_PADDING);
        }

        // Clamp scroll offset to valid range
        if total_visible_rows > visible_rows {
            let max_offset = total_visible_rows - visible_rows;
            self.list_scroll_offset = self.list_scroll_offset.min(max_offset);
        } else {
            self.list_scroll_offset = 0;
        }
    }

    /// Request async loading of the currently selected image
    /// If the selected clip is an image and not already cached, sends a load request
    fn request_image_load(&mut self) {
        if let Some(clip_id) = self.selected_clip_id() {
            // Check if already cached
            if self.image_cache.contains(&clip_id) {
                log::debug!("Image {} already cached", clip_id);
                return; // Already cached
            }

            // Check if this is an image clip
            if let Some(entry) = self.history.get_entry(clip_id)
                && let crate::models::ClipContent::Image { data, .. } = &entry.content
            {
                log::debug!("Requesting async load for clip {}", clip_id);
                // Send load request (non-blocking)
                let _ = self.image_load_tx.send(ImageLoadRequest {
                    clip_id,
                    image_data: data.clone(),
                });
            }
        }
    }

    /// Poll for completed image loads and update cache
    /// Should be called in the event loop before rendering
    pub fn update_image_cache(&mut self) {
        // Check for any completed image loads (non-blocking)
        while let Ok(result) = self.image_load_rx.try_recv() {
            if let Some(protocol_image) = result.protocol_image {
                log::debug!("Caching loaded image for clip {}", result.clip_id);
                // Add to LRU cache (automatically evicts least recently used if full)
                self.image_cache.put(result.clip_id, protocol_image);
            } else {
                log::warn!("Failed to load image for clip {}", result.clip_id);
                // Don't cache failed loads
            }
        }
    }

    /// Update search results based on current query
    pub fn update_search_results(&mut self) {
        let query = self.search_input.value();
        if query.is_empty() {
            self.search_results.clear();
            self.selected_index = 0;
            self.request_image_load();
            return;
        }

        // Perform fuzzy search
        let results = self.search_index.search(self.history.entries(), query);

        // Extract just the clip IDs
        self.search_results = results.into_iter().map(|(id, _score)| id).collect();

        // Reset selection to top of results
        self.selected_index = 0;
        self.request_image_load();
    }

    /// Select the currently highlighted entry and copy to clipboard
    pub fn select_entry(&mut self) -> Result<()> {
        let clip_id = self.selected_clip_id().context("No clip selected")?;

        let entry = self
            .history
            .get_entry(clip_id)
            .context("Clip not found in history")?;

        // Copy to clipboard using backend
        match &entry.content {
            crate::models::ClipContent::Text(text) => {
                self.clipboard_backend.write_text(text)?;
            }
            crate::models::ClipContent::Image { data, .. } => {
                self.clipboard_backend.write_image(data)?;
            }
            crate::models::ClipContent::File { .. } => {
                // For file references, we would copy the file path as text
                // This is a simplified implementation
                anyhow::bail!("File clipboard entries not yet supported for selection");
            }
        }

        // Exit if configured to do so
        if self.config.general.exit_on_select {
            self.should_quit = true;
        }

        Ok(())
    }

    /// Request paste after TUI exits
    ///
    /// Writes content to clipboard then simulates Ctrl-V to paste.
    /// Avoids shell escaping issues with special characters in text.
    pub fn paste_entry(&mut self) -> Result<()> {
        let clip_id = self.selected_clip_id().context("No clip selected")?;

        let entry = self
            .history
            .get_entry(clip_id)
            .context("Clip not found in history")?;

        self.paste_request = PasteRequest::PasteFromClipboard(entry.content.clone());
        self.should_quit = true;

        Ok(())
    }

    /// Toggle pin status of currently selected clip
    pub fn toggle_pin(&mut self) -> Result<()> {
        let clip_id = self.selected_clip_id().context("No clip selected")?;

        self.history.toggle_pin(clip_id)?;

        Ok(())
    }

    /// Delete the currently selected clip
    /// Cannot delete clips with permanent registers
    pub fn delete_entry(&mut self) -> Result<()> {
        let clip_id = self.selected_clip_id().context("No clip selected")?;

        // Check if clip can be deleted (no permanent registers)
        let entry = self
            .history
            .get_entry(clip_id)
            .context("Clip not found in history")?;

        if !entry.can_delete() {
            anyhow::bail!("Cannot delete clips with permanent registers");
        }

        // Remove any temporary register assignments first
        let temp_regs: Vec<char> = entry.temporary_registers.clone();
        for key in temp_regs {
            self.registers.remove_temporary(key, &mut self.history)?;
        }

        // Remove from history
        self.history.remove_entry(clip_id);

        // Adjust selection if needed
        let visible_count = self.visible_clips().len();
        if visible_count > 0 && self.selected_index >= visible_count {
            self.selected_index = visible_count - 1;
        }

        // Request image load for new selection
        self.request_image_load();

        Ok(())
    }

    /// Move selection up by n items
    pub fn move_up(&mut self, n: usize) {
        self.selected_index = self.selected_index.saturating_sub(n);
        self.request_image_load();
    }

    /// Move selection down by n items
    pub fn move_down(&mut self, n: usize) {
        let visible_count = self.visible_clips().len();
        if visible_count > 0 {
            self.selected_index = (self.selected_index + n).min(visible_count - 1);
        }
        self.request_image_load();
    }

    /// Jump to top of list
    pub fn jump_to_top(&mut self) {
        self.selected_index = 0;
        self.request_image_load();
    }

    /// Jump to bottom of list
    pub fn jump_to_bottom(&mut self) {
        let visible_count = self.visible_clips().len();
        if visible_count > 0 {
            self.selected_index = visible_count - 1;
        }
        self.request_image_load();
    }

    /// Jump to specific clip number (0-indexed)
    pub fn jump_to_number(&mut self, num: usize) {
        let visible_count = self.visible_clips().len();
        if num < visible_count {
            self.selected_index = num;
        }
        self.request_image_load();
    }

    /// Enter search mode
    pub fn enter_search_mode(&mut self) {
        // Enter search mode, keeping existing query if present
        // This allows re-entering search to refine the filter
        self.mode = AppMode::Search;
    }

    /// Exit search mode back to normal
    pub fn exit_search_mode(&mut self) {
        // Exit search mode but keep the search query and results
        // This allows users to navigate and manipulate filtered results
        self.mode = AppMode::Normal;
    }

    /// Clear search query and results, returning to full history
    pub fn clear_search(&mut self) {
        self.search_input.reset();
        self.search_results.clear();
        self.selected_index = 0;
        self.request_image_load();
    }

    /// Enter register assignment mode
    pub fn enter_register_mode(&mut self) {
        self.mode = AppMode::RegisterAssign;
        self.register_key = None;
    }

    /// Toggle temporary register assignment for current clip
    /// If the clip already has the register, remove it; otherwise add it
    pub fn assign_register(&mut self, key: char) -> Result<()> {
        let clip_id = self.selected_clip_id().context("No clip selected")?;

        // Check if the current clip already has this register
        let clip_has_register = self
            .history
            .get_entry(clip_id)
            .map(|clip| clip.temporary_registers.contains(&key))
            .unwrap_or(false);

        if clip_has_register {
            // Remove the register from this clip
            self.registers.remove_temporary(key, &mut self.history)?;
        } else {
            // Add to temporary registry (this updates both the registry and the clip)
            self.registers
                .assign_temporary(key, clip_id, &mut self.history)?;
        }

        // Exit register mode
        self.mode = AppMode::Normal;
        self.register_key = None;

        Ok(())
    }

    /// Toggle help overlay
    pub fn toggle_help(&mut self) {
        self.mode = match self.mode {
            AppMode::Help => AppMode::Normal,
            _ => {
                self.help_scroll = 0; // Reset scroll when entering help
                self.help_max_scroll = 0;
                AppMode::Help
            }
        };
    }

    /// Toggle temporary register filter
    pub fn toggle_temporary_filter(&mut self) {
        self.register_filter = match self.register_filter {
            RegisterFilter::Temporary => RegisterFilter::None,
            _ => RegisterFilter::Temporary,
        };
        self.selected_index = 0; // Reset selection when filter changes
        self.request_image_load();
    }

    /// Toggle permanent register filter
    pub fn toggle_permanent_filter(&mut self) {
        self.register_filter = match self.register_filter {
            RegisterFilter::Permanent => RegisterFilter::None,
            _ => RegisterFilter::Permanent,
        };
        self.selected_index = 0; // Reset selection when filter changes
        self.request_image_load();
    }

    /// Toggle pinned clips filter
    pub fn toggle_pinned_filter(&mut self) {
        self.register_filter = match self.register_filter {
            RegisterFilter::Pinned => RegisterFilter::None,
            _ => RegisterFilter::Pinned,
        };
        self.selected_index = 0; // Reset selection when filter changes
        self.request_image_load();
    }

    /// Toggle between Compact and Comfortable view modes
    pub fn toggle_view_mode(&mut self) {
        self.view_mode = match self.view_mode {
            ViewMode::Compact => ViewMode::Comfortable,
            ViewMode::Comfortable => ViewMode::Compact,
        };
    }

    /// Enter confirmation mode for clear all
    pub fn enter_confirm_clear_all(&mut self) {
        self.mode = AppMode::Confirm;
    }

    /// Clear all unpinned, non-registered entries
    pub fn clear_all_unpinned(&mut self) {
        self.history.clear_unpinned();
        self.selected_index = 0;
        self.mode = AppMode::Normal;
    }

    /// Cancel confirmation and return to normal mode
    pub fn cancel_confirm(&mut self) {
        self.mode = AppMode::Normal;
    }

    /// Request application exit
    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    /// Reload theme from config file
    /// Performs atomic swap: load → validate → apply only if valid
    /// On error, displays error modal and keeps previous theme
    pub fn reload_theme(&mut self) -> Result<()> {
        log::info!("Reloading theme: {}", self.current_theme_name);

        match Theme::load(&self.current_theme_name) {
            Ok(new_theme) => {
                // Atomic swap - only replace if load succeeded
                self.theme = new_theme;
                // Clear any previous error
                self.startup_error = None;
                log::info!("Theme reloaded successfully");
                Ok(())
            }
            Err(e) => {
                // Keep previous theme, show error modal
                let error_msg = format!(
                    "Failed to reload theme '{}':\n{}",
                    self.current_theme_name, e
                );
                log::error!("{}", error_msg);
                self.startup_error = Some(error_msg);
                Err(e)
            }
        }
    }

    /// Check for theme file changes and auto-reload if in development mode
    /// Called from main event loop before rendering
    /// Non-blocking check using try_recv()
    pub fn check_theme_reload(&mut self) {
        // Only check if watcher is active
        if let Some(ref rx) = self.theme_watch_rx {
            // Drain all pending events (multiple events can queue up)
            let mut has_changes = false;

            while let Ok(result) = rx.try_recv() {
                match result {
                    Ok(event) => {
                        // Check if this is a modify event for the theme file
                        if matches!(event.kind, notify::EventKind::Modify(_)) {
                            log::debug!("Theme file changed: {:?}", event.paths);
                            has_changes = true;
                        }
                    }
                    Err(e) => {
                        log::warn!("File watcher error: {}", e);
                    }
                }
            }

            // Reload theme if changes detected
            if has_changes {
                log::info!("Auto-reloading theme due to file changes");
                let _ = self.reload_theme();
            }
        }
    }

    /// Handle keyboard event based on current mode
    pub fn handle_key(&mut self, key: KeyEvent) -> Result<()> {
        // If there's a startup error modal, any key dismisses it
        if self.startup_error.is_some() {
            self.startup_error = None;
            return Ok(());
        }

        match self.mode {
            AppMode::Normal => self.handle_normal_key(key),
            AppMode::Search => self.handle_search_key(key),
            AppMode::RegisterAssign => self.handle_register_key(key),
            AppMode::Confirm => self.handle_confirm_key(key),
            AppMode::Help => self.handle_help_key(key),
            AppMode::Numeric => self.handle_numeric_key(key),
            AppMode::ThemePicker => self.handle_theme_picker_key(key),
        }
    }

    /// Handle keys in normal mode (vim-style navigation)
    fn handle_normal_key(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            // Entering a digit starts Numeric mode
            KeyCode::Char(c) if c.is_ascii_digit() && key.modifiers.is_empty() => {
                self.numeric_prefix.push(c);
                self.mode = AppMode::Numeric;
            }

            // Vim navigation (simple - no numeric prefix in Normal mode)
            KeyCode::Char('j') | KeyCode::Down => {
                self.move_down(1);
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.move_up(1);
            }
            KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                let count = self.half_page_size();
                self.move_down(count);
            }
            KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                let count = self.half_page_size();
                self.move_up(count);
            }
            KeyCode::Char('r') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                // Reload theme from config file
                let _ = self.reload_theme();
            }

            // Home/End for jump to top/bottom (replacing gg/G)
            KeyCode::Home => {
                self.jump_to_top();
            }
            KeyCode::End => {
                self.jump_to_bottom();
            }

            // PageUp/PageDown
            KeyCode::PageUp => {
                let count = self.full_page_size();
                self.move_up(count);
            }
            KeyCode::PageDown => {
                let count = self.full_page_size();
                self.move_down(count);
            }

            // Actions
            KeyCode::Enter => {
                self.select_entry()?;
            }
            KeyCode::Char(' ') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                // Ctrl-Space - paste entry via Ctrl-V simulation
                self.paste_entry()?;
            }
            KeyCode::Char('m') => {
                self.enter_register_mode();
            }
            KeyCode::Char('p') => {
                self.toggle_pin()?;
            }
            KeyCode::Char('P') => {
                self.toggle_pinned_filter();
            }
            KeyCode::Char('c') => {
                self.clear_flash_messages();
            }
            KeyCode::Char('L') => {
                // Test flash messages at different levels (capital L)
                log::trace!("Test trace message");
                log::debug!("Test debug message");
                log::info!("Test info message");
                log::warn!("Test warning message");
                log::error!("Test error message");
            }
            KeyCode::Char('/') => {
                self.enter_search_mode();
            }
            KeyCode::Char('?') => {
                self.toggle_help();
            }
            KeyCode::Char('\'') => {
                self.toggle_temporary_filter();
            }
            KeyCode::Char('"') => {
                self.toggle_permanent_filter();
            }
            KeyCode::Char('v') => {
                self.toggle_view_mode();
            }
            KeyCode::Char('t') if key.modifiers.contains(KeyModifiers::ALT) => {
                // Alt-T - save current theme as default
                let _ = self.save_theme_as_default();
            }
            KeyCode::Char('t') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                // Ctrl-T - cycle to next theme
                self.cycle_theme();
            }
            KeyCode::Char('T') => {
                // Capital T - open theme picker
                self.open_theme_picker();
            }
            KeyCode::Char('d') => {
                // Delete entry - silently ignore errors (e.g., can't delete permanent register clips)
                let _ = self.delete_entry();
            }
            KeyCode::Char('D') => {
                self.enter_confirm_clear_all();
            }
            KeyCode::Char('q') => {
                self.quit();
            }
            KeyCode::Esc => {
                // ESC clears filters in order: search filter, register filter, then quit
                if !self.search_input.value().is_empty() {
                    self.clear_search();
                } else if self.register_filter != RegisterFilter::None {
                    self.register_filter = RegisterFilter::None;
                    self.selected_index = 0;
                } else {
                    self.quit();
                }
            }

            _ => {
                // Unknown keys do nothing in Normal mode
            }
        }
        Ok(())
    }

    /// Handle keys in search mode
    fn handle_search_key(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Up => {
                self.move_up(1);
            }
            KeyCode::Down => {
                self.move_down(1);
            }
            KeyCode::Esc => {
                self.exit_search_mode();
            }
            KeyCode::Enter => {
                self.mode = AppMode::Normal;
                self.select_entry()?;
            }
            KeyCode::Char(' ') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                // Ctrl-Space - paste entry via Ctrl-V simulation
                self.paste_entry()?;
            }
            _ => {
                // Delegate all other keys to tui-input (characters, backspace,
                // delete, left/right arrows, Ctrl+A/E, Ctrl+W, etc.)
                let event = Event::Key(key);
                if self.search_input.handle_event(&event).is_some() {
                    self.update_search_results();
                }
            }
        }
        Ok(())
    }

    /// Handle keys in register assignment mode
    fn handle_register_key(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Char(c) if c.is_alphanumeric() => {
                self.assign_register(c)?;
            }
            KeyCode::Esc => {
                self.mode = AppMode::Normal;
                self.register_key = None;
            }
            _ => {}
        }
        Ok(())
    }

    /// Handle keys in confirmation mode
    fn handle_confirm_key(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Char('y') | KeyCode::Char('Y') => {
                self.clear_all_unpinned();
            }
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                self.cancel_confirm();
            }
            _ => {}
        }
        Ok(())
    }

    /// Handle keys in help mode
    fn handle_help_key(&mut self, key: KeyEvent) -> Result<()> {
        const HELP_HALF_PAGE: usize = 5;
        const HELP_FULL_PAGE: usize = 10;

        match key.code {
            // Line-by-line navigation with bounds checking
            KeyCode::Char('j') | KeyCode::Down => {
                if self.help_scroll < self.help_max_scroll {
                    self.help_scroll += 1;
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                if self.help_scroll > 0 {
                    self.help_scroll -= 1;
                }
            }
            // Page navigation with clamping
            KeyCode::PageDown => {
                self.help_scroll = (self.help_scroll + HELP_FULL_PAGE).min(self.help_max_scroll);
            }
            KeyCode::PageUp => {
                self.help_scroll = self.help_scroll.saturating_sub(HELP_FULL_PAGE);
            }
            // Half-page navigation with clamping
            KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.help_scroll = (self.help_scroll + HELP_HALF_PAGE).min(self.help_max_scroll);
            }
            KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.help_scroll = self.help_scroll.saturating_sub(HELP_HALF_PAGE);
            }
            // Jump to top/bottom
            KeyCode::Home => {
                self.help_scroll = 0;
            }
            KeyCode::End => {
                self.help_scroll = self.help_max_scroll;
            }
            // Close help with '?' or Esc
            KeyCode::Char('?') | KeyCode::Esc => {
                self.mode = AppMode::Normal;
            }
            // Any other key also closes help (for backward compatibility)
            _ => {
                self.mode = AppMode::Normal;
            }
        }
        Ok(())
    }

    /// Handle keys in numeric mode (command palette with numeric prefix)
    fn handle_numeric_key(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            // Additional digits extend the prefix
            KeyCode::Char(c) if c.is_ascii_digit() => {
                self.numeric_prefix.push(c);
            }

            // Commands that use the numeric prefix
            KeyCode::Char('j') | KeyCode::Down => {
                let count = self.numeric_prefix.parse::<usize>().unwrap_or(1);
                self.move_down(count);
                self.numeric_prefix.clear();
                self.mode = AppMode::Normal;
            }
            KeyCode::Char('k') | KeyCode::Up => {
                let count = self.numeric_prefix.parse::<usize>().unwrap_or(1);
                self.move_up(count);
                self.numeric_prefix.clear();
                self.mode = AppMode::Normal;
            }
            KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                let multiplier = self.numeric_prefix.parse::<usize>().unwrap_or(1);
                let count = self.half_page_size() * multiplier;
                self.move_down(count);
                self.numeric_prefix.clear();
                self.mode = AppMode::Normal;
            }
            KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                let multiplier = self.numeric_prefix.parse::<usize>().unwrap_or(1);
                let count = self.half_page_size() * multiplier;
                self.move_up(count);
                self.numeric_prefix.clear();
                self.mode = AppMode::Normal;
            }
            KeyCode::Enter => {
                // Enter jumps to the typed number
                let count = self.numeric_prefix.parse::<usize>().unwrap_or(0);
                self.jump_to_number(count);
                self.numeric_prefix.clear();
                self.mode = AppMode::Normal;
            }
            KeyCode::Esc => {
                // Escape cancels numeric mode
                self.numeric_prefix.clear();
                self.mode = AppMode::Normal;
            }

            _ => {
                // Any other key cancels numeric mode
                self.numeric_prefix.clear();
                self.mode = AppMode::Normal;
            }
        }
        Ok(())
    }

    /// Handle keys in theme picker mode
    fn handle_theme_picker_key(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Esc => {
                self.mode = AppMode::Normal;
            }
            KeyCode::Enter => {
                self.select_theme_from_picker();
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if self.theme_picker_selected > 0 {
                    self.theme_picker_selected -= 1;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.theme_picker_selected + 1 < self.theme_picker_themes.len() {
                    self.theme_picker_selected += 1;
                }
            }
            KeyCode::Home | KeyCode::Char('g') => {
                self.theme_picker_selected = 0;
            }
            KeyCode::End | KeyCode::Char('G') => {
                self.theme_picker_selected = self.theme_picker_themes.len().saturating_sub(1);
            }
            _ => {}
        }
        Ok(())
    }

    /// Cycle to the next available theme
    pub fn cycle_theme(&mut self) {
        let themes = Theme::get_all_theme_names();
        if themes.is_empty() {
            return;
        }

        let current_idx = themes
            .iter()
            .position(|t| t == &self.current_theme_name)
            .unwrap_or(0);
        let next_idx = (current_idx + 1) % themes.len();
        let next_theme_name = &themes[next_idx];

        match Theme::load(next_theme_name) {
            Ok(theme) => {
                self.theme = theme;
                self.current_theme_name = next_theme_name.clone();
            }
            Err(e) => {
                self.startup_error =
                    Some(format!("Failed to load theme '{}': {}", next_theme_name, e));
            }
        }
    }

    /// Save current theme as default in config file
    pub fn save_theme_as_default(&mut self) -> Result<()> {
        // Update config with current theme
        self.config.general.theme = self.current_theme_name.clone();

        // Write config to file
        self.config.save()?;

        Ok(())
    }

    /// Open theme picker modal
    pub fn open_theme_picker(&mut self) {
        self.theme_picker_themes = Theme::get_all_theme_names();
        // Find current theme index
        self.theme_picker_selected = self
            .theme_picker_themes
            .iter()
            .position(|t| t == &self.current_theme_name)
            .unwrap_or(0);
        self.mode = AppMode::ThemePicker;
    }

    /// Select theme from picker
    pub fn select_theme_from_picker(&mut self) {
        if self.theme_picker_selected < self.theme_picker_themes.len() {
            let theme_name = &self.theme_picker_themes[self.theme_picker_selected];
            match Theme::load(theme_name) {
                Ok(theme) => {
                    self.theme = theme;
                    self.current_theme_name = theme_name.clone();
                    self.mode = AppMode::Normal;
                }
                Err(e) => {
                    self.startup_error =
                        Some(format!("Failed to load theme '{}': {}", theme_name, e));
                    self.mode = AppMode::Normal;
                }
            }
        }
    }

    /// Render the TUI
    pub fn draw(&mut self, frame: &mut Frame) {
        let size = frame.area();

        // Set themed background for entire frame
        frame.render_widget(
            ratatui::widgets::Block::default()
                .style(ratatui::prelude::Style::default().bg(self.theme.default_bg)),
            size,
        );

        // Create layout: [clip_list, divider, preview, keyboard_hints]
        let chunks = ui::create_main_layout(size, self.view_mode);
        let clip_list_area = chunks[0];
        let divider_area = chunks[1];
        let preview_area = chunks[2];
        let keyboard_hints_area = chunks[3];

        // Update list height for page movement calculations
        // Account for header height based on view mode
        let header_height = match self.view_mode {
            ViewMode::Comfortable => 4,
            ViewMode::Compact => 1,
        };
        self.list_height = clip_list_area.height.saturating_sub(header_height);

        // Get visible clip count for scroll calculation
        let visible_clip_ids = self.visible_clips();
        let visible_count = visible_clip_ids.len();

        // Calculate total rows needed for all entries
        let rows_per_entry = match self.view_mode {
            ViewMode::Compact => 1,
            ViewMode::Comfortable => 3,
        };
        let total_visible_rows = visible_count * rows_per_entry;

        // Update scroll offset to maintain padding (must happen before borrowing entries)
        self.update_scroll_offset(total_visible_rows);

        // Get visible entries for rendering (after scroll offset update)
        let visible_entries: Vec<&crate::models::ClipEntry> = visible_clip_ids
            .iter()
            .filter_map(|&id| self.history.get_entry(id))
            .collect();

        // Render clip list (with inline search or numeric prefix display)
        ui::render_clip_list(
            frame,
            clip_list_area,
            &visible_entries,
            ui::clip_list::ClipListRenderContext {
                selected: self.selected_index,
                mode: self.mode,
                search_input: &self.search_input,
                numeric_prefix: &self.numeric_prefix,
                register_filter: self.register_filter,
                view_mode: self.view_mode,
                scroll_offset: self.list_scroll_offset,
                theme: &self.theme,
            },
        );

        // Render divider between history and preview
        ui::render_divider(frame, divider_area, self.view_mode, &self.theme);

        // Render preview for selected clip
        let selected_entry = self
            .selected_clip_id()
            .and_then(|id| self.history.get_entry(id));

        // Get cached image if available for current selection
        // peek() doesn't update LRU order, get_mut() does
        let cached_image = if let Some(clip_id) = self.selected_clip_id() {
            self.image_cache.get_mut(&clip_id)
        } else {
            None
        };

        ui::render_preview(
            frame,
            preview_area,
            selected_entry,
            cached_image,
            self.config.general.show_preview_metadata,
            &self.flash_messages,
            &self.theme,
        );

        // Render mode-specific keyboard hints
        ui::render_keyboard_hints(frame, keyboard_hints_area, self, &self.theme);

        // Render help overlay if in help mode
        if matches!(self.mode, AppMode::Help) {
            (self.help_scroll, self.help_max_scroll) =
                ui::render_help_overlay(frame, size, &self.theme, self.help_scroll);
        }

        // Render theme picker if in theme picker mode
        if matches!(self.mode, AppMode::ThemePicker) {
            ui::render_theme_picker(
                frame,
                size,
                &self.theme_picker_themes,
                self.theme_picker_selected,
                &self.current_theme_name,
                &self.theme,
            );
        }

        // Render confirmation dialog if in confirm mode
        if matches!(self.mode, AppMode::Confirm) {
            ui::render_confirm_overlay(frame, size, &self.theme);
        }

        // Render startup error modal if present (takes precedence over other overlays)
        if let Some(ref error_msg) = self.startup_error {
            ui::render_error_modal(frame, size, error_msg, &self.theme);
        }
    }
}
