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
        .constraints([Constraint::Min(0), Constraint::Length(1), Constraint::Length(1)].as_ref())
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

    // Middle pane: Status
    let status_text = if let Some(err) = &app.error_msg {
        Span::styled(format!(" ERROR: {} ", err), Style::default().bg(Color::Red).fg(Color::White).add_modifier(Modifier::BOLD))
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
            Style::default().bg(Color::Green).fg(Color::Black),
        )
    } else {
        Span::raw(" Idle ")
    };

    let status_bar = Paragraph::new(Line::from(status_text));
    f.render_widget(status_bar, chunks[1]);

    // Bottom pane: Help
    let help_spans = vec![
        Span::styled("[↑/↓]", Style::default().fg(Color::DarkGray)),
        Span::styled(" Navigate  ", Style::default().fg(Color::Gray)),
        Span::styled("[Enter]", Style::default().fg(Color::Cyan)),
        Span::styled(" Switch  ", Style::default().fg(Color::Gray)),
        Span::styled("[b]", Style::default().fg(Color::Yellow)),
        Span::styled(" Build  ", Style::default().fg(Color::Gray)),
        Span::styled("[d]", Style::default().fg(Color::Yellow)),
        Span::styled(" Dry-Build  ", Style::default().fg(Color::Gray)),
        Span::styled("[q]", Style::default().fg(Color::Yellow)),
        Span::styled(" Quit", Style::default().fg(Color::Gray)),
    ];
    let help_bar = Paragraph::new(Line::from(help_spans));
    f.render_widget(help_bar, chunks[2]);
}
