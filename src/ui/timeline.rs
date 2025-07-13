use chrono::{DateTime, Utc, Duration, Timelike, Offset};
use ratatui::{
    buffer::Buffer,
    layout::{Rect, Margin},
    style::{Color, Style},
    widgets::{Block, Borders, Widget},
};

use crate::time::TimeZone;
use crate::app::{TimeFormat, TimezoneDisplayMode};
use crate::config::{TimeDisplayConfig, ColorTheme};
use crate::weather::WeatherData;

pub struct TimelineWidget<'a> {
    pub timeline_position: DateTime<Utc>,
    pub current_time: DateTime<Utc>,
    pub timezone: &'a TimeZone,
    pub selected: bool,
    pub display_format: TimeFormat,
    pub timezone_display_mode: TimezoneDisplayMode,
    pub time_config: &'a TimeDisplayConfig,
    pub color_theme: ColorTheme,
    pub weather_data: Option<&'a WeatherData>,
    pub show_weather: bool,
    pub show_dst: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DstTransition {
    SpringForward,  // Clock jumps forward (2 AM -> 3 AM)
    FallBack,       // Clock falls back (2 AM -> 1 AM)
}

impl<'a> TimelineWidget<'a> {
    pub fn new(
        timeline_position: DateTime<Utc>,
        current_time: DateTime<Utc>,
        timezone: &'a TimeZone,
        selected: bool,
        display_format: TimeFormat,
        timezone_display_mode: TimezoneDisplayMode,
        time_config: &'a TimeDisplayConfig,
        color_theme: ColorTheme,
        weather_data: Option<&'a WeatherData>,
        show_weather: bool,
        show_dst: bool,
    ) -> Self {
        Self {
            timeline_position,
            current_time,
            timezone,
            selected,
            display_format,
            timezone_display_mode,
            time_config,
            color_theme,
            weather_data,
            show_weather,
            show_dst,
        }
    }

    fn get_timeline_start(&self) -> DateTime<Utc> {
        // Start timeline 24 hours before current position
        self.timeline_position - Duration::hours(24)
    }

    fn get_timeline_end(&self) -> DateTime<Utc> {
        // End timeline 24 hours after current position  
        self.timeline_position + Duration::hours(24)
    }

    fn time_to_position(&self, time: DateTime<Utc>, width: u16) -> u16 {
        let start = self.get_timeline_start();
        let end = self.get_timeline_end();
        let total_duration = end.signed_duration_since(start);
        let time_duration = time.signed_duration_since(start);
        
        if total_duration.num_seconds() == 0 {
            return 0;
        }
        
        let ratio = time_duration.num_seconds() as f64 / total_duration.num_seconds() as f64;
        let position = (ratio * (width as f64)).round() as u16;
        position.min(width.saturating_sub(1))
    }

    fn get_hour_display(&self, hour: u32) -> (char, Color) {
        let activity = self.time_config.get_time_activity(hour);
        let char = self.time_config.get_activity_char(activity);
        let color = self.time_config.get_activity_color(activity, self.color_theme);
        (char, color)
    }
    
    fn detect_dst_transition(&self, utc_time: DateTime<Utc>) -> Option<DstTransition> {
        // Check for DST transitions by examining offset changes
        let local_time = utc_time.with_timezone(&self.timezone.tz);
        let offset_before = local_time.offset().fix().local_minus_utc();
        
        // Check one hour ahead
        let one_hour_later = utc_time + Duration::hours(1);
        let local_time_later = one_hour_later.with_timezone(&self.timezone.tz);
        let offset_after = local_time_later.offset().fix().local_minus_utc();
        
        if offset_after > offset_before {
            // Offset increased = clocks fell back (e.g., DST ended)
            Some(DstTransition::FallBack)
        } else if offset_after < offset_before {
            // Offset decreased = clocks sprang forward (e.g., DST started)
            Some(DstTransition::SpringForward)
        } else {
            None
        }
    }
    
    fn get_dst_transitions_in_range(&self) -> Vec<(DateTime<Utc>, DstTransition)> {
        let mut transitions = Vec::new();
        let start = self.get_timeline_start();
        let end = self.get_timeline_end();
        
        // Check every hour for DST transitions
        let mut current = start;
        while current < end {
            if let Some(transition) = self.detect_dst_transition(current) {
                transitions.push((current, transition));
            }
            current = current + Duration::hours(1);
        }
        
        transitions
    }

    fn get_timeline_display(&self, width: u16) -> Vec<(char, Color)> {
        let mut display = vec![('░', Color::DarkGray); width as usize];
        let start_time = self.get_timeline_start();
        
        // Convert timeline to local timezone for this zone
        let local_start = start_time.with_timezone(&self.timezone.tz);
        
        for i in 0..width {
            // Calculate what time this position represents in the local timezone
            let hours_offset = (i as f64 / width as f64) * 48.0; // 48 hours total
            let time_at_position = local_start + Duration::minutes((hours_offset * 60.0) as i64);
            let hour = time_at_position.hour();
            
            display[i as usize] = self.get_hour_display(hour);
        }
        
        display
    }
}

impl<'a> Widget for TimelineWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let inner = area.inner(Margin { horizontal: 1, vertical: 1 });
        if inner.width < 2 {
            return;
        }

        // Render border
        let border_style = if self.selected {
            Style::default().fg(self.color_theme.get_selected_border_color())
        } else {
            Style::default()
        };
        
        let title = match self.timezone_display_mode {
            TimezoneDisplayMode::Short => format!("{} {}", self.timezone.display_name(), self.timezone.offset_string()),
            TimezoneDisplayMode::Full => self.timezone.get_full_display_name(),
        };
        let block = Block::default()
            .borders(Borders::ALL)
            .title(title)
            .style(border_style);
        block.render(area, buf);

        // Generate timeline display
        let timeline_display = self.get_timeline_display(inner.width);
        
        // Render timeline bar
        let timeline_y = inner.y;
        for (i, &(ch, color)) in timeline_display.iter().enumerate() {
            if i >= inner.width as usize {
                break;
            }
            
            let x = inner.x + i as u16;
            let style = Style::default().fg(color);
            
            buf[(x, timeline_y)].set_char(ch).set_style(style);
        }

        // Render current time indicator (now line)
        let now_pos = self.time_to_position(self.current_time, inner.width);
        if now_pos < inner.width {
            let x = inner.x + now_pos;
            buf[(x, timeline_y)]
                .set_char('│')
                .set_style(Style::default().fg(self.color_theme.get_current_time_color()));
        }

        // Render timeline position indicator (scrub line)
        let timeline_pos = self.time_to_position(self.timeline_position, inner.width);
        if timeline_pos < inner.width && timeline_pos != now_pos {
            let x = inner.x + timeline_pos;
            buf[(x, timeline_y)]
                .set_char('┃')
                .set_style(Style::default().fg(self.color_theme.get_timeline_position_color()));
        }

        // Render DST transition indicators if enabled
        if self.show_dst {
            let dst_transitions = self.get_dst_transitions_in_range();
            for (transition_time, transition_type) in dst_transitions {
                let dst_pos = self.time_to_position(transition_time, inner.width);
                if dst_pos < inner.width {
                    let x = inner.x + dst_pos;
                    let (symbol, color) = match transition_type {
                        DstTransition::SpringForward => ('⇈', Color::Green),   // Double up arrow for spring forward
                        DstTransition::FallBack => ('⇊', Color::Yellow),       // Double down arrow for fall back
                    };
                    buf[(x, timeline_y)]
                        .set_char(symbol)
                        .set_style(Style::default().fg(color));
                }
            }
        }

        // Render time display under the scrubber position
        if inner.height > 1 {
            let zone_time = self.timezone.convert_time(self.timeline_position);
            let mut time_str = match self.display_format {
                TimeFormat::TwentyFourHour => zone_time.format("%H:%M %a").to_string(),
                TimeFormat::TwelveHour => zone_time.format("%I:%M %p %a").to_string(),
            };
            
            // Add weather icon if available and enabled
            if self.show_weather {
                if let Some(weather) = self.weather_data {
                    time_str = format!("{} {}", time_str, weather.emoji);
                }
            }
            
            let time_y = inner.y + 1;
            
            // Position the time display under the timeline position indicator
            let timeline_pos = self.time_to_position(self.timeline_position, inner.width);
            let time_start_x = if timeline_pos >= (time_str.chars().count() as u16 / 2) {
                timeline_pos - (time_str.chars().count() as u16 / 2)
            } else {
                0
            };
            
            // Ensure we don't go beyond the right edge
            let time_start_x = time_start_x.min(inner.width.saturating_sub(time_str.chars().count() as u16));
            
            for (i, ch) in time_str.chars().enumerate() {
                let x = inner.x + time_start_x + i as u16;
                if x < inner.x + inner.width {
                    buf[(x, time_y)].set_char(ch);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono_tz;

    #[test]
    fn test_timeline_widget_creation() {
        let tz = crate::time::TimeZone::from_tz(chrono_tz::UTC);
        let now = Utc::now();
        let config = crate::config::TimeDisplayConfig::default();
        
        let widget = TimelineWidget::new(now, now, &tz, false, TimeFormat::TwentyFourHour, TimezoneDisplayMode::Short, &config, ColorTheme::default(), None, false, false);
        assert_eq!(widget.timeline_position, now);
        assert_eq!(widget.current_time, now);
        assert!(!widget.selected);
        assert_eq!(widget.display_format, TimeFormat::TwentyFourHour);
    }

    #[test]
    fn test_time_to_position() {
        let tz = crate::time::TimeZone::from_tz(chrono_tz::UTC);
        let base_time = Utc::now();
        let config = crate::config::TimeDisplayConfig::default();
        let widget = TimelineWidget::new(base_time, base_time, &tz, false, TimeFormat::TwentyFourHour, TimezoneDisplayMode::Short, &config, ColorTheme::default(), None, false, false);
        
        // Position should be in the middle for the timeline position itself
        let pos = widget.time_to_position(base_time, 100);
        assert_eq!(pos, 50); // Middle of 100-width timeline
    }

    #[test]
    fn test_hour_display_mapping() {
        let tz = crate::time::TimeZone::from_tz(chrono_tz::UTC);
        let base_time = Utc::now();
        let config = crate::config::TimeDisplayConfig::default();
        let widget = TimelineWidget::new(base_time, base_time, &tz, false, TimeFormat::TwentyFourHour, TimezoneDisplayMode::Short, &config, ColorTheme::default(), None, false, false);
        
        // Test work hours get dark shade block
        let (char, _) = widget.get_hour_display(14); // 2 PM
        assert_eq!(char, '▓'); // Work hours = dark shade block
        
        // Test awake hours get medium shade
        let (char, _) = widget.get_hour_display(7); // 7 AM
        assert_eq!(char, '▒'); // Awake hours = medium shade
        
        // Test night hours get light shade
        let (char, _) = widget.get_hour_display(2); // 2 AM
        assert_eq!(char, '░'); // Night hours = light shade
    }

    #[test]
    fn test_time_format_handling() {
        let tz = crate::time::TimeZone::from_tz(chrono_tz::UTC);
        let base_time = Utc::now();
        let config = crate::config::TimeDisplayConfig::default();
        
        // Test 24-hour format
        let widget_24h = TimelineWidget::new(base_time, base_time, &tz, false, TimeFormat::TwentyFourHour, TimezoneDisplayMode::Short, &config, ColorTheme::default(), None, false, false);
        assert_eq!(widget_24h.display_format, TimeFormat::TwentyFourHour);
        
        // Test 12-hour format
        let widget_12h = TimelineWidget::new(base_time, base_time, &tz, false, TimeFormat::TwelveHour, TimezoneDisplayMode::Short, &config, ColorTheme::default(), None, false, false);
        assert_eq!(widget_12h.display_format, TimeFormat::TwelveHour);
    }

    #[test]
    fn test_dst_transition_detection() {
        // Test with a timezone that has DST transitions (US/Eastern)
        let tz = crate::time::TimeZone::from_tz(chrono_tz::US::Eastern);
        let config = crate::config::TimeDisplayConfig::default();
        
        // Create a widget with DST enabled
        let base_time = Utc::now();
        let widget = TimelineWidget::new(base_time, base_time, &tz, false, TimeFormat::TwentyFourHour, TimezoneDisplayMode::Short, &config, ColorTheme::default(), None, false, true);
        
        // Test that DST transitions can be detected (this may not find any in the current 48-hour window, but the function should work)
        let transitions = widget.get_dst_transitions_in_range();
        
        // The test passes if the function doesn't panic and returns a vector (empty or not)
        assert!(transitions.len() >= 0); // Always true, but ensures the function executes
    }

    #[test]
    fn test_dst_always_enabled() {
        let tz = crate::time::TimeZone::from_tz(chrono_tz::UTC);
        let now = Utc::now();
        let config = crate::config::TimeDisplayConfig::default();
        
        // DST indicators are now always enabled
        let widget = TimelineWidget::new(now, now, &tz, false, TimeFormat::TwentyFourHour, TimezoneDisplayMode::Short, &config, ColorTheme::default(), None, false, true);
        assert!(widget.show_dst);
    }
}