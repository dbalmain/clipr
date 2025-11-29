# Quickstart Guide: Clipboard Manager TUI

**Feature**: 001-clipboard-manager-tui
**Date**: 2025-11-24
**Audience**: Developers joining the project

## Overview

Clipr is a high-performance TUI for managing clipboard history on Linux. Built in Rust with a focus on speed (<100ms cold start), simplicity (YAGNI), and vim-style interaction.

**Key Features**:
- Fuzzy search through clipboard history (fzf-style)
- Side-by-side preview panel for images and text
- Vim-style keybindings with mark/jump analogy for registers
- Event-driven clipboard monitoring (via wl-paste --watch)
- Persistent pinning and named registers (temporary + permanent)

---

## Quick Start

### Prerequisites

#### Option 1: Nix Flakes (Recommended)

If you have [Nix](https://nixos.org/download.html) with [flakes enabled](https://wiki.nixos.org/wiki/Flakes):

```bash
# Clone repository
git clone <repo-url>
cd clipr

# Enter development shell (installs all dependencies automatically)
nix develop
```

The flake provides Rust, wl-clipboard, and all development tools pre-configured.

#### Option 2: Manual Installation

```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install wl-clipboard (Wayland)
sudo apt install wl-clipboard  # Debian/Ubuntu
sudo pacman -S wl-clipboard    # Arch
sudo dnf install wl-clipboard  # Fedora

# Verify installation
wl-paste --version
```

### Build and Run

```bash
# Clone repository
git clone <repo-url>
cd clipr

# Build release binary
cargo build --release

# Run daemon (background clipboard monitoring)
./target/release/clipr --daemon &

# Copy some text/images to test
echo "test clipboard entry" | wl-copy

# Launch TUI to browse history
./target/release/clipr

# Or run directly with cargo
cargo run --release
```

### First Time Setup

1. **Config file** is created automatically at `~/.config/clipr/config.toml`
2. **Data directory** is created at `~/.local/share/clipr/`
3. **Edit config** to add permanent registers:

```bash
# Edit config
$EDITOR ~/.config/clipr/config.toml

# Example permanent registers:
[[registers]]
name = "e"
content = "your.email@example.com"

[[registers]]
name = "w"
content = "123 Main St, City, State"
```

---

## Architecture Overview

### Project Structure

```
clipr/
├── src/
│   ├── main.rs              # Entry point, mode dispatch
│   ├── app.rs               # Main TUI application state
│   ├── ui/                  # TUI components
│   │   ├── mod.rs
│   │   ├── layout.rs        # Panel layout management
│   │   ├── clip_list.rs     # Clip selector widget
│   │   ├── preview.rs       # Preview panel widget
│   │   ├── search.rs        # Search input widget
│   │   └── keybindings.rs   # Vim-style key handlers
│   ├── clipboard/           # Clipboard integration
│   │   ├── mod.rs
│   │   ├── backend.rs       # ClipboardBackend trait
│   │   ├── wayland.rs       # wl-clipboard integration
│   │   └── monitor.rs       # wl-paste --watch daemon
│   ├── storage/             # Persistence layer
│   │   ├── mod.rs
│   │   ├── history.rs       # Bincode history storage
│   │   ├── registers.rs     # Bincode register storage
│   │   └── config.rs        # TOML config parser
│   ├── models/              # Core data structures
│   │   ├── mod.rs
│   │   ├── clip.rs          # ClipEntry, ClipContent
│   │   ├── register.rs      # Register types
│   │   └── search_index.rs  # Nucleo fuzzy search
│   ├── image/               # Image display
│   │   ├── mod.rs
│   │   ├── protocol.rs      # ImageProtocol trait
│   │   └── kitty.rs         # Kitty graphics via viuer
│   └── lib.rs               # Library exports for testing
├── tests/
│   ├── integration/         # Integration tests
│   └── unit/                # Unit tests (mirror src/)
├── benches/                 # Criterion benchmarks
│   ├── startup.rs
│   ├── search.rs
│   └── clipboard_ops.rs
├── Cargo.toml               # Dependencies and metadata
└── clipr.toml.example       # Example config file
```

### Data Flow

```
┌─────────────────────────────────────────────────────┐
│                   User copies text                  │
└─────────────────────────────────────────────────────┘
                         │
                         v
┌─────────────────────────────────────────────────────┐
│  wl-paste --watch clipr --store-text (daemon)       │
│  ├─ Detects clipboard change (Wayland event)        │
│  ├─ Pipes content to clipr --store-text             │
│  └─ clipr appends to history.bin (atomic write)     │
└─────────────────────────────────────────────────────┘
                         │
                         v
┌─────────────────────────────────────────────────────┐
│              User launches clipr TUI                │
│  ├─ Load history.bin (Bincode deserialize)          │
│  ├─ Load registers.bin (temporary registers)        │
│  ├─ Load config.toml (permanent registers)          │
│  └─ Render UI (ratatui)                             │
└─────────────────────────────────────────────────────┘
                         │
                         v
┌─────────────────────────────────────────────────────┐
│           User searches with / key                  │
│  ├─ Nucleo fuzzy matcher filters entries            │
│  ├─ Results displayed in <100ms                     │
│  └─ Preview panel shows selected entry              │
└─────────────────────────────────────────────────────┘
                         │
                         v
┌─────────────────────────────────────────────────────┐
│        User presses Enter to select                 │
│  ├─ Copy entry to clipboard via wl-copy             │
│  └─ Exit TUI (or stay open if configured)           │
└─────────────────────────────────────────────────────┘
```

---

## Development Workflow

### Running Tests

```bash
# Unit tests (fast, run on every change)
cargo test --lib

# Integration tests
cargo test --test '*'

# All tests
cargo test

# Watch mode (re-run on file changes)
cargo install cargo-watch
cargo watch -x 'test --lib'
```

### Running Benchmarks

```bash
# All benchmarks
cargo bench

# Specific benchmark
cargo bench fuzzy_search

# Cold start measurement (requires hyperfine)
cargo install hyperfine
cargo build --release
hyperfine './target/release/clipr --help'
```

### Code Quality

```bash
# Format code
cargo fmt

# Lint
cargo clippy -- -D warnings

# Check compilation without building
cargo check
```

### Debugging

```bash
# Enable debug logging
RUST_LOG=debug cargo run

# Specific module logging
RUST_LOG=clipr::clipboard=debug cargo run

# Trace level (very verbose)
RUST_LOG=trace cargo run

# Disable logging (release builds)
cargo run --release
```

---

## Key Design Patterns

### 1. Trait Abstractions for Platform Portability

```rust
// src/clipboard/backend.rs
pub trait ClipboardBackend: Send + Sync {
    fn read_text(&self) -> Result<String>;
    fn write_text(&self, text: &str) -> Result<()>;
    fn read_image(&self) -> Result<Vec<u8>>;
    fn write_image(&self, data: &[u8]) -> Result<()>;
    fn supports_images(&self) -> bool;
    fn name(&self) -> &'static str;
}

// Implementations: WaylandBackend, X11Backend
// Factory: create_backend() -> Box<dyn ClipboardBackend>
```

**Why**: Isolates platform-specific code, enables easy testing with mock backends.

### 2. Atomic File Writes

```rust
// src/storage/history.rs
pub fn save(&self, history: &ClipboardHistory) -> Result<()> {
    let tmp_path = self.path.with_extension("tmp");

    // Write to temp file
    let encoded = bincode::serialize(history)?;
    fs::write(&tmp_path, encoded)?;

    // Atomic rename (POSIX guarantees atomicity)
    fs::rename(&tmp_path, &self.path)?;

    Ok(())
}
```

**Why**: Prevents data corruption if process crashes during write.

### 3. Event-Driven Monitoring via wl-paste

```rust
// src/clipboard/monitor.rs
pub fn start_daemon() -> Result<()> {
    // Spawn watcher for text
    Command::new("wl-paste")
        .args(&["--type", "text", "--watch", "clipr", "--store-text"])
        .spawn()?;

    // Spawn watcher for images
    Command::new("wl-paste")
        .args(&["--type", "image/png", "--watch", "clipr", "--store-image"])
        .spawn()?;

    // Keep daemon alive
    loop { sleep(Duration::from_secs(60)); }
}
```

**Why**: Leverages wl-clipboard for event-driven monitoring, avoids implementing Wayland protocol.

### 4. Elm Architecture for TUI

```rust
// src/app.rs
pub struct App {
    mode: AppMode,              // State
    history: ClipboardHistory,  // Model
    selected: usize,            // Model
}

impl App {
    pub fn handle_key(&mut self, key: KeyEvent) -> Result<()> {
        // Update (state transitions)
    }

    pub fn draw(&self, terminal: &mut Terminal) -> Result<()> {
        // View (render current state)
    }
}
```

**Why**: Clear separation of state/update/view, easy to reason about and test.

---

## Performance Targets

All performance requirements from the constitution:

| Metric | Target | How We Measure |
|--------|--------|----------------|
| Cold start | <100ms | `hyperfine './target/release/clipr --help'` |
| Clipboard event response | <10ms p99 | wl-paste --watch (instant) |
| UI frame time | <16ms (60fps) | `cargo bench ui_render` |
| Fuzzy search | <100ms for 1000 items | `cargo bench fuzzy_search` |
| Image preview | <200ms | `cargo bench image_preview` |
| Memory usage | <50MB for 1000 clips | `valgrind --tool=massif` |

### Running Performance Tests

```bash
# Micro-benchmarks (Criterion)
cargo bench

# Cold start (hyperfine)
cargo install hyperfine
cargo build --release
hyperfine --warmup 3 './target/release/clipr --help'

# Memory profiling
valgrind --tool=massif ./target/release/clipr
ms_print massif.out.*

# CPU profiling (flamegraph)
cargo install flamegraph
CARGO_PROFILE_RELEASE_DEBUG=true cargo flamegraph
# Opens flamegraph.svg in browser
```

---

## Common Tasks

### Add a New Keybinding

1. Edit `src/ui/keybindings.rs`
2. Add case to `handle_normal_key()`:
   ```rust
   KeyCode::Char('x') => self.custom_action(),
   ```
3. Implement action in `src/app.rs`
4. Update help overlay in `src/ui/help.rs`
5. Test: `cargo test keybindings`

### Add Support for New Clipboard Type

1. Extend `ClipContent` enum in `src/models/clip.rs`:
   ```rust
   pub enum ClipContent {
       Text(String),
       Image(Vec<u8>),
       RichText { html: String, plain: String },  // New
   }
   ```
2. Update serialization tests
3. Add preview rendering in `src/ui/preview.rs`
4. Update daemon to capture new type in `src/clipboard/monitor.rs`

### Add a New Storage Backend

1. Implement `HistoryStorage` trait in `src/storage/`
2. Example: `JsonHistoryStorage` for human-readable debug files
3. Add factory method or feature flag
4. Benchmark performance: `cargo bench storage`

### Profile Performance Bottleneck

```bash
# 1. Identify slow path with flamegraph
CARGO_PROFILE_RELEASE_DEBUG=true cargo flamegraph

# 2. Add focused benchmark
# benches/my_feature.rs
fn bench_my_feature(c: &mut Criterion) {
    c.bench_function("my_feature", |b| {
        b.iter(|| my_feature())
    });
}

# 3. Run benchmark
cargo bench my_feature

# 4. Optimize and compare
cargo bench my_feature --save-baseline before
# Make changes
cargo bench my_feature --baseline before
```

---

## Testing Strategy

### Unit Tests

**Location**: `tests/unit/` (mirrors `src/`)

**What to test**:
- Model methods (ClipEntry, Register logic)
- Storage serialization round-trips
- Clipboard backend mock operations
- Search index accuracy

**Example**:
```rust
#[test]
fn test_clip_entry_deduplication() {
    let entry1 = ClipEntry::new_text("test".to_string());
    let entry2 = ClipEntry::new_text("test".to_string());

    assert!(entry1.is_duplicate(&entry2));
}
```

### Integration Tests

**Location**: `tests/integration/`

**What to test**:
- End-to-end clipboard operations
- File persistence and recovery
- Daemon startup and clipboard capture
- TUI interaction flows

**Example**:
```rust
#[test]
fn test_daemon_captures_clipboard() {
    // Start daemon
    let daemon = start_test_daemon()?;

    // Simulate clipboard change
    Command::new("wl-copy").arg("test").spawn()?.wait()?;

    // Wait for daemon to process
    sleep(Duration::from_millis(100));

    // Verify history file updated
    let history = load_history()?;
    assert_eq!(history.entries.len(), 1);
    assert_eq!(history.entries[0].content, ClipContent::Text("test".into()));
}
```

### Benchmarks

**Location**: `benches/`

**What to benchmark**:
- Cold start time (hyperfine)
- Fuzzy search performance
- UI render time
- Clipboard read/write latency
- Storage load/save time

---

## Troubleshooting

### wl-clipboard not found

```bash
# Install wl-clipboard
sudo apt install wl-clipboard

# Verify
which wl-paste
wl-paste --version
```

### Daemon not capturing clipboard

```bash
# Check if daemon is running
ps aux | grep clipr

# Check logs
RUST_LOG=debug clipr --daemon

# Test wl-paste directly
echo "test" | wl-copy
wl-paste
```

### TUI not rendering images

```bash
# Check terminal support
echo $TERM  # Should include "kitty" or known terminal

# Test viuer directly
cargo run --example test_image

# Fallback: disable image preview in config
# ~/.config/clipr/config.toml
[general]
image_preview = false
```

### Build fails with missing dependencies

```bash
# Update Rust
rustup update stable

# Clean build
cargo clean
cargo build

# Check Cargo.lock
rm Cargo.lock
cargo build
```

---

## Contributing Guidelines

### Before Submitting PR

1. ✅ Run tests: `cargo test`
2. ✅ Run clippy: `cargo clippy -- -D warnings`
3. ✅ Format code: `cargo fmt`
4. ✅ Run benchmarks if performance-critical: `cargo bench`
5. ✅ Update documentation if adding features
6. ✅ Verify constitution compliance (see plan.md)

### Commit Message Format

```
<type>: <description>

Examples:
feat: add support for rich text clipboard
fix: prevent duplicate entries on rapid clipboard changes
perf: optimize fuzzy search for large histories
docs: update quickstart with troubleshooting section
```

### Constitution Compliance Checklist

For any PR, verify:
- ✅ **Performance-First**: No regressions in benchmarks
- ✅ **Simplicity/YAGNI**: Complexity justified in PR description
- ✅ **Debuggability**: Errors include context via anyhow
- ✅ **Iteration Speed**: Tests run in <1s for `cargo test --lib`
- ✅ **Platform Portability**: X11 fallback maintained if touching clipboard code

---

## Resources

### Documentation

- Feature spec: `specs/001-clipboard-manager-tui/spec.md`
- Implementation plan: `specs/001-clipboard-manager-tui/plan.md`
- Research findings: `specs/001-clipboard-manager-tui/research.md`
- Data model: `specs/001-clipboard-manager-tui/data-model.md`
- Contracts: `specs/001-clipboard-manager-tui/contracts/`
- Constitution: `.specify/memory/constitution.md`

### External Resources

- ratatui docs: https://ratatui.rs/
- nucleo (fuzzy search): https://docs.rs/nucleo/
- wl-clipboard: https://github.com/bugaevc/wl-clipboard
- Criterion.rs: https://bheisler.github.io/criterion.rs/
- Rust Performance Book: https://nnethercote.github.io/perf-book/

### Similar Projects (Reference)

- clipse: `./clipse/` (Go implementation, similar features)
- clipboard-rs: Rust clipboard library examples
- helix: Rust TUI editor (ratatui patterns)

---

## Next Steps

After reading this guide, you should:

1. **Build and run** the application locally
2. **Read the architecture docs** (plan.md, data-model.md, contracts/)
3. **Run the test suite** to verify your environment
4. **Pick a task** from tasks.md (will be generated via `/speckit.tasks`)
5. **Ask questions** if anything is unclear

Welcome to the project! 🎉

**Last Updated**: 2025-11-24
