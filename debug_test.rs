use alltz::time::TimeZoneManager;

fn main() {
    let manager = TimeZoneManager::with_default_zones();
    println!("Zone count: {}", manager.zones().len());
    
    for (i, zone) in manager.zones().iter().enumerate() {
        println!("Zone {}: {} ({}) offset: {}", 
                 i, zone.display_name(), zone.tz, zone.offset_string());
    }
}