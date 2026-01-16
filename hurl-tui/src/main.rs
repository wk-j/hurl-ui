//! Hurl TUI - A Terminal User Interface for Hurl HTTP testing
//!
//! This application provides an interactive terminal interface for working with
//! Hurl files, allowing users to browse, edit, run, and debug HTTP requests.

mod app;
mod config;
mod events;
mod parser;
mod runner;
mod ui;

use anyhow::Result;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{io, path::PathBuf};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use crate::app::App;
use crate::config::Config;
use crate::events::EventHandler;

/// Application entry point
#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    init_logging()?;

    // Load configuration
    let config = Config::load()?;

    // Get the working directory (current dir or from args)
    let working_dir = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());

    // Setup terminal
    let mut terminal = setup_terminal()?;

    // Create application state
    let mut app = App::new(config, working_dir)?;

    // Create event handler
    let event_handler = EventHandler::new(250); // 250ms tick rate

    // Run the application
    let result = run_app(&mut terminal, &mut app, event_handler).await;

    // Restore terminal
    restore_terminal(&mut terminal)?;

    // Handle any errors from the main loop
    if let Err(e) = result {
        eprintln!("Application error: {e}");
        std::process::exit(1);
    }

    Ok(())
}

/// Initialize the tracing subscriber for logging
fn init_logging() -> Result<()> {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("warn"));

    tracing_subscriber::registry()
        .with(filter)
        .with(tracing_subscriber::fmt::layer().with_writer(io::stderr))
        .init();

    Ok(())
}

/// Setup the terminal for TUI rendering
fn setup_terminal() -> Result<Terminal<CrosstermBackend<io::Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

/// Restore the terminal to its original state
fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    Ok(())
}

/// Main application loop
async fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
    mut event_handler: EventHandler,
) -> Result<()> {
    loop {
        // Draw the UI
        terminal.draw(|frame| ui::draw(frame, app))?;

        // Handle events
        match event_handler.next().await? {
            events::Event::Tick => {
                app.on_tick();
            }
            events::Event::Key(key_event) => {
                app.handle_key_event(key_event).await?;
            }
            events::Event::Mouse(mouse_event) => {
                app.handle_mouse_event(mouse_event);
            }
            events::Event::Resize(width, height) => {
                app.handle_resize(width, height);
            }
        }

        // Check if we should quit
        if app.should_quit() {
            break;
        }
    }

    Ok(())
}
