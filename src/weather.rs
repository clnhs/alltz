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
    // Currently only used for demo weather data
    // Future: can be extended to cache real API weather data
}

impl WeatherManager {
    pub fn new() -> Self {
        Self {}
    }
    
    /// Provides demo weather data for the given city.
    /// This is used as fallback when no real weather API is configured.
    pub fn get_demo_weather(&self, city: &str) -> WeatherData {
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
        let manager = WeatherManager::new();
        // Should create successfully
        let demo_weather = manager.get_demo_weather("London");
        assert_eq!(demo_weather.emoji, "🌦️");
    }

    #[test]
    fn test_demo_weather() {
        let manager = WeatherManager::new();
        let la_weather = manager.get_demo_weather("Los Angeles");
        assert_eq!(la_weather.emoji, "☀️");
        assert_eq!(la_weather.temperature, 22.0);
    }
}