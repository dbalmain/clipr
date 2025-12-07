# Implementation Plan: Advanced Theming System

**Branch**: `002-advanced-theming-system` | **Date**: 2025-12-04 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/002-advanced-theming-system/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/commands/plan.md` for the execution workflow.

## Summary

Implement a comprehensive theming system that allows users to customize every visible UI element through TOML configuration files. The system supports common colors for reusability, default fallback colors, and explicit element styling with RGB colors and text modifiers. Critical features include Ctrl-R manual reload and optional theme development mode with automatic file polling (<1 second) for maximum iteration speed. Built-in themes (Catppuccin, Tokyo Night) can be exported via `clipr export-theme` command as starting points for customization.

## Technical Context

**Language/Version**: Rust 1.75+ (stable channel, latest 3 releases supported)
**Primary Dependencies**:
- `ratatui` (TUI framework, already in use)
- `toml` (TOML parsing, already in use)
- `notify` or `inotify` (file watching for theme development mode) - NEEDS CLARIFICATION on best choice
- `serde` (serialization/deserialization, already in use)

**Storage**: TOML configuration files in `~/.config/clipr/` directory
**Testing**: `cargo test` (unit tests for theme parsing, validation, style resolution)
**Target Platform**: Linux (Wayland/X11), 24-bit color terminals
**Project Type**: Single Rust TUI application
**Performance Goals**:
- Theme load/reload: <100ms
- Theme development mode polling: <1 second change detection
- UI frame time: <16ms (60fps maintained after theme changes)
- No perceptible lag on Ctrl-R reload

**Constraints**:
- Must not regress UI rendering performance
- Theme reload must be atomic (no partial updates visible)
- File polling in development mode must not impact CPU usage (<1% idle)
- Error modals must display within 3 seconds of invalid theme detection

**Scale/Scope**:
- ~50-100 themed UI elements
- 5 built-in themes (existing ColorScheme themes + new system)
- Support for arbitrary number of user-defined common colors
- Theme files expected to be <10KB typically

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

### I. Performance-First ✅

- **Theme loading**: Target <100ms aligns with cold start requirement (<100ms)
- **UI rendering**: Theme changes must not regress 60fps responsiveness
- **File polling**: Must not introduce perceptible CPU overhead (<1% idle)
- **Atomic updates**: Prevents UI flicker during reload

**Assessment**: PASS - Performance constraints are explicit and measurable. Theme system is designed to be loaded once and cached, with reload being an explicit user action (Ctrl-R) or development-mode operation.

### II. Simplicity/YAGNI ✅

- **Common colors**: Simple reusability without complex inheritance (hierarchical groups deferred to P3)
- **RGB-only**: No multiple color format support (YAGNI for hex, named colors)
- **TOML format**: Reuses existing config infrastructure
- **No backward compatibility**: Explicitly rejected as anti-goal (nobody using yet)
- **File watching dependency**: Justified by theme development mode requirement

**Assessment**: PASS - Feature deliberately avoids over-engineering. Hierarchical groups moved to P3. Export command provides discoverability without GUI builder complexity.

### III. Debuggability ✅

- **Error modals**: Clear validation errors displayed on Ctrl-R
- **Auto-clear in dev mode**: Immediate feedback when issues fixed
- **Export command**: Users can inspect built-in themes to understand format
- **Fallback defaults**: System degrades gracefully on missing properties

**Assessment**: PASS - Error handling is comprehensive. Users get clear feedback through modals. Export command aids debugging by showing valid examples.

### IV. Iteration Speed ✅

- **Ctrl-R reload**: Manual reload without restart
- **Theme development mode**: Automatic reload within 1 second (fastest possible iteration)
- **Export command**: Quick starting point from built-in themes
- **TOML format**: Human-readable, hand-editable

**Assessment**: PASS - This feature explicitly prioritizes iteration speed. Theme development mode provides the absolute fastest workflow (edit → save → see results automatically).

### V. Platform Portability ✅

- **Terminal-agnostic**: Works on any 24-bit color terminal
- **File-based config**: Standard Linux config directory (`~/.config/clipr/`)
- **No platform-specific UI**: Pure terminal rendering through ratatui

**Assessment**: PASS - No platform-specific dependencies beyond existing Wayland/X11 clipboard backend. Theme system is entirely portable.

**Overall Gate Status**: ✅ PASS - All constitution principles satisfied. No complexity violations requiring justification.

## Project Structure

### Documentation (this feature)

```text
specs/[###-feature]/
├── plan.md              # This file (/speckit.plan command output)
├── research.md          # Phase 0 output (/speckit.plan command)
├── data-model.md        # Phase 1 output (/speckit.plan command)
├── quickstart.md        # Phase 1 output (/speckit.plan command)
├── contracts/           # Phase 1 output (/speckit.plan command)
└── tasks.md             # Phase 2 output (/speckit.tasks command - NOT created by /speckit.plan)
```

### Source Code (repository root)

```text
src/
├── ui/
│   ├── colorscheme.rs       # Current theme implementation (to be refactored)
│   ├── theme.rs             # New: Theme parsing, validation, and management
│   ├── clip_list.rs         # Update: Use new theme system
│   ├── preview.rs           # Update: Use new theme system
│   ├── status.rs            # Update: Use new theme system
│   ├── help.rs              # Update: Use new theme system
│   ├── error_modal.rs       # Update: Use new theme system + theme errors
│   └── mod.rs               # Update: Use new theme system
├── storage/
│   └── config.rs            # Update: Add theme_dev_mode config option
├── app.rs                   # Update: Add Ctrl-R handler, theme watcher
├── cli.rs                   # Update: Add export-theme subcommand
└── main.rs                  # No changes expected

tests/
├── unit/
│   ├── theme_parsing.rs     # New: Test TOML parsing, validation
│   ├── theme_resolution.rs  # New: Test common color resolution, defaults
│   └── theme_export.rs      # New: Test built-in theme export
└── integration/
    └── theme_reload.rs      # New: Test Ctrl-R reload, file watching
```

**Structure Decision**: Single Rust project structure. This feature extends existing TUI code with:
- New `src/ui/theme.rs` module for theme management
- Updates to all `src/ui/*.rs` files to use new theme system instead of hardcoded ColorScheme
- CLI extension for `export-theme` subcommand
- App-level changes for Ctrl-R handler and optional file watching

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

No violations - Constitution Check passed all principles.
