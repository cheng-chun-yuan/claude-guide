use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::actions::Action;
use crate::app::InputMode;

/// Maps key events to actions based on the current input mode
pub fn map_key_to_action(key: KeyEvent, mode: &InputMode) -> Option<Action> {
    match mode {
        InputMode::Normal => map_normal_mode(key),
        InputMode::Insert => map_insert_mode(key),
        InputMode::Modal => map_modal_mode(key),
    }
}

fn map_normal_mode(key: KeyEvent) -> Option<Action> {
    match key.code {
        // Vim-style navigation
        KeyCode::Char('j') | KeyCode::Down => Some(Action::MoveDown),
        KeyCode::Char('k') | KeyCode::Up => Some(Action::MoveUp),
        KeyCode::Char('h') | KeyCode::Left => Some(Action::MoveLeft),
        KeyCode::Char('l') | KeyCode::Right => Some(Action::MoveRight),

        // Tab navigation
        KeyCode::Tab => Some(Action::NextTab),
        KeyCode::BackTab => Some(Action::PrevTab),
        KeyCode::Char('1') => Some(Action::GoToTab(0)),
        KeyCode::Char('2') => Some(Action::GoToTab(1)),
        KeyCode::Char('3') => Some(Action::GoToTab(2)),
        KeyCode::Char('4') => Some(Action::GoToTab(3)),
        KeyCode::Char('5') => Some(Action::GoToTab(4)),
        KeyCode::Char('6') => Some(Action::GoToTab(5)),

        // Selection
        KeyCode::Enter => Some(Action::Select),
        KeyCode::Esc => Some(Action::Cancel),

        // CRUD operations
        KeyCode::Char('a') => Some(Action::Add),
        KeyCode::Char('e') => Some(Action::Edit),
        KeyCode::Char('d') => Some(Action::Delete),
        KeyCode::Char(' ') => Some(Action::Toggle),

        // Save with 's' or Ctrl+S
        KeyCode::Char('s') => Some(Action::Save),

        // Application
        KeyCode::Char('?') => Some(Action::ShowHelp),
        KeyCode::Char('p') => Some(Action::ChangePlatform),
        KeyCode::Char('v') => Some(Action::ManageVersions),
        KeyCode::Char('t') => Some(Action::ToggleScope),
        KeyCode::Char('q') => Some(Action::Quit),
        KeyCode::Char('Q') => Some(Action::ForceQuit),
        KeyCode::Char('r') => Some(Action::Refresh),
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            Some(Action::ForceQuit)
        }

        _ => None,
    }
}

fn map_insert_mode(key: KeyEvent) -> Option<Action> {
    match key.code {
        KeyCode::Esc => Some(Action::Cancel),
        KeyCode::Enter => Some(Action::Select),
        KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => Some(Action::Save),
        KeyCode::Backspace => Some(Action::InputBackspace),
        KeyCode::Delete => Some(Action::InputDelete),
        KeyCode::Left => Some(Action::InputLeft),
        KeyCode::Right => Some(Action::InputRight),
        KeyCode::Home => Some(Action::InputHome),
        KeyCode::End => Some(Action::InputEnd),
        KeyCode::Char(c) => Some(Action::InputChar(c)),
        _ => None,
    }
}

fn map_modal_mode(key: KeyEvent) -> Option<Action> {
    match key.code {
        KeyCode::Enter | KeyCode::Char('y') => Some(Action::Confirm),
        KeyCode::Esc | KeyCode::Char('n') => Some(Action::Dismiss),
        KeyCode::Char('j') | KeyCode::Down => Some(Action::MoveDown),
        KeyCode::Char('k') | KeyCode::Up => Some(Action::MoveUp),
        KeyCode::Char('d') => Some(Action::Delete),
        _ => None,
    }
}

/// Get keybinding hints for the status bar
pub fn get_keybinding_hints(mode: &InputMode) -> Vec<(&'static str, &'static str)> {
    match mode {
        InputMode::Normal => vec![
            ("j/k", "Navigate"),
            ("Tab", "Switch tab"),
            ("a", "Add"),
            ("e", "Edit"),
            ("d", "Delete"),
            ("s", "Save"),
            ("?", "Help"),
            ("p", "Platform"),
            ("t", "Scope"),
            ("v", "Versions"),
            ("q", "Quit"),
        ],
        InputMode::Insert => vec![("Esc", "Cancel"), ("Enter", "Confirm"), ("Ctrl+S", "Save")],
        InputMode::Modal => vec![
            ("y/Enter", "Confirm"),
            ("n/Esc", "Cancel"),
            ("j/k", "Navigate"),
        ],
    }
}
