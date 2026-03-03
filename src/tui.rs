use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
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
            let event = event::read()?;
            if let Event::Key(key) = event {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => {
                        app.should_quit = true;
                    }
                    KeyCode::Up | KeyCode::Char('k') => app.move_up(),
                    KeyCode::Down | KeyCode::Char('j') => app.move_down(),
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
                }
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
