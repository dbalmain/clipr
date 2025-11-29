# Contract: UI Component Interface

**Feature**: 001-clipboard-manager-tui
**Component**: `src/ui/`
**Purpose**: Define TUI component interfaces and interaction contracts

## Architecture Overview

The UI follows the **Elm Architecture** pattern (Model-View-Update):

```
┌─────────┐         ┌────────┐         ┌──────┐
│  Event  │────────>│ Update │────────>│ Model│
└─────────┘         └────────┘         └──────┘
                                           │
                                           v
                                       ┌──────┐
                                       │ View │
                                       └──────┘
                                           │
                                           v
                                       ┌────────┐
                                       │Terminal│
                                       └────────┘
```

---

## Core Components

### App (Main State Machine)

**Location**: `src/app.rs`

```rust
use ratatui::backend::Backend;
use ratatui::Terminal;
use crossterm::event::{self, Event, KeyCode, KeyEvent};

/// Main application state
pub struct App {
    /// Current UI mode
    mode: AppMode,

    /// Clipboard history
    history: Arc<RwLock<ClipboardHistory>>,

    /// Temporary registers
    registers: HashMap<char, Register>,

    /// Configuration
    config: Config,

    /// Current filter (if any)
    filter: Option<EntryFilter>,

    /// Selected entry index
    selected_index: usize,

    /// Search query (when in search mode)
    search_query: String,

    /// Whether to exit
    should_quit: bool,
}

/// Application modes (state machine)
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AppMode {
    /// Normal browsing mode
    Normal,

    /// Search/filter mode (/ pressed)
    Search,

    /// Register assignment mode (m pressed, waiting for letter)
    RegisterAssign,

    /// Help overlay
    Help,
}

impl App {
    /// Handle keyboard event
    pub fn handle_key(&mut self, key: KeyEvent) -> Result<()> {
        match self.mode {
            AppMode::Normal => self.handle_normal_key(key),
            AppMode::Search => self.handle_search_key(key),
            AppMode::RegisterAssign => self.handle_register_key(key),
            AppMode::Help => self.handle_help_key(key),
        }
    }

    /// Handle normal mode keys (vim-style)
    fn handle_normal_key(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Char('j') => self.move_selection_down(),
            KeyCode::Char('k') => self.move_selection_up(),
            KeyCode::Char('g') if key.modifiers.is_empty() => self.move_to_top(),
            KeyCode::Char('G') => self.move_to_bottom(),
            KeyCode::Char('/') => self.enter_search_mode(),
            KeyCode::Char('\'') => self.filter_temporary_registers(),
            KeyCode::Char('"') => self.filter_permanent_registers(),
            KeyCode::Char('m') => self.enter_register_mode(),
            KeyCode::Char('p') => self.toggle_pin(),
            KeyCode::Char('d') => self.delete_entry(),
            KeyCode::Char('?') => self.show_help(),
            KeyCode::Enter => self.select_entry(),
            KeyCode::Esc => self.clear_filter(),
            KeyCode::Char('q') => self.quit(),
            _ => {}
        }
        Ok(())
    }

    /// Handle search mode keys
    fn handle_search_key(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Char(c) => {
                self.search_query.push(c);
                self.update_search_results();
            }
            KeyCode::Backspace => {
                self.search_query.pop();
                self.update_search_results();
            }
            KeyCode::Up | KeyCode::Down => {
                // Navigate results without exiting search mode
                if key.code == KeyCode::Down {
                    self.move_selection_down();
                } else {
                    self.move_selection_up();
                }
            }
            KeyCode::Esc => {
                if self.filter.is_some() {
                    // First escape: exit search input, keep filter
                    self.mode = AppMode::Normal;
                    self.selected_index = 0;
                } else {
                    // Second escape: clear filter
                    self.search_query.clear();
                    self.mode = AppMode::Normal;
                }
            }
            KeyCode::Enter => {
                self.mode = AppMode::Normal;
                self.select_entry();
            }
            _ => {}
        }
        Ok(())
    }

    /// Render UI
    pub fn draw<B: Backend>(&self, terminal: &mut Terminal<B>) -> Result<()> {
        terminal.draw(|f| {
            let chunks = self.create_layout(f.size());

            // Render clip list
            self.render_clip_list(f, chunks[0]);

            // Render preview panel
            self.render_preview(f, chunks[1]);

            // Render status bar
            self.render_status(f, chunks[2]);

            // Render search input if in search mode
            if matches!(self.mode, AppMode::Search) {
                self.render_search_input(f);
            }

            // Render help overlay if active
            if matches!(self.mode, AppMode::Help) {
                self.render_help_overlay(f);
            }
        })?;

        Ok(())
    }
}
```

**Performance Requirements**:
- Frame render time: <16ms (60fps)
- Key event processing: <1ms
- Filter/search update: <100ms for 1000 entries

---

### Layout Manager

**Location**: `src/ui/layout.rs`

```rust
use ratatui::layout::{Constraint, Direction, Layout, Rect};

/// Create side-by-side layout for clip list and preview
pub fn create_main_layout(area: Rect) -> Vec<Rect> {
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(3),      // Main content area
            Constraint::Length(1),   // Status bar
        ])
        .split(area);

    let content_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(40),  // Clip list (left)
            Constraint::Percentage(60),  // Preview (right)
        ])
        .split(main_chunks[0]);

    vec![content_chunks[0], content_chunks[1], main_chunks[1]]
}

/// Handle terminal resize
pub fn handle_resize(area: Rect) -> Vec<Rect> {
    if area.width < 80 || area.height < 20 {
        // Minimal layout for small terminals
        create_minimal_layout(area)
    } else {
        create_main_layout(area)
    }
}
```

---

### ClipList Widget

**Location**: `src/ui/clip_list.rs`

```rust
use ratatui::widgets::{Block, Borders, List, ListItem, ListState};
use ratatui::style::{Color, Modifier, Style};

/// Render clip list with vim-style selection
pub fn render_clip_list(
    frame: &mut Frame,
    area: Rect,
    entries: &[ClipEntry],
    selected: usize,
    filter: &Option<EntryFilter>,
) {
    // Create list items with indicators
    let items: Vec<ListItem> = entries
        .iter()
        .enumerate()
        .map(|(i, entry)| {
            let mut indicators = String::new();

            if entry.pinned {
                indicators.push('📌');
            }
            if entry.register.is_some() {
                indicators.push('🔖');
            }

            let preview = entry.preview(60);
            let timestamp = format_timestamp(entry.timestamp);

            let content = format!("{} {} {}", indicators, timestamp, preview);

            let style = if i == selected {
                Style::default()
                    .bg(Color::Blue)
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            ListItem::new(content).style(style)
        })
        .collect();

    let title = match filter {
        Some(EntryFilter::Search(q)) => format!("Search: {}", q),
        Some(EntryFilter::TemporaryRegisters) => "Temporary Registers".to_string(),
        Some(EntryFilter::PermanentRegisters) => "Permanent Registers".to_string(),
        Some(EntryFilter::PinnedOnly) => "Pinned Clips".to_string(),
        _ => format!("Clipboard History ({} entries)", entries.len()),
    };

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(title))
        .highlight_symbol("► ");

    let mut list_state = ListState::default();
    list_state.select(Some(selected));

    frame.render_stateful_widget(list, area, &mut list_state);
}
```

**Virtualization** (for 1000+ items):
```rust
/// Only render visible entries (viewport optimization)
pub fn get_visible_range(total: usize, viewport_height: usize, selected: usize) -> (usize, usize) {
    let start = selected.saturating_sub(viewport_height / 2);
    let end = (start + viewport_height).min(total);
    (start, end)
}
```

---

### Preview Widget

**Location**: `src/ui/preview.rs`

```rust
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

/// Render preview panel for selected entry
pub fn render_preview(
    frame: &mut Frame,
    area: Rect,
    entry: Option<&ClipEntry>,
    image_renderer: &Option<Box<dyn ImageProtocol>>,
) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title("Preview");

    if let Some(entry) = entry {
        match &entry.content {
            ClipContent::Text(text) => {
                let paragraph = Paragraph::new(text.as_str())
                    .block(block)
                    .wrap(Wrap { trim: false });

                frame.render_widget(paragraph, area);
            }
            ClipContent::Image(data) => {
                if let Some(renderer) = image_renderer {
                    // Render image using kitty/sixel protocol
                    render_image(frame, area, data, renderer);
                } else {
                    let msg = Paragraph::new("[Image preview not available]")
                        .block(block);
                    frame.render_widget(msg, area);
                }
            }
        }
    } else {
        let msg = Paragraph::new("No selection")
            .block(block);
        frame.render_widget(msg, area);
    }
}

/// Render image using protocol (async to avoid blocking)
fn render_image(
    frame: &mut Frame,
    area: Rect,
    data: &[u8],
    renderer: &Box<dyn ImageProtocol>,
) {
    // Load image
    let img = match image::load_from_memory(data) {
        Ok(img) => img,
        Err(e) => {
            render_error(frame, area, format!("Failed to load image: {}", e));
            return;
        }
    };

    // Render via protocol (viuer handles terminal detection)
    let config = viuer::Config {
        width: Some((area.width as u32).saturating_sub(2)),
        height: Some((area.height as u32).saturating_sub(2)),
        absolute_offset: false,
        ..Default::default()
    };

    if let Err(e) = viuer::print(&img, &config) {
        render_error(frame, area, format!("Image render failed: {}", e));
    }
}
```

---

### Search Input

**Location**: `src/ui/search.rs`

```rust
/// Render search input bar at bottom of screen
pub fn render_search_input(frame: &mut Frame, area: Rect, query: &str) {
    let input_area = Rect {
        x: area.x,
        y: area.y + area.height - 3,
        width: area.width,
        height: 3,
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .title("Search (/ to start, Esc to clear)");

    let input = Paragraph::new(format!("/{}", query))
        .block(block)
        .style(Style::default().fg(Color::Yellow));

    frame.render_widget(input, input_area);
}
```

---

### Help Overlay

**Location**: `src/ui/help.rs`

```rust
/// Render help overlay with keybindings
pub fn render_help_overlay(frame: &mut Frame, area: Rect) {
    let help_text = r#"
CLIPR KEYBINDINGS

Navigation:
  j/k         Move down/up
  gg/G        Jump to top/bottom
  Ctrl-d/u    Half-page down/up

Search & Filter:
  /           Start fuzzy search
  '           Show temporary registers
  "           Show permanent registers
  Esc         Clear filter (press twice in search)

Actions:
  Enter       Copy to clipboard and exit
  m<letter>   Assign to register
  p           Toggle pin
  d           Delete entry
  q           Quit

Help:
  ?           Show this help
  Esc         Close help

Press any key to close...
"#;

    let overlay_area = centered_rect(60, 80, area);

    let paragraph = Paragraph::new(help_text)
        .block(Block::default()
            .borders(Borders::ALL)
            .title("Help")
            .style(Style::default().bg(Color::Black)))
        .style(Style::default().fg(Color::White))
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, overlay_area);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
```

---

### Status Bar

**Location**: `src/ui/status.rs`

```rust
/// Render status bar at bottom
pub fn render_status(frame: &mut Frame, area: Rect, app: &App) {
    let mode_str = match app.mode {
        AppMode::Normal => "NORMAL",
        AppMode::Search => "SEARCH",
        AppMode::RegisterAssign => "REGISTER",
        AppMode::Help => "HELP",
    };

    let stats = app.history.read().unwrap().stats();

    let status = format!(
        " {} | Total: {} | Pinned: {} | Registered: {} | Images: {} ",
        mode_str, stats.total, stats.pinned, stats.registered, stats.images
    );

    let paragraph = Paragraph::new(status)
        .style(Style::default().bg(Color::DarkGray).fg(Color::White));

    frame.render_widget(paragraph, area);
}
```

---

## Event Loop

**Location**: `src/main.rs`

```rust
fn main() -> Result<()> {
    // Initialize terminal
    let mut terminal = setup_terminal()?;

    // Create app state
    let mut app = App::new()?;

    // Main event loop
    loop {
        // Render
        app.draw(&mut terminal)?;

        // Handle events (with timeout for responsive UI)
        if event::poll(Duration::from_millis(16))? {  // 60fps
            if let Event::Key(key) = event::read()? {
                app.handle_key(key)?;
            }
        }

        // Check clipboard changes (via channel from monitor thread)
        if let Ok(content) = clipboard_rx.try_recv() {
            app.add_clipboard_entry(content)?;
        }

        // Exit check
        if app.should_quit {
            break;
        }
    }

    // Cleanup
    restore_terminal(&mut terminal)?;
    app.save_state()?;

    Ok(())
}
```

---

## Performance Contracts

### Render Performance
- Full frame render: <16ms (60fps)
- List virtualization: Only render visible items
- Image rendering: Async, non-blocking

### Input Latency
- Key event → state update: <1ms
- State update → re-render: <16ms
- Total perceived latency: <20ms

### Search Performance
- Fuzzy search 1000 items: <100ms (nucleo)
- UI update after search: <16ms

---

## Testing Contract

### Unit Tests
```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_vim_navigation() {
        let mut app = App::new_test();
        app.handle_key(key('j'));
        assert_eq!(app.selected_index, 1);

        app.handle_key(key('k'));
        assert_eq!(app.selected_index, 0);
    }

    #[test]
    fn test_search_mode_transition() {
        let mut app = App::new_test();
        app.handle_key(key('/'));
        assert_eq!(app.mode, AppMode::Search);

        app.handle_key(key_esc());
        assert_eq!(app.mode, AppMode::Normal);
    }
}
```

### Integration Tests
```rust
#[test]
fn test_full_interaction_flow() {
    // Test: launch → search → select → copy to clipboard
}
```

---

## Dependencies

```toml
[dependencies]
ratatui = "0.28"
crossterm = "0.28"
viuer = "0.10"
image = "0.25"
```

---

## Future Enhancements

1. **Mouse support**: Optional mouse clicks for selection
2. **Custom themes**: User-configurable colors
3. **Multi-select**: Select multiple entries for batch operations
4. **Preview scrolling**: Scroll long previews independently

**Contract Complete** ✅
