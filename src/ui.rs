use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
};

use crate::app::App;
use crate::types::LogStream;

pub fn render(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Min(0),
                Constraint::Length(1),
                Constraint::Length(1),
            ]
            .as_ref(),
        )
        .split(f.area());

    let top_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(25), Constraint::Percentage(75)].as_ref())
        .split(chunks[0]);

    // Left pane: Hosts
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
                display_text = format!("{} [running]", host);
                if i != app.selected_host_index {
                    style = style.fg(Color::Magenta).add_modifier(Modifier::BOLD);
                }
            }

            ListItem::new(display_text).style(style)
        })
        .collect();

    let hosts_list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Hosts (LazyNixOS) "),
    );
    f.render_widget(hosts_list, top_chunks[0]);

    // Right pane: Logs (per-host)
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
        Some(host) => format!(" Logs — {} ", host),
        None => " Logs ".to_string(),
    };

    let logs_view = Paragraph::new(log_lines)
        .block(Block::default().borders(Borders::ALL).title(log_title))
        .wrap(Wrap { trim: false })
        .scroll((
            app.current_log_scroll(top_chunks[1].height.saturating_sub(2)),
            0,
        ));

    f.render_widget(logs_view, top_chunks[1]);

    // Bottom pane 1: Status
    let status_text = if let Some(err) = &app.error_msg {
        Span::styled(
            format!(" ERROR: {} ", err),
            Style::default()
                .bg(Color::Red)
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )
    } else if !app.running_actions.is_empty() {
        let running_hosts: Vec<String> = app.running_actions.keys().cloned().collect();
        Span::styled(
            format!(
                " RUNNING ({}): {} ",
                running_hosts.len(),
                running_hosts.join(", ")
            ),
            Style::default()
                .bg(Color::Magenta)
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )
    } else if !app.status_msg.is_empty() {
        Span::styled(
            format!(" {} ", app.status_msg),
            Style::default()
                .bg(Color::Green)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        )
    } else {
        Span::styled(" Idle ", Style::default().fg(Color::DarkGray))
    };

    let status_bar = Paragraph::new(Line::from(status_text));
    f.render_widget(status_bar, chunks[1]);

    // Bottom pane 2: Help bar
    let help_spans = vec![
        Span::styled("[↑/↓] ", Style::default().fg(Color::DarkGray)),
        Span::styled("Navigate  ", Style::default().fg(Color::Gray)),
        Span::styled("[PgUp/PgDn] ", Style::default().fg(Color::DarkGray)),
        Span::styled("Logs  ", Style::default().fg(Color::Gray)),
        Span::styled("[Enter] ", Style::default().fg(Color::Cyan)),
        Span::styled("Switch  ", Style::default().fg(Color::Gray)),
        Span::styled("[b] ", Style::default().fg(Color::Yellow)),
        Span::styled("Build  ", Style::default().fg(Color::Gray)),
        Span::styled("[d] ", Style::default().fg(Color::Yellow)),
        Span::styled("Dry-Build  ", Style::default().fg(Color::Gray)),
        Span::styled("[q] ", Style::default().fg(Color::Yellow)),
        Span::styled("Quit", Style::default().fg(Color::Gray)),
    ];
    let help_bar = Paragraph::new(Line::from(help_spans));
    f.render_widget(help_bar, chunks[2]);
}
