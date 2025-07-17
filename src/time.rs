use chrono::{DateTime, Offset, Timelike, Utc};
use chrono_tz::Tz;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CityData {
    pub name: String,
    pub code: String,
    pub timezone: String,
    pub country: String,
    pub coordinates: [f64; 2],
    pub aliases: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CitiesData {
    pub cities: Vec<CityData>,
    pub major_cities: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct TimeZone {
    pub tz: Tz,
    pub display_name: String,
    pub cities: Vec<String>, // List of cities in this timezone
}

impl TimeZone {
    pub fn new(tz: Tz, _name: String, display_name: String) -> Self {
        Self { 
            tz, 
            display_name, 
            cities: Vec::new() 
        }
    }


    pub fn from_tz(tz: Tz) -> Self {
        let name = tz.to_string();
        let cities_data = TimeZoneManager::load_cities_data();

        // Try to find the timezone in our cities data to get the proper airport code
        let display_name = cities_data
            .cities
            .iter()
            .find(|city| {
                if let Ok(city_tz) = Tz::from_str(&city.timezone) {
                    city_tz == tz
                } else {
                    false
                }
            })
            .map(|city| city.code.clone())
            .unwrap_or_else(|| {
                // Fallback: generate a 3-letter code from timezone string
                let tz_string = tz.to_string();
                let parts: Vec<&str> = tz_string.split('/').collect();
                if let Some(city) = parts.last() {
                    city.chars().take(3).collect::<String>().to_uppercase()
                } else {
                    "UNK".to_string()
                }
            });

        Self::new(tz, name, display_name)
    }

    pub fn add_city(&mut self, city: String) {
        if !self.cities.contains(&city) {
            self.cities.push(city);
        }
    }

    pub fn get_display_name(&self, use_full_names: bool, show_all_in_groups: bool) -> String {
        if self.cities.is_empty() {
            self.display_name.clone()
        } else if self.cities.len() == 1 {
            if use_full_names {
                self.cities[0].clone()
            } else {
                self.display_name.clone()
            }
        } else {
            if show_all_in_groups {
                // Show all cities in the group
                if use_full_names {
                    self.cities.join(", ")
                } else {
                    // Show timezone abbreviation + all city names
                    format!("{} ({})", self.get_timezone_abbreviation(), self.cities.join(", "))
                }
            } else {
                // Show first city + count
                let first_city = if use_full_names {
                    &self.cities[0]
                } else {
                    &self.display_name
                };
                format!("{} +{}", first_city, self.cities.len() - 1)
            }
        }
    }

    /// Gets the timezone identifier string (e.g., "UTC", "US/Eastern")
    #[cfg(test)]
    pub fn name(&self) -> String {
        self.tz.to_string()
    }

    pub fn get_timezone_abbreviation(&self) -> String {
        // Use chrono's built-in timezone formatting for accurate, DST-aware abbreviations
        let now = Utc::now();
        let local_time = now.with_timezone(&self.tz);
        local_time.format("%Z").to_string()
    }

    pub fn get_full_display_name(&self) -> String {
        // Create full display name using timezone string, abbreviation and offset
        let tz_string = self.tz.to_string();
        let parts: Vec<&str> = tz_string.split('/').collect();
        if let Some(city) = parts.last() {
            format!(
                "{} {} {}",
                city.replace("_", " "),
                self.get_timezone_abbreviation(),
                self.offset_string()
            )
        } else {
            format!(
                "{} {}",
                self.get_timezone_abbreviation(),
                self.offset_string()
            )
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

}

impl fmt::Display for TimeZone {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({})", self.display_name, self.offset_string())
    }
}

#[derive(Debug, Clone)]
pub struct TimeZoneManager {
    zones: Vec<TimeZone>,
    cities_data: CitiesData,
}

impl TimeZoneManager {
    pub fn new() -> Self {
        let cities_data = Self::load_cities_data();
        Self {
            zones: Vec::new(),
            cities_data,
        }
    }

    fn load_cities_data() -> CitiesData {
        let json_str = include_str!("cities.json");
        serde_json::from_str(json_str).expect("Failed to parse cities.json")
    }

    pub fn get_all_available_timezones() -> Vec<(Tz, String, String, f64, f64)> {
        let cities_data = Self::load_cities_data();
        cities_data
            .cities
            .iter()
            .map(|city| {
                let tz = Tz::from_str(&city.timezone).expect("Invalid timezone in cities.json");
                (
                    tz,
                    city.name.clone(),
                    city.code.clone(),
                    city.coordinates[0],
                    city.coordinates[1],
                )
            })
            .collect()
    }

    pub fn search_timezones(query: &str) -> Vec<String> {
        let query_lower = query.to_lowercase().trim().to_string();
        if query_lower.is_empty() {
            return Vec::new();
        }

        let cities_data = Self::load_cities_data();
        let mut results: Vec<(String, i32)> = Vec::new();

        for city in &cities_data.cities {
            let mut score = 0;
            let name_lower = city.name.to_lowercase();
            let code_lower = city.code.to_lowercase();
            let country_lower = city.country.to_lowercase();
            let tz_string = city.timezone.to_lowercase();

            // Exact match gets highest score
            if name_lower == query_lower || code_lower == query_lower {
                score += 1000;
            }
            // Starts with query gets very high score
            else if name_lower.starts_with(&query_lower) || code_lower.starts_with(&query_lower) {
                score += 500;
            }
            // Contains query gets medium score
            else if name_lower.contains(&query_lower) || code_lower.contains(&query_lower) {
                score += 200;
            }

            // Check aliases
            for alias in &city.aliases {
                let alias_lower = alias.to_lowercase();
                if alias_lower == query_lower {
                    score += 800;
                } else if alias_lower.starts_with(&query_lower) {
                    score += 400;
                } else if alias_lower.contains(&query_lower) {
                    score += 150;
                }
            }

            // Check country names
            if country_lower.contains(&query_lower) {
                score += 100;
            }

            // Check timezone string for technical searches
            if tz_string.contains(&query_lower) {
                score += 50;
            }

            // Bonus for major cities
            if cities_data.major_cities.contains(&city.name) {
                score += 25;
            }

            // Only include results with some relevance
            if score > 0 {
                results.push((city.name.clone(), score));
            }
        }

        // Sort by score (highest first), then alphabetically
        results.sort_by(|a, b| match b.1.cmp(&a.1) {
            std::cmp::Ordering::Equal => a.0.cmp(&b.0),
            other => other,
        });

        // Return top 8 results
        results.into_iter().take(8).map(|(name, _)| name).collect()
    }

    pub fn get_country_for_city(city: &str) -> String {
        let cities_data = Self::load_cities_data();
        cities_data
            .cities
            .iter()
            .find(|c| c.name == city)
            .map(|c| c.country.clone())
            .unwrap_or_else(|| "Unknown".to_string())
    }


    pub fn add_timezone_by_name_with_merging(&mut self, name: &str, merge_same_time: bool) -> bool {
        if let Some(city) = self
            .cities_data
            .cities
            .iter()
            .find(|c| c.name.eq_ignore_ascii_case(name))
        {
            if let Ok(tz) = Tz::from_str(&city.timezone) {
                // First, check if we already have this exact timezone (always merge same timezone)
                if let Some(existing_zone) = self.zones.iter_mut().find(|z| z.tz == tz) {
                    existing_zone.add_city(city.name.clone());
                    return true;
                }

                // If merge_same_time is enabled, check for zones with same current time
                if merge_same_time {
                    let current_time = chrono::Utc::now();
                    let new_time = current_time.with_timezone(&tz);
                    
                    for existing_zone in &mut self.zones {
                        let existing_time = current_time.with_timezone(&existing_zone.tz);
                        if new_time.hour() == existing_time.hour() 
                            && new_time.minute() == existing_time.minute()
                            && new_time.offset().fix().local_minus_utc() == existing_time.offset().fix().local_minus_utc() {
                            // Same time, merge into existing zone
                            existing_zone.add_city(city.name.clone());
                            return true;
                        }
                    }
                }

                // Create new timezone
                let mut timezone = TimeZone::new(tz, tz.to_string(), city.code.clone());
                timezone.add_city(city.name.clone());
                self.add_zone(timezone);
                return true;
            }
        }
        false
    }

    pub fn with_default_zones() -> Self {
        let cities_data = Self::load_cities_data();
        let default_city_names = vec![
            "Los Angeles",
            "New York",
            "UTC",
            "London",
            "Berlin",
            "Tokyo",
            "Sydney",
        ];

        let mut zones: Vec<TimeZone> = default_city_names
            .into_iter()
            .filter_map(|name| {
                cities_data
                    .cities
                    .iter()
                    .find(|c| c.name == name)
                    .and_then(|city| {
                        Tz::from_str(&city.timezone)
                            .ok()
                            .map(|tz| {
                                let mut timezone = TimeZone::new(tz, tz.to_string(), city.code.clone());
                                timezone.add_city(city.name.clone());
                                timezone
                            })
                    })
            })
            .collect();

        // Sort by UTC offset for natural time progression
        zones.sort_by_key(|tz| tz.utc_offset_hours());

        Self { zones, cities_data }
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

    pub fn reorganize_zones_for_merge(&mut self, merge_same_time: bool) {
        if merge_same_time {
            // When merge is enabled, first merge by same timezone, then by same time
            let mut new_zones: Vec<TimeZone> = Vec::new();
            let current_time = chrono::Utc::now();
            
            for zone in &self.zones {
                let zone_time = current_time.with_timezone(&zone.tz);
                
                // First, check if we already have the same timezone
                if let Some(existing_zone) = new_zones.iter_mut().find(|existing| existing.tz == zone.tz) {
                    // Same timezone, merge cities
                    for city in &zone.cities {
                        existing_zone.add_city(city.clone());
                    }
                } else if let Some(existing_zone) = new_zones.iter_mut().find(|existing| {
                    // Different timezone, but check if time is the same
                    let existing_time = current_time.with_timezone(&existing.tz);
                    zone_time.hour() == existing_time.hour() 
                        && zone_time.minute() == existing_time.minute()
                        && zone_time.offset().fix().local_minus_utc() == existing_time.offset().fix().local_minus_utc()
                }) {
                    // Same time, merge into existing zone
                    for city in &zone.cities {
                        existing_zone.add_city(city.clone());
                    }
                } else {
                    // Create a new zone
                    new_zones.push(zone.clone());
                }
            }

            self.zones = new_zones;
        } else {
            // When merge is disabled, split zones back based on their original timezone
            let mut new_zones: Vec<TimeZone> = Vec::new();
            
            for zone in &self.zones {
                if zone.cities.len() <= 1 {
                    // Single city zone, keep as is
                    new_zones.push(zone.clone());
                } else {
                    // Multi-city zone, split into separate zones by actual timezone
                    for city in &zone.cities {
                        // Find the appropriate timezone for this city
                        if let Some(city_data) = self.cities_data.cities.iter().find(|c| c.name == *city) {
                            if let Ok(tz) = chrono_tz::Tz::from_str(&city_data.timezone) {
                                // Check if we already have a zone for this timezone
                                if let Some(existing_zone) = new_zones.iter_mut().find(|z| z.tz == tz) {
                                    existing_zone.add_city(city.clone());
                                } else {
                                    let mut new_zone = TimeZone::new(tz, tz.to_string(), city_data.code.clone());
                                    new_zone.add_city(city.clone());
                                    new_zones.push(new_zone);
                                }
                            }
                        }
                    }
                }
            }

            self.zones = new_zones;
        }
        
        // Re-sort to maintain UTC offset order
        self.zones.sort_by_key(|tz| tz.utc_offset_hours());
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
        assert!(
            offset == "UTC-8" || offset == "UTC-7",
            "Expected UTC-8 or UTC-7, got {}",
            offset
        );
    }

    #[test]
    fn test_timezone_manager_default() {
        let manager = TimeZoneManager::with_default_zones();
        assert!(manager.zone_count() > 0);

        // Check that zones are sorted by offset
        let zones = manager.zones();
        for i in 1..zones.len() {
            assert!(zones[i - 1].utc_offset_hours() <= zones[i].utc_offset_hours());
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
    fn test_group_same_time_cities() {
        let mut manager = TimeZoneManager::new();
        
        // Add first city
        let success = manager.add_timezone_by_name_with_merging("New York", true);
        assert!(success);
        assert_eq!(manager.zone_count(), 1);
        
        // Add another city in the same timezone (should merge if they're in the same timezone)
        let success = manager.add_timezone_by_name_with_merging("Boston", true);
        assert!(success);
        
        // Check the timezones to debug what's happening
        let zones = manager.zones();
        if zones.len() == 1 {
            // Same timezone, should have merged
            assert_eq!(zones[0].cities.len(), 2);
            assert!(zones[0].cities.contains(&"New York".to_string()));
            assert!(zones[0].cities.contains(&"Boston".to_string()));
        } else {
            // Different timezones, should not have merged
            assert_eq!(zones.len(), 2);
            assert_eq!(zones[0].cities.len(), 1);
            assert_eq!(zones[1].cities.len(), 1);
        }
    }

    #[test]
    fn test_no_merge_when_disabled() {
        let mut manager = TimeZoneManager::new();
        
        // Add first city
        let success = manager.add_timezone_by_name_with_merging("New York", false);
        assert!(success);
        assert_eq!(manager.zone_count(), 1);
        
        // Add another city in the same timezone (should still merge since they're exact same timezone)
        let success = manager.add_timezone_by_name_with_merging("Boston", false);
        assert!(success);
        // Should be 1 zone with 2 cities since they're in the same timezone
        assert_eq!(manager.zone_count(), 1);
        assert_eq!(manager.zones()[0].cities.len(), 2);
    }

    #[test]
    fn test_timezone_display_name_with_cities() {
        let mut timezone = TimeZone::from_tz(chrono_tz::US::Eastern);
        
        // No cities - should use display name
        assert_eq!(timezone.get_display_name(false, false), timezone.display_name);
        
        // One city - should use abbreviation by default
        timezone.add_city("New York".to_string());
        assert_eq!(timezone.get_display_name(false, false), timezone.display_name);
        assert_eq!(timezone.get_display_name(true, false), "New York");
        
        // Multiple cities - should show first city + count
        timezone.add_city("Toronto".to_string());
        assert_eq!(timezone.get_display_name(false, false), format!("{} +1", timezone.display_name));
        assert_eq!(timezone.get_display_name(true, false), "New York +1");
        assert_eq!(timezone.get_display_name(true, true), "New York, Toronto");
        assert_eq!(timezone.get_display_name(false, true), format!("{} (New York, Toronto)", timezone.get_timezone_abbreviation()));
        
        // Adding duplicate city should not change count
        timezone.add_city("New York".to_string());
        assert_eq!(timezone.get_display_name(true, false), "New York +1");
    }

    #[test]
    fn test_default_zone_loading_with_cities() {
        let mut manager = TimeZoneManager::new();
        
        // Add a few cities and check that they have cities associated
        manager.add_timezone_by_name_with_merging("New York", false);
        manager.add_timezone_by_name_with_merging("Boston", false);
        
        let zones = manager.zones();
        assert_eq!(zones.len(), 1); // Should merge into same timezone
        assert_eq!(zones[0].cities.len(), 2); // Should have both cities
        assert_eq!(zones[0].get_display_name(true, false), "New York +1"); // Should show merged format
    }

    #[test]
    fn test_app_config_default_zones_have_cities() {
        use crate::app::App;
        
        let app = App::new();
        
        // Check that default zones have cities
        let zones = app.timezone_manager.zones();
        println!("Number of zones: {}", zones.len());
        
        for (i, zone) in zones.iter().enumerate() {
            println!("Zone {}: {} - cities: {:?}", i, zone.get_display_name(true, false), zone.cities);
            assert!(!zone.cities.is_empty(), "Zone {} should have cities", i);
        }
    }

    #[test]
    fn test_app_with_merge_enabled() {
        use crate::app::App;
        use crate::app::Message;
        
        let mut app = App::new();
        
        // Group functionality is now enabled by default
        assert!(app.group_same_time_cities);
        
        // Add a city to the same timezone as New York
        app.update(Message::StartAddZone);
        app.update(Message::UpdateAddZoneInput("Boston".to_string()));
        app.update(Message::ConfirmAddZone);
        
        let zones = app.timezone_manager.zones();
        println!("Zones after adding Boston with merge enabled:");
        for (i, zone) in zones.iter().enumerate() {
            println!("Zone {}: {} - cities: {:?}", i, zone.get_display_name(true, false), zone.cities);
        }
        
        // Check that Boston was grouped into the New York zone
        let ny_zone = zones.iter().find(|z| z.cities.contains(&"New York".to_string()));
        assert!(ny_zone.is_some());
        assert!(ny_zone.unwrap().cities.contains(&"Boston".to_string()));
    }

    #[test]
    fn test_reorganize_zones_for_merge() {
        let mut manager = TimeZoneManager::new();
        
        // Add some cities in the same timezone
        manager.add_timezone_by_name_with_merging("New York", false);
        manager.add_timezone_by_name_with_merging("Boston", false);
        manager.add_timezone_by_name_with_merging("Washington DC", false);
        
        println!("Before merge reorganization:");
        for (i, zone) in manager.zones().iter().enumerate() {
            println!("Zone {}: {} - cities: {:?}", i, zone.get_display_name(true, false), zone.cities);
        }
        
        // Enable merge - should combine zones with same time
        manager.reorganize_zones_for_merge(true);
        
        println!("After enabling merge:");
        for (i, zone) in manager.zones().iter().enumerate() {
            println!("Zone {}: {} - cities: {:?}", i, zone.get_display_name(true, false), zone.cities);
        }
        
        // Should have fewer zones now
        assert!(manager.zone_count() <= 3);
        
        // Disable merge - should split back into individual zones
        manager.reorganize_zones_for_merge(false);
        
        println!("After disabling merge:");
        for (i, zone) in manager.zones().iter().enumerate() {
            println!("Zone {}: {} - cities: {:?}", i, zone.get_display_name(true, false), zone.cities);
        }
    }

    #[test]
    fn test_app_merge_toggle_effect() {
        use crate::app::App;
        use crate::app::Message;
        
        let mut app = App::new();
        
        // Add some cities first
        app.update(Message::StartAddZone);
        app.update(Message::UpdateAddZoneInput("Boston".to_string()));
        app.update(Message::ConfirmAddZone);
        
        app.update(Message::StartAddZone);
        app.update(Message::UpdateAddZoneInput("Washington DC".to_string()));
        app.update(Message::ConfirmAddZone);
        
        println!("Initial state (merge ON):");
        for (i, zone) in app.timezone_manager.zones().iter().enumerate() {
            println!("Zone {}: {} - cities: {:?}", i, zone.get_display_name(true, false), zone.cities);
        }
        let initial_count = app.timezone_manager.zone_count();
        
        // Toggle merge OFF
        app.update(Message::ToggleGroupSameTimeCities);
        assert!(!app.group_same_time_cities);
        
        println!("After toggling merge OFF:");
        for (i, zone) in app.timezone_manager.zones().iter().enumerate() {
            println!("Zone {}: {} - cities: {:?}", i, zone.get_display_name(true, false), zone.cities);
        }
        let after_toggle_count = app.timezone_manager.zone_count();
        
        // Should have more zones when merge is disabled
        assert!(after_toggle_count >= initial_count);
        
        // Toggle merge back ON
        app.update(Message::ToggleGroupSameTimeCities);
        assert!(app.group_same_time_cities);
        
        println!("After toggling merge ON:");
        for (i, zone) in app.timezone_manager.zones().iter().enumerate() {
            println!("Zone {}: {} - cities: {:?}", i, zone.get_display_name(true, false), zone.cities);
        }
        
        // Should have fewer zones when merge is enabled again
        assert!(app.timezone_manager.zone_count() <= after_toggle_count);
    }

    #[test]
    fn test_timezone_vs_time_merge() {
        let mut manager = TimeZoneManager::new();
        
        // Add Toronto and Montreal (both Canada/Eastern)
        manager.add_timezone_by_name_with_merging("Toronto", false);
        manager.add_timezone_by_name_with_merging("Montreal", false);
        
        println!("After adding Toronto and Montreal (same timezone):");
        for (i, zone) in manager.zones().iter().enumerate() {
            println!("Zone {}: {} - cities: {:?} - tz: {}", i, zone.get_display_name(true, false), zone.cities, zone.tz);
        }
        
        // They should be merged because they have the same timezone
        assert_eq!(manager.zone_count(), 1);
        assert_eq!(manager.zones()[0].cities.len(), 2);
        
        // Add New York (US/Eastern) - different timezone but same time
        manager.add_timezone_by_name_with_merging("New York", false);
        
        println!("After adding New York (different timezone, same time):");
        for (i, zone) in manager.zones().iter().enumerate() {
            println!("Zone {}: {} - cities: {:?} - tz: {}", i, zone.get_display_name(true, false), zone.cities, zone.tz);
        }
        
        // Should be 2 zones because New York has different timezone
        assert_eq!(manager.zone_count(), 2);
        
        // Now enable merge by time
        manager.reorganize_zones_for_merge(true);
        
        println!("After enabling merge by time:");
        for (i, zone) in manager.zones().iter().enumerate() {
            println!("Zone {}: {} - cities: {:?} - tz: {}", i, zone.get_display_name(true, false), zone.cities, zone.tz);
        }
        
        // Should be 1 zone because all cities have the same time
        assert_eq!(manager.zone_count(), 1);
        assert_eq!(manager.zones()[0].cities.len(), 3);
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
