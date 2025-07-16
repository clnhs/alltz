# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.1] - 2024-12-18

### Added
- Midnight markers (┊) on timelines showing day boundaries in each timezone
- Color legend showing timeline indicators and their meanings
- Adaptive timeline window that expands time view on wider screens

### Changed
- Timeline now shows 60+ hours on typical terminals instead of stretched 48 hours
- Midnight markers use night color from current theme for visual consistency
- Timeline character density optimized for better wide-screen experience

### Fixed
- Timeline appearance on ultra-wide monitors no longer looks stretched
- Improved readability with consistent character density across screen sizes

## [0.1.0] - 2024-12-17

### Added
- Multi-timezone timeline display with UTC offset ordering
- Visual timeline scrubbing with keyboard navigation
- 6 color themes (Default, Ocean, Forest, Sunset, Cyberpunk, Monochrome)
- DST transition indicators (⇈ spring forward, ⇊ fall back)
- Persistent configuration (~/.config/alltz/config.toml)
- CLI commands: `list`, `time <city>`, `zone <city>`
- Date display toggle
- 12/24 hour format toggle
- Real-time updates every second
- Vim-like navigation (h/j/k/l)
- Add/remove timezone functionality
- Interactive help system
- Cross-platform support (macOS, Linux, Windows)

[Unreleased]: https://github.com/abradburne/alltz/compare/v0.1.1...HEAD
[0.1.1]: https://github.com/abradburne/alltz/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/abradburne/alltz/releases/tag/v0.1.0