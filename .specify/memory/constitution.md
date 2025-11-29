<!--
Sync Impact Report (Version 1.1.0 - Refined Observability Principle)
====================================================================
Version: 1.0.0 → 1.1.0 (MINOR: Principle refinement to match project scope)
Project: Clipr - High-performance Rust TUI for clipboard history management

Modified Principles:
- III. Observability → Debuggability
  - Old: Heavy observability with structured logging (tracing crate), filterable logs, tracing spans
  - New: Lightweight debuggability with error context, optional debug logging, clear error messages
  - Rationale: Clipr is a simple TUI wrapping wl-paste/wl-copy, not a distributed service.
    Heavyweight observability infrastructure is overkill and violates Simplicity/YAGNI principle.

Technical Constraints Updated:
- Core dependency changed: `tracing` → `env_logger` or similar (lightweight, optional)

Templates Status:
✅ plan-template.md: No changes needed (Constitution Check remains generic)
✅ spec-template.md: No changes needed (requirements-focused, technology-agnostic)
✅ tasks-template.md: No changes needed (task categorization still aligns)
✅ All command files: No changes needed (generic guidance maintained)

Follow-up Actions:
- None - all placeholders resolved
- Constitution ready for use by /speckit.plan and other workflows

Rationale for Version 1.1.0:
MINOR bump appropriate because this refines an existing principle without removing it or
making backward-incompatible governance changes. The principle still addresses error handling
and debugging, just with appropriate scope for a simple TUI application.
-->

# Clipr Constitution

Clipr is a high-performance Terminal User Interface (TUI) for managing clipboard history, built in Rust with a focus on speed, simplicity, and developer iteration velocity.

## Core Principles

### I. Performance-First

**Goal**: Clipr MUST be the fastest TUI clipboard manager available.

**Non-Negotiable Rules**:
- All operations MUST prioritize low latency and minimal resource consumption
- Clipboard monitoring MUST NOT introduce perceptible lag in user workflows
- UI rendering MUST maintain 60fps responsiveness under normal load
- Memory footprint MUST remain minimal even with extensive clipboard history
- Performance regressions MUST be caught in benchmarks before merge

**Rationale**: Users choose Clipr specifically for speed. Performance is the primary differentiator and MUST NOT be sacrificed for convenience features.

### II. Simplicity/YAGNI (You Aren't Gonna Need It)

**Non-Negotiable Rules**:
- Start with the simplest implementation that solves the immediate need
- Features MUST solve real, demonstrated user problems (not hypothetical future needs)
- Complexity MUST be justified in writing before implementation
- Abstractions MUST have at least 2 concrete use cases before introduction
- Dependencies MUST be minimized; evaluate alternatives before adding crates

**Rationale**: Complexity slows iteration speed and makes performance optimization harder. Simple code is faster to write, test, debug, and optimize.

### III. Debuggability

**Non-Negotiable Rules**:
- Error paths MUST include sufficient context for debugging (error chains, backtraces in debug builds)
- Debug logging available for troubleshooting (can use simple `env_logger` or similar)
- Errors from wl-paste/wl-copy MUST be captured and presented clearly to users

**Rationale**: TUIs are harder to debug than typical CLI tools. Good error messages and optional debug logging enable quick issue resolution without heavyweight observability infrastructure.

### IV. Iteration Speed

**Non-Negotiable Rules**:
- Development workflow MUST support rapid compile-test-debug cycles
- Breaking changes during early development (pre-1.0) are acceptable if they accelerate progress
- Tests MUST be fast enough to run on every save (use `cargo test` with `--lib` for unit tests)
- Documentation and polish are secondary to working functionality until feature stability is achieved

**Rationale**: Fast iteration enables experimentation and quick validation of approaches, crucial for finding optimal solutions in performance-critical code.

### V. Platform Portability

**Non-Negotiable Rules**:
- Primary clipboard backend MUST be Wayland (wl-clipboard integration)
- X11 compatibility MUST be achievable through clear adapter interfaces
- Platform-specific code MUST be isolated behind trait abstractions
- Build process MUST clearly document platform requirements and feature flags

**Rationale**: Modern Linux systems use Wayland, but X11 remains widely deployed. Clear separation enables maintenance without coupling to specific display protocols.

## Technical Constraints

**Language**: Rust (stable channel, latest 3 releases supported)

**Performance Requirements**:
- Clipboard event response: <10ms p99
- UI frame time: <16ms (60fps)
- Memory usage: <50MB with 1000 clipboard entries
- Cold start time: <100ms

**Core Dependencies** (justified):
- `ratatui` or `tui-rs`: TUI framework (essential for terminal UI)
- `env_logger` or similar: Optional debug logging (debuggability principle)
- Wayland/X11 clipboard libraries: Platform integration (portability principle)

**Forbidden Patterns**:
- Global mutable state (use message passing or explicit state containers)
- Blocking operations on UI thread (use async or dedicated threads)
- Unbounded allocations (cap history size, implement eviction)

## Development Workflow

**Iteration-First Approach**:
1. Write minimal code to validate approach
2. Measure performance with benchmarks
3. Iterate on bottlenecks
4. Add tests for stable components
5. Document once behavior is stable

**Complexity Justification Process**:
When introducing complexity (new abstraction, dependency, architectural pattern):
1. Document the problem being solved
2. List simpler alternatives considered
3. Explain why simpler alternatives are insufficient
4. Get approval before implementation

**Performance Validation**:
- Benchmarks MUST exist for clipboard operations, UI rendering, and search
- Criterion.rs or similar for regression detection
- Profile with `cargo flamegraph` or `perf` before optimizing

## Governance

**Amendment Process**:
1. Propose change via documented rationale (why current principle insufficient)
2. Discuss impact on existing code and workflows
3. Update constitution with version bump
4. Update affected templates and documentation
5. Add migration guidance if breaking changes to development process

**Versioning Policy**:
- MAJOR: Backward incompatible governance changes (e.g., removing principle, changing non-negotiable rule)
- MINOR: New principle added or existing principle materially expanded
- PATCH: Clarifications, wording improvements, non-semantic refinements

**Compliance**:
- All features MUST validate against these principles during planning (`/speckit.plan`)
- Performance regressions are treated as bugs
- Complexity without justification MUST be rejected in review

**Constitution Supersedes**:
This constitution is the authoritative source for project governance. In case of conflict between this document and other guidance, this constitution takes precedence.

**Version**: 1.1.0 | **Ratified**: 2025-11-24 | **Last Amended**: 2025-11-24
