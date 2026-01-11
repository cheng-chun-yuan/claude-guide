use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
};

use crate::app::{App, HookType, ModalType};
use crate::config::HookEvent;

pub fn render_modal(frame: &mut Frame, app: &App) {
    if let Some(modal) = &app.modal {
        let area = centered_rect(60, 50, frame.area());

        // Clear the area behind the modal
        frame.render_widget(Clear, area);

        match modal {
            ModalType::Confirm { title, message, .. } => {
                render_confirm_modal(frame, area, title, message);
            }
            ModalType::AddHook {
                event,
                hook_type,
                target,
            } => {
                render_add_hook_modal(frame, area, event, hook_type, target, app.modal_index);
            }
            ModalType::AddSkill { name } => {
                render_simple_input_modal(frame, area, "Add Skill", "Name:", name);
            }
            ModalType::AddCommand { name } => {
                render_simple_input_modal(frame, area, "Add Command", "Name:", name);
            }
            ModalType::AddAgent { name } => {
                render_simple_input_modal(frame, area, "Add Agent", "Name:", name);
            }
            ModalType::AddMcp {
                name,
                command,
                args,
            } => {
                render_add_mcp_modal(frame, area, name, command, args, app.modal_index);
            }
            ModalType::Help => {
                render_help_modal(frame, area);
            }
        }
    }
}

fn render_confirm_modal(frame: &mut Frame, area: Rect, title: &str, message: &str) {
    let content = vec![
        Line::from(""),
        Line::from(Span::styled(message, Style::default().fg(Color::White))),
        Line::from(""),
        Line::from(""),
        Line::from(vec![
            Span::styled("[y]", Style::default().fg(Color::Green)),
            Span::raw(" Yes  "),
            Span::styled("[n]", Style::default().fg(Color::Red)),
            Span::raw(" No"),
        ]),
    ];

    let paragraph = Paragraph::new(content)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow))
                .title(format!(" {} ", title))
                .title_style(
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
        )
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, area);
}

fn render_add_hook_modal(
    frame: &mut Frame,
    area: Rect,
    event: &HookEvent,
    hook_type: &HookType,
    target: &str,
    selected_field: usize,
) {
    let event_style = if selected_field == 0 {
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::White)
    };

    let type_style = if selected_field == 1 {
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::White)
    };

    let target_style = if selected_field == 2 {
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::White)
    };

    let content = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("Event: ", Style::default().fg(Color::Cyan)),
            Span::styled(format!("< {} >", event), event_style),
        ]),
        Line::from(Span::styled(
            "(j/k to change)",
            Style::default().fg(Color::DarkGray),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("Type: ", Style::default().fg(Color::Cyan)),
            Span::styled(format!("< {} >", hook_type.as_str()), type_style),
        ]),
        Line::from(Span::styled(
            "(Space to toggle)",
            Style::default().fg(Color::DarkGray),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("Target: ", Style::default().fg(Color::Cyan)),
            Span::styled(
                if target.is_empty() {
                    "<enter value>"
                } else {
                    target
                },
                target_style,
            ),
            if selected_field == 2 {
                Span::styled("_", Style::default().fg(Color::Yellow))
            } else {
                Span::raw("")
            },
        ]),
        Line::from(""),
        Line::from(""),
        Line::from(vec![
            Span::styled("[Tab]", Style::default().fg(Color::Green)),
            Span::raw(" Next field  "),
            Span::styled("[Enter]", Style::default().fg(Color::Green)),
            Span::raw(" Save  "),
            Span::styled("[Esc]", Style::default().fg(Color::Red)),
            Span::raw(" Cancel"),
        ]),
    ];

    let paragraph = Paragraph::new(content)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow))
                .title(" Add Hook ")
                .title_style(
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
        )
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, area);
}

fn render_simple_input_modal(
    frame: &mut Frame,
    area: Rect,
    title: &str,
    label: &str,
    value: &str,
) {
    let content = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled(format!("{} ", label), Style::default().fg(Color::Cyan)),
            Span::styled(
                if value.is_empty() { "<enter name>" } else { value },
                Style::default().fg(Color::White),
            ),
            Span::styled("_", Style::default().fg(Color::Yellow)),
        ]),
        Line::from(""),
        Line::from(""),
        Line::from(vec![
            Span::styled("[Enter]", Style::default().fg(Color::Green)),
            Span::raw(" Create  "),
            Span::styled("[Esc]", Style::default().fg(Color::Red)),
            Span::raw(" Cancel"),
        ]),
    ];

    let paragraph = Paragraph::new(content)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow))
                .title(format!(" {} ", title))
                .title_style(
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
        )
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, area);
}

fn render_add_mcp_modal(
    frame: &mut Frame,
    area: Rect,
    name: &str,
    command: &str,
    args: &str,
    selected_field: usize,
) {
    let name_style = if selected_field == 0 {
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::White)
    };

    let command_style = if selected_field == 1 {
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::White)
    };

    let args_style = if selected_field == 2 {
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::White)
    };

    let cursor = |idx: usize| {
        if selected_field == idx {
            Span::styled("_", Style::default().fg(Color::Yellow))
        } else {
            Span::raw("")
        }
    };

    let content = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("Name: ", Style::default().fg(Color::Cyan)),
            Span::styled(
                if name.is_empty() { "<server name>" } else { name },
                name_style,
            ),
            cursor(0),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Command: ", Style::default().fg(Color::Cyan)),
            Span::styled(
                if command.is_empty() {
                    "<command>"
                } else {
                    command
                },
                command_style,
            ),
            cursor(1),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Args: ", Style::default().fg(Color::Cyan)),
            Span::styled(
                if args.is_empty() {
                    "<space separated>"
                } else {
                    args
                },
                args_style,
            ),
            cursor(2),
        ]),
        Line::from(""),
        Line::from(""),
        Line::from(vec![
            Span::styled("[Tab]", Style::default().fg(Color::Green)),
            Span::raw(" Next  "),
            Span::styled("[Enter]", Style::default().fg(Color::Green)),
            Span::raw(" Save  "),
            Span::styled("[Esc]", Style::default().fg(Color::Red)),
            Span::raw(" Cancel"),
        ]),
    ];

    let paragraph = Paragraph::new(content)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow))
                .title(" Add MCP Server ")
                .title_style(
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
        )
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, area);
}

fn render_help_modal(frame: &mut Frame, area: Rect) {
    let content = vec![
        Line::from(""),
        Line::from(Span::styled(
            "Navigation",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(vec![
            Span::styled("  j/k     ", Style::default().fg(Color::Yellow)),
            Span::raw("Move up/down"),
        ]),
        Line::from(vec![
            Span::styled("  h/l     ", Style::default().fg(Color::Yellow)),
            Span::raw("Switch focus left/right"),
        ]),
        Line::from(vec![
            Span::styled("  Tab     ", Style::default().fg(Color::Yellow)),
            Span::raw("Next tab"),
        ]),
        Line::from(vec![
            Span::styled("  1-6     ", Style::default().fg(Color::Yellow)),
            Span::raw("Jump to tab"),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "Actions",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(vec![
            Span::styled("  a       ", Style::default().fg(Color::Yellow)),
            Span::raw("Add new item"),
        ]),
        Line::from(vec![
            Span::styled("  e       ", Style::default().fg(Color::Yellow)),
            Span::raw("Edit selected item"),
        ]),
        Line::from(vec![
            Span::styled("  d       ", Style::default().fg(Color::Yellow)),
            Span::raw("Delete selected item"),
        ]),
        Line::from(vec![
            Span::styled("  Space   ", Style::default().fg(Color::Yellow)),
            Span::raw("Toggle (plugins)"),
        ]),
        Line::from(vec![
            Span::styled("  r       ", Style::default().fg(Color::Yellow)),
            Span::raw("Refresh"),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "Application",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(vec![
            Span::styled("  Ctrl+S  ", Style::default().fg(Color::Yellow)),
            Span::raw("Save settings"),
        ]),
        Line::from(vec![
            Span::styled("  q       ", Style::default().fg(Color::Yellow)),
            Span::raw("Quit"),
        ]),
        Line::from(vec![
            Span::styled("  ?       ", Style::default().fg(Color::Yellow)),
            Span::raw("Show this help"),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "Press any key to close",
            Style::default().fg(Color::DarkGray),
        )),
    ];

    let paragraph = Paragraph::new(content)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan))
                .title(" Help ")
                .title_style(
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
        )
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, area);
}

/// Helper function to create a centered rectangle
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
