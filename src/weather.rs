use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeatherData {
    pub temperature: f64,
    pub description: String,
    pub icon: String,
    pub emoji: String,
    pub last_updated: DateTime<Utc>,
}

impl WeatherData {
    pub fn new(temperature: f64, description: String, icon: String) -> Self {
        let emoji = Self::weather_icon_to_emoji(&icon);
        Self {
            temperature,
            description,
            icon,
            emoji,
            last_updated: Utc::now(),
        }
    }
    
    fn weather_icon_to_emoji(icon: &str) -> String {
        match icon {
            // Clear sky
            "01d" => "☀️".to_string(),  // Clear sky day
            "01n" => "🌙".to_string(),  // Clear sky night
            
            // Few clouds
            "02d" => "🌤️".to_string(),  // Few clouds day
            "02n" => "☁️".to_string(),  // Few clouds night
            
            // Scattered/broken clouds
            "03d" | "03n" | "04d" | "04n" => "☁️".to_string(),
            
            // Shower rain
            "09d" | "09n" => "🌧️".to_string(),
            
            // Rain
            "10d" => "🌦️".to_string(),  // Rain day
            "10n" => "🌧️".to_string(),  // Rain night
            
            // Thunderstorm
            "11d" | "11n" => "⛈️".to_string(),
            
            // Snow
            "13d" | "13n" => "❄️".to_string(),
            
            // Mist/fog
            "50d" | "50n" => "🌫️".to_string(),
            
            _ => "🌍".to_string(), // Default fallback
        }
    }
    
}

#[derive(Debug, Clone)]
pub struct WeatherManager {
    #[allow(dead_code)] // Stored for future real API implementation
    api_key: Option<String>,
    enabled: bool,
}

impl WeatherManager {
    pub fn new() -> Self {
        let api_key = std::env::var("OPENWEATHER_API_KEY").ok();
        let enabled = api_key.is_some();
        
        Self {
            api_key,
            enabled,
        }
    }
    
    /// Check if weather functionality is enabled (API key provided)
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
    
    /// Get weather data for a city, only if API key is configured
    /// Returns None if no API key is available
    pub fn get_weather(&self, city: &str) -> Option<WeatherData> {
        if !self.enabled {
            return None;
        }
        
        // For now, return demo data when API key is present
        // TODO: Implement real API calls in the future
        Some(self.get_demo_weather_internal(city))
    }
    
    /// Internal demo weather data for testing and when API key is present
    #[cfg(test)]
    pub fn get_demo_weather(&self, city: &str) -> WeatherData {
        self.get_demo_weather_internal(city)
    }
    
    fn get_demo_weather_internal(&self, city: &str) -> WeatherData {
        match city {
            "Los Angeles" => WeatherData::new(22.0, "Sunny".to_string(), "01d".to_string()),
            "New York" => WeatherData::new(15.0, "Partly cloudy".to_string(), "02d".to_string()),
            "London" => WeatherData::new(12.0, "Light rain".to_string(), "10d".to_string()),
            "Berlin" => WeatherData::new(18.0, "Cloudy".to_string(), "04d".to_string()),
            "Tokyo" => WeatherData::new(25.0, "Clear".to_string(), "01n".to_string()),
            "Sydney" => WeatherData::new(20.0, "Scattered clouds".to_string(), "03d".to_string()),
            "UTC" => WeatherData::new(16.0, "Mostly clear".to_string(), "02d".to_string()),
            _ => WeatherData::new(20.0, "Unknown".to_string(), "01d".to_string()),
        }
    }
}

impl Default for WeatherManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_weather_data_creation() {
        let weather = WeatherData::new(25.0, "Sunny".to_string(), "01d".to_string());
        assert_eq!(weather.temperature, 25.0);
        assert_eq!(weather.description, "Sunny");
        assert_eq!(weather.emoji, "☀️");
        assert!(weather.last_updated <= Utc::now()); // Timestamp should be valid
    }

    #[test]
    fn test_weather_icon_mapping() {
        let clear_day = WeatherData::new(25.0, "Clear".to_string(), "01d".to_string());
        assert_eq!(clear_day.emoji, "☀️");
        
        let clear_night = WeatherData::new(15.0, "Clear".to_string(), "01n".to_string());
        assert_eq!(clear_night.emoji, "🌙");
        
        let rain = WeatherData::new(18.0, "Rain".to_string(), "10d".to_string());
        assert_eq!(rain.emoji, "🌦️");
    }

    #[test]
    fn test_weather_manager_creation() {
        // Test without API key (should be disabled)
        let manager = WeatherManager::new();
        assert!(!manager.is_enabled());
        assert!(manager.get_weather("London").is_none());
    }
    
    #[test]
    fn test_weather_manager_with_api_key() {
        // Mock API key for testing
        std::env::set_var("OPENWEATHER_API_KEY", "test_key");
        let manager = WeatherManager::new();
        assert!(manager.is_enabled());
        
        let weather = manager.get_weather("London");
        assert!(weather.is_some());
        assert_eq!(weather.unwrap().emoji, "🌦️");
        
        // Clean up
        std::env::remove_var("OPENWEATHER_API_KEY");
    }

    #[test]
    fn test_demo_weather() {
        let manager = WeatherManager::new();
        let la_weather = manager.get_demo_weather("Los Angeles");
        assert_eq!(la_weather.emoji, "☀️");
        assert_eq!(la_weather.temperature, 22.0);
    }
}