# Quickstart: Advanced Theming System

**Feature**: 002-advanced-theming-system
**Date**: 2025-12-04

## Overview

This guide provides the fastest path to implementing the theming system. Focus on iteration speed - get something working quickly, then refine.

## Implementation Steps (High-Level)

### Step 1: Iterate on Theme Format (~2-4 hours)

**Goal**: Determine the actual TOML syntax we want to use.

**Approach**:
1. Create `src/ui/theme.rs` with minimal `Theme` struct
2. Try writing a theme file by hand with different syntax options
3. Experiment with parsing (use `toml` crate + `serde`)
4. Pick the syntax that feels best to write

**Syntax options to try**:
```toml
# Option A: Inline style syntax
[elements]
clip_number = "fg(#89b4fa) bold"
status_bar = "fg($text) bg($surface)"

# Option B: Nested TOML
[elements.clip_number]
fg = "#89b4fa"
bold = true

# Option C: Mixed
[elements]
clip_number.fg = "#89b4fa"
clip_number.bold = true
```

**Deliverable**: Chosen syntax documented in data-model.md, basic parser working

### Step 2: Implement Theme struct (~1-2 hours)

**Goal**: Create the runtime `Theme` struct with direct field access.

**Tasks**:
- Define `Theme` struct with all ~50-100 fields
- Map current `ColorScheme` fields to new `Theme` fields
- Create `Default` implementation using sensible defaults

**File**: `src/ui/theme.rs`

**Test**: Can construct a `Theme` and access fields

### Step 3: Implement Theme Loading (~2-3 hours)

**Goal**: Load themes from TOML files and built-in themes.

**Tasks**:
- Create `ThemeDefinition` struct for TOML deserialization
- Implement `Theme::from_definition()` to convert TOML → runtime struct
- Handle color reference resolution (`$primary` → RGB)
- Implement `BuiltInTheme` enum with hardcoded themes
- Add error types for theme validation

**Files**:
- `src/ui/theme.rs`

**Test**: Can load a TOML theme file, can get built-in theme

### Step 4: Implement export-theme Command (~1 hour)

**Goal**: Users can export themes to TOML.

**Tasks**:
- Add `export-theme` subcommand to `src/cli.rs`
- Implement `Theme::to_toml()` serialization
- Handle both built-in and custom theme export

**Files**:
- `src/cli.rs`
- `src/ui/theme.rs` (add `to_toml()` method)

**Test**: `clipr export-theme catppuccin-mocha` produces valid TOML

### Step 5: Add Ctrl-R Reload (~1-2 hours)

**Goal**: Press Ctrl-R to reload theme without restarting.

**Tasks**:
- Add Ctrl-R key handler in `src/app.rs`
- Implement `App::reload_theme()` method
- Add error modal display for invalid themes
- Ensure atomic theme swap (no partial updates)

**Files**:
- `src/app.rs`
- `src/ui/error_modal.rs` (update for theme errors)

**Test**: Change theme file, press Ctrl-R, see updates; break theme file, press Ctrl-R, see error modal

### Step 6: Add Theme Development Mode (~2-3 hours)

**Goal**: Auto-reload theme when file changes.

**Tasks**:
- Add `notify` crate dependency
- Add `theme_dev_mode` config option in `src/storage/config.rs`
- Set up file watcher in `App::new()` (only if dev mode enabled)
- Add `App::check_theme_reload()` method called in event loop
- Implement auto-error modal display/clear

**Files**:
- `Cargo.toml` (add `notify = "8.2"`)
- `src/storage/config.rs`
- `src/app.rs`
- `src/main.rs` (call `check_theme_reload()` in loop)

**Test**: Enable dev mode, edit theme file, see auto-reload; break theme, see error modal; fix theme, see error clear

### Step 7: Migrate UI Code (~3-5 hours)

**Goal**: Replace all `colors().field` calls with `theme.field`.

**Approach**:
- Can be broken during migration - don't worry about it
- Migrate one file at a time:
  1. `src/ui/clip_list.rs`
  2. `src/ui/preview.rs`
  3. `src/ui/status.rs`
  4. `src/ui/help.rs`
  5. `src/ui/error_modal.rs`
  6. `src/ui/mod.rs`
- Delete `src/ui/colorscheme.rs` when done

**Test**: Visual inspection of TUI, compare before/after

### Step 8: Testing & Polish (~2-3 hours)

**Goal**: Ensure everything works end-to-end.

**Tasks**:
- Write unit tests for theme parsing
- Write integration test for file watching
- Test all built-in themes
- Test export/import round-trip
- Handle edge cases (empty theme, missing fields)

**Files**:
- `tests/unit/theme_*.rs`
- `tests/integration/theme_reload.rs`

## Minimal Viable Product (MVP)

**What to build first** (can skip theme dev mode initially):
1. Theme struct with direct fields
2. Load from TOML (basic syntax)
3. Built-in themes
4. export-theme command
5. Ctrl-R reload
6. Migrate one UI file (e.g., status bar)

**Total time**: ~8-10 hours

**What to defer to iteration 2**:
- Theme development mode (auto-reload)
- All UI file migrations (do incrementally)
- Advanced error messages
- Full test coverage

## Iteration Strategy

**Iteration 0** (MVP):
- Basic TOML syntax (pick something simple)
- Ctrl-R reload only (no auto-watch)
- Migrate status bar to prove it works

**Iteration 1** (Full Feature):
- Refine TOML syntax based on usage
- Add theme development mode
- Migrate all UI code
- Full error handling

**Iteration 2** (Polish):
- Better error messages
- Documentation
- More built-in themes
- Hierarchical groups (P3 feature)

## Key Files

```
src/ui/theme.rs              # Core theme implementation
src/cli.rs                   # export-theme command
src/app.rs                   # Ctrl-R handler, file watching
src/storage/config.rs        # theme_dev_mode option
src/ui/colorscheme.rs        # DELETE after migration
tests/unit/theme_parsing.rs  # Theme tests
```

## Development Tips

1. **Start with one built-in theme**: Don't implement all 5 themes initially
2. **Test with real theme files**: Write themes by hand to validate syntax
3. **Use `cargo watch`**: Fast iteration on compile errors
4. **Visual testing**: Run clipr frequently to see theme changes
5. **Don't optimize early**: Get it working, then optimize
6. **Accept broken state**: Migration can leave things broken temporarily

## Success Criteria Checklist

- [ ] Can export catppuccin-mocha theme to TOML
- [ ] Can load exported theme and it looks identical
- [ ] Can press Ctrl-R to reload theme
- [ ] Invalid theme shows error modal
- [ ] Can enable theme_dev_mode and see auto-reload
- [ ] All UI elements are themeable (50-100 fields)
- [ ] Theme file is easy to write by hand
- [ ] Reload completes in <1 second
- [ ] Export command works for both built-in and custom themes

## Time Estimate

**Total implementation time**: 15-20 hours
- MVP (Iterations 0): 8-10 hours
- Full feature (Iteration 1): 5-7 hours
- Polish (Iteration 2): 2-3 hours

**Critical path**:
1. Decide TOML syntax (Step 1)
2. Implement Theme struct (Step 2)
3. Everything else can happen in parallel or incrementally
