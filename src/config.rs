use crate::app::{TimeFormat, TimezoneDisplayMode};
use ratatui::style::Color;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeDisplayConfig {
    pub work_hours_start: u32,  // 8 (8 AM)
    pub work_hours_end: u32,    // 18 (6 PM)
    pub awake_hours_start: u32, // 6 (6 AM)
    pub awake_hours_end: u32,   // 22 (10 PM)
                                // Night hours are the complement: 22-6
}

impl Default for TimeDisplayConfig {
    fn default() -> Self {
        Self {
            work_hours_start: 8,  // 8 AM
            work_hours_end: 18,   // 6 PM
            awake_hours_start: 6, // 6 AM
            awake_hours_end: 22,  // 10 PM
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TimeActivity {
    Night, // 10 PM - 6 AM
    Awake, // 6 AM - 8 AM, 6 PM - 10 PM
    Work,  // 8 AM - 6 PM
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ColorTheme {
    Default,
    Ocean,
    Forest,
    Sunset,
    Cyberpunk,
    Monochrome,
}

impl ColorTheme {
    pub fn all_themes() -> Vec<ColorTheme> {
        vec![
            ColorTheme::Default,
            ColorTheme::Ocean,
            ColorTheme::Forest,
            ColorTheme::Sunset,
            ColorTheme::Cyberpunk,
            ColorTheme::Monochrome,
        ]
    }

    pub fn next(&self) -> ColorTheme {
        let themes = Self::all_themes();
        let current_index = themes.iter().position(|t| t == self).unwrap_or(0);
        let next_index = (current_index + 1) % themes.len();
        themes[next_index]
    }

    pub fn get_night_color(&self) -> Color {
        match self {
            ColorTheme::Default => Color::DarkGray,
            ColorTheme::Ocean => Color::Blue,
            ColorTheme::Forest => Color::Green,
            ColorTheme::Sunset => Color::Red,
            ColorTheme::Cyberpunk => Color::Magenta,
            ColorTheme::Monochrome => Color::Gray,
        }
    }

    pub fn get_awake_color(&self) -> Color {
        match self {
            ColorTheme::Default => Color::Gray,
            ColorTheme::Ocean => Color::Cyan,
            ColorTheme::Forest => Color::LightGreen,
            ColorTheme::Sunset => Color::Yellow,
            ColorTheme::Cyberpunk => Color::LightBlue,
            ColorTheme::Monochrome => Color::White,
        }
    }

    pub fn get_work_color(&self) -> Color {
        match self {
            ColorTheme::Default => Color::Magenta,
            ColorTheme::Ocean => Color::LightCyan,
            ColorTheme::Forest => Color::LightYellow,
            ColorTheme::Sunset => Color::LightRed,
            ColorTheme::Cyberpunk => Color::LightMagenta,
            ColorTheme::Monochrome => Color::White,
        }
    }

    pub fn get_selected_border_color(&self) -> Color {
        match self {
            ColorTheme::Default => Color::Yellow,
            ColorTheme::Ocean => Color::LightCyan,
            ColorTheme::Forest => Color::LightGreen,
            ColorTheme::Sunset => Color::LightYellow,
            ColorTheme::Cyberpunk => Color::LightMagenta,
            ColorTheme::Monochrome => Color::White,
        }
    }

    pub fn get_timeline_position_color(&self) -> Color {
        match self {
            ColorTheme::Default => Color::Magenta,
            ColorTheme::Ocean => Color::Cyan,
            ColorTheme::Forest => Color::Green,
            ColorTheme::Sunset => Color::Yellow,
            ColorTheme::Cyberpunk => Color::LightMagenta,
            ColorTheme::Monochrome => Color::White,
        }
    }

    pub fn get_current_time_color(&self) -> Color {
        Color::Red // Keep consistent across all themes for clarity
    }
}

impl Default for ColorTheme {
    fn default() -> Self {
        ColorTheme::Default
    }
}

impl TimeDisplayConfig {
    pub fn get_time_activity(&self, hour: u32) -> TimeActivity {
        let hour = hour % 24; // Ensure valid hour range

        if hour >= self.work_hours_start && hour < self.work_hours_end {
            TimeActivity::Work
        } else if hour >= self.awake_hours_start && hour < self.awake_hours_end {
            TimeActivity::Awake
        } else {
            TimeActivity::Night
        }
    }

    pub fn get_activity_char(&self, activity: TimeActivity) -> char {
        match activity {
            TimeActivity::Night => '░', // Light shade - low activity
            TimeActivity::Awake => '▒', // Medium shade - moderate activity
            TimeActivity::Work => '▓',  // Dark shade - high activity (less intense than █)
        }
    }

    pub fn get_activity_color(&self, activity: TimeActivity, theme: ColorTheme) -> Color {
        match activity {
            TimeActivity::Night => theme.get_night_color(),
            TimeActivity::Awake => theme.get_awake_color(),
            TimeActivity::Work => theme.get_work_color(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub zones: Vec<String>,         // List of timezone names to load
    pub selected_zone_index: usize, // Currently selected timezone
    pub display_format: TimeFormat, // 12/24 hour format
    pub timezone_display_mode: TimezoneDisplayMode, // Short/Full names
    pub time_config: TimeDisplayConfig, // Work/awake/night hours
    pub color_theme: ColorTheme,    // Color theme for UI
    pub show_date: bool,            // Date display toggle
    pub group_same_time_cities: bool, // Group cities when they have the same time
    pub use_full_city_names: bool,  // Use full city names instead of abbreviations
    pub show_all_cities_in_groups: bool, // Show all cities in groups instead of "City +N"
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            zones: vec![
                "Los Angeles".to_string(),
                "New York".to_string(),
                "UTC".to_string(),
                "London".to_string(),
                "Berlin".to_string(),
                "Tokyo".to_string(),
                "Sydney".to_string(),
            ],
            selected_zone_index: 0,
            display_format: TimeFormat::TwentyFourHour,
            timezone_display_mode: TimezoneDisplayMode::Short,
            time_config: TimeDisplayConfig::default(),
            color_theme: ColorTheme::default(),
            show_date: false,
            group_same_time_cities: true,
            use_full_city_names: false,  // Default to abbreviations
            show_all_cities_in_groups: false, // Default to "City +N" format
        }
    }
}

impl AppConfig {
    pub fn config_path() -> Option<PathBuf> {
        dirs::home_dir().map(|home_dir| home_dir.join(".config").join("alltz").join("config.toml"))
    }

    pub fn load() -> Self {
        if let Some(config_path) = Self::config_path() {
            if let Ok(content) = fs::read_to_string(&config_path) {
                if let Ok(config) = toml::from_str::<AppConfig>(&content) {
                    return config;
                }
            }
        }
        Self::default()
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(config_path) = Self::config_path() {
            // Create config directory if it doesn't exist
            if let Some(parent) = config_path.parent() {
                fs::create_dir_all(parent)?;
            }

            let content = toml::to_string_pretty(self)?;
            fs::write(&config_path, content)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_time_periods() {
        let config = TimeDisplayConfig::default();

        // Test work hours
        assert_eq!(config.get_time_activity(9), TimeActivity::Work); // 9 AM
        assert_eq!(config.get_time_activity(14), TimeActivity::Work); // 2 PM
        assert_eq!(config.get_time_activity(17), TimeActivity::Work); // 5 PM

        // Test awake hours
        assert_eq!(config.get_time_activity(7), TimeActivity::Awake); // 7 AM
        assert_eq!(config.get_time_activity(19), TimeActivity::Awake); // 7 PM
        assert_eq!(config.get_time_activity(21), TimeActivity::Awake); // 9 PM

        // Test night hours
        assert_eq!(config.get_time_activity(23), TimeActivity::Night); // 11 PM
        assert_eq!(config.get_time_activity(2), TimeActivity::Night); // 2 AM
        assert_eq!(config.get_time_activity(5), TimeActivity::Night); // 5 AM
    }

    #[test]
    fn test_activity_characters() {
        let config = TimeDisplayConfig::default();

        assert_eq!(config.get_activity_char(TimeActivity::Night), '░');
        assert_eq!(config.get_activity_char(TimeActivity::Awake), '▒');
        assert_eq!(config.get_activity_char(TimeActivity::Work), '▓');
    }

    #[test]
    fn test_boundary_conditions() {
        let config = TimeDisplayConfig::default();

        // Boundary at work start (8 AM)
        assert_eq!(config.get_time_activity(7), TimeActivity::Awake);
        assert_eq!(config.get_time_activity(8), TimeActivity::Work);

        // Boundary at work end (6 PM)
        assert_eq!(config.get_time_activity(17), TimeActivity::Work);
        assert_eq!(config.get_time_activity(18), TimeActivity::Awake);

        // Boundary at awake end (10 PM)
        assert_eq!(config.get_time_activity(21), TimeActivity::Awake);
        assert_eq!(config.get_time_activity(22), TimeActivity::Night);
    }

    #[test]
    fn test_app_config_default() {
        let config = AppConfig::default();
        assert!(!config.zones.is_empty());
        assert_eq!(config.display_format, TimeFormat::TwentyFourHour);
        assert_eq!(config.timezone_display_mode, TimezoneDisplayMode::Short);
    }

    #[test]
    fn test_config_serialization() {
        let config = AppConfig::default();
        let toml_str = toml::to_string(&config).unwrap();
        let parsed: AppConfig = toml::from_str(&toml_str).unwrap();

        assert_eq!(config.zones, parsed.zones);
        assert_eq!(config.display_format, parsed.display_format);
        assert_eq!(config.timezone_display_mode, parsed.timezone_display_mode);
    }
}
