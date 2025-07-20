use chrono::{DateTime, Local, Offset, Timelike, Utc};
use ratatui::{
    layout::{Alignment, Constraint, Direction as LayoutDirection, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Clear, Paragraph, Row, Table, TableState, Wrap},
    Frame,
};

use crate::config::{AppConfig, ColorTheme, TimeDisplayConfig};
use crate::time::{TimeZone, TimeZoneManager};
use crate::ui::TimelineWidget;

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum TimeFormat {
    TwentyFourHour,
    TwelveHour,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum TimezoneDisplayMode {
    Short, // LAX, NYC, LON
    Full,  // Pacific Time (US) PDT UTC-7
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
    ToggleDate,
    ToggleSunTimes,
    ToggleHelp,
    CycleColorTheme,

    // Zone management
    StartAddZone,
    UpdateAddZoneInput(String),
    NavigateSearchResults(Direction),
    SelectSearchResult(usize),
    ConfirmAddZone,
    CancelAddZone,
    RemoveCurrentZone,

    // Zone renaming
    StartRenameZone,
    UpdateRenameInput(String),
    ConfirmRename,
    CancelRename,
    ClearCustomName,

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
    pub selected_search_result: usize,
    pub renaming_zone: bool,
    pub rename_zone_input: String,
    pub show_date: bool,
    pub show_sun_times: bool,

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
            selected_search_result: 0,
            renaming_zone: false,
            rename_zone_input: String::new(),
            show_date: false,
            show_sun_times: true,
            should_quit: false,
        }
    }
}

impl App {
    pub fn new() -> Self {
        let config = AppConfig::load();

        // Create default config file if it doesn't exist
        if let Some(config_path) = AppConfig::config_path() {
            if !config_path.exists() {
                // Only create if we successfully loaded default config
                if let Err(_) = config.save() {
                    // Don't fail if we can't save config, just continue with defaults
                    eprintln!(
                        "Warning: Could not create default config file at {}",
                        config_path.display()
                    );
                }
            }
        }

        let mut app = Self::from_config(config);
        app.select_local_timezone();
        app
    }

    pub fn from_config(config: AppConfig) -> Self {
        let mut timezone_manager = TimeZoneManager::new();

        // Load timezones from config with custom labels
        for zone_config in &config.zones {
            timezone_manager.add_timezone_with_label(
                zone_config.city_name(),
                zone_config.custom_label().map(|s| s.to_string()),
            );
        }

        // If no zones were loaded, use defaults
        if timezone_manager.zones().is_empty() {
            timezone_manager = TimeZoneManager::with_default_zones();
        }

        let now = Utc::now();
        let selected_zone_index = config
            .selected_zone_index
            .min(timezone_manager.zone_count().saturating_sub(1));

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
            selected_search_result: 0,
            renaming_zone: false,
            rename_zone_input: String::new(),
            show_date: config.show_date,
            show_sun_times: config.show_sun_times,
            should_quit: false,
        }
    }

    pub fn to_config(&self) -> AppConfig {
        AppConfig {
            zones: self
                .timezone_manager
                .zones()
                .iter()
                .map(|zone| {
                    // Use the source_city if available, otherwise find the original search name
                    let city_name = if let Some(source_city) = &zone.source_city {
                        source_city.clone()
                    } else {
                        let available = TimeZoneManager::get_all_available_timezones();
                        available
                            .iter()
                            .find(|(tz, _, _, _, _)| *tz == zone.tz)
                            .map(|(_, search_name, _, _, _)| search_name.clone())
                            .unwrap_or_else(|| zone.tz.to_string())
                    };

                    // Save as full ZoneConfig if custom label is present, otherwise as simple string
                    match &zone.custom_label {
                        Some(label) => {
                            crate::config::ZoneConfigCompat::Full(crate::config::ZoneConfig {
                                city_name,
                                custom_label: Some(label.clone()),
                            })
                        }
                        None => crate::config::ZoneConfigCompat::Simple(city_name),
                    }
                })
                .collect(),
            selected_zone_index: self.selected_zone_index,
            display_format: self.display_format.clone(),
            timezone_display_mode: self.timezone_display_mode.clone(),
            time_config: self.time_config.clone(),
            color_theme: self.color_theme,
            show_date: self.show_date,
            show_sun_times: self.show_sun_times,
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
                        if self.timeline_position.minute() == 0
                            && self.timeline_position.second() == 0
                            && self.timeline_position.nanosecond() == 0
                        {
                            // Already at hour boundary, go to previous hour
                            self.timeline_position - chrono::Duration::hours(1)
                        } else {
                            // Go to start of current hour
                            self.timeline_position
                                .with_minute(0)
                                .unwrap_or(self.timeline_position)
                                .with_second(0)
                                .unwrap_or(self.timeline_position)
                                .with_nanosecond(0)
                                .unwrap_or(self.timeline_position)
                        }
                    }
                    Direction::Right => {
                        // Go to the start of the next hour
                        let next_hour_start = self
                            .timeline_position
                            .with_minute(0)
                            .unwrap_or(self.timeline_position)
                            .with_second(0)
                            .unwrap_or(self.timeline_position)
                            .with_nanosecond(0)
                            .unwrap_or(self.timeline_position)
                            + chrono::Duration::hours(1);
                        next_hour_start
                    }
                    _ => self.timeline_position,
                };
                self.timeline_position = rounded_time;
                None
            }

            Message::ScrubTimelineWithShift(direction) => {
                // Fine scrub by 1 minute when shift is held
                let delta = match direction {
                    Direction::Left => chrono::Duration::minutes(-1),
                    Direction::Right => chrono::Duration::minutes(1),
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
                let zone_count = self.timezone_manager.zone_count();
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

            Message::ToggleDate => {
                self.show_date = !self.show_date;
                self.save_config();
                None
            }

            Message::ToggleSunTimes => {
                self.show_sun_times = !self.show_sun_times;
                self.save_config();
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
                // Clear other modal states
                self.renaming_zone = false;
                self.rename_zone_input.clear();
                
                self.adding_zone = true;
                self.add_zone_input.clear();
                self.zone_search_results.clear();
                self.selected_search_result = 0;
                None
            }

            Message::UpdateAddZoneInput(input) => {
                self.add_zone_input = input.clone();
                self.zone_search_results = crate::time::TimeZoneManager::search_timezones(&input);
                self.selected_search_result = 0; // Reset selection when search changes
                None
            }

            Message::NavigateSearchResults(direction) => {
                if !self.zone_search_results.is_empty() {
                    match direction {
                        Direction::Up => {
                            if self.selected_search_result > 0 {
                                self.selected_search_result -= 1;
                            }
                        }
                        Direction::Down => {
                            if self.selected_search_result < self.zone_search_results.len() - 1 {
                                self.selected_search_result += 1;
                            }
                        }
                        _ => {}
                    }
                }
                None
            }

            Message::SelectSearchResult(index) => {
                if let Some(zone_name) = self.zone_search_results.get(index) {
                    let success = self.timezone_manager.add_timezone_by_name(zone_name);

                    if success {
                        // Update selected index if needed
                        if self.selected_zone_index >= self.timezone_manager.zone_count() {
                            self.selected_zone_index =
                                self.timezone_manager.zone_count().saturating_sub(1);
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
                if !self.zone_search_results.is_empty() {
                    // Use the currently selected search result
                    if let Some(zone_name) =
                        self.zone_search_results.get(self.selected_search_result)
                    {
                        let success = self.timezone_manager.add_timezone_by_name(zone_name);

                        if success {
                            // Update selected index if needed
                            if self.selected_zone_index >= self.timezone_manager.zone_count() {
                                self.selected_zone_index =
                                    self.timezone_manager.zone_count().saturating_sub(1);
                            }
                            self.save_config();
                        }
                    }
                } else if !self.add_zone_input.is_empty() {
                    // Try to add the exact input if no search results
                    let success = self
                        .timezone_manager
                        .add_timezone_by_name(&self.add_zone_input);

                    if success {
                        // Update selected index if needed
                        if self.selected_zone_index >= self.timezone_manager.zone_count() {
                            self.selected_zone_index =
                                self.timezone_manager.zone_count().saturating_sub(1);
                        }
                        self.save_config();
                    }
                }
                self.adding_zone = false;
                self.add_zone_input.clear();
                self.zone_search_results.clear();
                self.selected_search_result = 0;
                None
            }

            Message::CancelAddZone => {
                self.adding_zone = false;
                self.add_zone_input.clear();
                self.zone_search_results.clear();
                self.selected_search_result = 0;
                None
            }

            Message::RemoveCurrentZone => {
                if self.timezone_manager.zone_count() > 1 {
                    // Keep at least one zone
                    self.timezone_manager.remove_zone(self.selected_zone_index);

                    // Adjust selected index if needed
                    if self.selected_zone_index >= self.timezone_manager.zone_count() {
                        self.selected_zone_index =
                            self.timezone_manager.zone_count().saturating_sub(1);
                    }
                    self.save_config();
                }
                None
            }

            Message::StartRenameZone => {
                if self.timezone_manager.zone_count() > 0 {
                    // Clear other modal states
                    self.adding_zone = false;
                    self.add_zone_input.clear();
                    self.zone_search_results.clear();
                    
                    self.renaming_zone = true;
                    // Pre-fill with current custom label or empty
                    self.rename_zone_input = self.timezone_manager.zones()
                        [self.selected_zone_index]
                        .custom_label
                        .clone()
                        .unwrap_or_default();
                }
                None
            }

            Message::UpdateRenameInput(input) => {
                self.rename_zone_input = input;
                None
            }

            Message::ConfirmRename => {
                if self.timezone_manager.zone_count() > 0 {
                    let custom_label = if self.rename_zone_input.trim().is_empty() {
                        None
                    } else {
                        Some(self.rename_zone_input.trim().to_string())
                    };
                    self.timezone_manager
                        .update_zone_label(self.selected_zone_index, custom_label);
                    self.save_config();
                }
                self.renaming_zone = false;
                self.rename_zone_input.clear();
                None
            }

            Message::CancelRename => {
                self.renaming_zone = false;
                self.rename_zone_input.clear();
                None
            }

            Message::ClearCustomName => {
                if self.timezone_manager.zone_count() > 0 {
                    self.timezone_manager
                        .update_zone_label(self.selected_zone_index, None);
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
                Constraint::Length(2), // Legend
                Constraint::Length(3), // Footer
            ])
            .split(f.area());

        self.render_header(f, chunks[0]);
        self.render_current_time_display(f, chunks[1]);
        self.render_zones(f, chunks[2]);
        self.render_legend(f, chunks[3]);
        self.render_footer(f, chunks[4]);

        // Render modals on top if needed
        if self.show_help {
            self.render_help_modal(f);
        } else if self.adding_zone {
            self.render_add_zone_modal(f);
        } else if self.renaming_zone {
            self.render_rename_zone_modal(f);
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
        let inner = area.inner(ratatui::layout::Margin {
            horizontal: 1,
            vertical: 1,
        });

        let chunks = Layout::default()
            .direction(LayoutDirection::Horizontal)
            .constraints([
                Constraint::Percentage(33), // App name (left)
                Constraint::Percentage(34), // Local time (center)
                Constraint::Percentage(33), // Timeline (right)
            ])
            .split(inner);

        // Left: App name
        let app_name = Paragraph::new(format!("alltz v{}", env!("CARGO_PKG_VERSION")))
            .alignment(Alignment::Left);
        f.render_widget(app_name, chunks[0]);

        // Center: Local time
        let local_display =
            Paragraph::new(format!("Local: {}", local_time_str)).alignment(Alignment::Center);
        f.render_widget(local_display, chunks[1]);

        // Right: Timeline time
        let timeline_display =
            Paragraph::new(format!("Timeline: {}", timeline_time_str)).alignment(Alignment::Right);
        f.render_widget(timeline_display, chunks[2]);

        let border = Block::default().borders(Borders::ALL);
        f.render_widget(border, area);
    }

    fn get_local_timezone_name(&self) -> String {
        // Try to get a better timezone name from our configured zones
        let local_time = self.current_time.with_timezone(&Local);
        let local_offset_hours = local_time.offset().fix().local_minus_utc() / 3600;

        // Look for a matching timezone in our list to get a better abbreviation
        for zone in self.timezone_manager.zones() {
            if zone.utc_offset_hours() == local_offset_hours {
                return zone.get_timezone_abbreviation();
            }
        }

        // Fallback: use system timezone name
        let tz_str = local_time.format("%Z").to_string();
        if tz_str.starts_with('+') || tz_str.starts_with('-') {
            // If it's still showing offset, try a different approach
            format!(
                "UTC{}",
                if local_offset_hours >= 0 {
                    format!("+{}", local_offset_hours)
                } else {
                    local_offset_hours.to_string()
                }
            )
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

        let zone_constraints = zones
            .iter()
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
        let timeline_widget = TimelineWidget::new(
            self.timeline_position,
            self.current_time,
            zone,
            is_selected,
            self.display_format.clone(),
            self.timezone_display_mode.clone(),
            &self.time_config,
            self.color_theme,
            self.show_date,
            true, // DST indicators always on
            self.show_sun_times,
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
            TimeFormat::TwentyFourHour => local_time.format("%H:%M %a").to_string(),
            TimeFormat::TwelveHour => local_time.format("%I:%M %p %a").to_string(),
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

    fn render_legend(&self, f: &mut Frame, area: Rect) {
        use ratatui::text::{Line, Span};

        // Create legend showing what the different timeline colors/characters mean
        let night_char = self
            .time_config
            .get_activity_char(crate::config::TimeActivity::Night);
        let awake_char = self
            .time_config
            .get_activity_char(crate::config::TimeActivity::Awake);
        let work_char = self
            .time_config
            .get_activity_char(crate::config::TimeActivity::Work);

        let night_color = self.color_theme.get_night_color();
        let awake_color = self.color_theme.get_awake_color();
        let work_color = self.color_theme.get_work_color();

        let legend_line = Line::from(vec![
            Span::styled(format!("{} ", night_char), Style::default().fg(night_color)),
            Span::raw("Night  "),
            Span::styled(format!("{} ", awake_char), Style::default().fg(awake_color)),
            Span::raw("Awake  "),
            Span::styled(format!("{} ", work_char), Style::default().fg(work_color)),
            Span::raw("Work  "),
            Span::styled("┊ ", Style::default().fg(night_color)),
            Span::raw("Midnight  "),
            Span::styled("│ ", Style::default().fg(Color::Red)),
            Span::raw("Now  "),
            Span::styled(
                "┃ ",
                Style::default().fg(self.color_theme.get_timeline_position_color()),
            ),
            Span::raw("Timeline"),
        ]);

        let legend = Paragraph::new(legend_line)
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center);

        f.render_widget(legend, area);
    }

    fn render_footer(&self, f: &mut Frame, area: Rect) {
        let footer_text = "?: help │ a: add │ q: quit";

        let footer = Paragraph::new(footer_text)
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center);

        f.render_widget(footer, area);
    }

    fn render_help_modal(&self, f: &mut Frame) {
        let area = f.area();

        // Calculate modal size to fit content
        let modal_width = area.width * 2 / 3; // Same as add city modal
                                              // Calculate height based on content: title + max column content + footer + borders
        let max_content_lines = 17; // Longest column has about 17 lines
        let modal_height = (2 + max_content_lines + 1 + 4).min(area.height.saturating_sub(2)); // title + content + footer + borders + margin

        let popup_area = Rect {
            x: (area.width.saturating_sub(modal_width)) / 2,
            y: (area.height.saturating_sub(modal_height)) / 2,
            width: modal_width,
            height: modal_height,
        };

        // Clear the background
        f.render_widget(Clear, popup_area);

        // Split into sections using layout
        let inner = popup_area.inner(ratatui::layout::Margin {
            horizontal: 1,
            vertical: 1,
        });
        let chunks = Layout::default()
            .direction(LayoutDirection::Vertical)
            .constraints([
                Constraint::Length(2), // Title
                Constraint::Min(1),    // Content
                Constraint::Length(1), // Footer
            ])
            .split(inner);

        // Render title
        let title = Paragraph::new("🕐 HELP & KEYBOARD SHORTCUTS")
            .style(
                Style::default()
                    .fg(self.color_theme.get_selected_border_color())
                    .add_modifier(Modifier::BOLD),
            )
            .alignment(Alignment::Center);
        f.render_widget(title, chunks[0]);

        // Split content into two columns for better space usage
        let content_chunks = Layout::default()
            .direction(LayoutDirection::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(chunks[1]);

        // Left column content
        let left_sections = vec![
            (
                "TIME NAVIGATION",
                vec![
                    "h/← or l/→     Scrub timeline (1 hour)",
                    "Shift + h/l    Fine scrub (1 minute)",
                    "[ or ]         Adjust by ±15 minutes",
                    "{ or }         Adjust by ±1 hour",
                    "t              Reset to current time",
                ],
            ),
            (
                "ZONE NAVIGATION",
                vec![
                    "j/↓ or k/↑     Navigate between zones",
                    "               Selected has colored border",
                ],
            ),
            (
                "DISPLAY OPTIONS",
                vec![
                    "m              Toggle 12/24 hour format",
                    "n              Toggle short/full names",
                    "d              Toggle date display",
                    "s              Toggle sunrise/sunset times",
                    "c              Cycle color themes",
                ],
            ),
        ];

        // Right column content
        let right_sections = vec![
            (
                "ZONE MANAGEMENT",
                vec![
                    "a              Add new timezone",
                    "r              Remove selected timezone",
                    "e              Rename selected timezone",
                    "E              Clear custom name",
                    "1-8            Quick-select search results",
                ],
            ),
            (
                "INDICATORS",
                vec![
                    "│              Red line: Current time",
                    "┃              Colored line: Timeline position",
                    "⇈              DST spring forward",
                    "⇊              DST fall back",
                    "░ ▒ ▓          Night, Awake, Work hours",
                ],
            ),
            (
                "CONTROLS",
                vec![
                    "?              Show/hide help",
                    "q              Quit",
                    "Esc            Cancel operation",
                ],
            ),
        ];

        // Render left column
        let left_text = left_sections
            .iter()
            .map(|(section, items)| {
                let mut section_text = format!("\n{}\n", section);
                for item in items {
                    section_text.push_str(&format!("  {}\n", item));
                }
                section_text
            })
            .collect::<Vec<_>>()
            .join("");

        let left_content = Paragraph::new(left_text)
            .style(Style::default().fg(Color::White))
            .wrap(Wrap { trim: true });
        f.render_widget(left_content, content_chunks[0]);

        // Render right column
        let right_text = right_sections
            .iter()
            .map(|(section, items)| {
                let mut section_text = format!("\n{}\n", section);
                for item in items {
                    section_text.push_str(&format!("  {}\n", item));
                }
                section_text
            })
            .collect::<Vec<_>>()
            .join("");

        let right_content = Paragraph::new(right_text)
            .style(Style::default().fg(Color::White))
            .wrap(Wrap { trim: true });
        f.render_widget(right_content, content_chunks[1]);

        // Render footer
        let footer = Paragraph::new("Press any key to close")
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Center);
        f.render_widget(footer, chunks[2]);

        // Render border around the entire modal
        let border = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.color_theme.get_selected_border_color()))
            .style(Style::default().bg(Color::Black));
        f.render_widget(border, popup_area);
    }

    fn render_add_zone_modal(&self, f: &mut Frame) {
        let area = f.area();

        // Calculate height more generously to ensure examples are shown
        let results_height = if !self.zone_search_results.is_empty() {
            self.zone_search_results.len() as u16 + 2 // +2 for header and spacing
        } else {
            8 // Space for examples and help text
        };

        let modal_height = (3 + results_height + 3 + 2).min(area.height.saturating_sub(4)); // header + content + controls + borders

        let popup_area = Rect {
            x: area.width / 6,
            y: (area.height.saturating_sub(modal_height)) / 2,
            width: area.width * 2 / 3,
            height: modal_height,
        };

        // Clear the background
        f.render_widget(Clear, popup_area);

        // Split the modal into sections
        let inner = popup_area.inner(ratatui::layout::Margin {
            horizontal: 1,
            vertical: 1,
        });
        let chunks = Layout::default()
            .direction(LayoutDirection::Vertical)
            .constraints([
                Constraint::Length(3), // Header and input
                Constraint::Min(1),    // Results table
                Constraint::Length(3), // Help text
            ])
            .split(inner);

        // Render header and input
        let header_text = format!("Search: {}", self.add_zone_input);
        let header = Paragraph::new(header_text)
            .style(ratatui::style::Style::default().fg(ratatui::style::Color::White));
        f.render_widget(header, chunks[0]);

        // Render results table or help text
        if !self.zone_search_results.is_empty() {
            self.render_search_results_table(f, chunks[1]);
        } else {
            self.render_search_help(f, chunks[1]);
        }

        // Render controls help
        let controls = if !self.zone_search_results.is_empty() {
            "↑↓: Navigate | Enter: Add selected | 1-8: Quick select | Esc: Cancel"
        } else {
            "Type to search cities, countries, or abbreviations | Esc: Cancel"
        };

        let controls_paragraph = Paragraph::new(controls)
            .style(ratatui::style::Style::default().fg(ratatui::style::Color::DarkGray))
            .alignment(Alignment::Center);
        f.render_widget(controls_paragraph, chunks[2]);

        // Render the modal border
        let border = Block::default()
            .borders(Borders::ALL)
            .title(" Add Timezone ")
            .title_style(
                ratatui::style::Style::default()
                    .fg(ratatui::style::Color::Green)
                    .add_modifier(Modifier::BOLD),
            )
            .border_style(ratatui::style::Style::default().fg(ratatui::style::Color::Green))
            .style(ratatui::style::Style::default().bg(ratatui::style::Color::Black));
        f.render_widget(border, popup_area);
    }

    fn render_search_results_table(&self, f: &mut Frame, area: Rect) {
        let mut rows = Vec::new();

        for (i, result) in self.zone_search_results.iter().enumerate() {
            if let Some((city_country, time_str, offset, code)) =
                self.get_search_result_parts(result)
            {
                rows.push(Row::new(vec![
                    Cell::from(format!("{}", i + 1)),
                    Cell::from(city_country),
                    Cell::from(time_str),
                    Cell::from(offset),
                    Cell::from(code),
                ]));
            } else {
                rows.push(Row::new(vec![
                    Cell::from(format!("{}", i + 1)),
                    Cell::from(result.clone()),
                    Cell::from(""),
                    Cell::from(""),
                    Cell::from(""),
                ]));
            }
        }

        let table = Table::new(
            rows,
            [
                Constraint::Length(3),  // #
                Constraint::Min(20),    // City, Country
                Constraint::Length(10), // Time
                Constraint::Length(8),  // Offset
                Constraint::Length(6),  // Code
            ],
        )
        .header(Row::new(vec![
            Cell::from("#").style(ratatui::style::Style::default().add_modifier(Modifier::BOLD)),
            Cell::from("City, Country")
                .style(ratatui::style::Style::default().add_modifier(Modifier::BOLD)),
            Cell::from("Time").style(ratatui::style::Style::default().add_modifier(Modifier::BOLD)),
            Cell::from("Offset")
                .style(ratatui::style::Style::default().add_modifier(Modifier::BOLD)),
            Cell::from("Code").style(ratatui::style::Style::default().add_modifier(Modifier::BOLD)),
        ]))
        .style(ratatui::style::Style::default().fg(ratatui::style::Color::White))
        .row_highlight_style(
            ratatui::style::Style::default()
                .fg(self.color_theme.get_work_color())
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> ")
        .column_spacing(1);

        // Create table state with selection
        let mut table_state = TableState::default();
        table_state.select(Some(self.selected_search_result));

        f.render_stateful_widget(table, area, &mut table_state);
    }

    fn render_search_help(&self, f: &mut Frame, area: Rect) {
        let help_text = if !self.add_zone_input.is_empty() {
            vec![
                "No matching timezones found.",
                "",
                "Try searching for:",
                "  • City names: Tokyo, New York, London",
                "  • Countries: Japan, USA, Germany",
                "  • Abbreviations: NYC, SF, LA",
                "  • Regions: Bay Area, Silicon Valley",
            ]
        } else {
            vec![
                "Search for cities, countries, or abbreviations:",
                "",
                "Examples:",
                "  Tokyo, London, Mumbai",
                "  NYC, SF, LA, DC",
                "  Japan, USA, Germany",
                "  Bay Area, Silicon Valley",
            ]
        };

        let help_paragraph = Paragraph::new(help_text.join("\n"))
            .style(ratatui::style::Style::default().fg(ratatui::style::Color::DarkGray))
            .alignment(Alignment::Left);
        f.render_widget(help_paragraph, area);
    }

    fn render_rename_zone_modal(&self, f: &mut Frame) {
        let area = f.area();

        let modal_height = 9; // Smaller modal for rename
        let modal_width = area.width.saturating_sub(area.width / 3).min(60);

        let popup_area = Rect {
            x: (area.width.saturating_sub(modal_width)) / 2,
            y: (area.height.saturating_sub(modal_height)) / 2,
            width: modal_width,
            height: modal_height,
        };

        // Clear the background
        f.render_widget(Clear, popup_area);

        // Get current zone info
        let current_zone = &self.timezone_manager.zones()[self.selected_zone_index];
        let city_name = self
            .timezone_manager
            .zones()
            .iter()
            .enumerate()
            .find(|(idx, zone)| *idx == self.selected_zone_index && zone.tz == current_zone.tz)
            .and_then(|(_, zone)| {
                let available = crate::time::TimeZoneManager::get_all_available_timezones();
                available
                    .iter()
                    .find(|(tz, _, _, _, _)| *tz == zone.tz)
                    .map(|(_, name, _, _, _)| name.clone())
            })
            .unwrap_or_else(|| current_zone.tz.to_string());

        // Split the modal into sections
        let inner = popup_area.inner(ratatui::layout::Margin {
            horizontal: 1,
            vertical: 1,
        });
        let chunks = Layout::default()
            .direction(LayoutDirection::Vertical)
            .constraints([
                Constraint::Length(2), // Zone info
                Constraint::Length(2), // Input field
                Constraint::Length(1), // Spacer
                Constraint::Length(2), // Controls help
            ])
            .split(inner);

        // Render zone info
        let zone_info = format!("Renaming: {}", city_name);
        let zone_paragraph = Paragraph::new(zone_info)
            .style(ratatui::style::Style::default().fg(ratatui::style::Color::Gray));
        f.render_widget(zone_paragraph, chunks[0]);

        // Render input field
        let input_text = format!("Custom name: {}", self.rename_zone_input);
        let input_paragraph = Paragraph::new(input_text)
            .style(ratatui::style::Style::default().fg(ratatui::style::Color::White));
        f.render_widget(input_paragraph, chunks[1]);

        // Render controls help
        let controls = "Enter: Save | Esc: Cancel | Empty to remove custom name";
        let controls_paragraph = Paragraph::new(controls)
            .style(ratatui::style::Style::default().fg(ratatui::style::Color::DarkGray))
            .alignment(Alignment::Center);
        f.render_widget(controls_paragraph, chunks[3]);

        // Render the modal border
        let border = Block::default()
            .borders(Borders::ALL)
            .title(" Rename Timezone ")
            .title_style(
                ratatui::style::Style::default()
                    .fg(ratatui::style::Color::Blue)
                    .add_modifier(Modifier::BOLD),
            )
            .border_style(ratatui::style::Style::default().fg(ratatui::style::Color::Blue))
            .style(ratatui::style::Style::default().bg(ratatui::style::Color::Black));
        f.render_widget(border, popup_area);
    }

    fn get_search_result_parts(&self, city_name: &str) -> Option<(String, String, String, String)> {
        let available = crate::time::TimeZoneManager::get_all_available_timezones();

        if let Some((tz, _, display_name, _, _)) = available
            .iter()
            .find(|(_, search_name, _, _, _)| search_name == city_name)
        {
            let timezone = crate::time::TimeZone::from_tz(*tz);
            let current_time_in_zone = timezone.convert_time(self.current_time);

            let time_str = match self.display_format {
                TimeFormat::TwentyFourHour => current_time_in_zone.format("%H:%M").to_string(),
                TimeFormat::TwelveHour => current_time_in_zone.format("%I:%M %p").to_string(),
            };

            let country = crate::time::TimeZoneManager::get_country_for_city(city_name);
            let city_country = format!("{}, {}", city_name, country);

            Some((
                city_country,
                time_str,
                timezone.offset_string(),
                display_name.clone(),
            ))
        } else {
            None
        }
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
        assert!(app.selected_zone_index < app.timezone_manager.zone_count());
        // Display format and timezone display mode are loaded from config, so they might vary
        assert!(matches!(
            app.display_format,
            TimeFormat::TwentyFourHour | TimeFormat::TwelveHour
        ));
        assert!(matches!(
            app.timezone_display_mode,
            TimezoneDisplayMode::Short | TimezoneDisplayMode::Full
        ));
        assert!(!app.show_help);
        assert!(!app.adding_zone);
        assert!(app.add_zone_input.is_empty());
        assert!(app.zone_search_results.is_empty());
        assert_eq!(app.selected_search_result, 0);
        // show_date is loaded from config, could be true or false
        // Color theme should match what's loaded from config (could be default or saved value)
        assert!(matches!(
            app.color_theme,
            crate::config::ColorTheme::Default
                | crate::config::ColorTheme::Ocean
                | crate::config::ColorTheme::Forest
                | crate::config::ColorTheme::Sunset
                | crate::config::ColorTheme::Cyberpunk
                | crate::config::ColorTheme::Monochrome
        ));
    }

    #[test]
    fn test_cycle_color_theme() {
        let mut app = App::new();
        let initial_theme = app.color_theme;

        app.update(Message::CycleColorTheme);
        assert_ne!(app.color_theme, initial_theme);

        // Cycle through all themes (6 total) and ensure we can get back to the start
        for _ in 0..4 {
            // We already cycled once, so 4 more cycles to complete the loop
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
    fn test_start_rename_zone() {
        let mut app = App::default(); // Use default instead of new() to avoid loading config
        assert!(!app.renaming_zone);
        assert!(app.rename_zone_input.is_empty());

        app.update(Message::StartRenameZone);
        assert!(app.renaming_zone);
        // Should be pre-filled with current custom label (from default zones)
        let current_label = app.timezone_manager.zones()[app.selected_zone_index]
            .custom_label
            .clone()
            .unwrap_or_default();
        assert_eq!(app.rename_zone_input, current_label);
    }

    #[test]
    fn test_update_rename_input() {
        let mut app = App::new();
        app.renaming_zone = true;

        app.update(Message::UpdateRenameInput("Alice".to_string()));
        assert_eq!(app.rename_zone_input, "Alice");

        app.update(Message::UpdateRenameInput(
            "Alice (Engineering)".to_string(),
        ));
        assert_eq!(app.rename_zone_input, "Alice (Engineering)");
    }

    #[test]
    fn test_confirm_rename() {
        let mut app = App::default(); // Use default to avoid loading config
        app.renaming_zone = true;
        app.rename_zone_input = "Bob (Sales)".to_string();

        // Store initial state (may or may not have custom label)
        let _initial_label = app.timezone_manager.zones()[app.selected_zone_index]
            .custom_label
            .clone();

        app.update(Message::ConfirmRename);

        // Should have the new custom label
        assert_eq!(
            app.timezone_manager.zones()[app.selected_zone_index]
                .custom_label
                .as_deref(),
            Some("Bob (Sales)")
        );
        assert!(!app.renaming_zone);
        assert!(app.rename_zone_input.is_empty());
    }

    #[test]
    fn test_confirm_rename_empty_input() {
        let mut app = App::new();
        app.renaming_zone = true;
        app.rename_zone_input = "  ".to_string(); // Whitespace only

        app.update(Message::ConfirmRename);

        // Empty/whitespace input should clear custom label
        assert_eq!(
            app.timezone_manager.zones()[app.selected_zone_index].custom_label,
            None
        );
        assert!(!app.renaming_zone);
        assert!(app.rename_zone_input.is_empty());
    }

    #[test]
    fn test_modal_state_exclusivity() {
        let mut app = App::new();
        
        // Add a timezone first so we can rename it
        app.update(Message::StartAddZone);
        app.update(Message::UpdateAddZoneInput("London".to_string()));
        app.update(Message::ConfirmAddZone);
        
        // Start add zone mode
        app.update(Message::StartAddZone);
        assert!(app.adding_zone);
        assert!(!app.renaming_zone);
        
        // Now start rename mode - should clear add zone state
        app.update(Message::StartRenameZone);
        assert!(!app.adding_zone);
        assert!(app.renaming_zone);
        assert!(app.add_zone_input.is_empty());
        assert!(app.zone_search_results.is_empty());
        
        // Start add zone mode again - should clear rename state
        app.update(Message::StartAddZone);
        assert!(app.adding_zone);
        assert!(!app.renaming_zone);
        assert!(app.rename_zone_input.is_empty());
    }

    #[test]
    fn test_cancel_rename() {
        let mut app = App::new();
        app.renaming_zone = true;
        app.rename_zone_input = "Some input".to_string();

        app.update(Message::CancelRename);

        assert!(!app.renaming_zone);
        assert!(app.rename_zone_input.is_empty());
    }

    #[test]
    fn test_clear_custom_name() {
        let mut app = App::new();

        // First set a custom label
        app.timezone_manager
            .update_zone_label(app.selected_zone_index, Some("Test Label".to_string()));
        assert_eq!(
            app.timezone_manager.zones()[app.selected_zone_index]
                .custom_label
                .as_deref(),
            Some("Test Label")
        );

        // Clear it
        app.update(Message::ClearCustomName);
        assert_eq!(
            app.timezone_manager.zones()[app.selected_zone_index].custom_label,
            None
        );
    }

    #[test]
    fn test_config_with_custom_labels() {
        let mut app = App::default(); // Use default to avoid loading config

        // Clear any existing labels first
        for i in 0..app.timezone_manager.zone_count() {
            app.timezone_manager.update_zone_label(i, None);
        }

        // Add custom labels to first two zones
        app.timezone_manager
            .update_zone_label(0, Some("Alice".to_string()));
        if app.timezone_manager.zone_count() > 1 {
            app.timezone_manager
                .update_zone_label(1, Some("Bob".to_string()));
        }

        // Convert to config
        let config = app.to_config();

        // Check that custom labels are preserved in config
        assert_eq!(config.zones[0].custom_label(), Some("Alice"));
        if config.zones.len() > 1 {
            assert_eq!(config.zones[1].custom_label(), Some("Bob"));
        }
        if config.zones.len() > 2 {
            assert_eq!(config.zones[2].custom_label(), None); // Third zone should have no label
        }
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

    #[test]
    fn test_manchester_london_save_load() {
        // Create a minimal config with just London and Manchester
        let config = AppConfig {
            zones: vec![
                crate::config::ZoneConfigCompat::Simple("London".to_string()),
                crate::config::ZoneConfigCompat::Simple("Manchester".to_string()),
            ],
            selected_zone_index: 0,
            display_format: TimeFormat::TwentyFourHour,
            timezone_display_mode: TimezoneDisplayMode::Short,
            time_config: crate::config::TimeDisplayConfig::default(),
            color_theme: crate::config::ColorTheme::default(),
            show_date: false,
            show_sun_times: true,
        };

        // Create app from config
        let app = App::from_config(config);

        // Verify both cities were loaded
        assert_eq!(app.timezone_manager.zone_count(), 2);

        // Convert back to config
        let saved_config = app.to_config();

        // Verify both cities are still there with correct names
        assert_eq!(saved_config.zones.len(), 2);

        // Check that we have both London and Manchester
        let city_names: Vec<String> = saved_config
            .zones
            .iter()
            .map(|z| z.city_name().to_string())
            .collect();

        assert!(
            city_names.contains(&"London".to_string()),
            "London should be preserved"
        );
        assert!(
            city_names.contains(&"Manchester".to_string()),
            "Manchester should be preserved"
        );
    }

    #[test]
    fn test_manchester_with_default_london() {
        // Start with default app (which includes London)
        let mut app = App::new();

        // Verify London is in defaults
        let initial_count = app.timezone_manager.zone_count();
        assert!(initial_count > 0, "Should have default zones");

        // Add Manchester
        let added = app.timezone_manager.add_timezone_by_name("Manchester");
        assert!(added, "Should be able to add Manchester");
        assert_eq!(
            app.timezone_manager.zone_count(),
            initial_count + 1,
            "Should have one more zone"
        );

        // Save to config
        let config = app.to_config();

        // Reload from config
        let reloaded_app = App::from_config(config);

        // Verify both cities are present
        let zones = reloaded_app.timezone_manager.zones();
        let display_names: Vec<&str> = zones.iter().map(|z| z.display_name.as_str()).collect();
        let source_cities: Vec<String> =
            zones.iter().filter_map(|z| z.source_city.clone()).collect();

        println!("Display names: {:?}", display_names);
        println!("Source cities: {:?}", source_cities);

        // Should have both LON and MAN
        assert!(display_names.contains(&"LON"), "Should have London (LON)");
        assert!(
            display_names.contains(&"MAN"),
            "Should have Manchester (MAN)"
        );

        // Source cities should include Manchester
        assert!(
            source_cities.contains(&"Manchester".to_string()),
            "Manchester source city should be preserved"
        );
    }

    #[test]
    fn test_search_navigation() {
        let mut app = App::new();
        
        // Start adding zone
        app.update(Message::StartAddZone);
        assert!(app.adding_zone);
        assert_eq!(app.selected_search_result, 0);
        
        // Add some search input to get results
        app.update(Message::UpdateAddZoneInput("London".to_string()));
        assert!(!app.zone_search_results.is_empty());
        assert_eq!(app.selected_search_result, 0);
        
        println!("Search results: {:?}", app.zone_search_results);
        println!("Number of results: {}", app.zone_search_results.len());
        
        // Navigate down
        app.update(Message::NavigateSearchResults(Direction::Down));
        assert_eq!(app.selected_search_result, 1);
        
        // Navigate down again if possible
        if app.zone_search_results.len() > 2 {
            app.update(Message::NavigateSearchResults(Direction::Down));
            assert_eq!(app.selected_search_result, 2);
        }
        
        // Navigate up
        app.update(Message::NavigateSearchResults(Direction::Up));
        if app.zone_search_results.len() > 2 {
            assert_eq!(app.selected_search_result, 1);
        } else {
            assert_eq!(app.selected_search_result, 0);
        }
        
        // Navigate up again
        app.update(Message::NavigateSearchResults(Direction::Up));
        assert_eq!(app.selected_search_result, 0);
        
        // Try to navigate up when already at top (should stay at 0)
        app.update(Message::NavigateSearchResults(Direction::Up));
        assert_eq!(app.selected_search_result, 0);
        
        // Navigate to bottom
        let max_index = app.zone_search_results.len() - 1;
        for _ in 0..app.zone_search_results.len() {
            app.update(Message::NavigateSearchResults(Direction::Down));
        }
        assert_eq!(app.selected_search_result, max_index);
        
        // Try to navigate down when already at bottom (should stay at max)
        app.update(Message::NavigateSearchResults(Direction::Down));
        assert_eq!(app.selected_search_result, max_index);
    }

    #[test]
    fn test_search_navigation_detailed() {
        let mut app = App::new();
        
        // Start adding zone
        app.update(Message::StartAddZone);
        println!("After StartAddZone: adding_zone={}, selected_search_result={}", app.adding_zone, app.selected_search_result);
        
        // Add search input
        app.update(Message::UpdateAddZoneInput("Lon".to_string()));
        println!("After search 'Lon': {} results, selected={}", app.zone_search_results.len(), app.selected_search_result);
        println!("Results: {:?}", app.zone_search_results);
        
        // Try navigation
        app.update(Message::NavigateSearchResults(Direction::Down));
        println!("After Down: selected={}", app.selected_search_result);
        
        app.update(Message::NavigateSearchResults(Direction::Down));
        println!("After Down again: selected={}", app.selected_search_result);
        
        app.update(Message::NavigateSearchResults(Direction::Up));
        println!("After Up: selected={}", app.selected_search_result);
        
        // Verify basic functionality
        assert!(app.adding_zone);
        assert!(!app.zone_search_results.is_empty());
    }
}
