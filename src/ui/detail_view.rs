use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph, Wrap},
};

use crate::app::{App, Focus, ListItem};

pub fn render_detail(frame: &mut Frame, area: Rect, app: &App) {
    let border_style = if app.focus == Focus::Detail {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let content = if let Some(item) = app.selected_item() {
        format_item_detail(&item)
    } else {
        vec![Line::from(Span::styled(
            "No item selected",
            Style::default().fg(Color::DarkGray),
        ))]
    };

    let paragraph = Paragraph::new(content)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(border_style)
                .title(" Details ")
                .title_style(Style::default().fg(Color::Cyan)),
        )
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, area);
}

fn format_item_detail(item: &ListItem) -> Vec<Line<'static>> {
    match item {
        ListItem::Hook { event, hook, .. } => {
            vec![
                Line::from(vec![
                    Span::styled("Event: ", Style::default().fg(Color::Cyan)),
                    Span::styled(event.to_string(), Style::default().fg(Color::White)),
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::styled("Type: ", Style::default().fg(Color::Cyan)),
                    Span::styled(
                        hook.get_type().to_string(),
                        Style::default().fg(Color::White),
                    ),
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::styled("Target: ", Style::default().fg(Color::Cyan)),
                    Span::styled(hook.get_target(), Style::default().fg(Color::Green)),
                ]),
                Line::from(""),
                if let Some(timeout) = hook.get_timeout() {
                    Line::from(vec![
                        Span::styled("Timeout: ", Style::default().fg(Color::Cyan)),
                        Span::styled(format!("{}ms", timeout), Style::default().fg(Color::White)),
                    ])
                } else {
                    Line::from("")
                },
            ]
        }
        ListItem::Skill(skill) => {
            let mut lines = vec![
                Line::from(vec![
                    Span::styled("Name: ", Style::default().fg(Color::Cyan)),
                    Span::styled(
                        skill.display_name().to_string(),
                        Style::default().fg(Color::White),
                    ),
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::styled("Description: ", Style::default().fg(Color::Cyan)),
                    Span::styled(
                        skill.description().to_string(),
                        Style::default().fg(Color::White),
                    ),
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::styled("Path: ", Style::default().fg(Color::Cyan)),
                    Span::styled(
                        skill.path.display().to_string(),
                        Style::default().fg(Color::DarkGray),
                    ),
                ]),
                Line::from(""),
                Line::from(Span::styled("Content:", Style::default().fg(Color::Cyan))),
                Line::from(""),
            ];

            for line in skill.content.lines().take(20) {
                lines.push(Line::from(Span::styled(
                    line.to_string(),
                    Style::default().fg(Color::White),
                )));
            }

            lines
        }
        ListItem::Plugin(plugin) => {
            vec![
                Line::from(vec![
                    Span::styled("Name: ", Style::default().fg(Color::Cyan)),
                    Span::styled(
                        plugin.display_name().to_string(),
                        Style::default().fg(Color::White),
                    ),
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::styled("Status: ", Style::default().fg(Color::Cyan)),
                    if plugin.enabled {
                        Span::styled("Enabled", Style::default().fg(Color::Green))
                    } else {
                        Span::styled("Disabled", Style::default().fg(Color::Red))
                    },
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::styled("Description: ", Style::default().fg(Color::Cyan)),
                    Span::styled(
                        plugin.description().to_string(),
                        Style::default().fg(Color::White),
                    ),
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::styled("Path: ", Style::default().fg(Color::Cyan)),
                    Span::styled(
                        plugin.path.display().to_string(),
                        Style::default().fg(Color::DarkGray),
                    ),
                ]),
                Line::from(""),
                Line::from(Span::styled(
                    "Press <Space> to toggle",
                    Style::default().fg(Color::Yellow),
                )),
            ]
        }
        ListItem::Command(cmd) => {
            let mut lines = vec![
                Line::from(vec![
                    Span::styled("Name: ", Style::default().fg(Color::Cyan)),
                    Span::styled(cmd.name.clone(), Style::default().fg(Color::White)),
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::styled("Path: ", Style::default().fg(Color::Cyan)),
                    Span::styled(
                        cmd.path.display().to_string(),
                        Style::default().fg(Color::DarkGray),
                    ),
                ]),
                Line::from(""),
                Line::from(Span::styled("Content:", Style::default().fg(Color::Cyan))),
                Line::from(""),
            ];

            for line in cmd.content.lines().take(30) {
                lines.push(Line::from(Span::styled(
                    line.to_string(),
                    Style::default().fg(Color::White),
                )));
            }

            lines
        }
        ListItem::Agent(agent) => {
            let mut lines = vec![
                Line::from(vec![
                    Span::styled("Name: ", Style::default().fg(Color::Cyan)),
                    Span::styled(
                        agent.display_name().to_string(),
                        Style::default().fg(Color::White),
                    ),
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::styled("Description: ", Style::default().fg(Color::Cyan)),
                    Span::styled(
                        agent.description().to_string(),
                        Style::default().fg(Color::White),
                    ),
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::styled("Model: ", Style::default().fg(Color::Cyan)),
                    Span::styled(agent.model().to_string(), Style::default().fg(Color::Green)),
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::styled("Path: ", Style::default().fg(Color::Cyan)),
                    Span::styled(
                        agent.path.display().to_string(),
                        Style::default().fg(Color::DarkGray),
                    ),
                ]),
                Line::from(""),
                Line::from(Span::styled("Content:", Style::default().fg(Color::Cyan))),
                Line::from(""),
            ];

            for line in agent.content.lines().take(20) {
                lines.push(Line::from(Span::styled(
                    line.to_string(),
                    Style::default().fg(Color::White),
                )));
            }

            lines
        }
        ListItem::Mcp { name, server } => {
            let mut lines = vec![
                Line::from(vec![
                    Span::styled("Name: ", Style::default().fg(Color::Cyan)),
                    Span::styled(name.clone(), Style::default().fg(Color::White)),
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::styled("Command: ", Style::default().fg(Color::Cyan)),
                    Span::styled(server.command.clone(), Style::default().fg(Color::Green)),
                ]),
                Line::from(""),
            ];

            if !server.args.is_empty() {
                lines.push(Line::from(vec![
                    Span::styled("Args: ", Style::default().fg(Color::Cyan)),
                    Span::styled(
                        server.args.join(" "),
                        Style::default().fg(Color::White),
                    ),
                ]));
                lines.push(Line::from(""));
            }

            if !server.env.is_empty() {
                lines.push(Line::from(Span::styled(
                    "Environment:",
                    Style::default().fg(Color::Cyan),
                )));
                lines.push(Line::from(""));
                for (key, value) in &server.env {
                    lines.push(Line::from(vec![
                        Span::styled(format!("  {}: ", key), Style::default().fg(Color::Yellow)),
                        Span::styled(value.clone(), Style::default().fg(Color::White)),
                    ]));
                }
            }

            lines
        }
    }
}
