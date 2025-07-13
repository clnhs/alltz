use chrono::{DateTime, Utc, Offset};
use chrono_tz::Tz;
use std::fmt;

#[derive(Debug, Clone)]
pub struct TimeZone {
    pub tz: Tz,
    pub display_name: String,
}

impl TimeZone {
    pub fn new(tz: Tz, _name: String, display_name: String) -> Self {
        Self {
            tz,
            display_name,
        }
    }
    
    pub fn from_tz(tz: Tz) -> Self {
        let name = tz.to_string();
        let display_name = Self::generate_display_name(&tz);
        Self::new(tz, name, display_name)
    }
    
    /// Gets the timezone identifier string (e.g., "UTC", "US/Eastern")
    #[cfg(test)]
    pub fn name(&self) -> String {
        self.tz.to_string()
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
    
    pub fn get_timezone_abbreviation(&self) -> String {
        // Return proper timezone abbreviations for header display
        match self.tz.to_string().as_str() {
            "UTC" => "UTC".to_string(),
            "US/Eastern" => "EST".to_string(),  // Will be EDT during DST
            "US/Pacific" => "PST".to_string(),  // Will be PDT during DST
            "Europe/London" => "GMT".to_string(), // Will be BST during DST
            "Europe/Berlin" => "CET".to_string(), // Will be CEST during DST
            "Asia/Tokyo" => "JST".to_string(),
            "Australia/Sydney" => "AEST".to_string(), // Will be AEDT during DST
            _ => {
                // Fallback to using chrono's timezone formatting
                let now = Utc::now();
                let local_time = now.with_timezone(&self.tz);
                local_time.format("%Z").to_string()
            }
        }
    }
    
    pub fn get_full_display_name(&self) -> String {
        // Return full timezone names for display
        match self.tz.to_string().as_str() {
            "UTC" => "Coordinated Universal Time UTC+0".to_string(),
            "US/Eastern" => format!("Eastern Time (US) {} {}", self.get_timezone_abbreviation(), self.offset_string()),
            "US/Pacific" => format!("Pacific Time (US) {} {}", self.get_timezone_abbreviation(), self.offset_string()),
            "Europe/London" => format!("Greenwich Mean Time {} {}", self.get_timezone_abbreviation(), self.offset_string()),
            "Europe/Berlin" => format!("Central European Time {} {}", self.get_timezone_abbreviation(), self.offset_string()),
            "Asia/Tokyo" => format!("Japan Standard Time {} {}", self.get_timezone_abbreviation(), self.offset_string()),
            "Australia/Sydney" => format!("Australian Eastern Time {} {}", self.get_timezone_abbreviation(), self.offset_string()),
            _ => {
                // Fallback to timezone name with abbreviation and offset
                let tz_string = self.tz.to_string();
                let parts: Vec<&str> = tz_string.split('/').collect();
                if let Some(city) = parts.last() {
                    format!("{} {} {}", city.replace("_", " "), self.get_timezone_abbreviation(), self.offset_string())
                } else {
                    format!("{} {}", self.get_timezone_abbreviation(), self.offset_string())
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
    
    pub fn get_all_available_timezones() -> Vec<(Tz, String, String, f64, f64)> {
        // Returns (timezone, search_name, display_name, lat, lon) tuples
        vec![
            // Americas
            (chrono_tz::US::Pacific, "Los Angeles".to_string(), "LAX".to_string(), 34.0522, -118.2437),
            (chrono_tz::US::Mountain, "Denver".to_string(), "DEN".to_string(), 39.7392, -104.9903),
            (chrono_tz::US::Central, "Chicago".to_string(), "CHI".to_string(), 41.8781, -87.6298),
            (chrono_tz::US::Eastern, "New York".to_string(), "NYC".to_string(), 40.7128, -74.0060),
            (chrono_tz::Canada::Pacific, "Vancouver".to_string(), "VAN".to_string(), 49.2827, -123.1207),
            (chrono_tz::Canada::Eastern, "Toronto".to_string(), "TOR".to_string(), 43.6532, -79.3832),
            (chrono_tz::America::Sao_Paulo, "São Paulo".to_string(), "SAO".to_string(), -23.5505, -46.6333),
            (chrono_tz::America::Argentina::Buenos_Aires, "Buenos Aires".to_string(), "BUE".to_string(), -34.6118, -58.3960),
            (chrono_tz::America::Mexico_City, "Mexico City".to_string(), "MEX".to_string(), 19.4326, -99.1332),
            
            // Europe
            (chrono_tz::UTC, "UTC".to_string(), "UTC".to_string(), 51.4769, -0.0005), // Greenwich
            (chrono_tz::Europe::London, "London".to_string(), "LON".to_string(), 51.5074, -0.1278),
            (chrono_tz::Europe::Dublin, "Dublin".to_string(), "DUB".to_string(), 53.3498, -6.2603),
            (chrono_tz::Europe::Paris, "Paris".to_string(), "PAR".to_string(), 48.8566, 2.3522),
            (chrono_tz::Europe::Berlin, "Berlin".to_string(), "BER".to_string(), 52.5200, 13.4050),
            (chrono_tz::Europe::Rome, "Rome".to_string(), "ROM".to_string(), 41.9028, 12.4964),
            (chrono_tz::Europe::Madrid, "Madrid".to_string(), "MAD".to_string(), 40.4168, -3.7038),
            (chrono_tz::Europe::Amsterdam, "Amsterdam".to_string(), "AMS".to_string(), 52.3676, 4.9041),
            (chrono_tz::Europe::Zurich, "Zurich".to_string(), "ZUR".to_string(), 47.3769, 8.5417),
            (chrono_tz::Europe::Stockholm, "Stockholm".to_string(), "STO".to_string(), 59.3293, 18.0686),
            (chrono_tz::Europe::Moscow, "Moscow".to_string(), "MOW".to_string(), 55.7558, 37.6173),
            
            // Asia
            (chrono_tz::Asia::Tokyo, "Tokyo".to_string(), "TOK".to_string(), 35.6762, 139.6503),
            (chrono_tz::Asia::Seoul, "Seoul".to_string(), "SEO".to_string(), 37.5665, 126.9780),
            (chrono_tz::Asia::Shanghai, "Shanghai".to_string(), "SHA".to_string(), 31.2304, 121.4737),
            (chrono_tz::Asia::Hong_Kong, "Hong Kong".to_string(), "HKG".to_string(), 22.3193, 114.1694),
            (chrono_tz::Asia::Singapore, "Singapore".to_string(), "SIN".to_string(), 1.3521, 103.8198),
            (chrono_tz::Asia::Kolkata, "Mumbai".to_string(), "BOM".to_string(), 19.0760, 72.8777),
            (chrono_tz::Asia::Dubai, "Dubai".to_string(), "DXB".to_string(), 25.2048, 55.2708),
            (chrono_tz::Asia::Bangkok, "Bangkok".to_string(), "BKK".to_string(), 13.7563, 100.5018),
            (chrono_tz::Asia::Jakarta, "Jakarta".to_string(), "JKT".to_string(), -6.2088, 106.8456),
            
            // Oceania
            (chrono_tz::Australia::Sydney, "Sydney".to_string(), "SYD".to_string(), -33.8688, 151.2093),
            (chrono_tz::Australia::Melbourne, "Melbourne".to_string(), "MEL".to_string(), -37.8136, 144.9631),
            (chrono_tz::Australia::Perth, "Perth".to_string(), "PER".to_string(), -31.9505, 115.8605),
            (chrono_tz::Pacific::Auckland, "Auckland".to_string(), "AKL".to_string(), -36.8485, 174.7633),
            
            // Africa
            (chrono_tz::Africa::Cairo, "Cairo".to_string(), "CAI".to_string(), 30.0444, 31.2357),
            (chrono_tz::Africa::Johannesburg, "Johannesburg".to_string(), "JNB".to_string(), -26.2041, 28.0473),
            (chrono_tz::Africa::Lagos, "Lagos".to_string(), "LOS".to_string(), 6.5244, 3.3792),
        ]
    }
    
    pub fn search_timezones(query: &str) -> Vec<String> {
        let query_lower = query.to_lowercase();
        let available = Self::get_all_available_timezones();
        
        available
            .iter()
            .filter(|(_, search_name, _, _, _)| {
                search_name.to_lowercase().contains(&query_lower)
            })
            .take(5) // Limit to top 5 results
            .map(|(_, search_name, _, _, _)| search_name.clone())
            .collect()
    }
    
    pub fn add_timezone_by_name(&mut self, name: &str) -> bool {
        let available = Self::get_all_available_timezones();
        
        if let Some((tz, _, display_name, _, _)) = available
            .iter()
            .find(|(_, search_name, _, _, _)| search_name.eq_ignore_ascii_case(name))
        {
            let timezone = TimeZone::new(
                *tz,
                tz.to_string(),
                display_name.clone(),
            );
            
            // Check if we already have this timezone
            if !self.zones.iter().any(|z| z.tz == *tz) {
                self.add_zone(timezone);
                return true;
            }
        }
        false
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
        assert_eq!(tz.name(), "UTC");
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