# Tasks: Clipboard History Manager TUI

**Input**: Design documents from `/specs/001-clipboard-manager-tui/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/

**Tests**: Tests are NOT explicitly requested in the specification, so test tasks are excluded. Unit/integration tests can be added later if needed.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Project initialization and basic structure

- [x] T001 Initialize Cargo project with Clipr metadata in Cargo.toml
- [x] T002 [P] Create project directory structure per plan.md (src/{models,storage,clipboard,ui,image}/, tests/, benches/)
- [x] T003 [P] Add core dependencies to Cargo.toml (ratatui, crossterm, nucleo, bincode, serde, toml, anyhow, log, env_logger)
- [x] T004 [P] Create example config file clipr.toml.example with permanent register examples
- [x] T005 [P] Create lib.rs to export modules for testing
- [x] T006 [P] Add .gitignore for Rust project (target/, Cargo.lock if library)

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core data structures and traits that ALL user stories depend on

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [x] T007 [P] Define ClipContent enum in src/models/clip.rs (Text/Image/File variants)
- [x] T008 [P] Define ClipEntry struct in src/models/clip.rs with id, content, timestamp, pinned, name, description, register fields
- [x] T009 [P] Implement ClipEntry methods in src/models/clip.rs (new_text, new_image, new_file, preview, register management)
- [x] T010 [P] Define Registry with HashMap<char, u64> in src/models/registry.rs
- [x] T011 [P] Define ClipboardHistory struct in src/models/clip.rs with entries, max_entries, hash_to_id
- [x] T012 [P] Implement ClipboardHistory methods in src/models/clip.rs (add_entry, remove_entry, get_pinned, rotation logic with register protection)
- [x] T013 Define ClipboardBackend trait in src/clipboard/backend.rs (read_text, write_text, read_image, write_image, supports_images, name)
- [x] T014 Define Config structs in src/storage/config.rs (Config, GeneralConfig, PermanentRegisterValue)
- [x] T015 Define HistoryStorage trait in src/storage/history.rs (load, save, path methods)
- [x] T016 Define RegisterStorage trait in src/storage/registers.rs (load, save, clear methods)
- [x] T017 Define ConfigStorage trait in src/storage/config.rs (load, save, path methods)
- [x] T018 Create module exports in src/models/mod.rs
- [x] T019 [P] Create module exports in src/storage/mod.rs
- [x] T020 [P] Create module exports in src/clipboard/mod.rs

**Checkpoint**: ✅ Foundation ready - user story implementation can now begin in parallel

---

## Phase 3: User Story 1 - Browse and Select Clipboard History (Priority: P1) 🎯 MVP

**Goal**: Users can view clipboard history, use fuzzy search, see text previews, and select items to copy back to clipboard

**Independent Test**: Copy several text items, launch Clipr, use fuzzy search with `/`, navigate with j/k, view preview panel, press Enter to select, verify item copied to clipboard

### Implementation for User Story 1

**Storage Layer:**

- [x] T021 [P] [US1] Implement BincodeHistoryStorage in src/storage/history.rs (atomic write pattern with .tmp file)
- [x] T022 [P] [US1] Implement TomlConfigStorage in src/storage/config.rs (load/save, create_default with examples)
- [x] T023 [US1] Implement ensure_directories() in src/storage/mod.rs to create XDG data/config dirs

**Clipboard Backend:**

- [x] T024 [US1] Implement WaylandBackend in src/clipboard/wayland.rs (write_text, write_image via wl-copy)
- [x] T025 [US1] Implement create_backend() factory in src/clipboard/mod.rs (detect WAYLAND_DISPLAY vs DISPLAY env var)
- [x] T026 [US1] Implement clipboard watchers with wl-paste --watch in src/clipboard/watch.rs (spawn detached background processes)

**Fuzzy Search:**

- [x] T027 [US1] Implement SearchIndex wrapper around nucleo in src/models/search_index.rs (SmartCase and CaseSensitive modes)

**TUI Components:**

- [ ] T028 [P] [US1] Define AppMode enum in src/app.rs (Normal, Search, RegisterAssign, Help)
- [ ] T029 [P] [US1] Define App struct in src/app.rs (mode, history, registers, config, filter, selected_index, search_query, should_quit)
- [ ] T030 [US1] Implement App::new() in src/app.rs (load history, config, registers from storage)
- [ ] T031 [US1] Implement vim movement handlers in src/app.rs (handle_normal_key: j/k, gg/G, Ctrl-d/u)
- [ ] T032 [US1] Implement search mode handlers in src/app.rs (handle_search_key: char input, backspace, up/down navigation, two-stage Escape)
- [ ] T033 [US1] Implement App::update_search_results() in src/app.rs (call nucleo fuzzy matcher)
- [ ] T034 [US1] Implement App::select_entry() in src/app.rs (copy to clipboard via backend, set should_quit)
- [ ] T035 [P] [US1] Implement create_main_layout() in src/ui/layout.rs (40% left clip list, 60% right preview, 1-line status bar)
- [ ] T036 [P] [US1] Implement render_clip_list() in src/ui/clip_list.rs (List widget with indicators, vim-style selection)
- [ ] T037 [P] [US1] Implement render_preview() for text in src/ui/preview.rs (Paragraph widget with text wrapping)
- [ ] T038 [P] [US1] Implement render_search_input() in src/ui/search.rs (search bar at bottom with /query display)
- [ ] T039 [P] [US1] Implement render_status() in src/ui/status.rs (mode indicator, entry counts)
- [ ] T040 [US1] Implement App::draw() in src/app.rs (orchestrate all render functions)
- [ ] T041 [P] [US1] Create module exports in src/ui/mod.rs

**Main Entry Point:**

- [ ] T042 [US1] Implement CLI argument parsing in src/main.rs (--daemon flag, --help, default TUI mode)
- [ ] T043 [US1] Implement setup_terminal() and restore_terminal() in src/main.rs (crossterm raw mode, alternate screen)
- [ ] T044 [US1] Implement TUI event loop in src/main.rs (16ms poll, handle keys, render, check clipboard channel, exit logic)
- [ ] T045 [US1] Implement daemon mode entry point in src/main.rs (spawn wl-paste --watch processes, keep alive loop)

**Error Handling:**

- [ ] T046 [US1] Add anyhow::Result returns throughout with .context() for all I/O operations
- [ ] T047 [US1] Implement corrupted history recovery in src/storage/history.rs (backup .bin.corrupted, return empty history)

**Configuration & Logging:**

- [ ] T048 [P] [US1] Initialize env_logger in src/main.rs (check RUST_LOG env var)
- [ ] T049 [P] [US1] Add debug!/info! logging to storage operations in src/storage/
- [ ] T050 [P] [US1] Add debug! logging to clipboard operations in src/clipboard/

**Checkpoint**: At this point, User Story 1 should be fully functional - browse history, fuzzy search, text preview, vim navigation, select to copy

---

## Phase 4: User Story 2 - Image Clipboard Support with Preview (Priority: P2)

**Goal**: Users can copy images, view image previews in the TUI, and paste images back to applications

**Independent Test**: Take screenshot, copy image, launch Clipr, see image indicator in list, navigate to image, see thumbnail in preview panel, press Enter to select, verify image pastes into application

### Implementation for User Story 2

**Image Protocol:**

- [ ] T051 [P] [US2] Define ImageProtocol trait in src/image/protocol.rs (render method, supports_terminal, name)
- [ ] T052 [US2] Implement KittyImageProtocol in src/image/kitty.rs (use viuer crate for rendering)
- [ ] T053 [US2] Implement create_image_protocol() factory in src/image/mod.rs (detect terminal via env vars, return Option<Box<dyn ImageProtocol>>)
- [ ] T054 [P] [US2] Create module exports in src/image/mod.rs

**Clipboard Backend Extensions:**

- [ ] T055 [US2] Add read_image/write_image to WaylandBackend in src/clipboard/wayland.rs (wl-paste --type image/png)
- [ ] T056 [US2] Update clipboard monitoring in src/clipboard/monitor.rs (add image watcher with wl-paste --type image/png --watch)

**TUI Updates:**

- [ ] T057 [US2] Update render_clip_list() in src/ui/clip_list.rs (add image indicator icon/label)
- [ ] T058 [US2] Implement render_preview() for images in src/ui/preview.rs (call ImageProtocol::render, show fallback message if no protocol)
- [ ] T059 [US2] Update App::new() in src/app.rs (initialize image_renderer via create_image_protocol())

**Storage Updates:**

- [ ] T060 [US2] Verify ClipContent::Image serialization in src/models/clip.rs (bincode handles Vec<u8>)
- [ ] T061 [US2] Add max_image_size_bytes config check in src/storage/config.rs (reject images >50MB default)

**Checkpoint**: At this point, User Stories 1 AND 2 should both work - text and image clipboard support with appropriate previews

---

## Phase 5: User Story 3 - Pin Important Clips (Priority: P3)

**Goal**: Users can pin clips to prevent removal by history rotation, with visual distinction in the list

**Independent Test**: Copy several items, pin 2 with `p` key, copy many more items exceeding history limit, reopen Clipr, verify pinned items still present with pin indicator

### Implementation for User Story 3

- [ ] T062 [US3] Implement toggle_pin() in src/app.rs (flip ClipEntry.pinned field, save history)
- [ ] T063 [US3] Update handle_normal_key() in src/app.rs (add case for 'p' key → toggle_pin)
- [ ] T064 [US3] Update render_clip_list() in src/ui/clip_list.rs (add pinned indicator like 📌 or [P])
- [ ] T065 [US3] Update ClipboardHistory rotation logic in src/models/clip.rs (skip pinned entries when removing old clips)
- [ ] T066 [US3] Verify pinned field serialization in src/storage/history.rs (persists across restarts)

**Checkpoint**: Pinned clips now exempt from history rotation and visually distinguished

---

## Phase 6: User Story 4 - Named Registers for Frequent Content (Priority: P4)

**Goal**: Users can save clips to temporary registers with m<letter>, filter by registers with ' and ", and configure permanent registers in TOML

**Independent Test**: Configure permanent registers in config.toml, launch Clipr, mark clip with `ma`, filter temporary registers with `'`, filter permanent with `"`, select register, verify content copied to clipboard

### Implementation for User Story 4

**Register Storage:**

- [ ] T067 [P] [US4] Implement BincodeRegisterStorage in src/storage/registers.rs (load, save, clear with atomic writes)
- [ ] T068 [US4] Update App::new() in src/app.rs (load temporary registers from BincodeRegisterStorage, merge permanent from config)

**Register Operations:**

- [ ] T069 [US4] Implement enter_register_mode() in src/app.rs (set mode to RegisterAssign)
- [ ] T070 [US4] Implement handle_register_key() in src/app.rs (wait for letter, save current clip to register, save registers to storage)
- [ ] T071 [US4] Update handle_normal_key() in src/app.rs (add case for 'm' key → enter_register_mode)
- [ ] T072 [US4] Implement filter_temporary_registers() in src/app.rs (set filter to show only register entries where register.is_some())
- [ ] T073 [US4] Implement filter_permanent_registers() in src/app.rs (set filter to show only permanent register entries from config)
- [ ] T074 [US4] Update handle_normal_key() in src/app.rs (add case for '\'' key → filter_temporary_registers)
- [ ] T075 [US4] Update handle_normal_key() in src/app.rs (add case for '\"' key → filter_permanent_registers)
- [ ] T076 [US4] Update clear_filter() in src/app.rs (handle Escape to clear register filters)

**TUI Updates:**

- [ ] T077 [US4] Update render_clip_list() in src/ui/clip_list.rs (add register indicator like 🔖 or [R], show register letter)
- [ ] T078 [US4] Update render_clip_list() title in src/ui/clip_list.rs (show "Temporary Registers" or "Permanent Registers" when filtered)
- [ ] T079 [US4] Update render_status() in src/ui/status.rs (add registered count to stats)

**Checkpoint**: Temporary and permanent registers fully functional with vim-style mark/jump bindings

---

## Phase 7: User Story 5 - Manage Clips Within TUI (Priority: P5)

**Goal**: Users can delete individual clips, clear all history (with confirmation), and view help overlay

**Independent Test**: Launch Clipr, select clip, press delete key, verify removed from list, trigger clear all, confirm prompt, verify history cleared except pinned items, press `?` to see help overlay

### Implementation for User Story 5

**Delete Operations:**

- [ ] T080 [US5] Implement delete_entry() in src/app.rs (remove from history, save to storage)
- [ ] T081 [US5] Update handle_normal_key() in src/app.rs (add case for 'd' key → delete_entry)
- [ ] T082 [US5] Implement clear_all() in src/app.rs (remove all non-pinned entries, save to storage)
- [ ] T083 [US5] Implement confirmation prompt for clear_all in src/app.rs (require second keypress or modal)

**Help System:**

- [ ] T084 [P] [US5] Implement render_help_overlay() in src/ui/help.rs (centered overlay with keybinding list)
- [ ] T085 [P] [US5] Create help text content in src/ui/help.rs (vim movement, search, registers, pinning, delete, quit)
- [ ] T086 [US5] Implement show_help() and handle_help_key() in src/app.rs (set mode to Help, Escape to exit)
- [ ] T087 [US5] Update handle_normal_key() in src/app.rs (add case for '?' key → show_help)
- [ ] T088 [P] [US5] Create module exports for src/ui/help.rs in src/ui/mod.rs

**Checkpoint**: All user stories (US1-US5) are now independently functional

---

## Phase 8: X11 Fallback Support (Future Enhancement)

**Goal**: Support X11 display servers as fallback when Wayland not available

**Note**: This is not required for MVP but included in constitution's platform portability principle

- [ ] T089 [P] Add arboard dependency to Cargo.toml for X11 clipboard access
- [ ] T090 Implement X11Backend in src/clipboard/x11.rs (use arboard for read/write operations)
- [ ] T091 Update create_backend() in src/clipboard/mod.rs (add X11Backend when DISPLAY set but not WAYLAND_DISPLAY)
- [ ] T092 Add conditional compilation or feature flag for X11 support in Cargo.toml

---

## Phase 9: Polish & Cross-Cutting Concerns

**Purpose**: Improvements that affect multiple user stories

- [ ] T093 [P] Add performance benchmarks in benches/startup.rs (cold start via hyperfine)
- [ ] T094 [P] Add performance benchmarks in benches/search.rs (fuzzy search with nucleo)
- [ ] T095 [P] Add performance benchmarks in benches/ui_render.rs (frame time measurement)
- [ ] T096 [P] Create README.md with quick start, features, and installation instructions
- [ ] T097 [P] Update clipr.toml.example with comprehensive config examples and comments
- [ ] T098 Code cleanup and clippy lint pass (cargo clippy -- -D warnings)
- [ ] T099 Format all code (cargo fmt)
- [ ] T100 Verify quickstart.md instructions (build, run, test scenarios)
- [ ] T101 [P] Add unit tests for ClipEntry model in tests/unit/models/clip_test.rs (if desired)
- [ ] T102 [P] Add unit tests for storage round-trip in tests/unit/storage/ (if desired)
- [ ] T103 [P] Add integration test for daemon clipboard capture in tests/integration/clipboard_ops.rs (if desired)

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS all user stories
- **User Stories (Phase 3-7)**: All depend on Foundational phase completion
  - User stories CAN proceed in parallel (if staffed) or sequentially in priority order
  - Each story is independently testable
- **X11 Fallback (Phase 8)**: Optional, can be done anytime after Foundational
- **Polish (Phase 9)**: Depends on desired user stories being complete

### User Story Dependencies

- **User Story 1 (P1)**: Can start after Foundational (Phase 2) - No dependencies on other stories ✅ MVP
- **User Story 2 (P2)**: Can start after Foundational (Phase 2) - Extends US1 but independently testable
- **User Story 3 (P3)**: Can start after Foundational (Phase 2) - Extends US1 but independently testable
- **User Story 4 (P4)**: Can start after Foundational (Phase 2) - Extends US1 but independently testable
- **User Story 5 (P5)**: Can start after Foundational (Phase 2) - Extends US1 but independently testable

### Within Each User Story

- Models/traits before services
- Services before UI components
- Core implementation before integrations
- Storage layer can be parallel with clipboard backend
- TUI components can be parallel if they don't depend on each other
- Story complete before moving to next priority

### Parallel Opportunities

**Phase 1 - Setup**: T002, T003, T004, T005, T006 can all run in parallel

**Phase 2 - Foundational**: T007-T012 (models) and T018-T020 (module exports) can run in parallel after trait definitions complete

**Phase 3 - US1**:
- T021, T022 (storage) can run in parallel
- T028, T029 (app struct setup) can run in parallel
- T035, T036, T037, T038, T039, T041 (TUI components) can run in parallel after App struct defined
- T048, T049, T050 (logging) can run in parallel

**Phase 4 - US2**:
- T051, T054 (image protocol) can run in parallel
- T057, T058 (TUI updates) can run in parallel

**Phase 5 - US3**: All tasks sequential (modify existing components)

**Phase 6 - US4**:
- T067 (register storage) can run in parallel with other tasks
- T077, T078, T079 (TUI updates) can run in parallel

**Phase 7 - US5**:
- T084, T085, T088 (help system) can run in parallel

**Phase 9 - Polish**: T093, T094, T095, T096, T097, T101, T102, T103 can all run in parallel

---

## Parallel Example: User Story 1

```bash
# After Foundational phase completes, launch these US1 tasks together:

# Storage layer (parallel)
Task T021: "Implement BincodeHistoryStorage in src/storage/history.rs"
Task T022: "Implement TomlConfigStorage in src/storage/config.rs"

# After storage + clipboard backend ready, launch TUI components (parallel):
Task T035: "Implement create_main_layout() in src/ui/layout.rs"
Task T036: "Implement render_clip_list() in src/ui/clip_list.rs"
Task T037: "Implement render_preview() for text in src/ui/preview.rs"
Task T038: "Implement render_search_input() in src/ui/search.rs"
Task T039: "Implement render_status() in src/ui/status.rs"

# Logging (parallel with any other work):
Task T048: "Initialize env_logger in src/main.rs"
Task T049: "Add debug!/info! logging to storage operations"
Task T050: "Add debug! logging to clipboard operations"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational (CRITICAL - blocks all stories)
3. Complete Phase 3: User Story 1
4. **STOP and VALIDATE**: Test US1 independently with acceptance scenarios from spec.md
5. Run benchmarks to verify performance targets (<100ms cold start, 60fps, <100ms search)
6. Deploy/demo if ready

**Estimated MVP**: ~50 tasks (T001-T050)

### Incremental Delivery

1. Foundation (Phases 1-2) → Core data structures and traits ready
2. Add User Story 1 (Phase 3) → Test independently → **MVP deployed!** ✅
3. Add User Story 2 (Phase 4) → Test independently → Image support released
4. Add User Story 3 (Phase 5) → Test independently → Pinning released
5. Add User Story 4 (Phase 6) → Test independently → Registers released
6. Add User Story 5 (Phase 7) → Test independently → Management features released
7. Polish (Phase 9) → Performance optimization, documentation, benchmarks

Each story adds value without breaking previous stories.

### Parallel Team Strategy

With multiple developers:

1. Team completes Setup + Foundational together (~20 tasks)
2. Once Foundational is done:
   - **Developer A**: User Story 1 (MVP - highest priority)
   - **Developer B**: User Story 2 (image support)
   - **Developer C**: User Story 3 (pinning)
3. Stories complete and integrate independently
4. Sequential execution still recommended (P1 → P2 → P3...) to ensure incremental value delivery

---

## Performance Validation

After completing each user story phase, verify against constitution targets:

- **Cold Start**: `hyperfine 'cargo run --release -- --help'` → Must be <100ms
- **Search Performance**: `cargo bench fuzzy_search` → Must be <100ms for 1000 items
- **UI Frame Time**: `cargo bench ui_render` → Must be <16ms (60fps)
- **Memory Usage**: `valgrind --tool=massif ./target/release/clipr` → Must be <50MB with 1000 clips
- **Clipboard Response**: wl-paste --watch provides instant events (<1ms)

---

## Notes

- [P] tasks = different files, no dependencies - can run in parallel
- [Story] label (e.g., [US1]) maps task to specific user story for traceability
- Each user story should be independently completable and testable
- Commit after each task or logical group of parallel tasks
- Stop at any checkpoint to validate story independently
- Use `nix develop` for reproducible dev environment with all dependencies
- Run `cargo watch -x 'test --lib'` during development for fast iteration
- Constitution compliance: Performance-first, Simplicity, Debuggability, Iteration Speed, Platform Portability

**Total Tasks**: 103 (Setup: 6, Foundational: 14, US1: 30, US2: 11, US3: 5, US4: 13, US5: 9, X11: 4, Polish: 11)

**Parallel Opportunities**: 38 tasks marked [P] can run in parallel with other tasks in their phase
