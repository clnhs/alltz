use chrono::{DateTime, Utc, Duration, Timelike};
use ratatui::{
    buffer::Buffer,
    layout::{Rect, Margin},
    style::{Color, Style},
    widgets::{Block, Borders, Widget},
};

use crate::time::TimeZone;
use crate::app::TimeFormat;
use crate::config::TimeDisplayConfig;

pub struct TimelineWidget<'a> {
    pub timeline_position: DateTime<Utc>,
    pub current_time: DateTime<Utc>,
    pub timezone: &'a TimeZone,
    pub selected: bool,
    pub display_format: TimeFormat,
    pub time_config: &'a TimeDisplayConfig,
}

impl<'a> TimelineWidget<'a> {
    pub fn new(
        timeline_position: DateTime<Utc>,
        current_time: DateTime<Utc>,
        timezone: &'a TimeZone,
        selected: bool,
        display_format: TimeFormat,
        time_config: &'a TimeDisplayConfig,
    ) -> Self {
        Self {
            timeline_position,
            current_time,
            timezone,
            selected,
            display_format,
            time_config,
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
        let color = self.time_config.get_activity_color(activity);
        (char, color)
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
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        };
        
        let title = format!("{} {}", self.timezone.display_name(), self.timezone.offset_string());
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
                .set_style(Style::default().fg(Color::Red));
        }

        // Render timeline position indicator (scrub line)
        let timeline_pos = self.time_to_position(self.timeline_position, inner.width);
        if timeline_pos < inner.width && timeline_pos != now_pos {
            let x = inner.x + timeline_pos;
            buf[(x, timeline_y)]
                .set_char('┃')
                .set_style(Style::default().fg(Color::Magenta));
        }

        // Render time display under the scrubber position
        if inner.height > 1 {
            let zone_time = self.timezone.convert_time(self.timeline_position);
            let time_str = match self.display_format {
                TimeFormat::TwentyFourHour => zone_time.format("%H:%M %a").to_string(),
                TimeFormat::TwelveHour => zone_time.format("%I:%M %p %a").to_string(),
            };
            let time_y = inner.y + 1;
            
            // Position the time display under the timeline position indicator
            let timeline_pos = self.time_to_position(self.timeline_position, inner.width);
            let time_start_x = if timeline_pos >= (time_str.len() as u16 / 2) {
                timeline_pos - (time_str.len() as u16 / 2)
            } else {
                0
            };
            
            // Ensure we don't go beyond the right edge
            let time_start_x = time_start_x.min(inner.width.saturating_sub(time_str.len() as u16));
            
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
        
        let widget = TimelineWidget::new(now, now, &tz, false, TimeFormat::TwentyFourHour, &config);
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
        let widget = TimelineWidget::new(base_time, base_time, &tz, false, TimeFormat::TwentyFourHour, &config);
        
        // Position should be in the middle for the timeline position itself
        let pos = widget.time_to_position(base_time, 100);
        assert_eq!(pos, 50); // Middle of 100-width timeline
    }

    #[test]
    fn test_hour_display_mapping() {
        let tz = crate::time::TimeZone::from_tz(chrono_tz::UTC);
        let base_time = Utc::now();
        let config = crate::config::TimeDisplayConfig::default();
        let widget = TimelineWidget::new(base_time, base_time, &tz, false, TimeFormat::TwentyFourHour, &config);
        
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
        let widget_24h = TimelineWidget::new(base_time, base_time, &tz, false, TimeFormat::TwentyFourHour, &config);
        assert_eq!(widget_24h.display_format, TimeFormat::TwentyFourHour);
        
        // Test 12-hour format
        let widget_12h = TimelineWidget::new(base_time, base_time, &tz, false, TimeFormat::TwelveHour, &config);
        assert_eq!(widget_12h.display_format, TimeFormat::TwelveHour);
    }
}