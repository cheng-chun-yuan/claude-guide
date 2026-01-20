mod actions;
mod app;
mod cli;
mod config;
mod event;
mod installer;
mod keybindings;
mod services;
mod ui;
mod version;

use anyhow::Result;
use clap::Parser;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::*;
use std::io;

use app::App;
use cli::{execute_command, Cli};
use event::{Event, EventHandler};

fn main() -> Result<()> {
    let cli = Cli::parse();

    // If a subcommand is provided, execute it and exit
    if let Some(command) = cli.command {
        return execute_command(command);
    }

    // Otherwise, launch the TUI
    run_tui()
}

fn run_tui() -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app and run
    let mut app = App::new()?;
    let result = run_app(&mut terminal, &mut app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = result {
        eprintln!("Error: {}", err);
        std::process::exit(1);
    }

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> Result<()> {
    let event_handler = EventHandler::default();

    while app.running {
        // Draw UI
        terminal.draw(|frame| ui::render(frame, app))?;

        // Handle events
        match event_handler.next()? {
            Event::Key(key) => {
                app.on_key(key)?;
            }
            Event::Resize => {
                // Terminal will redraw on next iteration
            }
            Event::Tick => {
                // Could update animations or check for external changes
            }
        }
    }

    Ok(())
}
