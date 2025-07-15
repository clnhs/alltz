use chrono::{DateTime, Offset, Utc};
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
}

impl TimeZone {
    pub fn new(tz: Tz, _name: String, display_name: String) -> Self {
        Self { tz, display_name }
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

    pub fn add_timezone_by_name(&mut self, name: &str) -> bool {
        if let Some(city) = self
            .cities_data
            .cities
            .iter()
            .find(|c| c.name.eq_ignore_ascii_case(name))
        {
            if let Ok(tz) = Tz::from_str(&city.timezone) {
                let timezone = TimeZone::new(tz, tz.to_string(), city.code.clone());

                // Check if we already have this timezone
                if !self.zones.iter().any(|z| z.tz == tz) {
                    self.add_zone(timezone);
                    return true;
                }
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
                            .map(|tz| TimeZone::new(tz, tz.to_string(), city.code.clone()))
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
    fn test_time_conversion() {
        let tokyo = TimeZone::from_tz(chrono_tz::Asia::Tokyo);
        let utc_time = Utc::now();
        let tokyo_time = tokyo.convert_time(utc_time);

        // Tokyo should be ahead of UTC
        assert!(tokyo_time.naive_local() >= utc_time.naive_utc());
    }
}
