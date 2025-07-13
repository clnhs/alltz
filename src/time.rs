use chrono::{DateTime, Utc, Offset};
use chrono_tz::Tz;
use std::fmt;

#[derive(Debug, Clone)]
pub struct TimeZone {
    pub tz: Tz,
    pub name: String,
    pub display_name: String,
}

impl TimeZone {
    pub fn new(tz: Tz, name: String, display_name: String) -> Self {
        Self {
            tz,
            name,
            display_name,
        }
    }
    
    pub fn from_tz(tz: Tz) -> Self {
        let name = tz.to_string();
        let display_name = Self::generate_display_name(&tz);
        Self::new(tz, name, display_name)
    }
    
    fn generate_display_name(tz: &Tz) -> String {
        match tz.to_string().as_str() {
            "UTC" => "UTC".to_string(),
            "US/Eastern" => "NYC".to_string(),
            "US/Pacific" => "LAX".to_string(),
            "Europe/London" => "LON".to_string(),
            "Europe/Berlin" => "BER".to_string(),
            "Asia/Tokyo" => "TOK".to_string(),
            "Australia/Sydney" => "SYD".to_string(),
            _ => {
                // Extract city name from timezone string
                let tz_string = tz.to_string();
                let parts: Vec<&str> = tz_string.split('/').collect();
                if let Some(city) = parts.last() {
                    city.chars().take(3).collect::<String>().to_uppercase()
                } else {
                    "UNK".to_string()
                }
            }
        }
    }
    
    pub fn convert_time(&self, utc_time: DateTime<Utc>) -> DateTime<Tz> {
        utc_time.with_timezone(&self.tz)
    }
    
    pub fn utc_offset_hours(&self) -> i32 {
        let now = Utc::now();
        let local_time = now.with_timezone(&self.tz);
        local_time.offset().fix().local_minus_utc() / 3600
    }
    
    pub fn offset_string(&self) -> String {
        let offset_hours = self.utc_offset_hours();
        if offset_hours >= 0 {
            format!("UTC+{}", offset_hours)
        } else {
            format!("UTC{}", offset_hours)
        }
    }
    
    pub fn display_name(&self) -> &str {
        &self.display_name
    }
}

impl fmt::Display for TimeZone {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({})", self.display_name, self.offset_string())
    }
}

#[derive(Debug, Clone)]
pub struct TimeZoneManager {
    zones: Vec<TimeZone>,
}

impl TimeZoneManager {
    pub fn new() -> Self {
        Self {
            zones: Vec::new(),
        }
    }
    
    pub fn with_default_zones() -> Self {
        let default_timezones = vec![
            chrono_tz::US::Pacific,      // UTC-8
            chrono_tz::US::Eastern,      // UTC-5
            chrono_tz::UTC,              // UTC+0
            chrono_tz::Europe::London,   // UTC+0/+1 (GMT/BST)
            chrono_tz::Europe::Berlin,   // UTC+1/+2 (CET/CEST)
            chrono_tz::Asia::Tokyo,      // UTC+9
            chrono_tz::Australia::Sydney,// UTC+10/+11 (AEST/AEDT)
        ];
        
        let mut zones: Vec<TimeZone> = default_timezones
            .into_iter()
            .map(TimeZone::from_tz)
            .collect();
        
        // Sort by UTC offset for natural time progression
        zones.sort_by_key(|tz| tz.utc_offset_hours());
        
        Self { zones }
    }
    
    pub fn add_zone(&mut self, timezone: TimeZone) {
        self.zones.push(timezone);
        // Re-sort to maintain UTC offset order
        self.zones.sort_by_key(|tz| tz.utc_offset_hours());
    }
    
    pub fn remove_zone(&mut self, index: usize) -> Option<TimeZone> {
        if index < self.zones.len() {
            Some(self.zones.remove(index))
        } else {
            None
        }
    }
    
    pub fn zones(&self) -> &[TimeZone] {
        &self.zones
    }
    
    pub fn zone_count(&self) -> usize {
        self.zones.len()
    }
    
    pub fn get_zone(&self, index: usize) -> Option<&TimeZone> {
        self.zones.get(index)
    }
}

impl Default for TimeZoneManager {
    fn default() -> Self {
        Self::with_default_zones()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono_tz;
    
    #[test]
    fn test_timezone_creation() {
        let tz = TimeZone::from_tz(chrono_tz::UTC);
        assert_eq!(tz.name, "UTC");
        assert_eq!(tz.display_name, "UTC");
        assert_eq!(tz.utc_offset_hours(), 0);
    }
    
    #[test]
    fn test_offset_string() {
        let utc = TimeZone::from_tz(chrono_tz::UTC);
        assert_eq!(utc.offset_string(), "UTC+0");
        
        let tokyo = TimeZone::from_tz(chrono_tz::Asia::Tokyo);
        assert_eq!(tokyo.offset_string(), "UTC+9");
        
        // Pacific timezone can be UTC-8 (PST) or UTC-7 (PDT) depending on DST
        let la = TimeZone::from_tz(chrono_tz::US::Pacific);
        let offset = la.offset_string();
        assert!(offset == "UTC-8" || offset == "UTC-7", "Expected UTC-8 or UTC-7, got {}", offset);
    }
    
    #[test]
    fn test_timezone_manager_default() {
        let manager = TimeZoneManager::with_default_zones();
        assert!(manager.zone_count() > 0);
        
        // Check that zones are sorted by offset
        let zones = manager.zones();
        for i in 1..zones.len() {
            assert!(zones[i-1].utc_offset_hours() <= zones[i].utc_offset_hours());
        }
    }
    
    #[test]
    fn test_add_remove_zones() {
        let mut manager = TimeZoneManager::new();
        assert_eq!(manager.zone_count(), 0);
        
        let utc_zone = TimeZone::from_tz(chrono_tz::UTC);
        manager.add_zone(utc_zone);
        assert_eq!(manager.zone_count(), 1);
        
        let removed = manager.remove_zone(0);
        assert!(removed.is_some());
        assert_eq!(manager.zone_count(), 0);
    }
    
    #[test]
    fn test_time_conversion() {
        let tokyo = TimeZone::from_tz(chrono_tz::Asia::Tokyo);
        let utc_time = Utc::now();
        let tokyo_time = tokyo.convert_time(utc_time);
        
        // Tokyo should be ahead of UTC
        assert!(tokyo_time.naive_local() >= utc_time.naive_utc());
    }
}