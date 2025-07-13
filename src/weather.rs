use serde::{Deserialize, Serialize};
use std::collections::HashMap;
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
    
    pub fn is_stale(&self) -> bool {
        let now = Utc::now();
        let duration = now.signed_duration_since(self.last_updated);
        duration.num_minutes() > 30 // Consider data stale after 30 minutes
    }
}

#[derive(Debug, Clone)]
pub struct WeatherManager {
    weather_data: HashMap<String, WeatherData>,
    api_key: Option<String>,
    enabled: bool,
}

impl WeatherManager {
    pub fn new() -> Self {
        let api_key = std::env::var("OPENWEATHER_API_KEY").ok();
        let enabled = api_key.is_some();
        
        Self {
            weather_data: HashMap::new(),
            api_key,
            enabled,
        }
    }
    
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
    
    pub fn get_weather(&self, city: &str) -> Option<&WeatherData> {
        self.weather_data.get(city)
    }
    
    pub async fn fetch_weather(&mut self, city: &str, lat: f64, lon: f64) -> Result<(), Box<dyn std::error::Error>> {
        if !self.enabled {
            return Ok(());
        }
        
        // Check if we have recent data
        if let Some(weather) = self.weather_data.get(city) {
            if !weather.is_stale() {
                return Ok(());
            }
        }
        
        let api_key = self.api_key.as_ref().unwrap();
        let url = format!(
            "https://api.openweathermap.org/data/2.5/weather?lat={}&lon={}&appid={}&units=metric",
            lat, lon, api_key
        );
        
        let client = reqwest::Client::new();
        let response: serde_json::Value = client.get(&url).send().await?.json().await?;
        
        if let (Some(main), Some(weather_array)) = (response.get("main"), response.get("weather")) {
            if let (Some(temp), Some(weather_obj)) = (main.get("temp"), weather_array.get(0)) {
                if let (Some(description), Some(icon)) = (weather_obj.get("description"), weather_obj.get("icon")) {
                    let weather_data = WeatherData::new(
                        temp.as_f64().unwrap_or(0.0),
                        description.as_str().unwrap_or("Unknown").to_string(),
                        icon.as_str().unwrap_or("01d").to_string(),
                    );
                    
                    self.weather_data.insert(city.to_string(), weather_data);
                }
            }
        }
        
        Ok(())
    }
    
    // Fallback weather data for demo purposes when API key is not available
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
        assert!(!weather.is_stale()); // Should be fresh when just created
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
        // Should work whether API key is set or not
        assert!(manager.weather_data.is_empty());
    }

    #[test]
    fn test_demo_weather() {
        let manager = WeatherManager::new();
        let la_weather = manager.get_demo_weather("Los Angeles");
        assert_eq!(la_weather.emoji, "☀️");
        assert_eq!(la_weather.temperature, 22.0);
    }
}