use chrono::{DateTime, Offset, Utc};
use chrono_tz::Tz;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use std::sync::OnceLock;
use sunrise::{Coordinates, SolarDay, SolarEvent};

static CITIES_DATA: OnceLock<CitiesData> = OnceLock::new();

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
    pub custom_label: Option<String>,
    pub source_city: Option<String>, // Store the original city name that was selected
}

impl TimeZone {
    pub fn new(tz: Tz, _name: String, display_name: String) -> Self {
        Self {
            tz,
            display_name,
            custom_label: None,
            source_city: None,
        }
    }

    #[cfg(test)]
    pub fn with_custom_label(tz: Tz, display_name: String, custom_label: Option<String>) -> Self {
        Self {
            tz,
            display_name,
            custom_label,
            source_city: None,
        }
    }

    pub fn with_source_city(
        tz: Tz,
        display_name: String,
        custom_label: Option<String>,
        source_city: Option<String>,
    ) -> Self {
        Self {
            tz,
            display_name,
            custom_label,
            source_city,
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

    #[cfg(test)]
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
            format!("UTC+{offset_hours}")
        } else {
            format!("UTC{offset_hours}")
        }
    }

    pub fn effective_display_name(&self) -> &str {
        self.custom_label.as_deref().unwrap_or(&self.display_name)
    }

    pub fn get_city_name(&self) -> String {
        // Use stored source city if available
        if let Some(source_city) = &self.source_city {
            return source_city.clone();
        }

        // Fallback: lookup by airport code
        let cities_data = TimeZoneManager::load_cities_data();
        if let Some(city) = cities_data
            .cities
            .iter()
            .find(|c| c.code == self.display_name)
        {
            return city.name.clone();
        }

        // Last resort: use display_name
        self.display_name.clone()
    }

    pub fn get_coordinates(&self) -> Option<(f64, f64)> {
        let cities_data = TimeZoneManager::load_cities_data();

        // First try to find by source city name
        if let Some(source_city) = &self.source_city {
            if let Some(city) = cities_data.cities.iter().find(|c| c.name == *source_city) {
                return Some((city.coordinates[0], city.coordinates[1]));
            }
        }

        // Fallback: lookup by airport code
        if let Some(city) = cities_data
            .cities
            .iter()
            .find(|c| c.code == self.display_name)
        {
            return Some((city.coordinates[0], city.coordinates[1]));
        }

        None
    }

    pub fn get_sunrise_sunset(&self, date: DateTime<Utc>) -> Option<(DateTime<Tz>, DateTime<Tz>)> {
        let (lat, lng) = self.get_coordinates()?;
        let coords = Coordinates::new(lat, lng)?;

        // Convert UTC date to local date for calculation
        let local_date = date.with_timezone(&self.tz).date_naive();

        // Create solar day for calculations
        let solar_day = SolarDay::new(coords, local_date);

        // Calculate sunrise and sunset times (returns UTC times)
        let sunrise_utc = solar_day.event_time(SolarEvent::Sunrise);
        let sunset_utc = solar_day.event_time(SolarEvent::Sunset);

        // Convert UTC times to local timezone
        let sunrise_tz = sunrise_utc.with_timezone(&self.tz);
        let sunset_tz = sunset_utc.with_timezone(&self.tz);

        Some((sunrise_tz, sunset_tz))
    }

    pub fn format_sun_times(&self, date: DateTime<Utc>, use_12_hour: bool) -> Option<String> {
        let (sunrise, sunset) = self.get_sunrise_sunset(date)?;
        if use_12_hour {
            Some(format!(
                "☀ {}:{}{} ☽ {}:{}{}",
                sunrise.format("%I"),
                sunrise.format("%M"),
                sunrise.format("%P"),
                sunset.format("%I"),
                sunset.format("%M"),
                sunset.format("%P")
            ))
        } else {
            Some(format!(
                "☀ {} ☽ {}",
                sunrise.format("%H:%M"),
                sunset.format("%H:%M")
            ))
        }
    }
}

impl fmt::Display for TimeZone {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} ({})",
            self.effective_display_name(),
            self.offset_string()
        )
    }
}

#[derive(Debug, Clone)]
pub struct TimeZoneManager {
    zones: Vec<TimeZone>,
}

impl TimeZoneManager {
    pub fn new() -> Self {
        Self { zones: Vec::new() }
    }

    fn load_cities_data() -> &'static CitiesData {
        CITIES_DATA.get_or_init(|| {
            let json_str = include_str!("cities.json");
            serde_json::from_str(json_str).expect("Failed to parse cities.json")
        })
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
                // Include country in the display name to disambiguate cities with same name
                let display_name = format!("{}, {}", city.name, city.country);
                results.push((display_name, score));
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
        self.add_timezone_with_label(name, None)
    }

    pub fn add_timezone_with_label(&mut self, name: &str, custom_label: Option<String>) -> bool {
        // Handle "City, Country" format from search results
        let (city_name, country) = if name.contains(", ") {
            let parts: Vec<&str> = name.splitn(2, ", ").collect();
            (parts[0], Some(parts[1]))
        } else {
            (name, None)
        };

        // Find city, considering country if provided
        let cities_data = Self::load_cities_data();
        let city = if let Some(country_name) = country {
            // Look for exact match with city name and country
            cities_data.cities.iter().find(|c| {
                c.name.eq_ignore_ascii_case(city_name)
                    && c.country.eq_ignore_ascii_case(country_name)
            })
        } else {
            // Fallback to just city name
            cities_data
                .cities
                .iter()
                .find(|c| c.name.eq_ignore_ascii_case(city_name))
        };

        if let Some(city) = city {
            if let Ok(tz) = Tz::from_str(&city.timezone) {
                let timezone = TimeZone::with_source_city(
                    tz,
                    city.code.clone(),
                    custom_label,
                    Some(city.name.clone()),
                );

                // Check if we already have this exact city (by airport code)
                if !self.zones.iter().any(|z| z.display_name == city.code) {
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

    pub fn update_zone_label(&mut self, index: usize, custom_label: Option<String>) -> bool {
        if index < self.zones.len() {
            self.zones[index].custom_label = custom_label;
            true
        } else {
            false
        }
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
            "Expected UTC-8 or UTC-7, got {offset}"
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

    #[test]
    fn test_custom_label() {
        let tz = chrono_tz::Asia::Tokyo;
        let timezone = TimeZone::with_custom_label(
            tz,
            "TYO".to_string(),
            Some("Alice (Engineering)".to_string()),
        );

        assert_eq!(timezone.display_name, "TYO");
        assert_eq!(
            timezone.custom_label.as_deref(),
            Some("Alice (Engineering)")
        );
        assert_eq!(timezone.effective_display_name(), "Alice (Engineering)");
    }

    #[test]
    fn test_custom_label_none() {
        let tz = chrono_tz::Asia::Tokyo;
        let timezone = TimeZone::with_custom_label(tz, "TYO".to_string(), None);

        assert_eq!(timezone.display_name, "TYO");
        assert_eq!(timezone.custom_label, None);
        assert_eq!(timezone.effective_display_name(), "TYO");
    }

    #[test]
    fn test_timezone_manager_update_label() {
        let mut manager = TimeZoneManager::new();
        manager.add_timezone_by_name("Tokyo");

        // Initially no custom label
        assert_eq!(manager.zones()[0].custom_label, None);

        // Update with custom label
        manager.update_zone_label(0, Some("Team Lead".to_string()));
        assert_eq!(
            manager.zones()[0].custom_label.as_deref(),
            Some("Team Lead")
        );

        // Clear custom label
        manager.update_zone_label(0, None);
        assert_eq!(manager.zones()[0].custom_label, None);
    }

    #[test]
    fn test_add_timezone_with_label() {
        let mut manager = TimeZoneManager::new();

        // Add timezone with custom label
        manager.add_timezone_with_label("New York", Some("NYC Office".to_string()));

        assert_eq!(manager.zone_count(), 1);
        assert_eq!(
            manager.zones()[0].custom_label.as_deref(),
            Some("NYC Office")
        );
    }

    #[test]
    fn test_search_london_disambiguation() {
        let results = TimeZoneManager::search_timezones("London");

        println!("Search results for 'London': {results:?}");

        // Should find both London, UK and London, Canada
        let london_uk = results.iter().find(|r| r.contains("London, UK"));
        let london_canada = results.iter().find(|r| r.contains("London, Canada"));

        assert!(london_uk.is_some(), "Should find London, UK in results");
        assert!(
            london_canada.is_some(),
            "Should find London, Canada in results"
        );

        // Should not have duplicate entries
        let london_uk_count = results.iter().filter(|r| r.contains("London, UK")).count();
        let london_canada_count = results
            .iter()
            .filter(|r| r.contains("London, Canada"))
            .count();

        assert_eq!(
            london_uk_count, 1,
            "Should have exactly one London, UK entry"
        );
        assert_eq!(
            london_canada_count, 1,
            "Should have exactly one London, Canada entry"
        );
    }

    #[test]
    fn test_add_both_london_cities() {
        let mut manager = TimeZoneManager::new();

        // Add both London cities
        let uk_added = manager.add_timezone_by_name("London, UK");
        let canada_added = manager.add_timezone_by_name("London, Canada");

        assert!(uk_added, "Should successfully add London, UK");
        assert!(canada_added, "Should successfully add London, Canada");
        assert_eq!(
            manager.zone_count(),
            2,
            "Should have 2 zones after adding both Londons"
        );

        // Verify they have different timezones
        let zones = manager.zones();
        let uk_zone = zones.iter().find(|z| {
            z.source_city.as_deref() == Some("London") && z.tz.to_string().contains("Europe")
        });
        let canada_zone = zones.iter().find(|z| {
            z.source_city.as_deref() == Some("London") && z.tz.to_string().contains("Canada")
        });

        assert!(uk_zone.is_some(), "Should find London, UK zone");
        assert!(canada_zone.is_some(), "Should find London, Canada zone");
    }
}
