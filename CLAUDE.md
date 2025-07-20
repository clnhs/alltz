# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**alltz** is a terminal-based timezone viewer for developers and remote teams. It provides visual timeline scrubbing across multiple timezones with DST indicators, color themes, and persistent configuration.

Built with Rust, using ratatui for terminal UI, designed for fast CLI workflow integration.

## Architecture

The project structure:
- `src/main.rs`: CLI entry point and TUI event loop
- `src/app.rs`: Main application state and message handling
- `src/time.rs`: Timezone management and time calculations  
- `src/config.rs`: Configuration persistence and color themes
- `src/ui/timeline.rs`: Timeline widget rendering
- `.github/workflows/`: CI/CD automation for releases

## Dependencies

- **ratatui** (0.29.0): Terminal user interface library
- **crossterm** (0.29.0): Terminal manipulation and events
- **chrono** (0.4): Date and time handling
- **chrono-tz** (0.10): Timezone database support
- **clap** (4.5): CLI argument parsing
- **serde/toml** (1.0): Configuration serialization

## Common Commands

### Building and Running
```bash
# Build and run in development
cargo run

# Run with CLI options
cargo run -- --help
cargo run -- time Tokyo
cargo run -- list

# Build for release
cargo build --release
```

### Development
```bash
# Run tests
cargo test

# Format code  
cargo fmt

# Run linter
cargo clippy

# Local development build
./release-local.sh
```

### Release Process
```bash
# Pre-release checks
- ensure you're on main branch with clean working directory
- ensure CHANGELOG.md is up to date with new version
- review README.md and update if necessary
- update version in Cargo.toml if needed
- ensure all tests pass: `cargo test`
- ensure code passes formatting: `cargo fmt -- --check`  
- ensure code passes linting: `cargo clippy -- -D warnings`
- test key functionality: `cargo run -- --help` and `cargo run`

# Create release (triggers GitHub Actions)
git tag v0.X.Y
git push origin v0.X.Y

# Verify release
- check GitHub Actions completed successfully
- verify release artifacts are created
- test download and installation works

# Local testing only
./release-local.sh
```

## Current Features

- ✅ **Multi-timezone display** with UTC offset ordering
- ✅ **Custom timezone names** - personalize with team member names or labels (e/E keys)
- ✅ **Timeline scrubbing** with visual indicators
- ✅ **6 color themes** (Default, Ocean, Forest, Sunset, Cyberpunk, Monochrome)
- ✅ **DST indicators** (⇈ spring forward, ⇊ fall back)
- ✅ **Persistent configuration** (~/.config/alltz/config.toml) with backward compatibility
- ✅ **CLI commands** (list, time <city>, zone <city>)
- ✅ **Date display** toggle
- ✅ **12/24 hour format** toggle

## Key Behaviors

- Configuration auto-loads from ~/.config/alltz/config.toml
- Zones ordered by UTC offset (-12 to +14)
- Timeline shows 48-hour span (yesterday → today → tomorrow)
- Real-time updates every second
- Vim-like navigation (h/j/k/l)

## Testing

- Run `cargo test` for unit tests
- GitHub Actions runs CI on push to main
- Release workflow builds cross-platform binaries
- Homebrew formula supports precompiled + source builds
