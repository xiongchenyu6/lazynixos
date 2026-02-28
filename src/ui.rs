use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
};

use crate::app::App;
use crate::types::{LogStream, RebuildAction};

pub fn render(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(3)].as_ref())
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
            let style = if i == app.selected_host_index {
                Style::default().bg(Color::Blue).fg(Color::White)
            } else {
                Style::default()
            };
            ListItem::new(host.as_str()).style(style)
        })
        .collect();

    let hosts_list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Hosts (LazyNixOS) "),
    );
    f.render_widget(hosts_list, top_chunks[0]);

    // Right pane: Logs
    let log_lines: Vec<Line> = app
        .logs
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

    let logs_view = Paragraph::new(log_lines)
        .block(Block::default().borders(Borders::ALL).title(" Logs "))
        .wrap(Wrap { trim: false })
        .scroll((
            (app.logs
                .len()
                .saturating_sub(top_chunks[1].height.saturating_sub(2) as usize))
                as u16,
            0,
        ));

    f.render_widget(logs_view, top_chunks[1]);

    // Bottom pane: Status / Keys
    let status_text = if let Some(err) = &app.error_msg {
        Span::styled(format!("ERROR: {}", err), Style::default().fg(Color::Red))
    } else if let Some((host, action)) = &app.running_action {
        Span::styled(
            format!(" RUNNING: {} on {} ", action, host),
            Style::default()
                .bg(Color::Magenta)
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )
    } else if !app.status_msg.is_empty() {
        Span::styled(
            format!(" {} ", app.status_msg),
            Style::default().fg(Color::Green),
        )
    } else {
        Span::raw(" Enter: switch | b: build | d: dry-build | ↑/↓: navigate | q: quit ")
    };

    let status_bar = Paragraph::new(Line::from(status_text))
        .block(Block::default().borders(Borders::ALL).title(" Status "));
    f.render_widget(status_bar, chunks[1]);
}
