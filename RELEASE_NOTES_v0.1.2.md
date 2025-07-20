# 🌍 alltz v0.1.2 Release Notes

## 🏷️ Custom Timezone Names - The Teamwork Update

alltz v0.1.2 introduces **custom timezone names**, allowing you to personalize your timezone display with team member names, office locations, or any custom labels you prefer!

### ✨ What's New

#### 🏷️ Custom Timezone Names
- **Press 'e'** to rename any timezone with a custom label
- **Press 'E'** to clear custom names and restore original timezone names
- Perfect for remote teams - label timezones with colleague names like "Alice (Engineering)" or "Tokyo Office"
- Custom names appear in both short and full display modes

#### 💾 Enhanced Configuration
- **Backward compatible** - existing configs continue to work seamlessly
- **Mixed format support** - combine simple timezone strings with custom label objects
- Configurations automatically upgrade when you add custom names

#### 🎨 Improved Display
- **Short mode**: Shows custom labels instead of airport codes when available
- **Full mode**: Shows "Custom Label (Original City Name)" format
- **Visual consistency** maintained across all color themes

### 🔧 Technical Improvements
- Added 14 new comprehensive tests for custom label functionality
- Enhanced config serialization with backward-compatible enum variants
- Improved timezone management with runtime label updates
- Extended UI state management for rename operations

### 📈 Usage Example

```toml
# Your config can now look like this:
zones = [
    "Los Angeles",
    { city_name = "Tokyo", custom_label = "Alice (Engineering)" },
    { city_name = "London", custom_label = "Bob (Sales)" }
]
```

### 🚀 Installation

#### Homebrew (macOS/Linux)
```bash
brew tap abradburne/alltz
brew install alltz
```

#### Direct Download
Download the appropriate binary for your platform from the [releases page](https://github.com/abradburne/alltz/releases/tag/v0.1.2).

#### From Source
```bash
cargo install --git https://github.com/abradburne/alltz --tag v0.1.2
```

### 🎮 New Controls
- `e` - Rename/customize current timezone with personal labels
- `E` - Clear custom name and restore original timezone name

### 🔄 Migration
Existing installations will continue to work exactly as before. No manual migration required - custom names are purely additive!

---

**Full Changelog**: [v0.1.1...v0.1.2](https://github.com/abradburne/alltz/compare/v0.1.1...v0.1.2)