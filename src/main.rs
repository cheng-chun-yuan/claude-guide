mod actions;
mod app;
mod config;
mod event;
mod keybindings;
mod ui;

use anyhow::Result;
use clap::Parser;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::*;
use std::io;

use app::{App, InputMode, ModalType};
use event::{Event, EventHandler};
use keybindings::map_key_to_action;

#[derive(Parser)]
#[command(name = "claude-guide")]
#[command(author = "Your Name")]
#[command(version = "0.1.0")]
#[command(about = "A TUI for managing Claude Code configurations", long_about = None)]
struct Cli {
    /// Enable debug mode
    #[arg(short, long)]
    debug: bool,
}

fn main() -> Result<()> {
    let _cli = Cli::parse();

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
                // Clear any previous message
                app.message = None;

                // Handle special cases for insert mode
                if app.input_mode == InputMode::Insert {
                    match key.code {
                        KeyCode::Tab => {
                            app.cycle_modal_field(true);
                            continue;
                        }
                        KeyCode::BackTab => {
                            app.cycle_modal_field(false);
                            continue;
                        }
                        KeyCode::Char(' ') => {
                            // In add hook modal, space on type field toggles type
                            if let Some(ModalType::AddHook { .. }) = &app.modal {
                                if app.modal_index == 1 {
                                    app.cycle_hook_type();
                                    continue;
                                }
                            }
                            // Otherwise, treat as character input
                            if app.modal_index == 2 || !matches!(&app.modal, Some(ModalType::AddHook { .. })) {
                                app.update_modal_field(' ');
                                continue;
                            }
                        }
                        KeyCode::Char(c) => {
                            // Only allow character input on the target field for hooks
                            if let Some(ModalType::AddHook { .. }) = &app.modal {
                                if app.modal_index == 2 {
                                    app.update_modal_field(c);
                                }
                            } else {
                                app.update_modal_field(c);
                            }
                            continue;
                        }
                        KeyCode::Backspace => {
                            app.backspace_modal_field();
                            continue;
                        }
                        KeyCode::Up => {
                            // In add hook modal, up/down on event field cycles events
                            if let Some(ModalType::AddHook { .. }) = &app.modal {
                                if app.modal_index == 0 {
                                    app.cycle_hook_event(false);
                                    continue;
                                }
                            }
                        }
                        KeyCode::Down => {
                            if let Some(ModalType::AddHook { .. }) = &app.modal {
                                if app.modal_index == 0 {
                                    app.cycle_hook_event(true);
                                    continue;
                                }
                            }
                        }
                        _ => {}
                    }
                }

                // Map key to action and handle
                if let Some(action) = map_key_to_action(key, &app.input_mode) {
                    app.handle_action(action)?;
                }
            }
            Event::Resize(_, _) => {
                // Terminal will redraw on next iteration
            }
            Event::Tick => {
                // Could update animations or check for external changes
            }
        }
    }

    Ok(())
}
