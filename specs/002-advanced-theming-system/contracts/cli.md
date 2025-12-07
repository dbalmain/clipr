# CLI Contract: export-theme Command

**Feature**: 002-advanced-theming-system
**Date**: 2025-12-04

## Command Specification

### `clipr export-theme <THEME_NAME>`

**Purpose**: Export a theme (built-in or custom) to TOML format with all default values filled in. Useful for:
- Creating a starting point from a built-in theme
- Seeing all available style properties with their current values
- Getting a complete reference showing what can be customized

**Arguments**:
- `<THEME_NAME>`: Name of theme to export (built-in or custom) (required)

**Output**: Complete TOML theme definition written to stdout with all fields populated

**Exit Codes**:
- `0`: Success - valid TOML written to stdout
- `1`: Error - invalid theme name or other error (message to stderr)

## Examples

### Export Built-in Theme

```bash
$ clipr export-theme catppuccin-mocha > ~/.config/clipr/themes/my-theme.toml
```

**stdout**:
```toml
# Catppuccin Mocha Theme
# Exported from clipr built-in themes

[colors]
primary = [137, 180, 250]
accent = [245, 194, 231]
background = [30, 30, 46]
text = [205, 214, 244]

[defaults]
fg = "$text"
bg = "$background"

[elements]
clip_number = "fg($accent) bold"
pin_indicator = "fg($text)"
# ... ALL elements with their values
temporary_register = "fg(#7dcfff)"
permanent_register = "fg($accent)"
# ... (every single themed element listed)
```

### Export Custom Theme (See Current Settings)

```bash
# User has a minimal custom theme with only a few overrides
$ cat ~/.config/clipr/themes/my-theme.toml
[defaults]
fg = [200, 200, 200]
bg = [20, 20, 20]

[elements]
clip_number = "fg(#ff0000) bold"

# Export it to see all values (including defaults)
$ clipr export-theme my-theme
```

**stdout**:
```toml
# my-theme
# Exported from custom theme

[defaults]
fg = [200, 200, 200]
bg = [20, 20, 20]

[elements]
clip_number = "fg(#ff0000) bold"
pin_indicator = "fg($text)"         # Uses default fg
temporary_register = "fg($text)"    # Uses default fg
status_bar_text = "fg($text)"       # Uses default fg
# ... ALL elements listed with their resolved values
```

**Value**: User can see exactly what every element looks like with their current theme, including fields they didn't explicitly set.

### Error Case: Unknown Theme

```bash
$ clipr export-theme nonexistent
```

**stdout**: (empty)

**stderr**:
```
Error: Unknown theme 'nonexistent'

Available themes:
  Built-in:
    - catppuccin-mocha
    - catppuccin-latte
    - tokyonight-night
    - tokyonight-storm
    - tokyonight-day
  Custom:
    - my-theme
    - dark-theme
```

**Exit code**: `1`

### Error Case: No Arguments

```bash
$ clipr export-theme
```

**stderr**:
```
Error: Missing required argument <THEME_NAME>

Usage: clipr export-theme <THEME_NAME>

Export a theme to TOML format with all values filled in.

Examples:
  clipr export-theme catppuccin-mocha
  clipr export-theme my-theme > ~/.config/clipr/themes/complete.toml
```

**Exit code**: `1`

## Functional Requirements Mapping

- **FR-012**: System MUST provide a `clipr export-theme <theme-name>` command that outputs themes to stdout
- **SC-005**: Users can export any theme and get a valid, working theme file
- **SC-007**: 90% of users can create a working custom theme by starting from an exported theme

## Implementation Notes

- Theme name matching is case-insensitive
- Export resolves the current active theme (built-in or loaded custom)
- ALL themed elements are included in export (not just user-defined ones)
- Default values are filled in for any undefined elements
- Output TOML must be valid and loadable by clipr
- Output should include comments for readability
- Exported theme should be identical when re-loaded
