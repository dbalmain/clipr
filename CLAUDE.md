# clipr Development Guidelines

Auto-generated from all feature plans. Last updated: 2025-11-26

## Active Technologies
- Rust 1.75+ (stable channel, latest 3 releases supported) (002-advanced-theming-system)
- TOML configuration files in `~/.config/clipr/` directory (002-advanced-theming-system)

- Rust (stable channel, latest 3 releases supported - currently 1.75+) (001-clipboard-manager-tui)

## Project Structure

```text
src/
tests/
```

## Commands

cargo test [ONLY COMMANDS FOR ACTIVE TECHNOLOGIES][ONLY COMMANDS FOR ACTIVE TECHNOLOGIES] cargo clippy

## Code Style

Rust (stable channel, latest 3 releases supported - currently 1.75+): Follow standard conventions

## Recent Changes
- 002-advanced-theming-system: Added Rust 1.75+ (stable channel, latest 3 releases supported)

- 001-clipboard-manager-tui: Added Rust (stable channel, latest 3 releases supported - currently 1.75+)

<!-- MANUAL ADDITIONS START -->
<!-- MANUAL ADDITIONS END -->
- Do not do `cargo build --release` unless requested. Always build do `cargo build`.
