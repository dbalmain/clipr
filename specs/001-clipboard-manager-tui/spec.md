# Feature Specification: Clipboard History Manager TUI

**Feature Branch**: `001-clipboard-manager-tui`
**Created**: 2025-11-24
**Status**: Draft
**Input**: User description: "Create a TUI for viewing and managing clipboard history. It will having a preview windows and will work with images and text. Unlike clipse, the preview window should show to the side of the clip selecter. The clip selecter should use fzf like finding to find the clips. It should use kitty protocol to show images, but should be built so a fallback method can be added in the future. It should have pinning, just like clipse, but it should also have registers, including some permanet registers for things like email address. The permanent registers should be configured in something like TOML, but everything else should be managed in the TUI."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Browse and Select Clipboard History (Priority: P1)

A user wants to view their clipboard history, search through it using fuzzy finding, see a preview of selected items, and paste a chosen item back to their clipboard.

**Why this priority**: This is the core functionality - without browsing and selecting from history, the tool has no value. This is the MVP that makes Clipr useful.

**Independent Test**: Can be fully tested by copying several text items, launching Clipr, using fuzzy search to filter items, viewing previews in the side panel, selecting an item, and verifying it's copied to clipboard.

**Acceptance Scenarios**:

1. **Given** clipboard contains 10+ text entries, **When** user launches the TUI, **Then** all entries are displayed in a list with the most recent at the top
2. **Given** user is viewing the clip list, **When** user activates search (/ key) and types search characters, **Then** the list filters in real-time showing only matching clips (fuzzy matching like fzf)
3. **Given** user is in search mode with filtered results, **When** user presses Escape once, **Then** search input closes, focus moves to first result, and filter remains active
4. **Given** user is viewing filtered results (after first Escape), **When** user presses Escape again, **Then** the filter clears and full clip list is restored
5. **Given** user is in search mode, **When** user presses up/down arrow keys, **Then** navigation works without clearing the filter or exiting search mode
6. **Given** user is browsing clips, **When** user navigates with vim keys (j/k for down/up), **Then** the preview panel on the side shows the full content of that clip
7. **Given** user has selected a clip, **When** user confirms selection (e.g., Enter key), **Then** the clip content is copied to the system clipboard and the TUI exits (or stays open based on configuration)

---

### User Story 2 - Image Clipboard Support with Preview (Priority: P2)

A user wants to copy images (screenshots, copied images from browsers), view thumbnail previews of images in the TUI, and paste images back to applications.

**Why this priority**: Image support is a key differentiator from basic text-only clipboard managers. Many users work with visual content and need this capability.

**Independent Test**: Can be tested by taking screenshots, copying images, launching Clipr, seeing image thumbnails in the preview panel, selecting an image, and verifying it pastes correctly into an image-capable application.

**Acceptance Scenarios**:

1. **Given** user copies an image to clipboard, **When** user launches the TUI, **Then** the image appears in the clip list with an indicator that it's an image
2. **Given** user navigates to an image clip, **When** the preview panel updates, **Then** a thumbnail of the image is displayed in the side preview panel
3. **Given** user selects an image clip, **When** user confirms selection, **Then** the image is copied to the system clipboard and can be pasted into other applications
4. **Given** the terminal supports image protocols, **When** displaying image previews, **Then** images are shown using the supported protocol (kitty protocol as primary)

---

### User Story 3 - Pin Important Clips (Priority: P3)

A user wants to mark certain clipboard entries as "pinned" so they persist and remain easily accessible even as new clipboard entries are added.

**Why this priority**: Pinning enables users to keep frequently-used snippets readily available without them being pushed out by the history limit. This improves efficiency for power users.

**Independent Test**: Can be tested by copying several items, pinning 2-3 of them via keyboard shortcut, copying many more items, reopening Clipr, and verifying pinned items are still present and marked distinctly.

**Acceptance Scenarios**:

1. **Given** user is viewing a clip in the list, **When** user triggers the pin action (keyboard shortcut), **Then** the clip is marked as pinned with a visual indicator
2. **Given** multiple clips are pinned, **When** user views the clip list, **Then** pinned clips appear in a separate section or are visually distinguished from regular clips
3. **Given** clipboard history reaches its limit, **When** new items are added, **Then** pinned items are never automatically removed
4. **Given** user no longer needs a pinned clip, **When** user unpins it, **Then** it reverts to a normal clip and can be removed by history rotation

---

### User Story 4 - Named Registers for Frequent Content (Priority: P4)

A user wants to store frequently-used text snippets (email addresses, common responses, code templates) in named registers that persist across sessions and are quickly accessible.

**Why this priority**: Registers transform Clipr from a passive history viewer into an active productivity tool for storing and recalling frequently-used content.

**Independent Test**: Can be tested by configuring permanent registers in a config file, launching Clipr, marking clips with m<letter>, filtering to registers with ' or ", selecting a register, and verifying its content is copied to clipboard.

**Acceptance Scenarios**:

1. **Given** user has configured permanent registers in a config file (e.g., "email", "work_address"), **When** user launches Clipr, **Then** permanent registers are loaded and accessible
2. **Given** user is viewing a clip, **When** user presses m followed by a letter (e.g., ma), **Then** the clip is saved to temporary register 'a' and persists across restarts
3. **Given** user is in the main view, **When** user presses single quote ('), **Then** view filters to show only temporary register contents
4. **Given** user is in the main view, **When** user presses double quote ("), **Then** view filters to show only permanent register contents
5. **Given** user is viewing filtered registers (temporary or permanent), **When** user navigates with j/k and selects a register, **Then** the register's content is copied to the clipboard
6. **Given** user is in register filter view, **When** user presses Escape, **Then** the filter clears and full clip list is restored
7. **Given** permanent registers are defined in config, **When** config is reloaded, **Then** permanent register updates take effect without losing temporary registers

---

### User Story 5 - Manage Clips Within TUI (Priority: P5)

A user wants to delete individual clips, clear all history, delete temporary registers, and perform other management tasks directly within the TUI without editing files.

**Why this priority**: Management features improve usability but aren't required for the core clipboard functionality. These are quality-of-life improvements.

**Independent Test**: Can be tested by launching Clipr, selecting a clip, deleting it, verifying it's removed from the list, clearing all history, and confirming the clip list is empty (except pinned items).

**Acceptance Scenarios**:

1. **Given** user is viewing a clip, **When** user triggers delete action (keyboard shortcut), **Then** the clip is removed from history and no longer appears in the list
2. **Given** user wants to clear all history, **When** user triggers clear all action (with confirmation prompt), **Then** all non-pinned clips are removed
3. **Given** user has temporary registers, **When** user selects a register and triggers delete, **Then** the register is removed permanently
4. **Given** user is viewing the TUI, **When** user requests help (e.g., ? key), **Then** a help overlay shows all available keyboard shortcuts and actions

---

### Edge Cases

- What happens when clipboard contains very large text items (>1MB)? Should they be truncated in the preview or list?
- How does the system handle binary clipboard content that isn't text or images?
- What happens when clipboard history file is corrupted or missing?
- How are clipboard entries handled when terminal window is too small to display preview panel?
- What happens when an image protocol is not supported and no fallback is available yet?
- How does the system handle clipboard content with special characters, unicode, or multi-line formatting?
- What happens when a permanent register name in config conflicts with a temporary register name?
- How are clipboard changes handled while the TUI is open - are they automatically added to history?

## Requirements *(mandatory)*

### Functional Requirements

**Core Clipboard Management:**

- **FR-001**: System MUST monitor system clipboard and capture all new clipboard entries (text and images)
- **FR-002**: System MUST store clipboard history persistently across application restarts
- **FR-003**: System MUST display clipboard entries in reverse chronological order (most recent first)
- **FR-004**: System MUST allow users to select any historical clipboard entry and copy it back to the system clipboard
- **FR-005**: System MUST limit clipboard history to a configurable maximum number of entries (with reasonable default like 1000)

**User Interface Layout:**

- **FR-006**: TUI MUST use a two-panel layout with clip selector on the left/main area and preview panel on the right/side
- **FR-007**: TUI MUST support keyboard navigation for all operations (no mouse required)
- **FR-008**: Preview panel MUST update in real-time as user navigates through clips in the selector
- **FR-009**: TUI MUST be responsive and maintain usability in terminal windows of varying sizes

**Search and Filtering:**

- **FR-010**: System MUST provide fuzzy search functionality for filtering clipboard entries (similar to fzf)
- **FR-011**: Search MUST filter entries in real-time as user types
- **FR-012**: Search MUST match against the text content of clips
- **FR-013**: Users MUST be able to clear search filter and return to full list

**Content Type Support:**

- **FR-014**: System MUST support text clipboard entries with full unicode support
- **FR-015**: System MUST support image clipboard entries
- **FR-016**: System MUST distinguish between text and image entries in the clip list (visual indicator)
- **FR-017**: Preview panel MUST display full text content for text clips
- **FR-018**: Preview panel MUST display image thumbnails for image clips using image display protocol
- **FR-019**: System MUST be architecturally designed to support adding alternative image display methods in the future

**Image Display Protocol:**

- **FR-020**: System MUST attempt to use kitty graphics protocol for displaying images when available
- **FR-021**: System MUST gracefully handle situations where image protocol is not supported (with clear user feedback)

**Pinning:**

- **FR-022**: Users MUST be able to pin individual clipboard entries
- **FR-023**: Pinned entries MUST be visually distinguished from regular entries
- **FR-024**: Pinned entries MUST never be automatically removed from history (exempt from history limit)
- **FR-025**: Users MUST be able to unpin previously pinned entries
- **FR-026**: Pinned status MUST persist across application restarts

**Registers:**

- **FR-027**: System MUST support permanent registers defined in a configuration file
- **FR-028**: Configuration file MUST use a human-readable structured format (e.g., TOML-like)
- **FR-029**: Users MUST be able to create temporary registers from clipboard entries using m<letter> syntax (e.g., ma creates register 'a')
- **FR-030**: Temporary register names MUST be single letters (a-z, A-Z)
- **FR-031**: Temporary registers MUST persist across application restarts until explicitly deleted
- **FR-032**: Single quote (') MUST filter the view to show only temporary register contents
- **FR-033**: Double quote (") MUST filter the view to show only permanent register contents
- **FR-034**: Users MUST be able to select and copy register content to clipboard from filtered register views
- **FR-035**: Users MUST be able to delete temporary registers from within the TUI
- **FR-036**: Permanent registers MUST NOT be deletable from the TUI (only via config file edit)
- **FR-037**: Escape key MUST clear register filters and return to full clip list view

**Management Operations:**

- **FR-038**: Users MUST be able to delete individual clipboard entries from within the TUI
- **FR-039**: Users MUST be able to clear all non-pinned clipboard history with a confirmation prompt
- **FR-040**: System MUST provide keyboard shortcuts for all operations
- **FR-041**: System MUST provide a help view showing all available keyboard shortcuts

**Keyboard Bindings:**

- **FR-042**: Keyboard bindings MUST follow vim conventions using mark/jump analogy for navigation and register operations
- **FR-043**: Movement keys MUST use vim navigation (j/k for up/down, gg/G for top/bottom, Ctrl-d/Ctrl-u for half-page scrolling)
- **FR-044**: Saving clips to registers MUST use vim mark syntax: m followed by register letter (e.g., ma saves current clip to register 'a')
- **FR-045**: Single quote (') MUST filter view to show only temporary register contents
- **FR-046**: Double quote (") MUST filter view to show only permanent register contents
- **FR-047**: Forward slash (/) MUST activate fuzzy search mode
- **FR-048**: When in search mode, up/down arrow keys MUST navigate results without exiting search mode or clearing the filter
- **FR-049**: When in search mode, first Escape press MUST exit search input, focus on first result, and preserve the active filter
- **FR-050**: When in search mode, second Escape press MUST clear the filter entirely and return to full clip list
- **FR-051**: Navigation keys (j/k) MUST work normally when focused on filtered results (after first Escape)
- **FR-052**: All vim-style bindings MUST be documented in the help view

**Performance:**

- **FR-053**: TUI MUST remain responsive with up to 1000 clipboard entries
- **FR-054**: Search/filter operations MUST provide results in under 100ms for typical history sizes
- **FR-055**: Application cold start MUST complete in under 100ms

### Key Entities

- **Clip Entry**: A single captured clipboard item with content (text or image data), timestamp, content type indicator, pinned status, and optional metadata
- **Register**: A named storage location for frequently-used content, with name, content, type (permanent or temporary), and creation timestamp
- **Configuration**: Permanent register definitions with name-to-content mappings, history size limits, UI preferences, and keybinding customizations
- **Clipboard History**: Ordered collection of clip entries with automatic rotation based on history limit (excluding pinned entries)

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Users can launch the application and view their clipboard history in under 100ms
- **SC-002**: Users can locate and select a specific clip from a list of 100 items in under 5 seconds using fuzzy search
- **SC-003**: Image previews render within 200ms of navigation for images under 5MB
- **SC-004**: Application handles 1000 clipboard entries without perceptible performance degradation (UI responsiveness, search speed)
- **SC-005**: Pinned clips remain accessible after 1000+ new clipboard entries have been added
- **SC-006**: Users can create, name, and recall temporary registers without ever editing configuration files
- **SC-007**: 95% of keyboard operations complete within 50ms (excluding clipboard I/O operations)
- **SC-008**: Application startup time remains under 100ms even with full clipboard history and 50+ registers

### Assumptions

- Users are running a terminal emulator on a Linux system with Wayland or X11
- Terminal emulator supports standard terminal capabilities (256 colors minimum, UTF-8)
- System clipboard is accessible via standard clipboard tools (wl-clipboard for Wayland)
- For image display: terminal emulator supports kitty graphics protocol (fallback methods will be added in future)
- Clipboard history storage location is writable and has sufficient disk space (configurable, default to user config directory)
- Users are comfortable with keyboard-driven interfaces and vim-style navigation keybindings
- Default history limit of 1000 entries is sufficient for most users (configurable)
- Image clipboard entries are limited to reasonable sizes (under 50MB) for performance
- Configuration file format will follow standard TOML conventions
- Permanent registers support both text content (inline) and file references (for text or images)
