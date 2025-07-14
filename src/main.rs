mod app;
mod time;
mod ui;
mod config;
mod weather;

use app::{App, Message, Direction};
use clap::{Parser, Subcommand};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
};
use std::{
    error::Error,
    io,
    time::{Duration, Instant},
};

/// Rate at which the UI updates (1 second for time changes and animations)
const TICK_RATE: Duration = Duration::from_millis(1000);

#[derive(Parser)]
#[command(name = "alltz")]
#[command(version = "0.1.0")]
#[command(about = "🌍 Terminal-based timezone viewer for developers and remote teams")]
#[command(long_about = "alltz is a beautiful terminal application for tracking multiple timezones simultaneously. Features include real-time weather icons, DST indicators, color themes, and intuitive timeline scrubbing.")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
    
    /// Start with a specific timezone selected
    #[arg(short, long)]
    timezone: Option<String>,
    
    /// Use 12-hour time format instead of 24-hour
    #[arg(long)]
    twelve_hour: bool,
    
    /// Start with a specific color theme
    #[arg(long, value_parser = parse_theme)]
    theme: Option<config::ColorTheme>,
}

#[derive(Subcommand)]
enum Commands {
    /// List all available timezones
    #[command(alias = "ls")]
    List,
    
    /// Show current time in a specific timezone
    #[command(alias = "show")]
    Time {
        /// City name to show time for
        city: String,
    },
    
    /// Show timezone information and current time
    #[command(alias = "info")]
    Zone {
        /// City name to get information for
        city: String,
    },
    
    /// Show configuration file path and content
    Config {
        /// Generate default config file if it doesn't exist
        #[arg(long)]
        generate: bool,
    },
}

/// Parse theme name from CLI argument into ColorTheme enum
fn parse_theme(s: &str) -> Result<config::ColorTheme, String> {
    match s.to_lowercase().as_str() {
        "default" => Ok(config::ColorTheme::Default),
        "ocean" => Ok(config::ColorTheme::Ocean),
        "forest" => Ok(config::ColorTheme::Forest),
        "sunset" => Ok(config::ColorTheme::Sunset),
        "cyberpunk" => Ok(config::ColorTheme::Cyberpunk),
        "monochrome" => Ok(config::ColorTheme::Monochrome),
        _ => Err(format!("Unknown theme: {}. Available themes: default, ocean, forest, sunset, cyberpunk, monochrome", s)),
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();
    
    if let Some(command) = cli.command {
        return handle_command(command).await;
    }
    
    // Initialize terminal for TUI mode
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    
    let mut app = create_app_with_options(cli)?;
    let result = run_app(&mut terminal, &mut app).await;
    
    // Cleanup: restore terminal to original state
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    
    if let Err(err) = result {
        println!("Error: {}", err);
    }
    
    Ok(())
}

/// Main event loop for the TUI application
/// Handles user input, renders the UI, and processes timed updates
async fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> io::Result<()> {
    let mut last_tick = Instant::now();
    
    loop {
        terminal.draw(|f| app.view(f))?;
        
        // Calculate timeout to maintain consistent TICK_RATE
        let timeout = TICK_RATE
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));
        
        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    let message = if app.show_help {
                        Some(Message::ToggleHelp)
                    } else if app.adding_zone {
                        // Special input handling for add zone modal
                        match key.code {
                            KeyCode::Char(c) => {
                                // Handle numeric selection of search results (1-9)
                                if c.is_ascii_digit() && !app.zone_search_results.is_empty() {
                                    let digit = c.to_digit(10).unwrap() as usize;
                                    if digit >= 1 && digit <= app.zone_search_results.len() {
                                        Some(Message::SelectSearchResult(digit - 1))
                                    } else {
                                        let mut input = app.add_zone_input.clone();
                                        input.push(c);
                                        Some(Message::UpdateAddZoneInput(input))
                                    }
                                } else {
                                    let mut input = app.add_zone_input.clone();
                                    input.push(c);
                                    Some(Message::UpdateAddZoneInput(input))
                                }
                            },
                            KeyCode::Backspace => {
                                let mut input = app.add_zone_input.clone();
                                input.pop();
                                Some(Message::UpdateAddZoneInput(input))
                            },
                            KeyCode::Up => Some(Message::NavigateSearchResults(Direction::Up)),
                            KeyCode::Down => Some(Message::NavigateSearchResults(Direction::Down)),
                            KeyCode::Enter => Some(Message::ConfirmAddZone),
                            KeyCode::Esc => Some(Message::CancelAddZone),
                            _ => None,
                        }
                    } else {
                        match key.code {
                            KeyCode::Char('q') => Some(Message::Quit),
                            KeyCode::Char('?') => Some(Message::ToggleHelp),
                            KeyCode::Char('a') => Some(Message::StartAddZone),
                            KeyCode::Char('d') => Some(Message::RemoveCurrentZone),
                            KeyCode::Char('m') => Some(Message::ToggleTimeFormat),
                            KeyCode::Char('n') => Some(Message::ToggleTimezoneDisplayMode),
                            KeyCode::Char('w') => Some(Message::ToggleWeather),
                            KeyCode::Char('e') => Some(Message::ToggleDate),
                            KeyCode::Char('c') => Some(Message::CycleColorTheme),
                            KeyCode::Char('t') => Some(Message::ResetToNow),
                            KeyCode::Char('h') | KeyCode::Left => {
                                if key.modifiers.contains(KeyModifiers::SHIFT) {
                                    Some(Message::ScrubTimelineWithShift(Direction::Left))
                                } else {
                                    Some(Message::ScrubTimeline(Direction::Left))
                                }
                            },
                            KeyCode::Char('l') | KeyCode::Right => {
                                if key.modifiers.contains(KeyModifiers::SHIFT) {
                                    Some(Message::ScrubTimelineWithShift(Direction::Right))
                                } else {
                                    Some(Message::ScrubTimeline(Direction::Right))
                                }
                            },
                            KeyCode::Char('j') | KeyCode::Down => Some(Message::NavigateZone(Direction::Down)),
                            KeyCode::Char('k') | KeyCode::Up => Some(Message::NavigateZone(Direction::Up)),
                            KeyCode::Char('[') => Some(Message::FineAdjust(-15)),
                            KeyCode::Char(']') => Some(Message::FineAdjust(15)),
                            KeyCode::Char('{') => Some(Message::FineAdjust(-60)),
                            KeyCode::Char('}') => Some(Message::FineAdjust(60)),
                            _ => None,
                        }
                    };
                    
                    if let Some(msg) = message {
                        app.update(msg);
                        if app.should_quit {
                            return Ok(());
                        }
                    }
                }
            }
        }
        
        // Send periodic tick for time updates and animations
        if last_tick.elapsed() >= TICK_RATE {
            app.update(Message::Tick);
            last_tick = Instant::now();
        }
    }
}

/// Handle CLI subcommands (list, time, zone) and exit without starting TUI
async fn handle_command(command: Commands) -> Result<(), Box<dyn Error>> {
    use chrono::{Utc, Local, Offset};
    use time::TimeZoneManager;
    
    match command {
        Commands::List => {
            println!("🌍 Available Timezones:");
            println!();
            let timezones = TimeZoneManager::get_all_available_timezones();
            for (_, city, code, lat, lon) in timezones {
                println!("  {:<15} {:<4} ({:>7.2}, {:>8.2})", city, code, lat, lon);
            }
            println!();
            println!("Use 'alltz time <city>' to see current time in any timezone");
        }
        
        Commands::Time { city } => {
            let timezones = TimeZoneManager::get_all_available_timezones();
            if let Some((tz, city_name, _, _, _)) = timezones.iter()
                .find(|(_, name, _, _, _)| name.eq_ignore_ascii_case(&city)) {
                
                let now = Utc::now();
                let local_time = now.with_timezone(tz);
                let local_system = now.with_timezone(&Local);
                
                println!("🕐 Current time in {}:", city_name);
                println!("   {}", local_time.format("%H:%M:%S %Z (%a, %b %d)"));
                println!();
                println!("🏠 Your local time:");
                println!("   {}", local_system.format("%H:%M:%S %Z (%a, %b %d)"));
            } else {
                eprintln!("❌ City '{}' not found. Use 'alltz list' to see available timezones.", city);
                std::process::exit(1);
            }
        }
        
        Commands::Zone { city } => {
            let timezones = TimeZoneManager::get_all_available_timezones();
            if let Some((tz, city_name, code, lat, lon)) = timezones.iter()
                .find(|(_, name, _, _, _)| name.eq_ignore_ascii_case(&city)) {
                
                let now = Utc::now();
                let local_time = now.with_timezone(tz);
                let offset_seconds = local_time.offset().fix().local_minus_utc();
                let offset_hours = offset_seconds / 3600;
                
                println!("🌍 Timezone Information for {}:", city_name);
                println!("   Code:         {}", code);
                println!("   Timezone:     {}", tz);
                println!("   UTC Offset:   UTC{:+}", offset_hours);
                println!("   Coordinates:  {:.2}°N, {:.2}°W", lat, lon.abs());
                println!("   Current Time: {}", local_time.format("%H:%M:%S %Z (%a, %b %d, %Y)"));
                
                // Simple DST status (just show current offset)
                println!("   DST Status:   Current offset UTC{:+}", offset_hours);
            } else {
                eprintln!("❌ City '{}' not found. Use 'alltz list' to see available timezones.", city);
                std::process::exit(1);
            }
        }
        
        Commands::Config { generate } => {
            use config::AppConfig;
            
            if let Some(config_path) = AppConfig::config_path() {
                println!("📁 Configuration file location:");
                println!("   {}", config_path.display());
                println!();
                
                if generate || !config_path.exists() {
                    // Generate/create config file
                    let default_config = AppConfig::default();
                    match default_config.save() {
                        Ok(()) => {
                            if generate {
                                println!("✅ Generated default configuration file");
                            } else {
                                println!("✅ Created default configuration file");
                            }
                        },
                        Err(e) => {
                            eprintln!("❌ Failed to create config file: {}", e);
                            std::process::exit(1);
                        }
                    }
                }
                
                // Show current config content
                if config_path.exists() {
                    match std::fs::read_to_string(&config_path) {
                        Ok(content) => {
                            println!("📄 Current configuration:");
                            println!("{}", content);
                        },
                        Err(e) => {
                            eprintln!("❌ Could not read config file: {}", e);
                        }
                    }
                } else {
                    println!("❌ Configuration file does not exist");
                    println!("   Run 'alltz config --generate' to create a default one");
                }
            } else {
                eprintln!("❌ Could not determine config directory path");
                std::process::exit(1);
            }
        }
    }
    
    Ok(())
}

/// Create App instance with CLI options applied (timezone, theme, format)
fn create_app_with_options(cli: Cli) -> Result<App, Box<dyn Error>> {
    let mut app = App::new();
    
    if let Some(timezone_name) = cli.timezone {
        let timezones = time::TimeZoneManager::get_all_available_timezones();
        if let Some(_) = timezones.iter().position(|(_, name, _, _, _)| name.eq_ignore_ascii_case(&timezone_name)) {
            app.timezone_manager.add_timezone_by_name(&timezone_name);
            
            // Set this timezone as selected
            if let Some(app_index) = app.timezone_manager.zones().iter().position(|zone| {
                timezones.iter().any(|(tz, name, _, _, _)| *tz == zone.tz && name.eq_ignore_ascii_case(&timezone_name))
            }) {
                app.selected_zone_index = app_index;
            }
        } else {
            eprintln!("⚠️  Warning: Timezone '{}' not found. Use 'alltz list' to see available options.", timezone_name);
        }
    }
    
    if cli.twelve_hour {
        app.display_format = app::TimeFormat::TwelveHour;
    }
    
    if let Some(theme) = cli.theme {
        app.color_theme = theme;
    }
    
    Ok(app)
}
