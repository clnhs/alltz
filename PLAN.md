# alltz - Terminal Time Zone Viewer
## Product Requirements Document

## Executive Summary

**alltz** is a terminal-based time zone viewer designed for developers, remote teams, and frequent schedulers. It provides an at-a-glance view of multiple time zones with visual timeline representation, real-time updates, and keyboard-driven workflow integration.

## Core Vision

Create a TUI application that presents time zones in a horizontally-scrollable timeline format, ordered by UTC offset, with a vertical "now" indicator showing current time across all zones. The interface prioritizes speed, clarity, and seamless CLI workflow integration.

### Key Principles
- **UTC Offset Ordering**: Zones always displayed from UTC-12 to UTC+14 for natural time progression
- **48-Hour Timeline**: Yesterday → Today → Tomorrow span for comprehensive time context
- **Real-time Interactivity**: Navigate through time and zones with keyboard controls
- **Visual Clarity**: Color coding, weather icons, and business hours highlighting

## Technical Architecture

### Dependencies
- **ratatui**: Terminal UI framework for rich TUI applications
- **crossterm**: Cross-platform terminal manipulation and event handling
- **chrono**: Date and time handling for precise time calculations
- **chrono-tz**: Timezone database support for accurate conversions and DST handling
- **tokio**: Async runtime for weather API requests and real-time updates
- **serde**: JSON serialization for configuration and weather data
- **reqwest**: HTTP client for weather API integration (optional)

### Performance Specifications
- **Startup time**: < 200ms cold start
- **Refresh rate**: 1-second updates for time display
- **Memory usage**: < 10MB resident
- **Terminal compatibility**: Support for 16-color minimum, 256-color optimal

### Platform Support
- **Primary**: macOS, Linux
- **Secondary**: Windows (via WSL)
- **Terminal requirements**: 80x24 minimum, 120x30 optimal

### Core Components

#### 1. Time Management (`src/time.rs`)
- Current time tracking and updates
- Timezone conversion logic
- Date range calculation (yesterday, today, tomorrow)
- UTC-based calculations with offset handling

#### 2. UI Components (`src/ui/`)
- **Shared Timeline Widget**: Unified horizontal time axis with all timezones aligned to UTC
- **Timezone Row**: Individual timezone display with visual offset positioning and clear time labels
- **Interactive Time Indicator**: Movable vertical bar for time navigation with precise positioning
- **Time Display Panel**: Shows exact time for selected moment across all timezones
- **Header/Footer**: Navigation help, current time, and status information

#### 3. App State (`src/app.rs`)
- Current time tracking and time navigation state
- Selected time indicator position (defaults to current time)
- Selected timezones list
- Display preferences (12/24 hour, etc.)
- Navigation state and user interaction handling

#### 4. Configuration (`src/config.rs`)
- Default timezone list
- User preferences
- Keyboard shortcuts
- Display settings

## User Interface Specification

### Primary Layout
```
┌─ Header ─────────────────────────────────────────────────────────────────┐
│ alltz v1.0 │ UTC: 14:32:15 │ Local: 09:32:15 EST │ [?] Help [q] Quit    │
├─ Timeline ──────────────────────────────────────────────────────────────┤
│        Yesterday    │      Today      │     Tomorrow                     │
│    ←─── 00:00 ──── 12:00 ──── 00:00 ──── 12:00 ──── 00:00 ────→         │
├─ Zone Display ──────────────────────────────────────────────────────────┤
│ ▲ UTC-8 LAX │░░░░░░░█████████████░░░░░░░░░░░░░░░░░░░░│ 06:32 Mon ☀️      │
│   UTC-5 NYC │░░░░░░░░░░░█████████████░░░░░░░░░░░░░░░│ 09:32 Mon ⛅      │
│   UTC+0 LON │░░░░░░░░░░░░░░░░█████████████░░░░░░░░░░│ 14:32 Mon 🌧️      │
│   UTC+1 BER │░░░░░░░░░░░░░░░░░█████████████░░░░░░░░░│ 15:32 Mon ⛅      │
│   UTC+9 TOK │░░░░░░░░░░░░░░░░░░░░░░█████████████░░░░│ 23:32 Mon 🌙      │
│   UTC+11 SYD│░░░░░░░░░░░░░░░░░░░░░░░█████████████░░░│ 01:32 Tue 🌙      │
│                               ║                                           │
│                               ║ ← Now indicator (14:32 UTC)              │
├─ Status ────────────────────────────────────────────────────────────────┤
│ 6 zones │ Auto-refresh: ON │ j/k: navigate │ h/l: scrub │ a: add zone    │
└─────────────────────────────────────────────────────────────────────────┘
```

### Zone Ordering System
Zones are always displayed in UTC offset order from most negative to most positive:
- UTC-12 (Baker Island) → UTC-11 (Samoa) → ... → UTC+0 (London) → ... → UTC+12 (Fiji) → UTC+14 (Kiribati)
- Creates natural "left-to-right" time progression
- Users navigate vertically through this ordered list
- New zones automatically inserted in correct offset position

### Visual Elements
- **Gradient time bars**: Dark (░) during night hours, bright (█) during business hours
- **Weather integration**: Real-time weather icons per zone (☀️⛅🌧️🌙)
- **Working hours highlight**: Distinct visual treatment for 9AM-5PM equivalent
- **DST indicators**: Visual markers during daylight saving transitions
- **Color coding**: Distinct colors for different offset groups

## Development Phases

### Phase 1: Core Foundation
**Goal**: Basic TUI application with time display and zone ordering

1. **Project Setup**
   - Initialize Cargo project with required dependencies
   - Set up basic module structure (`main.rs`, `app.rs`, `time.rs`, `ui/mod.rs`)
   - Configure development environment and tooling

2. **Time Management System**
   - Implement UTC-based time tracking with chrono/chrono-tz
   - Create timezone data structures with automatic UTC offset ordering
   - Build 48-hour timeline calculation (yesterday → today → tomorrow)
   - Add real-time updates with 1-second refresh

3. **Basic TUI Framework**
   - Set up ratatui application loop with crossterm
   - Create basic layout structure (header, timeline, zones, status)
   - Implement keyboard input handling (q to quit, basic navigation)
   - Add terminal setup/cleanup and error handling

### Phase 2: Timeline Visualization
**Goal**: Visual timeline with zone alignment and time scrubbing

1. **Timeline Widget Implementation**
   - Create shared horizontal timeline spanning 48 hours
   - Implement UTC offset positioning for each timezone row
   - Add gradient time bars (░ for night, █ for business hours)
   - Build responsive timeline scaling for different terminal widths

2. **Interactive Time Navigation**
   - Implement h/l keys for timeline scrubbing
   - Add vertical "now" indicator that moves with navigation
   - Create precise time calculation for any timeline position
   - Add 't' key to reset to current time

3. **Zone Display System**
   - Build timezone row widget with offset-based positioning
   - Add zone labels with UTC offset indicators (UTC-8, UTC+1, etc.)
   - Implement j/k navigation through ordered zone list
   - Add day boundary handling and date display

### Phase 3: Zone Management & Visual Enhancement
**Goal**: Dynamic zone management and improved visual design

1. **Zone Management Features**
   - Implement 'a' key fuzzy search for adding zones
   - Add 'd' key for removing zones with confirmation
   - Create zone favorites system with 's' key starring
   - Build zone persistence using TOML configuration

2. **Enhanced Visual Elements**
   - Add weather integration with API calls and emoji icons
   - Implement color themes with 'c' key cycling
   - Add DST transition indicators
   - Create progressive detail modes based on terminal width

3. **Display Options**
   - Implement 'm' key for 12/24 hour format toggle
   - Add compact/normal/detailed view modes with Tab key
   - Create working hours highlighting (9AM-5PM equivalent)
   - Add 'w' key for weather display toggle

### Phase 4: Advanced Features & Polish
**Goal**: Advanced navigation and professional polish

1. **Advanced Time Navigation**
   - Add [/] keys for 15-minute fine scrubbing
   - Implement {/} keys for 1-hour coarse scrubbing
   - Create ;/' keys for business hours quick jump
   - Add Home/End keys for timeline bounds navigation

2. **Configuration System**
   - Build comprehensive TOML configuration support
   - Implement zone aliases system for quick access
   - Add custom zone labels with 'r' key renaming
   - Create startup preferences and defaults

3. **Performance & Polish**
   - Optimize for <200ms startup and <10MB memory usage
   - Add comprehensive help system with '?' key
   - Implement proper error handling and recovery
   - Add command-line argument support for quick zone viewing

### Phase 5: Extended Features (Future)
**Goal**: Advanced productivity features

1. **Smart Meeting Tools**
   - Highlight overlapping business hours across zones
   - Add meeting time suggestion algorithms
   - Create business hours conflict detection

2. **Integration Features**
   - Add command-line arguments for scripting integration
   - Implement export formats (JSON, CSV, iCal)
   - Create system notification integration

## Core Features

### 1. Timeline Visualization
- **Horizontal time axis**: 48-hour span (yesterday → today → tomorrow)
- **Gradient time bars**: Dark during night hours, bright during business hours
- **Real-time "now" indicator**: Vertical line showing current UTC time across all zones
- **Working hours highlight**: Distinct visual treatment for 9AM-5PM local equivalent

### 2. Zone Management
- **Automatic ordering**: All zones sorted by UTC offset (UTC-12 to UTC+14)
- **Fuzzy search**: Quick zone addition with city/country/alias matching
- **Favorites system**: Star zones to keep them visible when filtering
- **Custom labels**: Override display names (e.g., "NYC Office" instead of "New York")

### 3. Visual Enhancement
- **Weather integration**: Real-time weather icons per zone
- **DST indicators**: Visual markers during daylight saving transitions
- **Color coding**: Distinct colors for different offset groups
- **Progressive detail**: More information as terminal width increases

### 4. User Interactions

#### Navigation Commands
| Key | Action | Description |
|-----|--------|-------------|
| j/k | Navigate zones | Move up/down through UTC-ordered zone list |
| h/l | Scrub timeline | Move "now" indicator to see past/future times |
| g/G | Jump to bounds | Go to earliest/latest timezone |
| Home/End | Timeline bounds | Jump to start/end of visible time range |

#### Zone Management Commands
| Key | Action | Description |
|-----|--------|-------------|
| a | Add zone | Fuzzy search interface for new timezone |
| d | Delete zone | Remove currently selected zone |
| r | Rename zone | Add custom label/alias |
| s | Star zone | Mark as favorite (visual indicator) |
| o | Sort options | Toggle secondary sort (name, favorites) |

#### View Controls
| Key | Action | Description |
|-----|--------|-------------|
| Space | Toggle zone | Hide/show selected zone |
| Enter | Focus mode | Expand selected zone with detailed info |
| Tab | Layout mode | Cycle between compact/normal/detailed views |
| m | Time format | Toggle 12/24 hour display |
| w | Weather toggle | Show/hide weather information |
| c | Color theme | Cycle through available color schemes |

#### Time Manipulation
| Key | Action | Description |
|-----|--------|-------------|
| t | Reset to now | Return "now" indicator to current time |
| [/] | Fine scrub | Move timeline by 15-minute increments |
| {/} | Coarse scrub | Move timeline by 1-hour increments |
| ;/' | Business hours | Jump to 9AM/5PM in selected zone |

## Configuration System

### Default Zones
```toml
# ~/.config/alltz/config.toml
[zones]
default = ["UTC-8:LAX", "UTC-5:NYC", "UTC+0:LON", "UTC+1:BER", "UTC+9:TOK"]

[display]
format_24h = true
show_weather = true
show_seconds = false
color_theme = "default"

[behavior]
auto_refresh = true
refresh_interval = 1000  # milliseconds
startup_to_now = true
```

### Zone Aliases
```toml
[aliases]
"sf" = "UTC-8:San Francisco"
"ny" = "UTC-5:New York"
"ldn" = "UTC+0:London"
"berlin" = "UTC+1:Berlin"
"tokyo" = "UTC+9:Tokyo"
"home" = "UTC-5:EST"  # User's local timezone
```

### Data Sources
- **Timezone data**: IANA Time Zone Database
- **Weather API**: OpenWeatherMap or similar (optional, configurable)
- **Configuration**: TOML/YAML file in ~/.config/alltz/

## Success Metrics
- Clear visual understanding of time across zones
- Quick identification of suitable meeting times
- Seamless integration into developer workflows
- Minimal learning curve for new users
- Reliable and accurate time calculations

## Future Enhancements
- Meeting time suggestions
- Timezone aliases and custom names
- Calendar integration
- Time zone history/favorites
- DST transition indicators
- Mobile/web companion