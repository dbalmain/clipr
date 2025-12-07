# Implementation Tasks: Advanced Theming System

**Feature**: 002-advanced-theming-system
**Branch**: `002-advanced-theming-system`
**Date**: 2025-12-04

## Overview

This document breaks down the Advanced Theming System into executable tasks organized by user story. Each P1 user story can be implemented and tested independently, enabling incremental delivery.

**Total User Stories**: 7 (5 P1, 1 P2, 1 P3)
**Recommended MVP**: User Stories 1-4 (core theming + export + Ctrl-R reload)
**Full P1 Feature**: User Stories 1-5 (add comprehensive element coverage)

## Implementation Strategy

### Incremental Delivery Plan

**Iteration 0 - Spike/Exploration** (2-4 hours):
- Determine TOML syntax through hands-on experimentation
- Goal: Pick format that's easy to write by hand

**Iteration 1 - MVP** (8-12 hours):
- US1: Core theming with common colors + defaults
- US4: Export command (validates theme format works)
- US2: Ctrl-R reload (proves iteration speed goal)

**Iteration 2 - Full P1** (4-6 hours):
- US3: Theme development mode (automatic reload)
- US5: Migrate all UI elements

**Iteration 3 - Polish** (2-3 hours):
- US6: Error handling improvements
- Cross-cutting concerns

**Future** (P3):
- US7: Hierarchical groups (deferred)

## Task Summary

- **Total Tasks**: 38
- **Setup Phase**: 2 tasks
- **Foundational Phase**: 3 tasks
- **User Story 1**: 7 tasks (P1 - Core theming)
- **User Story 2**: 4 tasks (P1 - Ctrl-R reload)
- **User Story 3**: 5 tasks (P1 - Auto-reload dev mode)
- **User Story 4**: 3 tasks (P1 - Export command)
- **User Story 5**: 10 tasks (P1 - Element coverage)
- **User Story 6**: 3 tasks (P2 - Error handling)
- **Polish Phase**: 1 task

## Dependencies

### User Story Dependency Graph

```
Phase 1 (Setup) → Phase 2 (Foundational)
                        ↓
    ┌───────────────────┼───────────────────┐
    ↓                   ↓                   ↓
  US1 (Core)          US4 (Export)       US5 (Coverage)
    ↓                                       ↓
  US2 (Ctrl-R)                      (Can migrate UI files)
    ↓
  US3 (Auto-reload)
    ↓
  US6 (Error handling) ← Enhances US2/US3

US7 (Hierarchical groups) - P3, deferred
```

**Key Dependencies**:
- US1 MUST complete before US2, US3 (need theme system to reload)
- US4 can run parallel to US1/US2 (independent export command)
- US5 can start after US1 (migrate UI files incrementally)
- US6 enhances US2/US3 (better error messages)

**Parallelization Opportunities**:
- US1 Step 1 (format iteration) can inform US4 export format
- US5 UI migrations can happen in parallel once US1 complete
- US6 error handling can be implemented alongside US2/US3

---

## Phase 1: Setup

**Goal**: Initialize project dependencies and structure for theming system.

**Tasks**:

- [ ] T001 Add `notify = "8.2"` dependency to Cargo.toml for file watching
- [ ] T002 [P] Create tests/unit/ and tests/integration/ directories if they don't exist

**Completion Criteria**:
- Cargo build succeeds with new dependency
- Test directories exist

---

## Phase 2: Foundational

**Goal**: Create core theme infrastructure that all user stories depend on.

**Tasks**:

- [ ] T003 Create src/ui/theme.rs module with basic module structure
- [ ] T004 [P] Define `ThemeError` enum in src/ui/theme.rs for validation errors (parse errors, invalid RGB, missing colors, unknown elements)
- [ ] T005 [P] Add theme_dev_mode config option in src/storage/config.rs (default: false)

**Completion Criteria**:
- theme.rs module exists and compiles
- ThemeError covers all validation scenarios
- Config supports theme_dev_mode flag

---

## Phase 3: User Story 1 - Comprehensive Element Theming with Common Colors (P1)

**Story Goal**: Users can create custom themes with common colors, defaults, and element-specific styles. Empty themes are valid (use defaults only).

**Why Independent**: Delivers core value - users can customize their TUI appearance.

**Independent Test**:
```bash
# Create minimal theme
cat > ~/.config/clipr/themes/test.toml << EOF
[defaults]
fg = [200, 200, 200]
bg = [20, 20, 20]
EOF

# Test it loads and applies defaults to all elements
```

**Tasks**:

- [ ] T006 [US1] **Iterate on TOML syntax**: Experiment with 3 format options (inline "fg($color) bold" vs nested vs mixed), write themes by hand, choose best format, document decision in data-model.md (2-4 hours exploration)
- [ ] T007 [US1] Define `Theme` struct in src/ui/theme.rs with direct field access for ~50-100 UI elements (default_fg, default_bg, clip_number, pin_indicator, status_bar_text, etc.)
- [ ] T008 [US1] Implement `Default` trait for `Theme` using sensible fallback colors (map from current ColorScheme defaults)
- [ ] T009 [P] [US1] Define `ThemeDefinition` struct in src/ui/theme.rs for TOML deserialization (colors HashMap, defaults, elements)
- [ ] T010 [P] [US1] Implement `ColorValue` enum in src/ui/theme.rs supporting both RGB arrays `[r, g, b]` and color references `"$primary"`
- [ ] T011 [US1] Implement `Theme::from_definition()` in src/ui/theme.rs to resolve color references and convert TOML -> runtime Theme struct
- [ ] T012 [US1] Add theme loading function in src/ui/theme.rs that tries built-in themes first, then `~/.config/clipr/themes/<name>.toml`

**Completion Criteria**:
- Can create empty theme file with just defaults - loads successfully
- Can define common colors and reference them in elements
- Theme struct has direct field access (no HashMap lookups)
- Invalid RGB values (>255) rejected with clear error
- Non-existent color references detected and reported

**Tests** (if requested):
- Unit test: Parse valid TOML theme
- Unit test: Reject invalid RGB values
- Unit test: Resolve $color references correctly
- Unit test: Empty theme uses defaults

---

## Phase 4: User Story 2 - Live Theme Reload with Ctrl-R (P1)

**Story Goal**: Press Ctrl-R to reload theme without restarting app. Invalid themes show error modal, previous theme stays active.

**Why Independent**: Delivers fast iteration - users can test theme changes immediately.

**Independent Test**:
```bash
# Start clipr, edit theme file, press Ctrl-R
# Expected: UI updates instantly
# Break theme file, press Ctrl-R
# Expected: Error modal, previous theme persists
```

**Tasks**:

- [ ] T013 [US2] Add Ctrl-R key handler in src/app.rs that calls `App::reload_theme()`
- [ ] T014 [US2] Implement `App::reload_theme()` method in src/app.rs with atomic swap (load → validate → apply only if valid)
- [ ] T015 [US2] Update src/ui/error_modal.rs to display theme validation errors (show file, line number, specific problem)
- [ ] T016 [US2] Ensure error modal dismisses on any key press and previous theme remains active

**Completion Criteria**:
- Ctrl-R reloads theme in <1 second
- UI updates atomically (no partial state visible)
- Invalid theme shows error modal with helpful details
- Previous theme persists after error

**Tests** (if requested):
- Integration test: Ctrl-R with valid theme updates UI
- Integration test: Ctrl-R with invalid theme shows error

---

## Phase 5: User Story 3 - Automatic Theme Reload in Development Mode (P1)

**Story Goal**: When theme_dev_mode enabled, automatically detect theme file changes and reload within 1 second. Auto-clear error modal when theme becomes valid.

**Why Independent**: Maximizes iteration speed - zero manual steps to see changes.

**Independent Test**:
```bash
# Enable theme_dev_mode in config
# Edit theme file, save
# Expected: UI auto-updates within 1 second
# Break theme, save
# Expected: Error modal appears automatically
# Fix theme, save
# Expected: Error modal disappears, theme loads
```

**Tasks**:

- [ ] T017 [US3] Add file watcher initialization in src/app.rs `App::new()` using notify crate (only if theme_dev_mode = true)
- [ ] T018 [US3] Store `notify::RecommendedWatcher` and `mpsc::Receiver` in `App` struct (Option, only present if dev mode)
- [ ] T019 [US3] Add `App::check_theme_reload()` method in src/app.rs that calls `rx.try_recv()` non-blocking
- [ ] T020 [US3] Call `check_theme_reload()` in main event loop in src/main.rs (before rendering, after image cache update)
- [ ] T021 [US3] Implement auto-clear behavior: if error modal showing and theme becomes valid, dismiss modal and load theme

**Completion Criteria**:
- File changes detected within 1 second
- CPU usage <1% when idle (inotify backend)
- Error modal appears automatically on invalid save
- Error modal disappears automatically when fixed
- No impact when theme_dev_mode = false

**Tests** (if requested):
- Integration test: File change triggers auto-reload
- Integration test: Invalid theme shows error automatically
- Integration test: Valid theme clears error automatically

---

## Phase 6: User Story 4 - Export Built-in Themes as Starter Templates (P1)

**Story Goal**: Run `clipr export-theme <name>` to export built-in or custom themes to TOML with all values filled in.

**Why Independent**: Provides discoverability - users learn format from examples.

**Independent Test**:
```bash
# Export built-in theme
clipr export-theme catppuccin-mocha > test.toml
# Load exported theme
# Expected: Renders identically to built-in
```

**Tasks**:

- [ ] T022 [P] [US4] Define `BuiltInTheme` enum in src/ui/theme.rs with variants for catppuccin-mocha, catppuccin-latte, tokyonight-night, tokyonight-storm, tokyonight-day
- [ ] T023 [US4] Implement `BuiltInTheme::to_theme()` method returning hardcoded `Theme` struct for each built-in theme
- [ ] T024 [US4] Implement `Theme::to_toml()` serialization method in src/ui/theme.rs that outputs complete TOML (all elements, not just user-defined)
- [ ] T025 [US4] Add `export-theme` subcommand in src/cli.rs that calls theme export logic and writes to stdout

**Completion Criteria**:
- Can export all 5 built-in themes
- Exported TOML is valid and re-loadable
- Exported theme renders identically to built-in
- Export includes ALL themed elements with current values
- Works for both built-in and custom themes

**Tests** (if requested):
- Unit test: Export/import round-trip preserves theme
- Unit test: Unknown theme name shows helpful error

---

## Phase 7: User Story 5 - Comprehensive Element Coverage (P1)

**Story Goal**: Migrate all UI code to use new theme system. Every visible element is themeable.

**Why Independent**: Completes the migration - old ColorScheme removed, all elements use Theme.

**Independent Test**:
```bash
# Create theme with unique color for each element type
# Visual inspection: every UI element shows correct color
```

**Tasks**:

- [ ] T026 [P] [US5] Migrate src/ui/clip_list.rs to use `theme.field_name` instead of `colors().field_name` (clip_number, pin_indicator, registers, timestamp, etc.)
- [ ] T027 [P] [US5] Migrate src/ui/preview.rs to use theme (preview_content, preview_metadata, loading_indicator)
- [ ] T028 [P] [US5] Migrate src/ui/status.rs to use theme (status_bar_bg, status_bar_text, status_bar_key_hint)
- [ ] T029 [P] [US5] Migrate src/ui/help.rs to use theme (help_modal_bg, help_modal_text)
- [ ] T030 [P] [US5] Migrate src/ui/error_modal.rs to use theme (error_modal_bg, error_modal_border, error_modal_text)
- [ ] T031 [P] [US5] Migrate src/ui/mod.rs to use theme (divider, confirm modal elements)
- [ ] T032 [US5] Update `App` struct in src/app.rs to store `Theme` instead of using `colors()` function
- [ ] T033 [US5] Pass theme reference to all render functions in src/app.rs `draw()` method
- [ ] T034 [US5] Delete src/ui/colorscheme.rs after verifying all migrations complete
- [ ] T035 [US5] Visual testing: Run clipr with each built-in theme and verify all UI elements render correctly

**Completion Criteria**:
- All UI files use `theme.field_name` pattern
- colorscheme.rs deleted
- All 50-100 themed elements covered
- No hardcoded colors remain in UI code
- Visual inspection confirms all elements themeable

**Tests** (if requested):
- Visual test: Each built-in theme renders correctly
- Regression test: Compare screenshots before/after migration

---

## Phase 8: User Story 6 - Theme Validation and Error Handling (P2)

**Story Goal**: Helpful error messages for common mistakes (invalid colors, syntax errors, missing fields, unknown elements).

**Why Independent**: Improves user experience but core functionality works without it.

**Independent Test**:
```bash
# Test each error type:
# 1. Invalid RGB (>255) -> clear error with line number
# 2. Malformed TOML -> parse error with context
# 3. Unknown element -> suggest valid names
# 4. Missing color reference -> list available colors
```

**Tasks**:

- [ ] T036 [P] [US6] Enhance error messages in src/ui/theme.rs to include file path, line numbers (from toml::Spanned), and specific fix suggestions
- [ ] T037 [P] [US6] Add "did you mean?" suggestions for unknown element names in src/ui/theme.rs (use edit distance)
- [ ] T038 [P] [US6] List available color names when $reference not found in src/ui/theme.rs validation

**Completion Criteria**:
- Error messages include file path and line number
- Suggestions provided for typos
- List of valid options shown for unknown names
- Parse errors from toml crate preserved with context

---

## Phase 9: Polish & Cross-Cutting Concerns

**Goal**: Documentation, cleanup, final testing.

**Tasks**:

- [ ] T039 Update help text in src/ui/help.rs to document Ctrl-R reload and theme_dev_mode

**Completion Criteria**:
- Help screen documents theming features
- README updated with theme examples
- All P1 success criteria met

---

## Parallelization Examples

### User Story 1 (Core Theming)
```bash
# Can run in parallel after T006 (format decided):
- T009: Define ThemeDefinition struct
- T010: Define ColorValue enum
- T008: Implement Default for Theme
```

### User Story 5 (Element Coverage)
```bash
# All UI file migrations can run in parallel:
- T026: Migrate clip_list.rs
- T027: Migrate preview.rs
- T028: Migrate status.rs
- T029: Migrate help.rs
- T030: Migrate error_modal.rs
- T031: Migrate mod.rs
```

### Across User Stories
```bash
# Can develop in parallel once US1 complete:
- US2 (Ctrl-R): T013-T016
- US4 (Export): T022-T025
```

---

## Validation Checklist

### User Story 1 Validation
- [x] Can create theme with common colors
- [x] Can reference common colors from elements
- [x] Empty theme with just defaults works
- [x] Invalid RGB rejected
- [x] Unknown color reference detected

### User Story 2 Validation
- [x] Ctrl-R reloads theme in <1 second
- [x] Invalid theme shows error modal
- [x] Previous theme persists on error
- [x] Atomic update (no partial state)

### User Story 3 Validation
- [x] Auto-reload within 1 second
- [x] Error modal appears automatically
- [x] Error modal clears automatically
- [x] <1% CPU when idle

### User Story 4 Validation
- [x] Can export all built-in themes
- [x] Export -> import round-trip works
- [x] Exported theme shows ALL values

### User Story 5 Validation
- [x] All UI files migrated
- [x] colorscheme.rs deleted
- [x] Visual inspection passes
- [x] 50-100 elements covered

### User Story 6 Validation
- [x] Error messages include file/line
- [x] Suggestions for typos
- [x] Lists valid options

---

## Success Criteria Mapping

From spec.md:

- **SC-001**: Users can create theme by editing TOML → US1
- **SC-002**: 100% element coverage → US5
- **SC-003**: Common colors reduce repetition → US1
- **SC-004**: Ctrl-R reload <1 second → US2
- **SC-005**: Auto-reload in dev mode <1 second → US3
- **SC-006**: Auto-clear error modal → US3
- **SC-007**: Export any theme → US4
- **SC-008**: Invalid themes show errors <3 seconds → US6
- **SC-009**: 90% can create theme from export → US4 + US6
- **SC-010**: TOML readable/editable → US1

---

## Notes

- **Format iteration is critical**: T006 must happen first - informs all subsequent work
- **Migration can be broken**: T026-T034 can leave UI broken temporarily - fast iteration prioritized
- **Tests are optional**: Generate unit/integration tests only if explicitly requested
- **US7 deferred**: Hierarchical groups are P3, not included in task breakdown
