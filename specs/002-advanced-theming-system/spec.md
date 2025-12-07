# Feature Specification: Advanced Theming System

**Feature Branch**: `002-advanced-theming-system`
**Created**: 2025-12-04
**Status**: Draft
**Input**: User description: "I want to do a deep dive on theming the app. I'd like to specify all the different parts of the app that need color, text style, and possible background. There will be a lot, so we should be able to group some of them. If you're creating a style, you should be able to set the style to the whole group and optionally override individual members of the group."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Comprehensive Element Theming with Common Colors (Priority: P1)

A user wants to create a custom theme by defining all visible UI elements with explicit colors, while also being able to define common color values that can be reused across multiple elements.

**Why this priority**: This is the foundation of the entire theming system. Users need to be able to theme every visible element and define reusable colors to create consistent themes efficiently.

**Independent Test**: Can be fully tested by creating a theme configuration file with common colors and element definitions, loading it, and verifying that all UI elements render with the correct colors. Delivers immediate value by allowing complete theme customization.

**Acceptance Scenarios**:

1. **Given** a user defines common colors (e.g., "primary", "accent", "background") in their theme, **When** elements reference these colors, **Then** the elements display using the defined common color values
2. **Given** a theme defines a default text color and background color, **When** any element is not explicitly styled, **Then** it falls back to these default colors
3. **Given** a user defines explicit colors for specific elements, **When** rendering those elements, **Then** they display with their explicit colors regardless of common color definitions
4. **Given** a user specifies foreground color, background color, and text modifiers (bold, italic) for an element, **When** rendering that element, **Then** all three style properties apply correctly
5. **Given** an element only specifies foreground color, **When** rendering, **Then** it uses the specified foreground and falls back to the default background color

---

### User Story 2 - Live Theme Reload with Ctrl-R (Priority: P1)

A user wants to edit their theme file and instantly see changes by pressing Ctrl-R, without restarting the application.

**Why this priority**: Fast iteration is crucial for theme development. Without live reload, users waste time restarting the app repeatedly, making theme creation frustrating and slow.

**Independent Test**: Can be tested by modifying a theme file while the app is running, pressing Ctrl-R, and verifying the UI immediately reflects the changes. Delivers immediate value by enabling rapid theme iteration.

**Acceptance Scenarios**:

1. **Given** the application is running with a theme loaded, **When** the user modifies the theme file and presses Ctrl-R, **Then** the UI immediately reloads and displays the updated theme
2. **Given** the user makes an error in the theme file, **When** they press Ctrl-R, **Then** an error modal displays with details about the problem and the previous theme remains active
3. **Given** the error modal is displayed after Ctrl-R, **When** the user presses any key, **Then** the modal closes and the previous theme remains active
4. **Given** multiple elements are changed in the theme, **When** pressing Ctrl-R, **Then** all changes apply atomically (no partial updates visible)

---

### User Story 3 - Automatic Theme Reload in Development Mode (Priority: P1)

A user wants to configure the application for theme development so it automatically polls the theme file and reloads when changes are detected, without requiring Ctrl-R.

**Why this priority**: Automatic reload removes even the Ctrl-R step, enabling the absolute fastest iteration cycle. Users can edit and save their theme file and immediately see results without touching the terminal.

**Independent Test**: Can be tested by starting clipr with theme development mode enabled, modifying the theme file, saving it, and verifying the UI automatically updates within 1 second. Delivers immediate value by maximizing iteration speed.

**Acceptance Scenarios**:

1. **Given** the application is running with theme development mode enabled, **When** the user modifies and saves the theme file, **Then** the UI automatically reloads and displays the updated theme within 1 second
2. **Given** theme development mode is enabled, **When** the user saves an invalid theme file, **Then** an error modal automatically appears displaying the validation errors
3. **Given** an error modal is displayed in theme development mode, **When** the user fixes the theme file and saves it, **Then** the error modal automatically disappears and the corrected theme loads
4. **Given** an error modal is displayed in theme development mode, **When** the user presses any key, **Then** the modal closes (but will reappear on next poll if error persists)
5. **Given** theme development mode is disabled (default), **When** the user modifies the theme file, **Then** no automatic reload occurs (user must press Ctrl-R)

---

### User Story 4 - Export Built-in Themes as Starter Templates (Priority: P1)

A user wants to export a built-in theme (like "catppuccin-mocha") to use as a starting point for their custom theme.

**Why this priority**: Starting from scratch is harder than modifying an existing theme. Export provides a discoverable way to learn the theme format and get a working baseline.

**Independent Test**: Can be tested by running `clipr export-theme catppuccin-mocha`, capturing the output, saving it to a file, and verifying it loads correctly as a custom theme. Delivers immediate value by making theme creation accessible.

**Acceptance Scenarios**:

1. **Given** a user runs `clipr export-theme catppuccin-mocha`, **When** the command executes, **Then** it outputs a complete, valid theme configuration to stdout
2. **Given** the exported theme output, **When** the user saves it to their config directory and loads it, **Then** it renders identically to the built-in theme
3. **Given** a user modifies the exported theme, **When** they reload it with Ctrl-R, **Then** their customizations apply correctly
4. **Given** an invalid theme name, **When** running export-theme, **Then** the command displays available theme names

---

### User Story 5 - Comprehensive Element Coverage (Priority: P1)

A user wants to customize the visual appearance of every visible element in the application, including clip list items, preview panels, status bars, overlays, and interactive states.

**Why this priority**: Without comprehensive element coverage, users cannot fully customize their experience. Missing elements would be stuck with hardcoded colors, breaking theme consistency.

**Independent Test**: Can be tested by creating a theme that intentionally uses distinct colors for every element type, then visually inspecting each part of the UI to verify the correct color appears. Delivers value by ensuring no UI element is "un-themeable".

**Acceptance Scenarios**:

1. **Given** a custom theme with unique colors for each element type, **When** viewing the clip list, **Then** all visible elements (clip number, pin indicator, preview text, registers, selection) display their assigned colors
2. **Given** a theme with custom preview panel styling, **When** viewing clip details, **Then** the preview content, metadata sections, and borders use the theme's colors
3. **Given** a theme with custom status bar colors, **When** viewing keyboard hints, **Then** the status bar background, text, and key indicators display the theme's colors
4. **Given** a theme with custom overlay styling, **When** opening help or confirmation dialogs, **Then** overlay backgrounds, borders, and text use the theme's colors
5. **Given** a theme with distinct register colors, **When** viewing clips with temporary and permanent registers, **Then** each register type displays its assigned color

---

### User Story 6 - Theme Validation and Error Handling (Priority: P2)

A user wants to receive helpful feedback when their theme configuration has errors, including missing required properties, invalid color values, or malformed syntax.

**Why this priority**: Good error handling prevents user frustration and helps them debug theme configurations. However, it's secondary to the actual theming functionality.

**Independent Test**: Can be tested by intentionally creating malformed theme files and verifying that clear, actionable error messages are displayed. Delivers value by making theme creation less error-prone.

**Acceptance Scenarios**:

1. **Given** a theme file with an invalid color value (e.g., "notacolor"), **When** loading the theme, **Then** the system displays an error identifying the invalid value and suggesting correct formats
2. **Given** a theme file missing a required group, **When** loading the theme, **Then** the system displays a warning and uses default values for the missing group
3. **Given** a theme file with malformed syntax, **When** loading the theme, **Then** the system displays the parsing error with line numbers and context
4. **Given** a theme references a non-existent element, **When** loading the theme, **Then** the system displays a warning listing valid element names

---

### User Story 7 - Hierarchical Style Groups (Priority: P3)

A user wants to organize theme elements into hierarchical groups where group-level defaults can be set and individual elements can override them.

**Why this priority**: This is a convenience feature that can make complex themes easier to maintain, but the core theming functionality (common colors and element definitions) already provides similar benefits. Can be added later as an enhancement.

**Independent Test**: Can be tested by creating a theme with style groups, setting group defaults, adding element overrides, and verifying inheritance works correctly. Delivers value by reducing repetition in large themes.

**Acceptance Scenarios**:

1. **Given** a theme defines a "text" group with default foreground color, **When** elements are assigned to that group, **Then** they inherit the group's color unless overridden
2. **Given** a group defines defaults and an element overrides one property, **When** rendering, **Then** the element uses its override for that property and inherits other properties from the group
3. **Given** multiple groups with different defaults, **When** elements reference their respective groups, **Then** each element correctly inherits from its assigned group

---

### Edge Cases

- What happens when a theme file is syntactically valid but semantically incomplete (e.g., missing some element definitions)?
- What happens when a theme file has invalid color values (and user presses Ctrl-R)?
- What happens when a theme file has invalid color values (and theme development mode is enabled)?
- What happens if the theme file is modified while the application is running in theme development mode?
- What happens when referencing a common color that doesn't exist?
- What happens if the theme file is deleted while theme development mode is enabled?
- What happens when the user fixes an invalid theme file after seeing an error modal in theme development mode?

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST allow defining common color values in themes that can be referenced by multiple elements
- **FR-002**: System MUST allow defining a default text color and default background color that apply to all elements not explicitly styled
- **FR-003**: System MUST allow individual elements to specify explicit style properties (foreground color, background color, text modifiers)
- **FR-004**: System MUST support RGB color format for 24-bit color terminals
- **FR-005**: System MUST support all ratatui text modifiers (BOLD, ITALIC, UNDERLINE, DIM, etc.)
- **FR-006**: System MUST theme all clipboard list elements including:
  - Clip numbers
  - Pin indicators
  - Preview text (selected and unselected states)
  - Temporary register indicators (with single quote prefix)
  - Permanent register indicators (with double quote prefix)
  - Timestamp/date text
  - Item count display
  - Search/filter header text
- **FR-007**: System MUST theme all preview panel elements including:
  - Content text (plain text, file paths)
  - Loading indicators
  - Metadata section (name, size, mime-type, description)
  - Register display in metadata
- **FR-008**: System MUST theme all status bar elements including:
  - Background
  - Normal text
  - Key binding indicators
  - Mode indicators
- **FR-009**: System MUST theme all overlay elements including:
  - Help dialog background and text
  - Confirmation dialog background and text
  - Error modal background, border, and text
  - Divider lines
- **FR-010**: System MUST theme special states including:
  - Selected item highlighting (bold text with color change)
  - Search input text
  - Register filter indicators
  - Numeric prefix display
- **FR-011**: Users MUST be able to load custom themes from configuration files
- **FR-012**: System MUST provide a `clipr export-theme <theme-name>` command that outputs built-in themes to stdout
- **FR-013**: Users MUST be able to reload the current theme by pressing Ctrl-R without restarting the application
- **FR-014**: System MUST display an error modal when Ctrl-R is pressed and the theme file is invalid, keeping the previous theme active
- **FR-015**: System MUST support a theme development mode that can be enabled via configuration
- **FR-016**: When theme development mode is enabled, system MUST automatically poll the theme file and reload when changes are detected (within 1 second)
- **FR-017**: When theme development mode is enabled and an invalid theme is detected, system MUST display an error modal automatically
- **FR-018**: When theme development mode is enabled and an error modal is displayed, system MUST automatically clear the modal and load the theme when the file becomes valid
- **FR-019**: System MUST validate theme files on load and provide clear error messages for invalid configurations
- **FR-020**: System MUST provide fallback behavior when theme properties are missing (use defaults)
- **FR-021**: System MUST store theme configuration in a human-readable, editable format (TOML)

### Key Entities *(include if feature involves data)*

- **Theme**: Represents a complete visual style, containing common colors, default colors, and element-specific styles
- **Common Colors**: Named color values (e.g., "primary", "accent") that can be referenced by multiple elements to maintain consistency
- **Default Colors**: A default text color and default background color that apply to any element not explicitly styled
- **Themed Element**: An individual UI component with style properties (foreground color, background color, modifiers)
- **Style Properties**: The three types of styling that can be applied to an element:
  - Foreground color (text color) - can reference a common color or specify RGB directly
  - Background color (fill color) - can reference a common color or specify RGB directly
  - Text modifiers (bold, italic, underline, dim, etc.)

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Users can create a complete custom theme by editing a single TOML configuration file
- **SC-002**: Users can customize any visible UI element (100% coverage of all text, backgrounds, and interactive states)
- **SC-003**: Users can define common colors and reference them from multiple elements, reducing repetition
- **SC-004**: Users can reload their theme with Ctrl-R and see changes within 1 second
- **SC-005**: Users can enable theme development mode and see automatic reloads within 1 second of saving the theme file
- **SC-006**: When theme development mode is enabled and an error occurs, users see an error modal that automatically clears when they fix the issue
- **SC-007**: Users can export any built-in theme using `clipr export-theme <name>` and get a valid, working theme file
- **SC-008**: Invalid theme configurations display error messages that identify the specific problem within 3 seconds
- **SC-009**: 90% of users can create a working custom theme by starting from an exported built-in theme
- **SC-010**: Theme files remain readable and editable by hand (TOML format, not binary or minified)

## Constraints

- Theme system must use ratatui Style, Color, and Modifier types to ensure compatibility
- Configuration format is TOML to match existing application configuration
- Only RGB color format is supported (24-bit color)
- Performance overhead of theme loading should be negligible (< 100ms)
- Theme reload with Ctrl-R should complete within 1 second
- Theme development mode polling should detect changes within 1 second
- Fast iteration is critical - theme changes must be visible immediately (Ctrl-R or automatic in dev mode)

## Dependencies

- Existing `src/ui/colorscheme.rs` implementation (will be refactored/replaced)
- Existing `src/storage/config.rs` configuration system
- ratatui's Style, Color, and Modifier APIs
- TOML parsing library (already in use: toml crate)

## Assumptions

1. Users creating custom themes have basic familiarity with RGB color values
2. The TOML format is acceptable for theme configuration (human-readable and supports nested structures)
3. Themes will be created by editing configuration files, not through a GUI
4. Users have 24-bit color terminal support
5. The number of themed elements will remain manageable (under 100)
6. Users can visually check their themes for readability and accessibility
7. Existing preset themes (Catppuccin, Tokyo Night) will be available as built-in themes that can be exported

## Out of Scope

- Backward compatibility with existing ColorScheme format (nobody is using clipr yet)
- Support for non-24-bit terminals (8-color, 16-color) - can wait for community contribution
- Color accessibility analysis or contrast checking - users can check visually
- Hierarchical style groups with inheritance (lower priority, can be added later as P3)
- Real-time theme editing GUI or interactive theme builder
- Theme marketplace or sharing platform
- Automatic theme generation from images or other sources
- Per-element animation or transition effects when switching themes
- Support for multiple themes loaded simultaneously
- Theme versioning or migration system
- Dynamic theme switching based on time of day or system preferences
