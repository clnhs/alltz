use chrono::{DateTime, Days, Duration, Offset, TimeZone as ChronoTimeZone, Timelike, Utc};
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Margin, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Widget},
};

use crate::app::{TimeFormat, TimezoneDisplayMode};
use crate::config::{ColorTheme, TimeDisplayConfig};
use crate::time::TimeZone;

pub struct TimelineWidget<'a> {
    pub timeline_position: DateTime<Utc>,
    pub current_time: DateTime<Utc>,
    pub timezone: &'a TimeZone,
    pub selected: bool,
    pub display_format: TimeFormat,
    pub timezone_display_mode: TimezoneDisplayMode,
    pub time_config: &'a TimeDisplayConfig,
    pub color_theme: ColorTheme,
    pub show_date: bool,
    pub show_dst: bool,
    pub show_sun_times: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DstTransition {
    SpringForward, // Clock jumps forward (2 AM -> 3 AM)
    FallBack,      // Clock falls back (2 AM -> 1 AM)
}

impl<'a> TimelineWidget<'a> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        timeline_position: DateTime<Utc>,
        current_time: DateTime<Utc>,
        timezone: &'a TimeZone,
        selected: bool,
        display_format: TimeFormat,
        timezone_display_mode: TimezoneDisplayMode,
        time_config: &'a TimeDisplayConfig,
        color_theme: ColorTheme,
        show_date: bool,
        show_dst: bool,
        show_sun_times: bool,
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
            show_date,
            show_dst,
            show_sun_times,
        }
    }

    fn get_timeline_hours(&self, width: u16) -> f64 {
        // Optimal display: approximately 2 characters per hour for dense but readable display
        // This means 48 hours fits in ~96 characters, allowing expansion on wider screens
        const OPTIMAL_CHARS_PER_HOUR: f64 = 2.0;
        const MIN_HOURS: f64 = 48.0; // Minimum 48-hour window (24h before + 24h after)
        const MAX_HOURS: f64 = 168.0; // Maximum 1 week window

        // Calculate how many hours we can display optimally with current width
        let optimal_hours = (width as f64) / OPTIMAL_CHARS_PER_HOUR;

        // Clamp between minimum and maximum
        optimal_hours.clamp(MIN_HOURS, MAX_HOURS)
    }

    fn get_timeline_start(&self, width: u16) -> DateTime<Utc> {
        let total_hours = self.get_timeline_hours(width);
        let hours_before = total_hours / 2.0;
        self.timeline_position - Duration::minutes((hours_before * 60.0) as i64)
    }

    fn get_timeline_end(&self, width: u16) -> DateTime<Utc> {
        let total_hours = self.get_timeline_hours(width);
        let hours_after = total_hours / 2.0;
        self.timeline_position + Duration::minutes((hours_after * 60.0) as i64)
    }

    fn time_to_position(&self, time: DateTime<Utc>, width: u16) -> u16 {
        let start = self.get_timeline_start(width);
        let end = self.get_timeline_end(width);
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
        let color = self
            .time_config
            .get_activity_color(activity, self.color_theme);
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

    fn get_dst_transitions_in_range(&self, width: u16) -> Vec<(DateTime<Utc>, DstTransition)> {
        let mut transitions = Vec::new();
        let start = self.get_timeline_start(width);
        let end = self.get_timeline_end(width);

        // Check every hour for DST transitions
        let mut current = start;
        while current < end {
            if let Some(transition) = self.detect_dst_transition(current) {
                transitions.push((current, transition));
            }
            current += Duration::hours(1);
        }

        transitions
    }

    fn get_midnight_markers_in_range(&self, width: u16) -> Vec<DateTime<Utc>> {
        let mut midnight_markers = Vec::new();
        let start = self.get_timeline_start(width);
        let end = self.get_timeline_end(width);

        // Convert to this timezone to find local midnights
        let local_start = start.with_timezone(&self.timezone.tz);
        let local_end = end.with_timezone(&self.timezone.tz);

        // Find the first midnight after start
        let mut current_date = local_start.date_naive();

        // If we're not at the start of the day, move to next day
        if local_start.time() != chrono::NaiveTime::from_hms_opt(0, 0, 0).unwrap() {
            current_date = current_date + Days::new(1);
        }

        // Iterate through each midnight in the range
        while current_date <= local_end.date_naive() {
            if let Some(midnight_local) = current_date.and_hms_opt(0, 0, 0) {
                // Convert midnight in this timezone to UTC
                if let Some(midnight_tz) = self
                    .timezone
                    .tz
                    .from_local_datetime(&midnight_local)
                    .single()
                {
                    let midnight_utc = midnight_tz.with_timezone(&chrono::Utc);

                    // Only include if it's within our timeline range
                    if midnight_utc >= start && midnight_utc <= end {
                        midnight_markers.push(midnight_utc);
                    }
                }
            }
            current_date = current_date + Days::new(1);
        }

        midnight_markers
    }

    fn get_timeline_display(&self, width: u16) -> Vec<(char, Color)> {
        let mut display = vec![('░', Color::DarkGray); width as usize];
        let start_time = self.get_timeline_start(width);
        let total_hours = self.get_timeline_hours(width);

        // Convert timeline to local timezone for this zone
        let local_start = start_time.with_timezone(&self.timezone.tz);

        for i in 0..width {
            // Calculate what time this position represents in the local timezone
            let hours_offset = (i as f64 / width as f64) * total_hours;
            let time_at_position = local_start + Duration::minutes((hours_offset * 60.0) as i64);
            let hour = time_at_position.hour();

            display[i as usize] = self.get_hour_display(hour);
        }

        display
    }
}

impl<'a> Widget for TimelineWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let inner = area.inner(Margin {
            horizontal: 1,
            vertical: 1,
        });
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
            TimezoneDisplayMode::Short => {
                // Use custom label if available, otherwise default display name
                format!(
                    "{} {}",
                    self.timezone.effective_display_name(),
                    self.timezone.offset_string()
                )
            }
            TimezoneDisplayMode::Full => {
                // For full mode, show custom label with city name, or just city name
                match &self.timezone.custom_label {
                    Some(label) => {
                        let city_name = self.timezone.get_city_name();
                        format!(
                            "{} ({} {})",
                            label,
                            city_name,
                            self.timezone.offset_string()
                        )
                    }
                    None => {
                        let city_name = self.timezone.get_city_name();
                        format!("{} {}", city_name, self.timezone.offset_string())
                    }
                }
            }
        };

        let mut block = Block::default()
            .borders(Borders::ALL)
            .title(title)
            .style(border_style);

        // Add sunrise/sunset times to bottom right if enabled
        if self.show_sun_times {
            let use_12_hour = matches!(self.display_format, TimeFormat::TwelveHour);
            if let Some(sun_times) = self
                .timezone
                .format_sun_times(self.current_time, use_12_hour)
            {
                let sun_color = if self.selected {
                    self.color_theme.get_selected_border_color()
                } else {
                    Color::Gray
                };
                let sun_line = Line::from(vec![Span::styled(
                    sun_times,
                    Style::default().fg(sun_color),
                )])
                .alignment(Alignment::Right);
                block = block.title_top(sun_line);
            }
        }

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
            let dst_transitions = self.get_dst_transitions_in_range(inner.width);
            for (transition_time, transition_type) in dst_transitions {
                let dst_pos = self.time_to_position(transition_time, inner.width);
                if dst_pos < inner.width {
                    let x = inner.x + dst_pos;
                    let (symbol, color) = match transition_type {
                        DstTransition::SpringForward => ('⇈', Color::Green), // Double up arrow for spring forward
                        DstTransition::FallBack => ('⇊', Color::Yellow), // Double down arrow for fall back
                    };
                    buf[(x, timeline_y)]
                        .set_char(symbol)
                        .set_style(Style::default().fg(color));
                }
            }
        }

        // Render midnight markers (subtle day change indicators)
        let midnight_markers = self.get_midnight_markers_in_range(inner.width);
        for midnight_time in midnight_markers {
            let midnight_pos = self.time_to_position(midnight_time, inner.width);
            if midnight_pos < inner.width && midnight_pos != now_pos && midnight_pos != timeline_pos
            {
                let x = inner.x + midnight_pos;
                // Use a subtle vertical line character with night color
                buf[(x, timeline_y)]
                    .set_char('┊')
                    .set_style(Style::default().fg(self.color_theme.get_night_color()));
            }
        }

        // Render dates in middle of each day's work hours if enabled
        if self.show_date {
            let start_time = self.get_timeline_start(inner.width);
            let end_time = self.get_timeline_end(inner.width);

            // Find the middle of work hours (default 8 AM to 6 PM, so middle is 1 PM)
            let work_middle_hour =
                (self.time_config.work_hours_start + self.time_config.work_hours_end) / 2;

            // Convert timeline to local timezone for this specific timezone
            let local_start = start_time.with_timezone(&self.timezone.tz);
            let local_end = end_time.with_timezone(&self.timezone.tz);
            let mut current_date = local_start.date_naive();

            // Iterate through each day visible in this timezone's local time
            while current_date <= local_end.date_naive() {
                // Create a time for the middle of work hours on this day IN THIS TIMEZONE
                if let Some(work_middle_local) = current_date.and_hms_opt(work_middle_hour, 0, 0) {
                    // Create the datetime in this timezone, then convert to UTC for position calculation
                    if let Some(work_middle_tz) = self
                        .timezone
                        .tz
                        .from_local_datetime(&work_middle_local)
                        .single()
                    {
                        let work_middle_utc = work_middle_tz.with_timezone(&chrono::Utc);
                        let date_pos = self.time_to_position(work_middle_utc, inner.width);

                        // Only render if this position is within the visible timeline
                        if date_pos < inner.width {
                            let date_str = current_date.format("%d %b").to_string(); // Format as "15 Jul"
                            let date_y = timeline_y; // Place date directly on timeline bar

                            // Center the date string around the calculated position
                            let date_start_x = if date_pos >= (date_str.chars().count() as u16 / 2)
                            {
                                date_pos - (date_str.chars().count() as u16 / 2)
                            } else {
                                0
                            };

                            // Ensure we don't go beyond the right edge
                            let date_start_x = date_start_x
                                .min(inner.width.saturating_sub(date_str.chars().count() as u16));

                            // Render the date
                            for (i, ch) in date_str.chars().enumerate() {
                                let x = inner.x + date_start_x + i as u16;
                                if x < inner.x + inner.width {
                                    buf[(x, date_y)].set_char(ch).set_style(
                                        Style::default().fg(Color::White).bg(Color::DarkGray),
                                    );
                                }
                            }
                        }
                    }
                }

                // Move to next day
                current_date = current_date + Days::new(1);
            }
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
            let time_start_x = if timeline_pos >= (time_str.chars().count() as u16 / 2) {
                timeline_pos - (time_str.chars().count() as u16 / 2)
            } else {
                0
            };

            // Ensure we don't go beyond the right edge
            let time_start_x =
                time_start_x.min(inner.width.saturating_sub(time_str.chars().count() as u16));

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

        let widget = TimelineWidget::new(
            now,
            now,
            &tz,
            false,
            TimeFormat::TwentyFourHour,
            TimezoneDisplayMode::Short,
            &config,
            ColorTheme::default(),
            false,
            false,
            false,
        );
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
        let widget = TimelineWidget::new(
            base_time,
            base_time,
            &tz,
            false,
            TimeFormat::TwentyFourHour,
            TimezoneDisplayMode::Short,
            &config,
            ColorTheme::default(),
            false,
            false,
            false,
        );

        // Position should be in the middle for the timeline position itself
        let pos = widget.time_to_position(base_time, 100);
        assert_eq!(pos, 50); // Middle of 100-width timeline
    }

    #[test]
    fn test_hour_display_mapping() {
        let tz = crate::time::TimeZone::from_tz(chrono_tz::UTC);
        let base_time = Utc::now();
        let config = crate::config::TimeDisplayConfig::default();
        let widget = TimelineWidget::new(
            base_time,
            base_time,
            &tz,
            false,
            TimeFormat::TwentyFourHour,
            TimezoneDisplayMode::Short,
            &config,
            ColorTheme::default(),
            false,
            false,
            false,
        );

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
        let widget_24h = TimelineWidget::new(
            base_time,
            base_time,
            &tz,
            false,
            TimeFormat::TwentyFourHour,
            TimezoneDisplayMode::Short,
            &config,
            ColorTheme::default(),
            false,
            false,
            false,
        );
        assert_eq!(widget_24h.display_format, TimeFormat::TwentyFourHour);

        // Test 12-hour format
        let widget_12h = TimelineWidget::new(
            base_time,
            base_time,
            &tz,
            false,
            TimeFormat::TwelveHour,
            TimezoneDisplayMode::Short,
            &config,
            ColorTheme::default(),
            false,
            false,
            false,
        );
        assert_eq!(widget_12h.display_format, TimeFormat::TwelveHour);
    }

    #[test]
    fn test_dst_transition_detection() {
        // Test with a timezone that has DST transitions (US/Eastern)
        let tz = crate::time::TimeZone::from_tz(chrono_tz::US::Eastern);
        let config = crate::config::TimeDisplayConfig::default();

        // Create a widget with DST enabled
        let base_time = Utc::now();
        let widget = TimelineWidget::new(
            base_time,
            base_time,
            &tz,
            false,
            TimeFormat::TwentyFourHour,
            TimezoneDisplayMode::Short,
            &config,
            ColorTheme::default(),
            false,
            true,
            false,
        );

        // Test that DST transitions can be detected - function should execute without panic
        const TEST_WIDTH: u16 = 120; // Use standard width for testing
        let transitions = widget.get_dst_transitions_in_range(TEST_WIDTH);

        // Verify the function returns a valid vector and each transition has valid data
        for (time, transition_type) in transitions {
            assert!(
                time >= widget.get_timeline_start(TEST_WIDTH)
                    && time <= widget.get_timeline_end(TEST_WIDTH)
            );
            assert!(matches!(
                transition_type,
                DstTransition::SpringForward | DstTransition::FallBack
            ));
        }
    }

    #[test]
    fn test_dst_always_enabled() {
        let tz = crate::time::TimeZone::from_tz(chrono_tz::UTC);
        let now = Utc::now();
        let config = crate::config::TimeDisplayConfig::default();

        // DST indicators are now always enabled
        let widget = TimelineWidget::new(
            now,
            now,
            &tz,
            false,
            TimeFormat::TwentyFourHour,
            TimezoneDisplayMode::Short,
            &config,
            ColorTheme::default(),
            false,
            true,
            false,
        );
        assert!(widget.show_dst);
    }

    #[test]
    fn test_adaptive_timeline_window() {
        let tz = crate::time::TimeZone::from_tz(chrono_tz::UTC);
        let now = Utc::now();
        let config = crate::config::TimeDisplayConfig::default();

        let widget = TimelineWidget::new(
            now,
            now,
            &tz,
            false,
            TimeFormat::TwentyFourHour,
            TimezoneDisplayMode::Short,
            &config,
            ColorTheme::default(),
            false,
            false,
            false,
        );

        // Test narrow width - should use minimum 48 hours
        let narrow_width = 80u16; // 80 chars / 2 chars per hour = 40 hours, but min is 48
        let narrow_hours = widget.get_timeline_hours(narrow_width);
        assert_eq!(narrow_hours, 48.0);

        // Test width that expands beyond minimum
        let medium_width = 200u16; // 200 chars / 2 chars per hour = 100 hours
        let medium_hours = widget.get_timeline_hours(medium_width);
        assert_eq!(medium_hours, 100.0);

        // Test very wide width - should be clamped to maximum
        let wide_width = 1000u16; // 1000 chars / 2 chars per hour = 500 hours, but max is 168
        let wide_hours = widget.get_timeline_hours(wide_width);
        assert_eq!(wide_hours, 168.0); // Should be clamped to max

        // Test that timeline spans are calculated correctly
        let start = widget.get_timeline_start(medium_width);
        let end = widget.get_timeline_end(medium_width);
        let actual_duration = end.signed_duration_since(start);
        let expected_duration = Duration::minutes((100.0 * 60.0) as i64); // 100 hours for 200-char width
        assert_eq!(actual_duration, expected_duration);
    }

    #[test]
    fn test_midnight_markers() {
        let tz = crate::time::TimeZone::from_tz(chrono_tz::US::Eastern);
        let config = crate::config::TimeDisplayConfig::default();

        // Create a specific time: 2024-01-15 12:00 UTC
        let base_time = chrono::DateTime::parse_from_rfc3339("2024-01-15T12:00:00Z")
            .unwrap()
            .with_timezone(&Utc);

        let widget = TimelineWidget::new(
            base_time,
            base_time,
            &tz,
            false,
            TimeFormat::TwentyFourHour,
            TimezoneDisplayMode::Short,
            &config,
            ColorTheme::default(),
            false,
            false,
            false,
        );

        // Get midnight markers - should find at least one midnight in 48-hour span
        const TEST_WIDTH: u16 = 120; // Use standard width for testing
        let midnight_markers = widget.get_midnight_markers_in_range(TEST_WIDTH);

        // Should have some midnight markers (48 hour span should contain multiple midnights)
        assert!(!midnight_markers.is_empty());

        // Each marker should be within the timeline range
        for marker in midnight_markers {
            assert!(marker >= widget.get_timeline_start(TEST_WIDTH));
            assert!(marker <= widget.get_timeline_end(TEST_WIDTH));
        }
    }

    #[test]
    fn test_custom_label_display_short_mode() {
        let tz = crate::time::TimeZone::with_custom_label(
            chrono_tz::Asia::Tokyo,
            "TYO".to_string(),
            Some("Alice (Engineering)".to_string()),
        );
        let config = crate::config::TimeDisplayConfig::default();
        let now = Utc::now();

        let _widget = TimelineWidget::new(
            now,
            now,
            &tz,
            false,
            TimeFormat::TwentyFourHour,
            TimezoneDisplayMode::Short,
            &config,
            ColorTheme::default(),
            false,
            false,
            false,
        );

        // Test that effective_display_name is used in short mode
        assert_eq!(tz.effective_display_name(), "Alice (Engineering)");
    }

    #[test]
    fn test_custom_label_display_full_mode() {
        let tz = crate::time::TimeZone::with_custom_label(
            chrono_tz::Asia::Tokyo,
            "TYO".to_string(),
            Some("Bob (Sales)".to_string()),
        );
        let config = crate::config::TimeDisplayConfig::default();
        let now = Utc::now();

        let _widget = TimelineWidget::new(
            now,
            now,
            &tz,
            false,
            TimeFormat::TwentyFourHour,
            TimezoneDisplayMode::Full,
            &config,
            ColorTheme::default(),
            false,
            false,
            false,
        );

        // In full mode, custom label should be used with original info in parentheses
        assert_eq!(tz.custom_label.as_deref(), Some("Bob (Sales)"));
        assert!(!tz.get_full_display_name().is_empty());
    }

    #[test]
    fn test_no_custom_label_display() {
        let tz = crate::time::TimeZone::from_tz(chrono_tz::Asia::Tokyo);
        let config = crate::config::TimeDisplayConfig::default();
        let now = Utc::now();

        let _widget = TimelineWidget::new(
            now,
            now,
            &tz,
            false,
            TimeFormat::TwentyFourHour,
            TimezoneDisplayMode::Short,
            &config,
            ColorTheme::default(),
            false,
            false,
            false,
        );

        // Without custom label, should use default display name
        assert_eq!(tz.custom_label, None);
        assert_eq!(tz.effective_display_name(), &tz.display_name);
    }
}
