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
    event::{DisableMouseCapture, EnableMouseCapture, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::*;
use std::io;

use app::{App, InputMode, ModalType};
use cli::{execute_command, Cli};
use event::{Event, EventHandler};
use keybindings::map_key_to_action;

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
                            if app.modal_index == 2
                                || !matches!(&app.modal, Some(ModalType::AddHook { .. }))
                            {
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
