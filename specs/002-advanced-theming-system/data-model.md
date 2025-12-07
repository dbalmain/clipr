# Data Model: Advanced Theming System

**Feature**: 002-advanced-theming-system
**Date**: 2025-12-04
**Status**: Iteration 0 - Exploratory

## ⚠️ Important Notice

**This is iteration 0 of the theme format design.** The TOML structure and syntax shown here are **NOT FINAL** and will be refined during implementation (task step 1). Expect multiple iterations on:
- Color reference syntax (`$name` vs other approaches)
- Style specification format (exploring inline syntax like `"fg(#ff83d6) bg($primary) bold"`)
- TOML structure (flat vs nested sections)
- Element naming conventions

The goal is to start simple and iterate based on actual usage during implementation.

## Design Goals

1. **Human-readable and writable**: TOML format, concise syntax
2. **Zero-cost runtime**: Theme is a flat struct with direct field access (no HashMap lookups)
3. **Empty theme support**: With default fg/bg, a theme file can be completely empty and still valid
4. **Common colors**: Support reusable color definitions
5. **Export/import**: Built-in themes can be exported to TOML as starting points

## Core Entities

### Theme (Runtime Struct)

**Purpose**: Direct field access for all themed elements. Zero iteration, zero overhead.

**Structure** (conceptual - will evolve):
```rust
pub struct Theme {
    // Defaults
    pub default_fg: Color,
    pub default_bg: Color,

    // Clip List Elements (direct fields - no lookup needed)
    pub clip_number: Style,
    pub pin_indicator: Style,
    pub preview_text: Style,
    pub preview_text_selected: Style,
    pub temporary_register: Style,
    pub permanent_register: Style,
    pub timestamp: Style,
    pub item_count: Style,
    pub search_filter_header: Style,

    // Preview Panel Elements
    pub preview_content: Style,
    pub preview_loading: Style,
    pub preview_metadata_label: Style,
    pub preview_metadata_value: Style,

    // Status Bar Elements
    pub status_bar_bg: Color,  // Background might be just Color, not full Style
    pub status_bar_text: Style,
    pub status_bar_key_hint: Style,

    // Overlay Elements
    pub help_modal_bg: Color,
    pub help_modal_text: Style,
    pub error_modal_bg: Color,
    pub error_modal_border: Style,
    pub error_modal_text: Style,
    pub divider: Style,

    // Interactive States
    pub selection_highlight: Style,
    pub search_input: Style,
    pub register_filter: Style,
    pub numeric_prefix: Style,

    // ... all other elements as direct fields
}
```

**Usage**:
```rust
// Zero-cost access - just struct field lookup
let style = theme.temporary_register;
frame.render_widget(Span::styled(text, style), area);
```

**NOT this**:
```rust
// ❌ Don't do this - no Element enum, no lookups
let style = theme.get(Element::TemporaryRegister);
```

### ThemeDefinition (TOML Deserialization - Iteration 0)

**Purpose**: Intermediate format for loading from TOML. This structure will change as we iterate on syntax.

**Current thinking** (subject to change):
```toml
# Common color palette (optional)
[colors]
primary = "#89b4fa"      # or [137, 180, 250]? TBD in step 1
accent = "#f5c2e7"
background = "#1e1e2e"
text = "#cdd6f4"

# Defaults (required, but can reference colors or use direct values)
[defaults]
fg = "$text"             # or just "text"? or "#cdd6f4"? TBD
bg = "$background"

# Elements (all optional - fall back to defaults)
# Syntax to be determined in step 1 - exploring inline format:
[elements]
clip_number = "fg($accent) bold"
temporary_register = "fg(#7dcfff)"
status_bar_text = "fg($text)"
selection_highlight = "fg($accent) bold"

# Alternative syntax under consideration:
# clip_number.fg = "$accent"
# clip_number.bold = true
```

**Open Questions for Step 1**:
1. Color format: `"#89b4fa"` vs `[137, 180, 250]` vs both?
2. Color reference: `"$primary"` vs `"primary"` vs `"{primary}"`?
3. Style syntax: Inline `"fg($accent) bold"` vs nested TOML tables?
4. Modifier names: `"bold"` vs `"b"` vs `"BOLD"`?

### BuiltInTheme

**Purpose**: Hardcoded themes that can be exported via `clipr export-theme`.

**Structure**:
```rust
pub enum BuiltInTheme {
    CatppuccinMocha,
    CatppuccinLatte,
    TokyonightNight,
    TokyonightStorm,
    TokyonightDay,
}

impl BuiltInTheme {
    pub fn from_name(name: &str) -> Option<Self> { ... }

    pub fn to_theme(&self) -> Theme {
        // Return hardcoded Theme struct with all fields set
        match self {
            Self::CatppuccinMocha => Theme {
                default_fg: Color::Rgb(205, 214, 244),
                default_bg: Color::Rgb(30, 30, 46),
                clip_number: Style::default().fg(Color::Rgb(245, 194, 231)),
                // ... all other fields
            },
            // ... other themes
        }
    }

    pub fn to_toml(&self) -> String {
        // Export to TOML format (for export-theme command)
        // This is where we serialize the hardcoded theme to the TOML syntax
    }
}
```

## Loading Process (High-Level)

```
1. Check config for theme name
2. Is it a built-in theme name?
   ├─ Yes: Use BuiltInTheme::to_theme()
   └─ No: Load from ~/.config/clipr/themes/<name>.toml
3. If loading from file:
   ├─ Parse TOML -> ThemeDefinition
   ├─ Validate (resolve color references, check RGB ranges)
   └─ Convert ThemeDefinition -> Theme (resolve all fields)
4. If any field missing in TOML, use default_fg/default_bg
```

## Validation Rules

**Iteration 0 - Will refine in step 1**:
- RGB values: 0-255 range
- Color references: Must exist in `[colors]` section (if we keep this approach)
- Defaults: Required `fg` and `bg` in `[defaults]`
- Elements: All optional (fall back to defaults)

**Empty theme example**:
```toml
[defaults]
fg = "#cdd6f4"
bg = "#1e1e2e"

# No elements defined - everything uses defaults
```

## Migration Strategy

**Simple approach - can be broken temporarily**:

1. Create new `src/ui/theme.rs` with `Theme` struct
2. Add `clipr export-theme` command
3. Migrate UI code file-by-file to use `theme.field_name` instead of `colors().field_name`
4. Accept that things will be broken/incomplete during migration
5. Once migration complete, delete old `colorscheme.rs`

No need for both systems to coexist. We can iterate fast and fix as we go.

## Performance

**Zero-cost access**: `theme.temporary_register` is a direct struct field access. No:
- HashMap lookups
- Enum matching
- Function calls
- Dynamic dispatch

**Load-time cost**: All color reference resolution happens once at theme load/reload. Runtime is pure struct field access.

## Next Steps for Implementation

**Step 1** of tasks will focus on:
1. Experimenting with TOML syntax (inline vs nested, color format, reference syntax)
2. Implementing TOML parser with chosen syntax
3. Testing ergonomics of writing theme files by hand
4. Iterating on syntax based on actual usage

**Don't treat this document as gospel** - it's a starting point for iteration.
