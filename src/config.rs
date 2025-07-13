use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeDisplayConfig {
    pub work_hours_start: u32,    // 8 (8 AM)
    pub work_hours_end: u32,      // 18 (6 PM)
    pub awake_hours_start: u32,   // 6 (6 AM) 
    pub awake_hours_end: u32,     // 22 (10 PM)
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
    Night,      // 10 PM - 6 AM
    Awake,      // 6 AM - 8 AM, 6 PM - 10 PM  
    Work,       // 8 AM - 6 PM
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
            TimeActivity::Night => '░',  // Light shade - low activity
            TimeActivity::Awake => '▒',  // Medium shade - moderate activity  
            TimeActivity::Work => '▓',   // Dark shade - high activity (less intense than █)
        }
    }
    
    pub fn get_activity_color(&self, activity: TimeActivity) -> ratatui::style::Color {
        match activity {
            TimeActivity::Night => ratatui::style::Color::DarkGray,
            TimeActivity::Awake => ratatui::style::Color::Gray,
            TimeActivity::Work => ratatui::style::Color::Magenta,  // Less bright than white
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_time_periods() {
        let config = TimeDisplayConfig::default();
        
        // Test work hours
        assert_eq!(config.get_time_activity(9), TimeActivity::Work);   // 9 AM
        assert_eq!(config.get_time_activity(14), TimeActivity::Work);  // 2 PM
        assert_eq!(config.get_time_activity(17), TimeActivity::Work);  // 5 PM
        
        // Test awake hours  
        assert_eq!(config.get_time_activity(7), TimeActivity::Awake);  // 7 AM
        assert_eq!(config.get_time_activity(19), TimeActivity::Awake); // 7 PM
        assert_eq!(config.get_time_activity(21), TimeActivity::Awake); // 9 PM
        
        // Test night hours
        assert_eq!(config.get_time_activity(23), TimeActivity::Night); // 11 PM
        assert_eq!(config.get_time_activity(2), TimeActivity::Night);  // 2 AM
        assert_eq!(config.get_time_activity(5), TimeActivity::Night);  // 5 AM
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
}