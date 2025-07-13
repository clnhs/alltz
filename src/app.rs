use chrono::{DateTime, Utc};
use ratatui::{
    layout::{Constraint, Direction as LayoutDirection, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::time::{TimeZone, TimeZoneManager};

#[derive(Debug, Clone, PartialEq)]
pub enum TimeFormat {
    TwentyFourHour,
    TwelveHour,
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
    ResetToNow,
    FineAdjust(i32), // minutes
    
    // Zone navigation
    NavigateZone(Direction),
    
    // Display options
    ToggleTimeFormat,
    
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
            should_quit: false,
        }
    }
}

impl App {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn update(&mut self, msg: Message) -> Option<Message> {
        match msg {
            Message::Tick => {
                self.current_time = Utc::now();
                None
            }
            
            Message::ScrubTimeline(direction) => {
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
                }
                None
            }
            
            Message::ToggleTimeFormat => {
                self.display_format = match self.display_format {
                    TimeFormat::TwentyFourHour => TimeFormat::TwelveHour,
                    TimeFormat::TwelveHour => TimeFormat::TwentyFourHour,
                };
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
                Constraint::Min(1),    // Main content
                Constraint::Length(3), // Footer
            ])
            .split(f.area());
        
        self.render_header(f, chunks[0]);
        self.render_zones(f, chunks[1]);
        self.render_footer(f, chunks[2]);
    }
    
    fn render_header(&self, f: &mut Frame, area: Rect) {
        let current_time_str = self.current_time.format("%H:%M:%S UTC").to_string();
        let timeline_time_str = match self.display_format {
            TimeFormat::TwentyFourHour => self.timeline_position.format("%H:%M").to_string(),
            TimeFormat::TwelveHour => self.timeline_position.format("%I:%M %p").to_string(),
        };
        
        let header_text = format!(
            "alltz v0.1.0 │ Current: {} │ Timeline: {} │ [q] Quit [?] Help",
            current_time_str, timeline_time_str
        );
        
        let header = Paragraph::new(header_text)
            .block(Block::default().borders(Borders::ALL).title("alltz"));
        
        f.render_widget(header, area);
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
            .map(|_| Constraint::Length(2))
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
        let zone_time = zone.convert_time(self.timeline_position);
        
        let time_str = match self.display_format {
            TimeFormat::TwentyFourHour => zone_time.format("%H:%M").to_string(),
            TimeFormat::TwelveHour => zone_time.format("%I:%M %p").to_string(),
        };
        
        let date_str = zone_time.format("%a %b %d").to_string();
        
        let content = format!(
            "{} │ {} │ {} {}",
            zone.display_name(),
            zone.offset_string(),
            time_str,
            date_str
        );
        
        let style = if is_selected {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        };
        
        let widget = Paragraph::new(content)
            .style(style)
            .block(Block::default().borders(Borders::ALL));
        
        f.render_widget(widget, area);
    }
    
    fn render_footer(&self, f: &mut Frame, area: Rect) {
        let footer_text = "j/k: navigate zones │ h/l: scrub timeline │ t: reset to now │ m: toggle format";
        
        let footer = Paragraph::new(footer_text)
            .block(Block::default().borders(Borders::ALL).title("Controls"));
        
        f.render_widget(footer, area);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_app_creation() {
        let app = App::new();
        assert!(!app.should_quit);
        assert_eq!(app.selected_zone_index, 0);
        assert_eq!(app.display_format, TimeFormat::TwentyFourHour);
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
        
        app.update(Message::ScrubTimeline(Direction::Right));
        assert!(app.timeline_position > initial_time);
        
        app.update(Message::ScrubTimeline(Direction::Left));
        assert_eq!(app.timeline_position, initial_time);
    }
}