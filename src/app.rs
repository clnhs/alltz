use chrono::{DateTime, Utc, Local, Offset, Timelike};
use ratatui::{
    layout::{Alignment, Constraint, Direction as LayoutDirection, Layout, Rect},
    style::{Modifier, Style, Color},
    widgets::{Block, Borders, Clear, Paragraph, Table, Row, Cell, TableState, Wrap},
    Frame,
};

use crate::time::{TimeZone, TimeZoneManager};
use crate::ui::TimelineWidget;
use crate::config::{TimeDisplayConfig, AppConfig, ColorTheme};

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
    ToggleDate,
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
    pub show_date: bool,

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
            show_date: false,
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
                    eprintln!("Warning: Could not create default config file at {}", config_path.display());
                }
            }
        }

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
        let selected_zone_index = config.selected_zone_index.min(timezone_manager.zone_count().saturating_sub(1));

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
            show_date: config.show_date,
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
            show_date: self.show_date,
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
                            self.selected_zone_index = self.timezone_manager.zone_count().saturating_sub(1);
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
                    if let Some(zone_name) = self.zone_search_results.get(self.selected_search_result) {
                        let success = self.timezone_manager.add_timezone_by_name(zone_name);

                        if success {
                            // Update selected index if needed
                            if self.selected_zone_index >= self.timezone_manager.zone_count() {
                                self.selected_zone_index = self.timezone_manager.zone_count().saturating_sub(1);
                            }
                            self.save_config();
                        }
                    }
                } else if !self.add_zone_input.is_empty() {
                    // Try to add the exact input if no search results
                    let success = self.timezone_manager.add_timezone_by_name(&self.add_zone_input);

                    if success {
                        // Update selected index if needed
                        if self.selected_zone_index >= self.timezone_manager.zone_count() {
                            self.selected_zone_index = self.timezone_manager.zone_count().saturating_sub(1);
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
                if self.timezone_manager.zone_count() > 1 { // Keep at least one zone
                    self.timezone_manager.remove_zone(self.selected_zone_index);

                    // Adjust selected index if needed
                    if self.selected_zone_index >= self.timezone_manager.zone_count() {
                        self.selected_zone_index = self.timezone_manager.zone_count().saturating_sub(1);
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

        let chunks = Layout::default()
            .direction(LayoutDirection::Horizontal)
            .constraints([
                Constraint::Percentage(33),  // App name (left)
                Constraint::Percentage(34),  // Local time (center)
                Constraint::Percentage(33),  // Timeline (right)
            ])
            .split(inner);

        // Left: App name
        let app_name = Paragraph::new("alltz v0.1.0")
            .alignment(Alignment::Left);
        f.render_widget(app_name, chunks[0]);

        // Center: Local time
        let local_display = Paragraph::new(format!("Local: {}", local_time_str))
            .alignment(Alignment::Center);
        f.render_widget(local_display, chunks[1]);

        // Right: Timeline time
        let timeline_display = Paragraph::new(format!("Timeline: {}", timeline_time_str))
            .alignment(Alignment::Right);
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
        let inner = popup_area.inner(ratatui::layout::Margin { horizontal: 1, vertical: 1 });
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
            .style(Style::default()
                .fg(self.color_theme.get_selected_border_color())
                .add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center);
        f.render_widget(title, chunks[0]);

        // Split content into two columns for better space usage
        let content_chunks = Layout::default()
            .direction(LayoutDirection::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(chunks[1]);

        // Left column content
        let left_sections = vec![
            ("TIME NAVIGATION", vec![
                "h/← or l/→     Scrub timeline (1 hour)",
                "Shift + h/l    Fine scrub (1 minute)",
                "[ or ]         Adjust by ±15 minutes",
                "{ or }         Adjust by ±1 hour",
                "t              Reset to current time",
            ]),
            ("ZONE NAVIGATION", vec![
                "j/↓ or k/↑     Navigate between zones",
                "               Selected has colored border",
            ]),
            ("DISPLAY OPTIONS", vec![
                "m              Toggle 12/24 hour format",
                "n              Toggle short/full names",
                "d              Toggle date display",
                "c              Cycle color themes",
            ]),
        ];

        // Right column content
        let right_sections = vec![
            ("ZONE MANAGEMENT", vec![
                "a              Add new timezone",
                "r              Remove selected timezone",
                "1-8            Quick-select search results",
            ]),
            ("INDICATORS", vec![
                "│              Red line: Current time",
                "┃              Colored line: Timeline position",
                "⇈              DST spring forward",
                "⇊              DST fall back",
                "░ ▒ ▓          Night, Awake, Work hours",
            ]),
            ("CONTROLS", vec![
                "?              Show/hide help",
                "q              Quit",
                "Esc            Cancel operation",
            ]),
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
        let inner = popup_area.inner(ratatui::layout::Margin { horizontal: 1, vertical: 1 });
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
            .title_style(ratatui::style::Style::default().fg(ratatui::style::Color::Green).add_modifier(Modifier::BOLD))
            .border_style(ratatui::style::Style::default().fg(ratatui::style::Color::Green))
            .style(ratatui::style::Style::default().bg(ratatui::style::Color::Black));
        f.render_widget(border, popup_area);
    }

    fn render_search_results_table(&self, f: &mut Frame, area: Rect) {
        let mut rows = Vec::new();

        for (i, result) in self.zone_search_results.iter().enumerate() {
            if let Some((city_country, time_str, offset, code)) = self.get_search_result_parts(result) {
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
            ]
        )
        .header(Row::new(vec![
            Cell::from("#").style(ratatui::style::Style::default().add_modifier(Modifier::BOLD)),
            Cell::from("City, Country").style(ratatui::style::Style::default().add_modifier(Modifier::BOLD)),
            Cell::from("Time").style(ratatui::style::Style::default().add_modifier(Modifier::BOLD)),
            Cell::from("Offset").style(ratatui::style::Style::default().add_modifier(Modifier::BOLD)),
            Cell::from("Code").style(ratatui::style::Style::default().add_modifier(Modifier::BOLD)),
        ]))
        .style(ratatui::style::Style::default().fg(ratatui::style::Color::White))
        .row_highlight_style(ratatui::style::Style::default()
            .fg(self.color_theme.get_work_color())
            .add_modifier(Modifier::BOLD))
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

            Some((city_country, time_str, timezone.offset_string(), display_name.clone()))
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
        assert!(matches!(app.display_format, TimeFormat::TwentyFourHour | TimeFormat::TwelveHour));
        assert!(matches!(app.timezone_display_mode, TimezoneDisplayMode::Short | TimezoneDisplayMode::Full));
        assert!(!app.show_help);
        assert!(!app.adding_zone);
        assert!(app.add_zone_input.is_empty());
        assert!(app.zone_search_results.is_empty());
        assert_eq!(app.selected_search_result, 0);
        assert!(!app.show_date); // Default is false
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