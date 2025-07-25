#[macro_use]
extern crate rust_i18n;

// Load translations from locales directory
i18n!("locales");

mod app;
mod config;
mod time;
mod ui;

use app::{App, Direction, Message};
use clap::{Parser, Subcommand};
use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, KeyModifiers,
    },
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{
    error::Error,
    io,
    time::{Duration, Instant},
};

/// Rate at which the UI updates (1 second for time changes and animations)
const TICK_RATE: Duration = Duration::from_millis(1000);

#[derive(Parser)]
#[command(name = "alltz")]
#[command(version = "0.1.3")]
#[command(about = "🌍 Terminal-based timezone viewer for developers and remote teams")]
#[command(
    long_about = "alltz is a terminal application for tracking multiple timezones simultaneously. Features include DST indicators, color themes, and intuitive timeline scrubbing."
)]
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
        _ => Err(t!("cli.unknown_theme_error", s = s).to_string()),
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    // Set default locale
    rust_i18n::set_locale("en");

    let cli = Cli::parse();

    if let Some(command) = cli.command {
        return handle_command(command);
    }

    // Initialize terminal for TUI mode
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = create_app_with_options(cli)?;
    let result = run_app(&mut terminal, &mut app);

    // Cleanup: restore terminal to original state
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = result {
        println!("{}", t!("cli.general_error", err = err));
    }

    Ok(())
}

/// Main event loop for the TUI application
/// Handles user input, renders the UI, and processes timed updates
fn run_app<B: ratatui::backend::Backend>(
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
                    } else if app.renaming_zone {
                        // Special input handling for rename zone modal
                        match key.code {
                            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                                Some(Message::Quit)
                            }
                            KeyCode::Char(c) => {
                                let mut input = app.rename_zone_input.clone();
                                input.push(c);
                                Some(Message::UpdateRenameInput(input))
                            }
                            KeyCode::Backspace => {
                                let mut input = app.rename_zone_input.clone();
                                input.pop();
                                Some(Message::UpdateRenameInput(input))
                            }
                            KeyCode::Enter => Some(Message::ConfirmRename),
                            KeyCode::Esc => Some(Message::CancelRename),
                            _ => None,
                        }
                    } else if app.adding_zone {
                        // Special input handling for add zone modal
                        match key.code {
                            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                                Some(Message::Quit)
                            }
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
                            }
                            KeyCode::Backspace => {
                                let mut input = app.add_zone_input.clone();
                                input.pop();
                                Some(Message::UpdateAddZoneInput(input))
                            }
                            KeyCode::Up => Some(Message::NavigateSearchResults(Direction::Up)),
                            KeyCode::Down => Some(Message::NavigateSearchResults(Direction::Down)),
                            KeyCode::Enter => Some(Message::ConfirmAddZone),
                            KeyCode::Esc => Some(Message::CancelAddZone),
                            _ => None,
                        }
                    } else {
                        match key.code {
                            KeyCode::Char('q') => Some(Message::Quit),
                            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                                Some(Message::Quit)
                            }
                            KeyCode::Char('?') => Some(Message::ToggleHelp),
                            KeyCode::Char('a') => Some(Message::StartAddZone),
                            KeyCode::Char('r') => Some(Message::RemoveCurrentZone),
                            KeyCode::Char('e') => Some(Message::StartRenameZone),
                            KeyCode::Char('E') => Some(Message::ClearCustomName),
                            KeyCode::Char('m') => Some(Message::ToggleTimeFormat),
                            KeyCode::Char('n') => Some(Message::ToggleTimezoneDisplayMode),
                            KeyCode::Char('d') => Some(Message::ToggleDate),
                            KeyCode::Char('s') => Some(Message::ToggleSunTimes),
                            KeyCode::Char('c') => Some(Message::CycleColorTheme),
                            KeyCode::Char('t') => Some(Message::ResetToNow),
                            KeyCode::Char('h') | KeyCode::Left => {
                                if key.modifiers.contains(KeyModifiers::SHIFT) {
                                    Some(Message::ScrubTimelineWithShift(Direction::Left))
                                } else {
                                    Some(Message::ScrubTimeline(Direction::Left))
                                }
                            }
                            KeyCode::Char('l') | KeyCode::Right => {
                                if key.modifiers.contains(KeyModifiers::SHIFT) {
                                    Some(Message::ScrubTimelineWithShift(Direction::Right))
                                } else {
                                    Some(Message::ScrubTimeline(Direction::Right))
                                }
                            }
                            // Handle uppercase H and L (some terminals send these with Shift)
                            KeyCode::Char('H') => {
                                Some(Message::ScrubTimelineWithShift(Direction::Left))
                            }
                            KeyCode::Char('L') => {
                                Some(Message::ScrubTimelineWithShift(Direction::Right))
                            }
                            KeyCode::Char('j') | KeyCode::Down => {
                                Some(Message::NavigateZone(Direction::Down))
                            }
                            KeyCode::Char('k') | KeyCode::Up => {
                                Some(Message::NavigateZone(Direction::Up))
                            }
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
fn handle_command(command: Commands) -> Result<(), Box<dyn Error>> {
    use chrono::{Local, Offset, Utc};
    use time::TimeZoneManager;

    match command {
        Commands::List => {
            use std::io::{self, Write};

            let stdout = io::stdout();
            let mut handle = stdout.lock();

            // Handle broken pipe gracefully
            let result = (|| -> io::Result<()> {
                writeln!(handle, "{}", t!("cli.list.header"))?;
                writeln!(handle)?;
                let timezones = TimeZoneManager::get_all_available_timezones();
                for (_, city, code, lat, lon) in timezones {
                    writeln!(handle, "  {city:<15} {code:<4} ({lat:>7.2}, {lon:>8.2})")?;
                }
                writeln!(handle)?;
                writeln!(handle, "{}", t!("cli.list.footer"))?;
                Ok(())
            })();

            // Ignore broken pipe errors (when output is piped to head, etc.)
            if let Err(e) = result {
                if e.kind() != io::ErrorKind::BrokenPipe {
                    return Err(e.into());
                }
            }
        }

        Commands::Time { city } => {
            let timezones = TimeZoneManager::get_all_available_timezones();
            if let Some((tz, city_name, _, _, _)) = timezones
                .iter()
                .find(|(_, name, _, _, _)| name.eq_ignore_ascii_case(&city))
            {
                let now = Utc::now();
                let local_time = now.with_timezone(tz);
                let local_system = now.with_timezone(&Local);

                println!("{}", t!("cli.time.header", city_name = city_name));
                println!("   {}", local_time.format("%H:%M:%S %Z (%a, %b %d)"));
                println!();
                println!("{}", t!("cli.time.local_header"));
                println!("   {}", local_system.format("%H:%M:%S %Z (%a, %b %d)"));
            } else {
                eprintln!("{}", t!("cli.time.not_found", city = city));
                std::process::exit(1);
            }
        }

        Commands::Zone { city } => {
            let timezones = TimeZoneManager::get_all_available_timezones();
            if let Some((tz, city_name, code, lat, lon)) = timezones
                .iter()
                .find(|(_, name, _, _, _)| name.eq_ignore_ascii_case(&city))
            {
                let now = Utc::now();
                let local_time = now.with_timezone(tz);
                let offset_seconds = local_time.offset().fix().local_minus_utc();
                let offset_hours = offset_seconds / 3600;

                println!("{}", t!("cli.zone.header", city_name = city_name));
                println!("{}", t!("cli.zone.code", code = code));
                println!("{}", t!("cli.zone.timezone", tz = tz));
                println!("{}", t!("cli.zone.utc_offset", offset_hours = offset_hours));
                if *lat >= 0.0 && *lon <= 0.0 {
                    println!(
                        "{}",
                        t!("cli.zone.coordinates_n_w", lat = lat, lon = lon.abs())
                    );
                } else if *lat >= 0.0 && *lon > 0.0 {
                    println!("{}", t!("cli.zone.coordinates_n_e", lat = lat, lon = lon));
                } else if *lat < 0.0 && *lon <= 0.0 {
                    println!(
                        "{}",
                        t!("cli.zone.coordinates_s_w", lat = lat.abs(), lon = lon.abs())
                    );
                } else {
                    println!(
                        "{}",
                        t!("cli.zone.coordinates_s_e", lat = lat.abs(), lon = lon)
                    );
                }
                println!(
                    "{}",
                    t!(
                        "cli.zone.current_time",
                        time = local_time.format("%H:%M:%S %Z (%a, %b %d, %Y)")
                    )
                );

                // Simple DST status (just show current offset)
                println!("{}", t!("cli.zone.dst_status", offset_hours = offset_hours));
            } else {
                eprintln!("{}", t!("cli.zone.not_found", city = city));
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
        if timezones
            .iter()
            .any(|(_, name, _, _, _)| name.eq_ignore_ascii_case(&timezone_name))
        {
            app.timezone_manager.add_timezone_by_name(&timezone_name);

            // Set this timezone as selected
            if let Some(app_index) = app.timezone_manager.zones().iter().position(|zone| {
                timezones.iter().any(|(tz, name, _, _, _)| {
                    *tz == zone.tz && name.eq_ignore_ascii_case(&timezone_name)
                })
            }) {
                app.selected_zone_index = app_index;
            }
        } else {
            eprintln!(
                "{}",
                t!(
                    "cli.timezone_not_found_warning",
                    timezone_name = timezone_name
                )
            );
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
