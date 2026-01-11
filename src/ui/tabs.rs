use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Tabs},
};

use crate::app::{App, Tab};

pub fn render_tabs(frame: &mut Frame, area: Rect, app: &App) {
    let titles: Vec<Line> = Tab::all()
        .iter()
        .map(|t| {
            let style = if *t == app.tab {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            Line::from(Span::styled(t.title(), style))
        })
        .collect();

    let tabs = Tabs::new(titles)
        .block(
            Block::default()
                .borders(Borders::BOTTOM)
                .title(" Claude Guide ")
                .title_style(
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
        )
        .select(app.tab.index())
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().fg(Color::Yellow))
        .divider(Span::raw(" | "));

    frame.render_widget(tabs, area);
}
