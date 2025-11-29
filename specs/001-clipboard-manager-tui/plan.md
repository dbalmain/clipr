# Implementation Plan: Clipboard History Manager TUI

**Branch**: `001-clipboard-manager-tui` | **Date**: 2025-11-24 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/001-clipboard-manager-tui/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/commands/plan.md` for the execution workflow.

## Summary

Build a high-performance TUI for managing clipboard history on Linux (Wayland/X11) with fuzzy search, side-by-side preview, image support via kitty protocol, vim-style keybindings using mark/jump analogy, pinning, and permanent/temporary registers. Target <100ms cold start, 60fps UI, <50MB memory for 1000 clips.

## Technical Context

**Language/Version**: Rust (stable channel, latest 3 releases supported - currently 1.75+)
**Primary Dependencies**: NEEDS CLARIFICATION (TUI framework, fuzzy matching library, clipboard integration, image protocol support)
**Storage**: Local files (clipboard history, temporary registers, configuration) in user config directory
**Testing**: `cargo test` with unit tests for models/services, integration tests for clipboard operations
**Target Platform**: Linux (Wayland primary via wl-clipboard, X11 secondary)
**Project Type**: Single project (standalone TUI binary)
**Performance Goals**: <100ms cold start, <10ms clipboard event response, 60fps UI (16ms frame time), <100ms fuzzy search
**Constraints**: <50MB memory with 1000 clips, <200ms image preview render, no blocking on UI thread
**Scale/Scope**: Single-user local tool, 1000 clipboard entries, 50+ registers, 5-10 core modules

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

### I. Performance-First ✅

**Initial Requirements Met**:
- Cold start <100ms specified in FR-055
- Clipboard response <10ms p99 aligns with constitution
- UI 60fps (16ms frame time) aligns with constitution
- Memory <50MB with 1000 entries aligns with constitution
- Search/filter <100ms specified in FR-054

**Concrete Evidence (Post-Design)**:
- **TUI Framework**: ratatui proven to achieve 60fps in production (Helix, gitui, bottom)
- **Fuzzy Search**: nucleo benchmarks at <5ms for 1000 items (well under 100ms target)
- **Clipboard Monitoring**: wl-paste --watch provides instant event notification (<1ms)
- **Storage**: Bincode serialization ~10ms for 1000 entries (meets <50ms target)
- **Image Preview**: viuer async rendering prevents UI blocking
- **Cold Start**: Minimal dependencies + mold linker supports <100ms target
- **Benchmarking**: Criterion.rs + hyperfine + github-action-benchmark for CI regression detection

**Status**: ✅ PASS - Technology choices meet all performance targets with margin

### II. Simplicity/YAGNI ✅

**Initial Requirements Met**:
- Single project structure (not multi-service)
- Minimal feature set for MVP (P1 = browse/search clipboard)
- Incremental delivery (P1→P2→P3→P4→P5)
- Dependencies minimized (research phase will evaluate alternatives)

**Concrete Evidence (Post-Design)**:
- **Dependencies**: 7 core crates (ratatui, crossterm, nucleo, bincode, serde, toml, anyhow) - all well-justified
- **Clipboard Integration**: Direct wl-paste/wl-copy calls (no heavy abstraction), event-driven via --watch (simpler than polling or protocol implementation)
- **Storage**: Plain Bincode files (simpler than SQLite/database)
- **Architecture**: Elm pattern (clear Model-View-Update separation)
- **No Premature Optimization**: Image support deferred via trait (can add protocols incrementally)
- **Single Binary**: No daemons, no IPC complexity - just stdin/stdout with wl-clipboard

**Status**: ✅ PASS - Minimal complexity, well-justified dependencies, YAGNI principles followed

### III. Debuggability ✅

**Initial Requirements Met**:
- Error context required (spec assumptions mention error handling)
- Debug logging capability (will use env_logger or similar)
- Clear error messages for wl-paste/wl-copy failures required

**Concrete Evidence (Post-Design)**:
- **Error Handling**: anyhow for context-rich errors with `.context()` on all operations
- **Logging**: log + env_logger (lightweight, RUST_LOG=debug enables verbally, disabled by default)
- **Storage Safety**: Atomic writes (temp file + rename) prevent corruption on crashes
- **Error Recovery**: Corrupted history backed up and fresh start (user data preserved)
- **User-Facing Errors**: Clear messages for missing wl-clipboard, invalid UTF-8, clipboard unavailable
- **Debug Logging**: Module-level logging (e.g., RUST_LOG=clipr::clipboard=debug)

**Status**: ✅ PASS - Comprehensive debuggability without heavyweight observability

### IV. Iteration Speed ✅

**Initial Requirements Met**:
- Fast compile-test-debug cycle (Rust with incremental compilation)
- Tests run on save (`cargo test --lib` for unit tests)
- MVP-first approach (P1 is independently testable)

**Concrete Evidence (Post-Design)**:
- **Nix Flakes**: Reproducible dev environment with mold linker (faster linking)
- **Fast Tests**: Unit tests isolated from I/O, <1s target for `cargo test --lib`
- **cargo-watch**: Auto-run tests on file changes (in flake.nix)
- **Modular Design**: Trait abstractions enable testing without platform dependencies
- **Mock Backends**: ClipboardBackend trait allows mock for TUI testing
- **Incremental Build**: Minimal macros, domain-organized modules for fast recompilation
- **Hot Paths**: Critical performance code isolated for focused benchmarking

**Status**: ✅ PASS - Development environment optimized for rapid iteration

### V. Platform Portability ✅

**Initial Requirements Met**:
- Primary: Wayland (wl-clipboard) as per spec assumptions
- Secondary: X11 compatibility required
- Trait abstractions for platform-specific code required
- Feature flags for platform selection

**Concrete Evidence (Post-Design)**:
- **ClipboardBackend Trait**: Platform abstraction defined in contracts/clipboard.md
- **WaylandBackend**: Direct wl-paste/wl-copy subprocess calls (2-5ms latency)
- **X11Backend**: arboard crate for X11 compatibility (handles selection complexity)
- **Auto-Detection**: Factory pattern checks WAYLAND_DISPLAY vs DISPLAY env vars
- **ImageProtocol Trait**: Kitty/Sixel/iTerm2 protocols abstracted for terminal portability
- **No Platform Leakage**: All platform code isolated behind traits
- **Nix Flakes**: Cross-platform dev environment (NixOS, Linux, macOS)

**Status**: ✅ PASS - Clean platform abstractions with Wayland primary, X11 fallback

### Overall Constitution Compliance

**Result**: ✅ **PASS** - All 5 principles satisfied with concrete implementations

**Summary**:
- ✅ **Performance**: All targets met with proven technologies (ratatui 60fps, nucleo <5ms, wl-paste --watch instant)
- ✅ **Simplicity**: 7 core dependencies, event-driven architecture, single binary, no unnecessary abstraction
- ✅ **Debuggability**: anyhow + env_logger, atomic writes, corruption recovery, clear user errors
- ✅ **Iteration Speed**: Nix + mold, cargo-watch, <1s unit tests, modular design
- ✅ **Portability**: ClipboardBackend + ImageProtocol traits, Wayland/X11 support, auto-detection

**Design Decisions Validated**:
- Event-driven monitoring (wl-paste --watch) over polling: Simpler AND faster ✅
- Bincode over JSON: Faster AND smaller ✅
- Direct wl-clipboard calls over abstraction libraries: Simpler AND more performant ✅
- Separate files (history.bin, registers.bin): Prevents write conflicts ✅
- Trait abstractions: Clean separation without complexity ✅

**Next Steps**: Proceed to implementation with `/speckit.tasks` and `/speckit.implement`

## Project Structure

### Documentation (this feature)

```text
specs/001-clipboard-manager-tui/
├── spec.md              # Feature specification (completed)
├── plan.md              # This file (/speckit.plan command output)
├── research.md          # Phase 0 output (/speckit.plan command)
├── data-model.md        # Phase 1 output (/speckit.plan command)
├── quickstart.md        # Phase 1 output (/speckit.plan command)
├── contracts/           # Phase 1 output (/speckit.plan command)
│   ├── clipboard.md     # Clipboard backend interface contract
│   ├── storage.md       # Persistence interface contract
│   └── ui.md            # UI component contracts
└── tasks.md             # Phase 2 output (/speckit.tasks command - NOT created by /speckit.plan)
```

### Source Code (repository root)

```text
src/
├── main.rs              # Entry point, CLI argument parsing
├── app.rs               # Main application state and event loop
├── ui/                  # TUI rendering and interaction
│   ├── mod.rs
│   ├── layout.rs        # Two-panel layout management
│   ├── clip_list.rs     # Clip selector widget
│   ├── preview.rs       # Preview panel widget
│   ├── search.rs        # Search/filter widget
│   └── keybindings.rs   # Vim-style key handler
├── clipboard/           # Clipboard integration layer
│   ├── mod.rs
│   ├── backend.rs       # ClipboardBackend trait
│   ├── wayland.rs       # Wayland (wl-clipboard) implementation
│   ├── x11.rs           # X11 implementation (future)
│   └── monitor.rs       # Clipboard change monitoring
├── storage/             # Persistence layer
│   ├── mod.rs
│   ├── history.rs       # Clipboard history storage
│   ├── registers.rs     # Register storage (temp + permanent)
│   └── config.rs        # Configuration file parsing (TOML)
├── models/              # Core data structures
│   ├── mod.rs
│   ├── clip.rs          # ClipEntry model
│   ├── register.rs      # Register model
│   └── search_index.rs  # Fuzzy search index
├── image/               # Image handling
│   ├── mod.rs
│   ├── protocol.rs      # ImageProtocol trait
│   └── kitty.rs         # Kitty graphics protocol implementation
└── lib.rs               # Library exports for testing

tests/
├── integration/
│   ├── clipboard_ops.rs # End-to-end clipboard operations
│   ├── search.rs        # Fuzzy search accuracy tests
│   └── registers.rs     # Register persistence tests
└── unit/
    ├── models/
    ├── storage/
    └── clipboard/

benches/
├── startup.rs           # Cold start benchmarks
├── search.rs            # Fuzzy search performance
└── ui_render.rs         # Frame time benchmarks

Cargo.toml               # Project manifest and dependencies
clipr.toml.example       # Example configuration with permanent registers
README.md                # Project overview and quick start
```

**Structure Decision**:

Single project structure selected because:
- Standalone TUI application (not web/mobile)
- All code runs in single process
- No separate backend/frontend or multi-service architecture
- Simple src/ layout with domain-organized modules aligns with Rust conventions and Simplicity/YAGNI principle

Module organization:
- `ui/` - TUI-specific rendering logic (isolated from business logic)
- `clipboard/` - Platform abstractions for clipboard backends
- `storage/` - Persistence and configuration (isolated from UI)
- `models/` - Core domain types (shared across layers)
- `image/` - Image protocol abstractions (isolated capability)

This structure supports:
- Fast compilation (small modules, clear boundaries)
- Platform portability (clipboard/ and image/ are abstracted)
- Iteration speed (can test models/storage without UI)
- Simplicity (flat module hierarchy, no deep nesting)

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

No violations identified. Constitution Check: ✅ PASS

## Phase 0: Research Tasks

The following research tasks must be completed to resolve NEEDS CLARIFICATION items and validate technology choices before Phase 1 design.

### Research Task 1: TUI Framework Selection

**Question**: Which Rust TUI framework best meets performance and feature requirements?

**Candidates to evaluate**:
- `ratatui` (modern fork of tui-rs, active maintenance)
- `tui-rs` (original, possibly less active)
- `cursive` (higher-level, might be heavier)

**Evaluation criteria**:
- Performance: Can it maintain 60fps with 1000-item lists?
- Features: Two-panel layout, keyboard event handling, custom widgets
- Cold start overhead: Does it add significant startup latency?
- Community and maintenance: Active development, good documentation?
- Image support: Can integrate with kitty graphics protocol?

**Expected output**: Recommendation with rationale in research.md

### Research Task 2: Fuzzy Matching Library

**Question**: Which fuzzy matching algorithm/library provides best speed and quality?

**Candidates to evaluate**:
- `skim` (Rust port of fzf)
- `nucleo` (helix editor's fuzzy matcher)
- `fuzzy-matcher` (simple crate)
- Custom implementation using substring/trigram matching

**Evaluation criteria**:
- Match quality: Ranks relevant results highly
- Performance: <100ms for 1000 items
- Memory overhead: Minimal additional allocation
- Incremental search: Can update results as user types

**Expected output**: Recommendation with benchmarks in research.md

### Research Task 3: Clipboard Backend Integration

**Question**: How to integrate with wl-clipboard and structure platform abstraction?

**Research areas**:
- Wayland: How to call wl-paste/wl-copy (std::process::Command, library wrapper, direct wayland protocol?)
- Clipboard monitoring: Polling vs event-based detection of changes
- X11: What's the equivalent tooling (xclip, xsel) and abstraction strategy
- Trait design: What operations does ClipboardBackend trait need?

**Evaluation criteria**:
- Latency: <10ms p99 for clipboard operations
- Reliability: Doesn't miss clipboard changes
- Simplicity: Minimal dependencies
- Portability: Easy to swap Wayland/X11 implementations

**Expected output**: Architecture decision and trait definition in research.md

### Research Task 4: Image Protocol Implementation

**Question**: How to implement kitty graphics protocol for terminal image display?

**Research areas**:
- Kitty protocol specification and escape sequences
- Existing Rust implementations (viuer crate, custom)
- Terminal capability detection (how to know if kitty protocol supported?)
- Fallback strategy architecture (how to plug in alternative protocols later?)

**Evaluation criteria**:
- Works with kitty terminal emulator
- Image rendering <200ms for typical screenshots
- Graceful degradation when protocol unsupported
- Extensible design for future fallbacks

**Expected output**: Implementation approach and protocol trait design in research.md

### Research Task 5: Storage and Serialization

**Question**: How to persist clipboard history, registers, and configuration efficiently?

**Research areas**:
- File format for clipboard history (JSON, MessagePack, Bincode, custom binary?)
- Configuration parsing (toml crate, serde integration)
- Storage location (XDG_CONFIG_HOME, XDG_DATA_HOME standards)
- Image data storage (embed in history file vs separate files?)
- Concurrent access (multiple clipr instances? file locking?)

**Evaluation criteria**:
- Load time: Contributes <50ms to cold start budget
- Reliability: Corruption-resistant (atomic writes, checksums?)
- Size efficiency: 1000 text clips + metadata should be <5MB
- Human-readable config (TOML requirement from spec)

**Expected output**: Storage architecture and format decisions in research.md

### Research Task 6: Logging and Error Handling

**Question**: Which lightweight logging approach meets debuggability requirements?

**Candidates to evaluate**:
- `env_logger` (simple, env-var controlled)
- `simple_logger` (even simpler)
- `log` crate alone with custom backend
- No logging crate (manual debug macros)

**Evaluation criteria**:
- Binary size overhead: <100KB
- Runtime overhead when disabled: ~0ms
- Ease of use: Simple to add debug statements
- Integration: Works with error chain context

**Expected output**: Logging strategy and error handling patterns in research.md

### Research Task 7: Performance Benchmarking Strategy

**Question**: How to set up benchmarks for performance requirements?

**Research areas**:
- Criterion.rs for micro-benchmarks (search, render, clipboard ops)
- Cold start measurement approach (hyperfine, custom harness?)
- CI integration (regression detection on PRs)
- Profiling tools (cargo flamegraph, perf, cachegrind)

**Expected output**: Benchmarking setup plan in research.md

## Phase 1: Design Artifacts

*Will be generated after Phase 0 research completion*

### data-model.md
Data structures for ClipEntry, Register, Configuration, SearchIndex

### contracts/
Interface contracts for clipboard backends, storage layer, UI components

### quickstart.md
Developer onboarding guide: build, run, test, architecture overview

### Agent context update
Add selected dependencies to appropriate agent context file

## Next Steps

1. Complete Phase 0 research (7 tasks above)
2. Document findings in research.md
3. Generate Phase 1 design artifacts
4. Re-run Constitution Check
5. Ready for `/speckit.tasks` to generate implementation task list
