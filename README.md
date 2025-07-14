# alltz 🌍

A terminal-based timezone viewer. Track multiple timezones simultaneously with timeline scrubbing, weather integration, DST indicators, and multiple color themes.

![Demo](https://img.shields.io/badge/Status-Active-brightgreen)
![Rust](https://img.shields.io/badge/Language-Rust-orange)
![License](https://img.shields.io/badge/License-MIT-blue)

## ✨ Features

- 🌍 **Multi-timezone display** with visual timeline scrubbing
- 🌤️ **Weather integration** with real weather icons (requires API key)
- 📅 **Date display** with timezone-aware positioning on timelines
- 🕐 **DST transition indicators** (⇈ spring forward, ⇊ fall back)
- 🎨 **6 color themes** (Default, Ocean, Forest, Sunset, Cyberpunk, Monochrome)
- 📍 **Local time display** shows scrubbed time in your timezone with day and UTC offset
- 💾 **Persistent configuration** saves your timezone list and preferences
- 📱 **Intuitive TUI controls** with vim-like navigation
- 💻 **CLI commands** for quick timezone queries without entering TUI
- ⚡ **Fast and lightweight** - built in Rust for performance

## 📦 Installation

### Option 1: Homebrew (macOS/Linux) - Recommended

```bash
# Add the tap (once alltz is published)
brew tap your-username/alltz
brew install alltz
```

### Option 2: Install from Source

```bash
# Clone the repository
git clone https://github.com/your-username/alltz.git
cd alltz

# Build and install with Cargo
cargo install --path .

# Or build release binary manually
cargo build --release
sudo cp target/release/alltz /usr/local/bin/
```

### Option 3: Download Pre-built Binary

1. Download the latest release from [GitHub Releases](https://github.com/your-username/alltz/releases)
2. Extract the binary to your PATH:
   ```bash
   tar -xzf alltz-v0.1.0-your-platform.tar.gz
   sudo mv alltz /usr/local/bin/
   ```

## 🚀 Quick Start

```bash
# Launch interactive TUI
alltz

# Show all available timezones
alltz list

# Check current time in a specific city
alltz time Tokyo

# Get detailed timezone information
alltz zone "New York"

# Start with specific options
alltz --timezone London --twelve-hour --theme ocean
```

## ⚙️ Weather Setup (Optional)

alltz supports real weather data from OpenWeatherMap. **Without an API key, weather display is disabled** - no fake data is shown.

### Getting an API Key

1. Sign up for a free account at [OpenWeatherMap](https://openweathermap.org/api)
2. Generate an API key (free tier allows 1,000 calls/day)
3. Set the environment variable:

```bash
# For current session
export OPENWEATHER_API_KEY="your_api_key_here"

# For permanent setup, add to your shell profile
echo 'export OPENWEATHER_API_KEY="your_api_key_here"' >> ~/.bashrc
# or for zsh users:
echo 'export OPENWEATHER_API_KEY="your_api_key_here"' >> ~/.zshrc
```

### Verifying Weather Setup

```bash
# Check if weather is enabled
alltz
# Press 'w' to toggle weather display
# If API key is valid, you'll see weather icons
# If no API key, weather toggle has no effect
```

## 🎮 TUI Controls

### Navigation
- `j/k` or `↑/↓` - Navigate between timezones
- `h/l` or `←/→` - Scrub timeline (1 hour steps)
- `Shift + h/l` - Fine scrub timeline (1 minute steps)
- `[/]` - Adjust time by ±15 minutes
- `{/}` - Adjust time by ±1 hour

### Time Management
- `t` - Reset to current time
- `m` - Toggle 12/24 hour format
- `n` - Toggle timezone display mode (short/full names)

### Zone Management
- `a` - Add new timezone (with search)
- `d` - Remove current timezone
- `1-9` - Quick select search results when adding zones

### Display Options
- `w` - Toggle weather display (only works with valid API key)
- `e` - Toggle date display on timelines
- `c` - Cycle through color themes
- `?` - Show/hide help
- `q` - Quit

## 🛠️ CLI Commands

### List Timezones
```bash
alltz list
# Shows all available timezones with coordinates
```

### Check Time
```bash
alltz time Tokyo
# Shows current time in Tokyo and your local time
```

### Timezone Information
```bash
alltz zone "Los Angeles"
# Shows detailed timezone info including DST status
```

### Configuration Management
```bash
alltz config                    # Show config path and current content
alltz config --generate         # Generate default config file
```

### CLI Options
```bash
alltz --help                           # Show all options
alltz --timezone Tokyo                 # Start with Tokyo selected
alltz --twelve-hour                    # Use 12-hour format
alltz --theme cyberpunk                # Start with cyberpunk theme
alltz --timezone London --theme ocean  # Combine options
```

## 🎨 Themes

Switch between 6 beautiful color themes using the `c` key:

1. **Default** - Classic terminal colors
2. **Ocean** - Blues and cyans for a calming water theme
3. **Forest** - Greens for a natural, earthy feel
4. **Sunset** - Warm oranges and reds
5. **Cyberpunk** - Neon magentas and electric blues
6. **Monochrome** - Clean black and white

Themes affect all UI elements including borders, timeline colors, and status indicators.

## ⚙️ Configuration

alltz automatically saves your configuration to `~/.config/alltz/config.toml`:

```toml
zones = ["Los Angeles", "New York", "UTC", "London", "Tokyo"]
selected_zone_index = 0
display_format = "TwentyFourHour"
timezone_display_mode = "Short"
color_theme = "Default"
show_weather = true
show_date = false

[time_config]
work_hours_start = 8
work_hours_end = 18
awake_hours_start = 6
awake_hours_end = 22
```

### Configuration Commands

```bash
# View current configuration
alltz config

# Generate default config file (if missing)
alltz config --generate

# Configuration is automatically created on first run
# Edit ~/.config/alltz/config.toml to customize defaults
```

### Customizing Work Hours

The timeline visualization shows different activity levels:
- **Night hours** (light shade): Sleep time
- **Awake hours** (medium shade): Personal time
- **Work hours** (dark shade): Working time

Edit the config file to match your schedule.

## 🌍 Supported Timezones

alltz includes 100+ major cities worldwide:

**Americas**: Los Angeles, Denver, Chicago, New York, Vancouver, Toronto, São Paulo, Buenos Aires
**Europe**: London, Berlin, Paris, Madrid, Rome, Amsterdam, Stockholm, Warsaw
**Asia**: Tokyo, Beijing, Seoul, Hong Kong, Singapore, Mumbai, Dubai, Istanbul
**Africa**: Cairo, Cape Town, Lagos, Nairobi
**Oceania**: Sydney, Melbourne, Auckland

Plus UTC and many more. Use `alltz list` to see all available timezones.

## 🔧 Troubleshooting

### Weather Not Showing
- Verify API key is set: `echo $OPENWEATHER_API_KEY`
- Check key is valid by testing manually:
  ```bash
  curl "https://api.openweathermap.org/data/2.5/weather?q=London&appid=$OPENWEATHER_API_KEY"
  ```
- Restart alltz after setting the environment variable

### Terminal Display Issues
- Ensure your terminal supports Unicode characters
- Try different color themes if colors appear wrong
- Use a monospace font for best alignment

### Configuration Issues
- Configuration is automatically created on first run at `~/.config/alltz/config.toml`
- Use `alltz config` to view current configuration
- Use `alltz config --generate` to recreate default configuration
- Delete `~/.config/alltz/config.toml` to reset to defaults
- Check file permissions if saving fails

### Performance
- alltz is built for performance - if you experience lag, check if your terminal supports hardware acceleration
- Timeline scrubbing is optimized for smooth interaction

## 🧪 Development

### Building from Source

```bash
git clone https://github.com/your-username/alltz.git
cd alltz

# Run in development mode
cargo run

# Run with CLI options
cargo run -- --help
cargo run -- time Tokyo

# Run tests
cargo test

# Build optimized release
cargo build --release
```

### Project Structure

```
src/
├── main.rs          # CLI and TUI setup
├── app.rs           # Application state and logic
├── time.rs          # Timezone management
├── weather.rs       # Weather API integration
├── config.rs        # Configuration and themes
└── ui/
    └── timeline.rs  # Timeline visualization widget
```

### Testing

```bash
# Run all tests
cargo test

# Run with weather API key for full testing
OPENWEATHER_API_KEY="test_key" cargo test

# Test specific module
cargo test weather
```

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request. For major changes, please open an issue first to discuss what you would like to change.

### Areas for Contribution
- Additional timezone support
- Weather API improvements
- New color themes
- Performance optimizations
- Documentation improvements

## 🙋 Support

- **Issues**: Report bugs and request features on [GitHub Issues](https://github.com/your-username/alltz/issues)
- **Discussions**: Join conversations on [GitHub Discussions](https://github.com/your-username/alltz/discussions)
- **Documentation**: Check the [installation guide](INSTALL.md) for detailed setup instructions

## 🎯 Roadmap

- [ ] Real weather API integration with caching
- [ ] Custom timezone support
- [ ] Export timezone schedules
- [ ] Meeting scheduler integration
- [ ] Mobile-friendly web interface
- [ ] Calendar integration
- [ ] Team timezone sharing

---

**Made with ❤️ and ☕ by developers, for developers.**

*alltz - All timezones, all the time.*