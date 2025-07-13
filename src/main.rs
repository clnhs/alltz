mod app;
mod time;
mod ui;
mod config;

use app::{App, Message, Direction};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
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

const TICK_RATE: Duration = Duration::from_millis(1000);

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    
    // Create app and run
    let mut app = App::new();
    let result = run_app(&mut terminal, &mut app).await;
    
    // Restore terminal
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

async fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> io::Result<()> {
    let mut last_tick = Instant::now();
    
    loop {
        // Render
        terminal.draw(|f| app.view(f))?;
        
        // Handle input
        let timeout = TICK_RATE
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));
        
        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    let message = match key.code {
                        KeyCode::Char('q') => Some(Message::Quit),
                        KeyCode::Char('m') => Some(Message::ToggleTimeFormat),
                        KeyCode::Char('t') => Some(Message::ResetToNow),
                        KeyCode::Char('h') | KeyCode::Left => Some(Message::ScrubTimeline(Direction::Left)),
                        KeyCode::Char('l') | KeyCode::Right => Some(Message::ScrubTimeline(Direction::Right)),
                        KeyCode::Char('j') | KeyCode::Down => Some(Message::NavigateZone(Direction::Down)),
                        KeyCode::Char('k') | KeyCode::Up => Some(Message::NavigateZone(Direction::Up)),
                        KeyCode::Char('[') => Some(Message::FineAdjust(-15)),
                        KeyCode::Char(']') => Some(Message::FineAdjust(15)),
                        KeyCode::Char('{') => Some(Message::FineAdjust(-60)),
                        KeyCode::Char('}') => Some(Message::FineAdjust(60)),
                        _ => None,
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
        
        // Tick
        if last_tick.elapsed() >= TICK_RATE {
            app.update(Message::Tick);
            last_tick = Instant::now();
        }
    }
}
