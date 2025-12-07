# Research: Advanced Theming System

**Feature**: 002-advanced-theming-system
**Date**: 2025-12-04
**Status**: Complete

## Overview

This document captures research decisions and technical choices for implementing the advanced theming system in clipr.

## Research Questions & Decisions

### 1. File Watching Library for Theme Development Mode

**Question**: Which Rust crate should we use for detecting theme file changes in development mode?

**Decision**: Use the `notify` crate (version 8.2.0)

**Rationale**:
- **Event-driven architecture**: Uses Linux inotify backend natively, providing <100ms detection latency with ~0% CPU overhead when idle
- **Production-proven**: Used by rust-analyzer, cargo-watch, Alacritty, Deno, mdBook, and 64.9k other projects
- **Simple integration**: Works perfectly with existing synchronous event loop using `std::sync::mpsc` and `try_recv()` (non-blocking)
- **Active maintenance**: Latest release August 2025, 3.2k stars, 852 commits, 114 contributors
- **YAGNI-compliant**: Provides exactly what we need (~10 lines of setup) without unnecessary features
- **Cross-platform ready**: Though we target Linux, this choice doesn't lock us out of future platform support

**Alternatives Considered**:
1. **`inotify` crate (Linux-specific)**: Rejected because `notify` already wraps inotify on Linux and provides better abstraction with no performance penalty
2. **Manual polling with fs::metadata()**: Rejected due to higher CPU overhead (0.1-0.3% idle), slower detection (200-1000ms), and more code to maintain

**Implementation Notes**:
- Create watcher only when `theme_dev_mode` config option is enabled
- Use `RecursiveMode::NonRecursive` (single file, not directory tree)
- Call `rx.try_recv()` in main event loop (non-blocking check)
- Handle `EventKind::Modify` events to trigger theme reload

### 2. TOML Theme File Format

**Question**: What structure should the TOML theme file use?

**Decision**: Flat structure with sections for common colors, defaults, and elements

**Rationale**:
- **Simplicity**: TOML already in use for main config, no new parsing infrastructure
- **Human-readable**: Users can hand-edit with syntax highlighting in editors
- **Validation**: Serde provides automatic deserialization with clear error messages
- **Common colors**: Named color definitions that can be referenced via `$color_name` syntax
- **Defaults**: Top-level `default_fg` and `default_bg` for fallback colors
- **Elements**: Flat key-value structure (e.g., `clip_number.fg = "$primary"`)

**Example Structure**:
```toml
[colors]
primary = [137, 180, 250]
accent = [245, 194, 231]
background = [30, 30, 46]

[defaults]
fg = "$primary"
bg = "$background"

[elements.clip_number]
fg = "$accent"
bold = true

[elements.status_bar]
fg = [205, 214, 244]
bg = [69, 71, 90]
```

**Alternatives Considered**:
1. **JSON**: Rejected due to less human-friendly syntax (trailing commas, verbose)
2. **YAML**: Rejected to avoid adding new dependency and potential whitespace issues
3. **Hierarchical groups**: Deferred to P3 per spec (common colors provide similar benefits)

### 3. Theme Storage and Loading

**Question**: Where should themes be stored and how should they be loaded?

**Decision**:
- Built-in themes: Hardcoded in `src/ui/theme.rs` (can export to TOML)
- Custom themes: `~/.config/clipr/themes/` directory
- Active theme: Specified in main config as either built-in name or custom file path

**Rationale**:
- **Built-in themes as code**: Compile-time guaranteed correctness, zero I/O on startup
- **Export command**: `clipr export-theme <name>` outputs built-in theme to stdout
- **Custom theme directory**: Standard XDG config location, discoverable, version-controllable
- **Config reference**: Single source of truth for which theme is active

**Loading Priority**:
1. Check if theme name matches built-in (e.g., "catppuccin-mocha")
2. If not, look for `~/.config/clipr/themes/<name>.toml`
3. If neither found, error with helpful message listing available themes

**Migration from ColorScheme**:
- Convert existing `ColorScheme` themes to built-in `Theme` structs
- Keep same theme names for continuity
- Remove old `colorscheme.rs` after migration complete

### 4. Error Handling for Invalid Themes

**Question**: How should theme validation errors be presented to users?

**Decision**: Display detailed error modal with specific problem identification

**Rationale**:
- **Immediate feedback**: Error modal appears within 3 seconds (SC-008)
- **Detailed context**: Show line number, field name, and validation error
- **TOML parse errors**: Display full `toml::de::Error` with span information
- **Validation errors**: Check for invalid color values, missing required fields, non-existent color references
- **Graceful fallback**: Keep previous theme active on Ctrl-R error; built-in default on startup error

**Error Categories**:
1. **Parse errors**: Malformed TOML syntax (missing quotes, invalid tables)
2. **Validation errors**: Invalid RGB values (>255), missing required sections
3. **Reference errors**: `$color_name` references non-existent common color
4. **I/O errors**: File not found, permission denied

**Error Modal Content**:
```
Theme Error: Invalid Color Value

File: ~/.config/clipr/themes/custom.toml
Line: 15

Error: RGB value 300 exceeds maximum of 255
Field: elements.clip_number.fg

Press any key to dismiss
```

### 5. Atomic Theme Reload

**Question**: How do we ensure atomic theme updates (no partial state visible)?

**Decision**: Load and validate entire new theme before swapping global theme reference

**Rationale**:
- **Parse → Validate → Apply**: Three-phase process ensures consistency
- **Atomic swap**: Replace entire `Theme` struct in single operation
- **No partial updates**: UI always sees complete, valid theme state
- **Rollback on error**: Keep previous theme active if new theme fails validation

**Implementation Strategy**:
```rust
pub fn reload_theme(&mut self) -> Result<()> {
    // Phase 1: Load from file
    let theme_content = std::fs::read_to_string(&self.theme_path)?;

    // Phase 2: Parse TOML
    let theme_def: ThemeDefinition = toml::from_str(&theme_content)?;

    // Phase 3: Validate and resolve references
    let new_theme = Theme::from_definition(theme_def)?;

    // Phase 4: Atomic swap (only if all above succeeded)
    self.current_theme = new_theme;

    Ok(())
}
```

## Best Practices

### Theme Development Mode

- Enable with `theme_dev_mode = true` in config
- Use `notify` crate with non-blocking `try_recv()`
- Poll file watcher in main event loop (alongside key events)
- Display error modal automatically on invalid theme
- Auto-clear error modal when theme becomes valid

### Performance Optimization

- Cache theme lookups in hot paths (e.g., render functions)
- Pre-resolve common color references at load time
- Use `Arc<Theme>` if sharing across threads (not currently needed)
- Keep theme file parsing off UI thread (already satisfied - happens on Ctrl-R or file change)

### Testing Strategy

- **Unit tests**: TOML parsing, validation, color reference resolution
- **Integration tests**: File watching, Ctrl-R reload, export command
- **Property tests**: Random RGB values always validate correctly
- **Snapshot tests**: Built-in theme exports match expected format

## Open Questions

None - all technical decisions resolved.

## References

- `notify` crate: https://crates.io/crates/notify
- `toml` crate: https://crates.io/crates/toml
- ratatui Color API: https://docs.rs/ratatui/latest/ratatui/style/enum.Color.html
- ratatui Style API: https://docs.rs/ratatui/latest/ratatui/style/struct.Style.html
- XDG Base Directory Spec: https://specifications.freedesktop.org/basedir-spec/
