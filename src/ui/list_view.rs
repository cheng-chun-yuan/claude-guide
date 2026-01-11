use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, ListState},
};

use crate::app::{App, Focus};

pub fn render_list(frame: &mut Frame, area: Rect, app: &App) {
    let items: Vec<ListItem> = app
        .current_list()
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let style = if i == app.list_index {
                if app.focus == Focus::List {
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                }
            } else {
                Style::default().fg(Color::White)
            };
            ListItem::new(Span::styled(item.display_name(), style))
        })
        .collect();

    let border_style = if app.focus == Focus::List {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(border_style)
                .title(format!(" {} ", app.tab.title()))
                .title_style(Style::default().fg(Color::Cyan)),
        )
        .highlight_symbol("> ");

    let mut state = ListState::default();
    state.select(Some(app.list_index));

    frame.render_stateful_widget(list, area, &mut state);
}
