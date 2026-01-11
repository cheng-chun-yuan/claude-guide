use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph},
};

use crate::app::App;
use crate::keybindings::get_keybinding_hints;

pub fn render_status_bar(frame: &mut Frame, area: Rect, app: &App) {
    let hints = get_keybinding_hints(&app.input_mode);
    let hint_spans: Vec<Span> = hints
        .iter()
        .flat_map(|(key, desc)| {
            vec![
                Span::styled(
                    format!("[{}]", key),
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
                ),
                Span::styled(format!(" {} ", desc), Style::default().fg(Color::White)),
                Span::raw(" "),
            ]
        })
        .collect();

    let mut line_spans = hint_spans;

    // Add unsaved changes indicator
    if app.unsaved_changes {
        line_spans.push(Span::styled(
            " [UNSAVED] ",
            Style::default()
                .fg(Color::Red)
                .add_modifier(Modifier::BOLD),
        ));
    }

    // Add message if present
    if let Some(msg) = &app.message {
        line_spans.push(Span::styled(
            format!(" {} ", msg),
            Style::default().fg(Color::Green),
        ));
    }

    let paragraph = Paragraph::new(Line::from(line_spans)).block(
        Block::default()
            .borders(Borders::TOP)
            .border_style(Style::default().fg(Color::DarkGray)),
    );

    frame.render_widget(paragraph, area);
}
