use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
};
use ratatui_image::StatefulImage;

use crate::app::App;
use crate::types::LogStream;

/// Render the full UI. Returns the `Rect` of the hosts list block (for mouse click mapping).
pub fn render(f: &mut Frame, app: &mut App) -> Rect {
    let outer = f.area();

    // 3-row vertical: [main content] [status] [help]
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![
            Constraint::Min(0),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .split(outer);

    // Determine header height based on whether we have an image
    let header_height: u16 = if app.image_state.is_some() { 8 } else { 0 };

    // Split main area into header + body
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(if header_height > 0 {
            vec![Constraint::Length(header_height), Constraint::Min(0)]
        } else {
            vec![Constraint::Length(0), Constraint::Min(0)]
        })
        .split(chunks[0]);

    // --- Header: Image + Title ---
    if app.image_state.is_some() && main_chunks[0].height > 0 {
        render_header(f, app, main_chunks[0]);
    }

    // --- Body: hosts (left) + logs (right) ---
    let body_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![Constraint::Percentage(25), Constraint::Percentage(75)])
        .split(main_chunks[1]);

    // Left pane: Hosts
    let hosts_area = render_hosts(f, app, body_chunks[0]);

    // Right pane: Logs
    render_logs(f, app, body_chunks[1]);

    // --- Status bar ---
    render_status(f, app, chunks[1]);

    // --- Help bar ---
    render_help(f, chunks[2]);

    hosts_area
}

fn render_header(f: &mut Frame, app: &mut App, area: Rect) {
    let header_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![Constraint::Length(14), Constraint::Min(0)])
        .split(area);

    // Image (left side of header) - needs mutable state
    if let Some(ref mut state) = app.image_state {
        let image_widget = StatefulImage::default();
        f.render_stateful_widget(image_widget, header_chunks[0], state);
    }

    // Title text (right side of header)
    let title_lines = vec![
        Line::from(vec![Span::styled(
            "  NixOS",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![
            Span::styled(
                "  Lazy ",
                Style::default()
                    .fg(Color::Blue)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "TUI",
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  ", Style::default()),
            Span::styled(
                app.hosts.len().to_string(),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" hosts configured", Style::default().fg(Color::Gray)),
        ]),
        Line::from(vec![
            Span::styled("  ", Style::default()),
            if !app.running_actions.is_empty() {
                Span::styled(
                    format!("[ {} active ]", app.running_actions.len()),
                    Style::default()
                        .fg(Color::Magenta)
                        .add_modifier(Modifier::BOLD),
                )
            } else {
                Span::styled("[ ready ]", Style::default().fg(Color::Green))
            },
        ]),
    ];

    let title_widget = Paragraph::new(title_lines);
    f.render_widget(title_widget, header_chunks[1]);
}

fn render_hosts(f: &mut Frame, app: &App, area: Rect) -> Rect {
    let items: Vec<ListItem> = app
        .hosts
        .iter()
        .enumerate()
        .map(|(i, host)| {
            let mut style = if i == app.selected_host_index {
                Style::default().bg(Color::Blue).fg(Color::White)
            } else {
                Style::default()
            };

            let mut display_text = host.clone();
            if app.running_actions.contains_key(host) {
                display_text = format!("\u{25CF} {}", host);
                if i != app.selected_host_index {
                    style = style.fg(Color::Magenta).add_modifier(Modifier::BOLD);
                }
            } else if i == app.selected_host_index {
                display_text = format!("\u{25B6} {}", host);
            }

            ListItem::new(display_text).style(style)
        })
        .collect();

    let hosts_list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Blue))
            .title(Span::styled(
                " Hosts ",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )),
    );
    f.render_widget(hosts_list, area);

    area
}

fn render_logs(f: &mut Frame, app: &App, area: Rect) {
    let selected_host = app.selected_host();
    let empty_logs = std::collections::VecDeque::new();
    let logs = app.selected_host_logs().unwrap_or(&empty_logs);

    let log_lines: Vec<Line> = logs
        .iter()
        .map(|log| {
            let style = match log.stream {
                LogStream::Stdout => Style::default().fg(Color::Gray),
                LogStream::Stderr => Style::default().fg(Color::Red),
                LogStream::System => Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            };
            Line::from(Span::styled(&log.text, style))
        })
        .collect();

    let log_title = match selected_host {
        Some(host) => format!(" Logs \u{2014} {} ", host),
        None => " Logs ".to_string(),
    };

    let follow_indicator = if app.follow_logs {
        Span::styled(" \u{25CF} ", Style::default().fg(Color::Green))
    } else {
        Span::styled(" \u{25A1} ", Style::default().fg(Color::DarkGray))
    };

    let logs_view = Paragraph::new(log_lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray))
                .title(Line::from(vec![
                    Span::styled(
                        log_title,
                        Style::default()
                            .fg(Color::White)
                            .add_modifier(Modifier::BOLD),
                    ),
                    follow_indicator,
                ])),
        )
        .wrap(Wrap { trim: false })
        .scroll((app.current_log_scroll(area.height.saturating_sub(2)), 0));

    f.render_widget(logs_view, area);
}

fn render_status(f: &mut Frame, app: &App, area: Rect) {
    let status_text = if let Some(err) = &app.error_msg {
        Span::styled(
            format!(" \u{2716} {} ", err),
            Style::default()
                .bg(Color::Red)
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )
    } else if !app.running_actions.is_empty() {
        let running_hosts: Vec<String> = app.running_actions.keys().cloned().collect();
        Span::styled(
            format!(" \u{25CF} {} ", running_hosts.join(", ")),
            Style::default()
                .bg(Color::Magenta)
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )
    } else if !app.status_msg.is_empty() {
        Span::styled(
            format!(" \u{2714} {} ", app.status_msg),
            Style::default()
                .bg(Color::Green)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        )
    } else {
        Span::styled(" idle ", Style::default().fg(Color::DarkGray))
    };

    let status_bar = Paragraph::new(Line::from(status_text));
    f.render_widget(status_bar, area);
}

fn render_help(f: &mut Frame, area: Rect) {
    let help_spans = vec![
        Span::styled(
            " \u{2191}/\u{2193} ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("Nav  ", Style::default().fg(Color::Gray)),
        Span::styled(
            "Click ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("Select  ", Style::default().fg(Color::Gray)),
        Span::styled(
            "Scroll ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("Logs  ", Style::default().fg(Color::Gray)),
        Span::styled(
            "Enter ",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("Switch  ", Style::default().fg(Color::Gray)),
        Span::styled(
            "b ",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("Build  ", Style::default().fg(Color::Gray)),
        Span::styled(
            "d ",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("Dry  ", Style::default().fg(Color::Gray)),
        Span::styled(
            "q ",
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        ),
        Span::styled("Quit", Style::default().fg(Color::Gray)),
    ];
    let help_bar = Paragraph::new(Line::from(help_spans));
    f.render_widget(help_bar, area);
}
