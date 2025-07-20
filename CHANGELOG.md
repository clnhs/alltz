# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.3] - 2025-07-20

### Added
- Expanded cities database from 102 to 505+ cities for comprehensive global coverage
- Added cities from underrepresented regions including Africa, Caribbean, Central America, and more tech hubs
- Sunrise/sunset times display for each timezone (toggle with 's' key)
- Support for sunrise/sunset times in both 12/24 hour formats

### Fixed
- Fixed Manchester/London bug where adding Manchester would display as London
- Fixed name toggle (n key) to properly switch between airport codes and city names
- Fixed config persistence issue where original city selections weren't saved correctly
- Fixed dead code warnings in compilation by marking test-only functions appropriately
- Improved search disambiguation to correctly show "London, UK" vs "London, Canada"
- Fixed Shift+h/l keyboard shortcut for 1-minute fine timeline scrubbing
- Fixed sunrise/sunset time formatting to respect 12/24 hour preference
- Fixed icon spacing in sunrise/sunset display to prevent overlap

### Changed
- Enhanced city data structure to preserve original city selection with `source_city` field
- Improved search results to include country names for better disambiguation
- Updated README with acknowledgment to everytimezone.com as inspiration
- Fixed LICENSE file formatting
- Version number now automatically pulled from Cargo.toml
- Sunrise/sunset times only highlighted for selected timezone, gray for others
- Sunrise/sunset times displayed at top-right of timezone border

### Technical
- Added comprehensive test coverage for search navigation and city persistence
- Enhanced timezone management to support multiple cities in the same timezone
- Improved config serialization to maintain original city names
- Major performance optimization: cities.json now parsed once and cached (10-100x faster lookups)
- Added `sunrise` crate dependency for accurate solar calculations
- Backward compatible config with default-enabled sunrise/sunset feature

## [0.1.2] - 2025-07-20

### Added
- Custom timezone names/labels - personalize timezones with team member names or custom identifiers
- Rename timezone functionality with 'e' key - opens modal dialog for entering custom names
- Clear custom names with 'E' key - quickly remove custom labels
- Mixed config format support - combine simple timezone strings with custom label objects
- Full backward compatibility - existing configs continue to work seamlessly

### Changed
- Timeline titles now show custom labels when available
- Short mode displays custom labels instead of airport codes
- Full mode shows "Custom Label (Original City Name)" format
- Config format enhanced to support both simple strings and objects with custom labels

### Technical
- Added comprehensive test coverage for custom label functionality (14 new tests)
- Enhanced config serialization with backward-compatible enum variants
- Improved timezone management with label update capabilities
- Extended UI state management for rename operations

## [0.1.1] - 2025-07-16

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

## [0.1.0] - 2025-07-16

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

[Unreleased]: https://github.com/abradburne/alltz/compare/v0.1.3...HEAD
[0.1.3]: https://github.com/abradburne/alltz/compare/v0.1.2...v0.1.3
[0.1.2]: https://github.com/abradburne/alltz/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/abradburne/alltz/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/abradburne/alltz/releases/tag/v0.1.0