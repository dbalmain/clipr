# clipr Development Guidelines

Auto-generated from all feature plans. Last updated: 2025-11-26

## Active Technologies

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

- 001-clipboard-manager-tui: Added Rust (stable channel, latest 3 releases supported - currently 1.75+)

<!-- MANUAL ADDITIONS START -->
<!-- MANUAL ADDITIONS END -->
- Do not do `cargo build --release` unless requested. Always build do `cargo build`.