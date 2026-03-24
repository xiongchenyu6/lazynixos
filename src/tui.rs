use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, MouseEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::path::PathBuf;
use std::time::Duration;
use tokio::sync::mpsc;

use crate::app::App;
use crate::types::{AppEvent, RebuildAction};
use crate::{cmd, ui};

pub async fn run(flake_dir: PathBuf) -> anyhow::Result<()> {
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let (tx, mut rx) = mpsc::channel(100);
    let mut app = App::new();

    let tx_discover = tx.clone();
    let flake_dir_clone = flake_dir.clone();
    tokio::spawn(async move {
        match cmd::discover_hosts(&flake_dir_clone).await {
            Ok(hosts) => {
                let _ = tx_discover.send(AppEvent::HostsLoaded(Ok(hosts))).await;
            }
            Err(e) => {
                let _ = tx_discover
                    .send(AppEvent::HostsLoaded(Err(e.to_string())))
                    .await;
            }
        }
    });

    loop {
        terminal.draw(|f| ui::render(f, &app))?;

        if event::poll(Duration::from_millis(50))? {
            let log_viewport_height = terminal.size()?.height.saturating_sub(4);
            let event = event::read()?;
            match event {
                Event::Key(key) => match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => {
                        app.should_quit = true;
                    }
                    KeyCode::Up | KeyCode::Char('k') => app.move_up(),
                    KeyCode::Down | KeyCode::Char('j') => app.move_down(),
                    KeyCode::PageUp => app.scroll_logs_up(log_viewport_height, 10),
                    KeyCode::PageDown => app.scroll_logs_down(log_viewport_height, 10),
                    KeyCode::Home => app.scroll_logs_to_top(log_viewport_height),
                    KeyCode::End => app.scroll_logs_to_bottom(),
                    KeyCode::Enter => {
                        if app.can_start_action() {
                            let host = app.selected_host().cloned();
                            if let Some(h) = host {
                                tokio::spawn(cmd::run_rebuild(
                                    flake_dir.clone(),
                                    h,
                                    RebuildAction::Switch,
                                    tx.clone(),
                                ));
                            }
                        }
                    }
                    KeyCode::Char('b') => {
                        if app.can_start_action() {
                            let host = app.selected_host().cloned();
                            if let Some(h) = host {
                                tokio::spawn(cmd::run_rebuild(
                                    flake_dir.clone(),
                                    h,
                                    RebuildAction::Build,
                                    tx.clone(),
                                ));
                            }
                        }
                    }
                    KeyCode::Char('d') => {
                        if app.can_start_action() {
                            let host = app.selected_host().cloned();
                            if let Some(h) = host {
                                tokio::spawn(cmd::run_rebuild(
                                    flake_dir.clone(),
                                    h,
                                    RebuildAction::DryBuild,
                                    tx.clone(),
                                ));
                            }
                        }
                    }
                    _ => {}
                },
                Event::Mouse(mouse) => match mouse.kind {
                    MouseEventKind::ScrollUp => app.scroll_logs_up(log_viewport_height, 3),
                    MouseEventKind::ScrollDown => app.scroll_logs_down(log_viewport_height, 3),
                    _ => {}
                },
                _ => {}
            }
        }

        while let Ok(event) = rx.try_recv() {
            app.apply_event(event);
        }

        if app.should_quit {
            break;
        }
    }

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}
