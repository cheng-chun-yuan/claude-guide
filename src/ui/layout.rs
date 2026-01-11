use ratatui::prelude::*;

use super::{render_detail, render_list, render_modal, render_status_bar, render_tabs};
use crate::app::App;

pub fn render(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Tab bar
            Constraint::Min(10),   // Main content
            Constraint::Length(2), // Status bar
        ])
        .split(frame.area());

    // Render tab bar
    render_tabs(frame, chunks[0], app);

    // Split main content area horizontally
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(35), // List
            Constraint::Percentage(65), // Detail
        ])
        .split(chunks[1]);

    // Render list and detail views
    render_list(frame, main_chunks[0], app);
    render_detail(frame, main_chunks[1], app);

    // Render status bar
    render_status_bar(frame, chunks[2], app);

    // Render modal if present (on top of everything)
    if app.modal.is_some() {
        render_modal(frame, app);
    }
}
