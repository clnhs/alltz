use chrono::{DateTime, Utc, Local, Offset, Timelike};
use ratatui::{
    layout::{Alignment, Constraint, Direction as LayoutDirection, Layout, Rect},
    style::Modifier,
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::time::{TimeZone, TimeZoneManager};
use crate::ui::TimelineWidget;
use crate::config::{TimeDisplayConfig, AppConfig, ColorTheme};
use crate::weather::WeatherManager;

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum TimeFormat {
    TwentyFourHour,
    TwelveHour,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum TimezoneDisplayMode {
    Short,  // LAX, NYC, LON
    Full,   // Pacific Time (US) PDT UTC-7
}

#[derive(Debug, Clone, PartialEq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug, Clone)]
pub enum Message {
    // Time navigation
    Tick,
    ScrubTimeline(Direction),
    ScrubTimelineWithShift(Direction),
    ResetToNow,
    FineAdjust(i32), // minutes
    
    // Zone navigation
    NavigateZone(Direction),
    
    // Display options
    ToggleTimeFormat,
    ToggleTimezoneDisplayMode,
    ToggleWeather,
    ToggleHelp,
    CycleColorTheme,
    
    // Zone management
    StartAddZone,
    UpdateAddZoneInput(String),
    SelectSearchResult(usize),
    ConfirmAddZone,
    CancelAddZone,
    RemoveCurrentZone,
    
    // App lifecycle
    Quit,
}

#[derive(Debug)]
pub struct App {
    // Time management
    pub current_time: DateTime<Utc>,
    pub timeline_position: DateTime<Utc>,
    
    // Zone management
    pub timezone_manager: TimeZoneManager,
    pub selected_zone_index: usize,
    
    // UI state
    pub display_format: TimeFormat,
    pub timezone_display_mode: TimezoneDisplayMode,
    pub time_config: TimeDisplayConfig,
    pub color_theme: ColorTheme,
    pub show_help: bool,
    pub adding_zone: bool,
    pub add_zone_input: String,
    pub zone_search_results: Vec<String>,
    pub show_weather: bool,
    pub weather_manager: WeatherManager,
    
    // App state
    pub should_quit: bool,
}

impl Default for App {
    fn default() -> Self {
        let now = Utc::now();
        Self {
            current_time: now,
            timeline_position: now,
            timezone_manager: TimeZoneManager::with_default_zones(),
            selected_zone_index: 0,
            display_format: TimeFormat::TwentyFourHour,
            timezone_display_mode: TimezoneDisplayMode::Short,
            time_config: TimeDisplayConfig::default(),
            color_theme: ColorTheme::default(),
            show_help: false,
            adding_zone: false,
            add_zone_input: String::new(),
            zone_search_results: Vec::new(),
            show_weather: true,
            weather_manager: WeatherManager::new(),
            should_quit: false,
        }
    }
}

impl App {
    pub fn new() -> Self {
        let config = AppConfig::load();
        let mut app = Self::from_config(config);
        app.select_local_timezone();
        app
    }
    
    pub fn from_config(config: AppConfig) -> Self {
        let mut timezone_manager = TimeZoneManager::new();
        
        // Load timezones from config
        for zone_name in &config.zones {
            timezone_manager.add_timezone_by_name(zone_name);
        }
        
        // If no zones were loaded, use defaults
        if timezone_manager.zones().is_empty() {
            timezone_manager = TimeZoneManager::with_default_zones();
        }
        
        let now = Utc::now();
        let selected_zone_index = config.selected_zone_index.min(timezone_manager.zones().len().saturating_sub(1));
        
        Self {
            current_time: now,
            timeline_position: now,
            timezone_manager,
            selected_zone_index,
            display_format: config.display_format,
            timezone_display_mode: config.timezone_display_mode,
            time_config: config.time_config,
            color_theme: config.color_theme,
            show_help: false,
            adding_zone: false,
            add_zone_input: String::new(),
            zone_search_results: Vec::new(),
            show_weather: true,
            weather_manager: WeatherManager::new(),
            should_quit: false,
        }
    }
    
    pub fn to_config(&self) -> AppConfig {
        AppConfig {
            zones: self.timezone_manager.zones()
                .iter()
                .map(|zone| {
                    // Try to find the original search name for this timezone
                    let available = TimeZoneManager::get_all_available_timezones();
                    available
                        .iter()
                        .find(|(tz, _, _, _, _)| *tz == zone.tz)
                        .map(|(_, search_name, _, _, _)| search_name.clone())
                        .unwrap_or_else(|| zone.tz.to_string())
                })
                .collect(),
            selected_zone_index: self.selected_zone_index,
            display_format: self.display_format.clone(),
            timezone_display_mode: self.timezone_display_mode.clone(),
            time_config: self.time_config.clone(),
            color_theme: self.color_theme,
        }
    }
    
    pub fn save_config(&self) {
        let config = self.to_config();
        if let Err(e) = config.save() {
            // In a real app, you might want to show an error message to the user
            eprintln!("Failed to save config: {}", e);
        }
    }
    
    fn select_local_timezone(&mut self) {
        let local_time = self.current_time.with_timezone(&Local);
        let local_offset_seconds = local_time.offset().fix().local_minus_utc();
        let local_offset_hours = local_offset_seconds / 3600;
        
        // Find the timezone that matches our local offset
        for (index, zone) in self.timezone_manager.zones().iter().enumerate() {
            let zone_offset_hours = zone.utc_offset_hours();
            if zone_offset_hours == local_offset_hours {
                self.selected_zone_index = index;
                break;
            }
        }
    }
    
    pub fn update(&mut self, msg: Message) -> Option<Message> {
        match msg {
            Message::Tick => {
                self.current_time = Utc::now();
                None
            }
            
            Message::ScrubTimeline(direction) => {
                // Round to the next/previous hour boundary
                let rounded_time = match direction {
                    Direction::Left => {
                        // Go to the start of the current hour, or previous hour if already at start
                        if self.timeline_position.minute() == 0 && self.timeline_position.second() == 0 && self.timeline_position.nanosecond() == 0 {
                            // Already at hour boundary, go to previous hour
                            self.timeline_position - chrono::Duration::hours(1)
                        } else {
                            // Go to start of current hour
                            self.timeline_position
                                .with_minute(0).unwrap_or(self.timeline_position)
                                .with_second(0).unwrap_or(self.timeline_position)
                                .with_nanosecond(0).unwrap_or(self.timeline_position)
                        }
                    },
                    Direction::Right => {
                        // Go to the start of the next hour
                        let next_hour_start = self.timeline_position
                            .with_minute(0).unwrap_or(self.timeline_position)
                            .with_second(0).unwrap_or(self.timeline_position)
                            .with_nanosecond(0).unwrap_or(self.timeline_position)
                            + chrono::Duration::hours(1);
                        next_hour_start
                    },
                    _ => self.timeline_position,
                };
                self.timeline_position = rounded_time;
                None
            }
            
            Message::ScrubTimelineWithShift(direction) => {
                // Preserve minutes when shift is held - use the old hourly increment
                let delta = match direction {
                    Direction::Left => chrono::Duration::hours(-1),
                    Direction::Right => chrono::Duration::hours(1),
                    _ => chrono::Duration::zero(),
                };
                self.timeline_position = self.timeline_position + delta;
                None
            }
            
            Message::ResetToNow => {
                self.timeline_position = self.current_time;
                None
            }
            
            Message::FineAdjust(minutes) => {
                let delta = chrono::Duration::minutes(minutes as i64);
                self.timeline_position = self.timeline_position + delta;
                None
            }
            
            Message::NavigateZone(direction) => {
                let zone_count = self.timezone_manager.zones().len();
                if zone_count > 0 {
                    let old_index = self.selected_zone_index;
                    match direction {
                        Direction::Up => {
                            if self.selected_zone_index > 0 {
                                self.selected_zone_index -= 1;
                            }
                        }
                        Direction::Down => {
                            if self.selected_zone_index < zone_count - 1 {
                                self.selected_zone_index += 1;
                            }
                        }
                        _ => {}
                    }
                    // Save config if selection changed
                    if old_index != self.selected_zone_index {
                        self.save_config();
                    }
                }
                None
            }
            
            Message::ToggleTimeFormat => {
                self.display_format = match self.display_format {
                    TimeFormat::TwentyFourHour => TimeFormat::TwelveHour,
                    TimeFormat::TwelveHour => TimeFormat::TwentyFourHour,
                };
                self.save_config();
                None
            }
            
            Message::ToggleTimezoneDisplayMode => {
                self.timezone_display_mode = match self.timezone_display_mode {
                    TimezoneDisplayMode::Short => TimezoneDisplayMode::Full,
                    TimezoneDisplayMode::Full => TimezoneDisplayMode::Short,
                };
                self.save_config();
                None
            }
            
            Message::ToggleWeather => {
                self.show_weather = !self.show_weather;
                None
            }
            
            Message::CycleColorTheme => {
                self.color_theme = self.color_theme.next();
                self.save_config();
                None
            }
            
            Message::ToggleHelp => {
                self.show_help = !self.show_help;
                None
            }
            
            Message::StartAddZone => {
                self.adding_zone = true;
                self.add_zone_input.clear();
                self.zone_search_results.clear();
                None
            }
            
            Message::UpdateAddZoneInput(input) => {
                self.add_zone_input = input.clone();
                self.zone_search_results = crate::time::TimeZoneManager::search_timezones(&input);
                None
            }
            
            Message::SelectSearchResult(index) => {
                if let Some(zone_name) = self.zone_search_results.get(index) {
                    let success = self.timezone_manager.add_timezone_by_name(zone_name);
                    
                    if success {
                        // Update selected index if needed
                        if self.selected_zone_index >= self.timezone_manager.zones().len() {
                            self.selected_zone_index = self.timezone_manager.zones().len().saturating_sub(1);
                        }
                        self.save_config();
                    }
                }
                self.adding_zone = false;
                self.add_zone_input.clear();
                self.zone_search_results.clear();
                None
            }
            
            Message::ConfirmAddZone => {
                if !self.add_zone_input.is_empty() {
                    // Try to add the exact input first, then try the first search result
                    let success = self.timezone_manager.add_timezone_by_name(&self.add_zone_input) ||
                        (self.zone_search_results.get(0)
                            .map(|name| self.timezone_manager.add_timezone_by_name(name))
                            .unwrap_or(false));
                    
                    if success {
                        // Update selected index if needed
                        if self.selected_zone_index >= self.timezone_manager.zones().len() {
                            self.selected_zone_index = self.timezone_manager.zones().len().saturating_sub(1);
                        }
                        self.save_config();
                    }
                }
                self.adding_zone = false;
                self.add_zone_input.clear();
                self.zone_search_results.clear();
                None
            }
            
            Message::CancelAddZone => {
                self.adding_zone = false;
                self.add_zone_input.clear();
                self.zone_search_results.clear();
                None
            }
            
            Message::RemoveCurrentZone => {
                if self.timezone_manager.zones().len() > 1 { // Keep at least one zone
                    self.timezone_manager.remove_zone(self.selected_zone_index);
                    
                    // Adjust selected index if needed
                    if self.selected_zone_index >= self.timezone_manager.zones().len() {
                        self.selected_zone_index = self.timezone_manager.zones().len().saturating_sub(1);
                    }
                    self.save_config();
                }
                None
            }
            
            Message::Quit => {
                self.should_quit = true;
                None
            }
        }
    }
    
    pub fn view(&self, f: &mut Frame) {
        let chunks = Layout::default()
            .direction(LayoutDirection::Vertical)
            .constraints([
                Constraint::Length(3), // Header
                Constraint::Length(4), // Current time display (taller)
                Constraint::Min(1),    // Main content
                Constraint::Length(3), // Footer
            ])
            .split(f.area());
        
        self.render_header(f, chunks[0]);
        self.render_current_time_display(f, chunks[1]);
        self.render_zones(f, chunks[2]);
        self.render_footer(f, chunks[3]);
        
        // Render modals on top if needed
        if self.show_help {
            self.render_help_modal(f);
        } else if self.adding_zone {
            self.render_add_zone_modal(f);
        }
    }
    
    fn render_header(&self, f: &mut Frame, area: Rect) {
        let local_time = self.current_time.with_timezone(&Local);
        
        // Use chrono-tz names for better abbreviations, fall back to %Z
        let tz_name = self.get_local_timezone_name();
        let local_time_str = match self.display_format {
            TimeFormat::TwentyFourHour => format!("{} {}", local_time.format("%H:%M:%S"), tz_name),
            TimeFormat::TwelveHour => format!("{} {}", local_time.format("%I:%M:%S %p"), tz_name),
        };
        
        let timeline_time_str = match self.display_format {
            TimeFormat::TwentyFourHour => self.timeline_position.format("%H:%M UTC").to_string(),
            TimeFormat::TwelveHour => self.timeline_position.format("%I:%M %p UTC").to_string(),
        };
        
        // Create inner area for content
        let inner = area.inner(ratatui::layout::Margin { horizontal: 1, vertical: 1 });
        
        // Use ratatui's layout system for even distribution
        let chunks = Layout::default()
            .direction(LayoutDirection::Horizontal)
            .constraints([
                Constraint::Percentage(25),  // App name
                Constraint::Percentage(30),  // Local time
                Constraint::Percentage(30),  // Timeline 
                Constraint::Percentage(15),  // Controls
            ])
            .split(inner);
        
        // Render each section
        let app_name = Paragraph::new("alltz v0.1.0");
        f.render_widget(app_name, chunks[0]);
        
        let local_display = Paragraph::new(format!("Local: {}", local_time_str));
        f.render_widget(local_display, chunks[1]);
        
        let timeline_display = Paragraph::new(format!("Timeline: {}", timeline_time_str));
        f.render_widget(timeline_display, chunks[2]);
        
        let controls = Paragraph::new("q: Quit | ?: Help");
        f.render_widget(controls, chunks[3]);
        
        // Render the border around the whole header
        let border = Block::default().borders(Borders::ALL).title("alltz");
        f.render_widget(border, area);
    }
    
    fn get_local_timezone_name(&self) -> String {
        // Try to get a better timezone name from our configured zones
        let local_time = self.current_time.with_timezone(&Local);
        let local_offset_hours = local_time.offset().fix().local_minus_utc() / 3600;
        
        // Look for a matching timezone in our list to get a better abbreviation
        for zone in self.timezone_manager.zones() {
            if zone.utc_offset_hours() == local_offset_hours {
                // Use the proper timezone abbreviation (JST, EST, etc.)
                return zone.get_timezone_abbreviation();
            }
        }
        
        // Fallback to chrono's timezone formatting
        let tz_str = local_time.format("%Z").to_string();
        if tz_str.starts_with('+') || tz_str.starts_with('-') {
            // If it's still showing offset, try a different approach
            format!("UTC{}", if local_offset_hours >= 0 { 
                format!("+{}", local_offset_hours) 
            } else { 
                local_offset_hours.to_string() 
            })
        } else {
            tz_str
        }
    }
    
    fn render_zones(&self, f: &mut Frame, area: Rect) {
        let zones = self.timezone_manager.zones();
        
        if zones.is_empty() {
            let empty_msg = Paragraph::new("No timezones configured")
                .block(Block::default().borders(Borders::ALL).title("Timezones"));
            f.render_widget(empty_msg, area);
            return;
        }
        
        
        let zone_constraints = zones.iter()
            .map(|_| Constraint::Length(4))
            .collect::<Vec<_>>();
        
        let zone_chunks = Layout::default()
            .direction(LayoutDirection::Vertical)
            .constraints(zone_constraints)
            .split(area);
        
        for (i, zone) in zones.iter().enumerate() {
            if i < zone_chunks.len() {
                self.render_zone(f, zone_chunks[i], zone, i == self.selected_zone_index);
            }
        }
    }
    
    fn render_zone(&self, f: &mut Frame, area: Rect, zone: &TimeZone, is_selected: bool) {
        // Get weather data for this zone - find the search name
        let available = TimeZoneManager::get_all_available_timezones();
        let zone_name = available
            .iter()
            .find(|(tz, _, _, _, _)| *tz == zone.tz)
            .map(|(_, search_name, _, _, _)| search_name.as_str())
            .unwrap_or("Unknown");
            
        // For now, always use demo weather data since we need a simpler approach
        let demo_weather = self.weather_manager.get_demo_weather(zone_name);
        
        let timeline_widget = TimelineWidget::new(
            self.timeline_position,
            self.current_time,
            zone,
            is_selected,
            self.display_format.clone(),
            self.timezone_display_mode.clone(),
            &self.time_config,
            self.color_theme,
            Some(&demo_weather),
            self.show_weather,
            true, // DST indicators always on
        );
        
        f.render_widget(timeline_widget, area);
    }
    
    fn render_current_time_display(&self, f: &mut Frame, area: Rect) {
        // Show what the scrubbed timeline position is in the user's local timezone
        let local_time = self.timeline_position.with_timezone(&chrono::Local);
        
        // Format as two lines: local timezone info with UTC offset on first line, time with day on second
        let local_offset_seconds = local_time.offset().fix().local_minus_utc();
        let local_offset_hours = local_offset_seconds / 3600;
        let utc_offset_str = if local_offset_hours >= 0 {
            format!("UTC+{}", local_offset_hours)
        } else {
            format!("UTC{}", local_offset_hours)
        };
        let timezone_line = format!("{} ({})", self.get_local_timezone_name(), utc_offset_str);
        
        let time_line = match self.display_format {
            TimeFormat::TwentyFourHour => local_time.format("%H:%M:%S %a").to_string(),
            TimeFormat::TwelveHour => local_time.format("%I:%M:%S %p %a").to_string(),
        };
        
        let display_text = format!("{}\n{}", timezone_line, time_line);
        
        // Calculate the width needed for the longest line plus borders and padding
        let max_line_width = timezone_line.len().max(time_line.len());
        let box_width = (max_line_width as u16 + 4).min(area.width); // +4 for borders and padding
        let center_x = (area.width.saturating_sub(box_width)) / 2;
        
        // Create a centered, narrow area for the time box
        let time_area = Rect {
            x: area.x + center_x,
            y: area.y,
            width: box_width,
            height: area.height,
        };
        
        // Center the time display in a bordered box
        let time_display = Paragraph::new(display_text)
            .style(ratatui::style::Style::default().fg(ratatui::style::Color::Cyan))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL).title("Local"));
        
        f.render_widget(time_display, time_area);
    }
    
    fn render_footer(&self, f: &mut Frame, area: Rect) {
        let footer_text = format!("j/k: navigate zones │ h/l: scrub timeline │ a: add zone │ del: delete zone │ t: reset │ m: format │ n: names │ w: weather │ c: theme ({}) │ ?: help", self.color_theme.name());
        
        let footer = Paragraph::new(footer_text)
            .block(Block::default().borders(Borders::ALL).title("Controls"));
        
        f.render_widget(footer, area);
    }
    
    fn render_help_modal(&self, f: &mut Frame) {
        let help_text = [
            "",
            "╔══════════════════════════════════════════════════════════════════════════════╗",
            "║                          🌍 ALLTZ - TIMEZONE VIEWER 🕐                       ║",
            "║                          Real-time global timezone tracker                   ║",
            "╚══════════════════════════════════════════════════════════════════════════════╝",
            "",
            "┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓",
            "┃ NAVIGATION                                                                   ┃",
            "┣━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┫",
            "┃  j k  ↑ ↓       Navigate between timezone panels                           ┃",
            "┃  h l  ← →       Scrub timeline by hour (snaps to hour boundaries)         ┃",
            "┃  H L  Shift+←→  Scrub timeline by hour (preserves minutes)                ┃",
            "┃  [ ]            Fine adjustment ±15 minutes                               ┃",
            "┃  { }            Fine adjustment ±60 minutes                               ┃",
            "┃  t              Reset timeline to current time                            ┃",
            "┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛",
            "",
            "┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓",
            "┃ VISUAL DISPLAY                                                               ┃",
            "┣━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┫",
            "┃  m              Toggle between 12-hour and 24-hour time formats           ┃",
            "┃  n              Toggle timezone display (short codes vs full names)       ┃",
            "┃  w              Toggle weather icons (☀️ 🌧️ ⛈️ ❄️ 🌫️)                     ┃",
            "┃  c              Cycle through color themes (6 beautiful themes)           ┃",
            &format!("┃                 Current theme: {:<40}           ┃", self.color_theme.name()),
            "┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛",
            "",
            "┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓",
            "┃ TIMEZONE MANAGEMENT                                                          ┃",
            "┣━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┫",
            "┃  a              Add new timezone (search by city name)                    ┃",
            "┃  d              Delete currently selected timezone                        ┃",
            "┃  1-5            Quick-select search result when adding timezones          ┃",
            "┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛",
            "",
            "┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓",
            "┃ SPECIAL INDICATORS                                                           ┃",
            "┣━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┫",
            "┃  │              Red line: Current real time                                ┃",
            "┃  ┃              Colored line: Timeline position (scrub time)              ┃",
            "┃  ⇈              Green arrows: Spring forward DST transition (clocks +1h)  ┃",
            "┃  ⇊              Yellow arrows: Fall back DST transition (clocks -1h)      ┃",
            "┃  ░ ▒ ▓          Activity blocks: Night, Awake, Work hours                 ┃",
            "┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛",
            "",
            "┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓",
            "┃ APPLICATION CONTROL                                                          ┃",
            "┣━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┫",
            "┃  ?              Show/hide this help screen                                ┃",
            "┃  q              Quit alltz                                                ┃",
            "┃  Esc            Cancel current operation (add zone, help, etc.)          ┃",
            "┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛",
            "",
            "                          Press any key to close help",
            "",
        ].join("\n");
        
        let area = f.area();
        let popup_area = Rect {
            x: area.width / 20,
            y: area.height / 20,
            width: area.width * 9 / 10,
            height: area.height * 9 / 10,
        };
        
        // Clear the background
        f.render_widget(Clear, popup_area);
        
        let help_popup = Paragraph::new(help_text)
            .block(Block::default()
                .borders(Borders::ALL)
                .title("📚 HELP & KEYBOARD SHORTCUTS")
                .title_style(ratatui::style::Style::default()
                    .fg(self.color_theme.get_selected_border_color())
                    .add_modifier(Modifier::BOLD))
                .border_style(ratatui::style::Style::default()
                    .fg(self.color_theme.get_selected_border_color()))
                .style(ratatui::style::Style::default().bg(ratatui::style::Color::Black)))
            .alignment(Alignment::Left)
            .style(ratatui::style::Style::default().fg(ratatui::style::Color::White));
        
        f.render_widget(help_popup, popup_area);
    }
    
    fn render_add_zone_modal(&self, f: &mut Frame) {
        let area = f.area();
        let popup_area = Rect {
            x: area.width / 4,
            y: area.height / 3,
            width: area.width / 2,
            height: area.height / 3,
        };
        
        // Clear the background
        f.render_widget(Clear, popup_area);
        
        // Create the modal content
        let mut content = vec![
            "Type city name to search for timezone:".to_string(),
            "".to_string(),
            format!("> {}", self.add_zone_input),
            "".to_string(),
        ];
        
        if !self.zone_search_results.is_empty() {
            content.push("Search results:".to_string());
            for (i, result) in self.zone_search_results.iter().enumerate() {
                content.push(format!("  {}. {}", i + 1, result));
            }
        } else if !self.add_zone_input.is_empty() {
            content.push("No matching timezones found.".to_string());
        }
        
        content.push("".to_string());
        content.push("Enter: Add zone | 1-5: Select result | Esc: Cancel".to_string());
        
        let modal_text = content.join("\n");
        
        let add_zone_popup = Paragraph::new(modal_text)
            .block(Block::default()
                .borders(Borders::ALL)
                .title(" Add Timezone ")
                .title_style(ratatui::style::Style::default().fg(ratatui::style::Color::Green).add_modifier(Modifier::BOLD))
                .border_style(ratatui::style::Style::default().fg(ratatui::style::Color::Green))
                .style(ratatui::style::Style::default().bg(ratatui::style::Color::Black)))
            .alignment(Alignment::Left)
            .style(ratatui::style::Style::default().fg(ratatui::style::Color::White));
        
        f.render_widget(add_zone_popup, popup_area);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_app_creation() {
        let app = App::new();
        assert!(!app.should_quit);
        // Selected zone index is now set to match local timezone, not necessarily 0
        assert!(app.selected_zone_index < app.timezone_manager.zones().len());
        assert_eq!(app.display_format, TimeFormat::TwentyFourHour);
        assert_eq!(app.timezone_display_mode, TimezoneDisplayMode::Short);
        assert!(!app.show_help);
        assert!(!app.adding_zone);
        assert!(app.add_zone_input.is_empty());
        assert!(app.zone_search_results.is_empty());
        assert!(app.show_weather);
        // Color theme should match what's loaded from config (could be default or saved value)
        assert!(matches!(app.color_theme, crate::config::ColorTheme::Default | crate::config::ColorTheme::Ocean | crate::config::ColorTheme::Forest | crate::config::ColorTheme::Sunset | crate::config::ColorTheme::Cyberpunk | crate::config::ColorTheme::Monochrome));
    }
    
    #[test]
    fn test_cycle_color_theme() {
        let mut app = App::new();
        let initial_theme = app.color_theme;
        
        app.update(Message::CycleColorTheme);
        assert_ne!(app.color_theme, initial_theme);
        
        // Cycle through all themes (6 total) and ensure we can get back to the start
        for _ in 0..4 { // We already cycled once, so 4 more cycles to complete the loop
            app.update(Message::CycleColorTheme);
        }
        
        app.update(Message::CycleColorTheme);
        assert_eq!(app.color_theme, initial_theme); // Should be back to the original after 6 cycles total
    }
    
    #[test]
    fn test_quit_message() {
        let mut app = App::new();
        assert!(!app.should_quit);
        
        app.update(Message::Quit);
        assert!(app.should_quit);
    }
    
    #[test]
    fn test_toggle_time_format() {
        let mut app = App::new();
        assert_eq!(app.display_format, TimeFormat::TwentyFourHour);
        
        app.update(Message::ToggleTimeFormat);
        assert_eq!(app.display_format, TimeFormat::TwelveHour);
        
        app.update(Message::ToggleTimeFormat);
        assert_eq!(app.display_format, TimeFormat::TwentyFourHour);
    }
    
    #[test]
    fn test_timeline_scrubbing() {
        let mut app = App::new();
        let initial_time = app.timeline_position;
        
        // Scrub right - should round to next hour
        app.update(Message::ScrubTimeline(Direction::Right));
        assert!(app.timeline_position > initial_time);
        
        // Test that scrubbing works in both directions
        let after_right = app.timeline_position;
        app.update(Message::ScrubTimeline(Direction::Left));
        assert!(app.timeline_position < after_right);
    }
    
    #[test]
    fn test_local_timezone_selection() {
        let app = App::new();
        let zones = app.timezone_manager.zones();
        
        // Verify that a valid zone is selected
        assert!(app.selected_zone_index < zones.len());
        
        // The selected zone should have an offset that matches local time
        let local_time = app.current_time.with_timezone(&Local);
        let local_offset_hours = local_time.offset().fix().local_minus_utc() / 3600;
        
        let selected_zone = &zones[app.selected_zone_index];
        let selected_offset_hours = selected_zone.utc_offset_hours();
        
        // They should match (allowing for DST differences)
        assert_eq!(selected_offset_hours, local_offset_hours);
    }
}