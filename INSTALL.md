# alltz Installation Guide

## 🌍 Terminal Timezone Viewer

alltz is a beautiful terminal application for tracking multiple timezones simultaneously with weather icons, DST indicators, and color themes.

## Installation Options

### Option 1: Pre-built Binary (Recommended)

1. Download the latest release binary:
   ```bash
   # Extract and install
   tar -xzf alltz-v0.1.0-macos.tar.gz
   sudo mv alltz /usr/local/bin/
   ```

2. Verify installation:
   ```bash
   alltz --version
   ```

### Option 2: Install with Cargo (Rust Required)

```bash
# Clone and install from source
git clone <repository-url>
cd alltz
cargo install --path .
```

### Option 3: Build from Source

```bash
# Clone repository
git clone <repository-url>
cd alltz

# Build release binary
cargo build --release

# Copy to system PATH
sudo cp target/release/alltz /usr/local/bin/
```

## Quick Start

```bash
# Launch interactive TUI
alltz

# Show available timezones
alltz list

# Check time in specific city
alltz time London

# Get detailed timezone info
alltz zone Tokyo

# View configuration file  
alltz config

# Start with specific options
alltz --timezone London --twelve-hour --theme ocean
```

## TUI Controls

- `?` - Show help
- `h/l` or `←/→` - Scrub timeline (Shift for fine control)
- `j/k` or `↑/↓` - Navigate timezones
- `a` - Add timezone
- `d` - Remove current timezone
- `m` - Toggle 12/24 hour format
- `n` - Toggle timezone display mode
- `w` - Toggle weather icons
- `e` - Toggle date display
- `c` - Cycle color themes
- `t` - Reset to current time
- `[/]` - Fine adjust ±15 minutes
- `{/}` - Fine adjust ±1 hour
- `q` - Quit

## Features

✨ **Timeline Scrubbing** - Navigate through time with visual timeline
🌤️ **Weather Integration** - Real-time weather icons for each location
📅 **Date Display** - Timezone-aware date positioning on timelines
🎨 **6 Color Themes** - Default, Ocean, Forest, Sunset, Cyberpunk, Monochrome
🕐 **DST Indicators** - Visual spring forward (⇈) and fall back (⇊) arrows
📍 **Local Time Display** - Shows scrubbed time in your timezone with day and UTC offset
💾 **Persistent Config** - Saves your timezone list and preferences
🌍 **Global Coverage** - 100+ major cities worldwide

## Configuration

alltz automatically saves your configuration to `~/.config/alltz/config.toml`

## System Requirements

- macOS 10.15+ / Linux / Windows
- Terminal with Unicode support
- No additional dependencies required

## Troubleshooting

If you encounter issues:
1. Ensure your terminal supports Unicode characters
2. Try different color themes if colors appear wrong
3. Check that your system timezone is properly configured

For support, visit: https://github.com/your-repo/alltz/issues