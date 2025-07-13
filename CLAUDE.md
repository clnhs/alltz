# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**alltz** is a terminal-based time zone viewer designed for developers, remote teams, and frequent schedulers. Inspired by Every Time Zone (everytimezone.com), it provides an at-a-glance view of major timezones with visual horizontal representation of yesterday, today, and tomorrow, with a vertical "now" indicator.

Built with Rust edition 2024, using ratatui for terminal UI and crossterm for cross-platform terminal manipulation.

## Architecture

The project has a simple structure:
- `src/main.rs`: Entry point with main function
- `Cargo.toml`: Project configuration with ratatui and crossterm dependencies

## Dependencies

- **ratatui** (0.29.0): Terminal user interface library for building rich TUI applications
- **crossterm** (0.29.0): Cross-platform terminal manipulation library
- **chrono** (future): Date and time handling for timezone calculations
- **chrono-tz** (future): Timezone database support for accurate conversions

## Common Commands

### Building and Running
```bash
# Build the project
cargo build

# Run the project
cargo run

# Build for release
cargo build --release
```

### Development
```bash
# Check code without building
cargo check

# Format code
cargo fmt

# Run linter
cargo clippy

# Run tests
cargo test
```

### Dependencies
```bash
# Add a new dependency
cargo add <crate_name>

# Update dependencies
cargo update
```

## Development Notes

- Project uses Rust edition 2024
- Built for terminal UI development with ratatui framework
- Uses crossterm for terminal event handling and manipulation
- See PLAN.md for detailed project architecture and development phases
- Core features: real-time timezone display, visual timeline, "now" indicator
- Target: developers, remote teams, and frequent schedulers
- Design inspired by everytimezone.com but optimized for CLI workflows