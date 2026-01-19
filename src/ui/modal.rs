use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
};

use crate::app::{App, HookType, ModalType};
use crate::config::{AgentType, HookEvent, PRESET_MCP_SERVERS};
use crate::version::SkillVersion;

/// Returns a highlighted style if selected, otherwise a default white style
fn field_style(is_selected: bool) -> Style {
    match is_selected {
        true => Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
        false => Style::default().fg(Color::White),
    }
}

/// Returns a cursor span if the field is selected, otherwise an empty span
fn cursor_span(is_selected: bool) -> Span<'static> {
    match is_selected {
        true => Span::styled("_", Style::default().fg(Color::Yellow)),
        false => Span::raw(""),
    }
}

/// Returns the value or a placeholder if empty
fn display_or_placeholder<'a>(value: &'a str, placeholder: &'a str) -> &'a str {
    if value.is_empty() {
        placeholder
    } else {
        value
    }
}

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
            ModalType::AddProfile { name } => {
                render_simple_input_modal(frame, area, "Create Profile", "Name:", name);
            }
            ModalType::AddMcp {
                name,
                command,
                args,
            } => {
                render_add_mcp_modal(frame, area, name, command, args, app.modal_index);
            }
            ModalType::SelectMcpPreset { selected_index } => {
                render_select_mcp_preset_modal(frame, area, *selected_index);
            }
            ModalType::ChangePlatform { selected_index } => {
                render_change_platform_modal(frame, area, *selected_index);
            }
            ModalType::ManageVersions {
                skill_name,
                versions,
                selected_index,
            } => {
                render_manage_versions_modal(frame, area, skill_name, versions, *selected_index);
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
    let target_display = display_or_placeholder(target, "<enter value>");

    let content = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("Event: ", Style::default().fg(Color::Cyan)),
            Span::styled(format!("< {} >", event), field_style(selected_field == 0)),
        ]),
        Line::from(Span::styled(
            "(j/k to change)",
            Style::default().fg(Color::DarkGray),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("Type: ", Style::default().fg(Color::Cyan)),
            Span::styled(
                format!("< {} >", hook_type.as_str()),
                field_style(selected_field == 1),
            ),
        ]),
        Line::from(Span::styled(
            "(Space to toggle)",
            Style::default().fg(Color::DarkGray),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("Target: ", Style::default().fg(Color::Cyan)),
            Span::styled(target_display, field_style(selected_field == 2)),
            cursor_span(selected_field == 2),
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

fn render_simple_input_modal(frame: &mut Frame, area: Rect, title: &str, label: &str, value: &str) {
    let display_value = display_or_placeholder(value, "<enter name>");
    let content = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled(format!("{} ", label), Style::default().fg(Color::Cyan)),
            Span::styled(display_value, Style::default().fg(Color::White)),
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
    let name_display = display_or_placeholder(name, "<server name>");
    let command_display = display_or_placeholder(command, "<command>");
    let args_display = display_or_placeholder(args, "<space separated>");

    let content = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("Name: ", Style::default().fg(Color::Cyan)),
            Span::styled(name_display, field_style(selected_field == 0)),
            cursor_span(selected_field == 0),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Command: ", Style::default().fg(Color::Cyan)),
            Span::styled(command_display, field_style(selected_field == 1)),
            cursor_span(selected_field == 1),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Args: ", Style::default().fg(Color::Cyan)),
            Span::styled(args_display, field_style(selected_field == 2)),
            cursor_span(selected_field == 2),
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

fn render_select_mcp_preset_modal(frame: &mut Frame, _area: Rect, selected_index: usize) {
    let larger_area = centered_rect(70, 80, frame.area());
    frame.render_widget(Clear, larger_area);

    let mut content = vec![
        Line::from(""),
        Line::from(Span::styled(
            "Select an MCP Server to install:",
            Style::default().fg(Color::Cyan),
        )),
        Line::from(""),
    ];

    // Add preset servers
    for (i, preset) in PRESET_MCP_SERVERS.iter().enumerate() {
        let is_selected = i == selected_index;
        let prefix = if is_selected { "> " } else { "  " };
        let style = field_style(is_selected);

        content.push(Line::from(vec![
            Span::styled(prefix, style),
            Span::styled(preset.name, style),
            Span::styled(" - ", Style::default().fg(Color::DarkGray)),
            Span::styled(preset.description, Style::default().fg(Color::DarkGray)),
        ]));
    }

    // Add "Custom..." option
    let is_custom_selected = selected_index == PRESET_MCP_SERVERS.len();
    let prefix = if is_custom_selected { "> " } else { "  " };
    let style = field_style(is_custom_selected);

    content.push(Line::from(""));
    content.push(Line::from(vec![
        Span::styled(prefix, style),
        Span::styled("Custom...", style),
        Span::styled(
            " - Enter custom MCP server details",
            Style::default().fg(Color::DarkGray),
        ),
    ]));

    content.push(Line::from(""));
    content.push(Line::from(""));
    content.push(Line::from(vec![
        Span::styled("[j/k]", Style::default().fg(Color::Green)),
        Span::raw(" Navigate  "),
        Span::styled("[Enter]", Style::default().fg(Color::Green)),
        Span::raw(" Select  "),
        Span::styled("[Esc]", Style::default().fg(Color::Red)),
        Span::raw(" Cancel"),
    ]));

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

    frame.render_widget(paragraph, larger_area);
}

fn render_change_platform_modal(frame: &mut Frame, _area: Rect, selected_index: usize) {
    let larger_area = centered_rect(60, 60, frame.area());
    frame.render_widget(Clear, larger_area);

    let mut content = vec![
        Line::from(""),
        Line::from(Span::styled(
            "Select Agent Platform:",
            Style::default().fg(Color::Cyan),
        )),
        Line::from(""),
    ];

    for (i, agent) in AgentType::all().iter().enumerate() {
        let is_selected = i == selected_index;
        let prefix = if is_selected { "> " } else { "  " };
        let style = field_style(is_selected);

        content.push(Line::from(vec![
            Span::styled(prefix, style),
            Span::styled(agent.display_name(), style),
            Span::styled(" - ", Style::default().fg(Color::DarkGray)),
            Span::styled(agent.short_name(), Style::default().fg(Color::DarkGray)),
        ]));
    }

    content.push(Line::from(""));
    content.push(Line::from(""));
    content.push(Line::from(vec![
        Span::styled("[j/k]", Style::default().fg(Color::Green)),
        Span::raw(" Navigate  "),
        Span::styled("[Enter]", Style::default().fg(Color::Green)),
        Span::raw(" Select  "),
        Span::styled("[Esc]", Style::default().fg(Color::Red)),
        Span::raw(" Cancel"),
    ]));

    let paragraph = Paragraph::new(content)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow))
                .title(" Switch Platform ")
                .title_style(
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
        )
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, larger_area);
}

fn render_manage_versions_modal(
    frame: &mut Frame,
    _area: Rect,
    skill_name: &str,
    versions: &[SkillVersion],
    selected_index: usize,
) {
    let larger_area = centered_rect(70, 70, frame.area());
    frame.render_widget(Clear, larger_area);

    let mut content = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("Versions for ", Style::default().fg(Color::Cyan)),
            Span::styled(
                skill_name,
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
    ];

    if versions.is_empty() {
        content.push(Line::from(Span::styled(
            "No versions found.",
            Style::default().fg(Color::DarkGray),
        )));
    } else {
        for (i, version) in versions.iter().enumerate() {
            let is_selected = i == selected_index;
            let prefix = if is_selected { "> " } else { "  " };
            let style = field_style(is_selected);

            content.push(Line::from(vec![
                Span::styled(prefix, style),
                Span::styled(format!("v{}", version.version), style),
                Span::styled(" - ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    version.installed_at.format("%Y-%m-%d %H:%M").to_string(),
                    Style::default().fg(Color::DarkGray),
                ),
            ]));
        }
    }

    content.push(Line::from(""));
    content.push(Line::from(""));
    content.push(Line::from(vec![
        Span::styled("[j/k]", Style::default().fg(Color::Green)),
        Span::raw(" Navigate  "),
        Span::styled("[Enter]", Style::default().fg(Color::Green)),
        Span::raw(" Switch  "),
        Span::styled("[d]", Style::default().fg(Color::Red)),
        Span::raw(" Delete  "),
        Span::styled("[Esc]", Style::default().fg(Color::Red)),
        Span::raw(" Close"),
    ]));

    let paragraph = Paragraph::new(content)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow))
                .title(" Manage Versions ")
                .title_style(
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
        )
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, larger_area);
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
            Span::styled("  s       ", Style::default().fg(Color::Yellow)),
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
