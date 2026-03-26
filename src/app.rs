use std::collections::{HashMap, VecDeque};

use ratatui_image::protocol::StatefulProtocol;

use crate::types::{AppEvent, LogLine, RebuildAction};

pub struct App {
    pub hosts: Vec<String>,
    pub selected_host_index: usize,
    pub host_logs: HashMap<String, VecDeque<LogLine>>,
    pub running_actions: HashMap<String, RebuildAction>,
    pub status_msg: String,
    pub error_msg: Option<String>,
    pub log_scroll: usize,
    pub follow_logs: bool,
    pub should_quit: bool,
    pub image_state: Option<StatefulProtocol>,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    pub fn new() -> Self {
        Self {
            hosts: Vec::new(),
            selected_host_index: 0,
            host_logs: HashMap::new(),
            running_actions: HashMap::new(),
            status_msg: "Loading hosts...".to_string(),
            error_msg: None,
            log_scroll: 0,
            follow_logs: true,
            should_quit: false,
            image_state: None,
        }
    }

    pub fn selected_host(&self) -> Option<&String> {
        self.hosts.get(self.selected_host_index)
    }

    pub fn move_up(&mut self) {
        if self.selected_host_index > 0 {
            self.selected_host_index -= 1;
            self.reset_log_scroll();
        }
    }

    pub fn move_down(&mut self) {
        if !self.hosts.is_empty() && self.selected_host_index < self.hosts.len() - 1 {
            self.selected_host_index += 1;
            self.reset_log_scroll();
        }
    }

    pub fn select_host(&mut self, index: usize) {
        if index < self.hosts.len() && index != self.selected_host_index {
            self.selected_host_index = index;
            self.reset_log_scroll();
        }
    }

    /// Handle a left click in the hosts list area.
    /// `list_area` is the full block area of the hosts list widget.
    /// `click_x`, `click_y` are the absolute terminal coordinates of the click.
    pub fn handle_hosts_click(
        &mut self,
        click_x: u16,
        click_y: u16,
        list_area: ratatui::layout::Rect,
    ) {
        if self.hosts.is_empty() {
            return;
        }

        // Inside block borders?
        if click_x <= list_area.x
            || click_x >= list_area.x + list_area.width - 1
            || click_y <= list_area.y
            || click_y >= list_area.y + list_area.height - 1
        {
            return;
        }

        let inner_height = list_area.height.saturating_sub(2) as usize;
        let local_y = (click_y - list_area.y - 1) as usize;
        let offset = if self.hosts.len() > inner_height {
            self.hosts.len() - inner_height
        } else {
            0
        };
        let index = local_y + offset;

        if index < self.hosts.len() {
            self.select_host(index);
        }
    }

    pub fn apply_event(&mut self, event: AppEvent) {
        match event {
            AppEvent::HostsLoaded(Ok(hosts)) => {
                self.hosts = hosts;
                self.status_msg = String::new();
            }
            AppEvent::HostsLoaded(Err(e)) => {
                self.error_msg = Some(e);
                self.status_msg = String::new();
            }
            AppEvent::Log { host, line } => {
                let is_selected_host = self
                    .selected_host()
                    .is_some_and(|selected_host| selected_host == &host);
                let logs = self.host_logs.entry(host).or_default();
                logs.push_back(line);
                if logs.len() > 1000 {
                    logs.pop_front();
                    if is_selected_host && !self.follow_logs {
                        self.log_scroll = self.log_scroll.saturating_sub(1);
                    }
                }
            }
            AppEvent::CommandStarted { host, action } => {
                self.running_actions.insert(host.clone(), action);
                self.status_msg = format!("Started on {}", host);
            }
            AppEvent::CommandFinished { host, success, .. } => {
                self.running_actions.remove(&host);
                if success {
                    self.status_msg = format!("{} finished successfully", host);
                } else {
                    self.status_msg = format!("{} failed", host);
                }
            }
            AppEvent::CommandErrored { host, error, .. } => {
                self.running_actions.remove(&host);
                self.error_msg = Some(format!("{}: {}", host, error));
            }
        }
    }

    pub fn can_start_action(&self) -> bool {
        if self.hosts.is_empty() {
            return false;
        }
        if let Some(host) = self.selected_host() {
            !self.running_actions.contains_key(host)
        } else {
            false
        }
    }

    pub fn selected_host_logs(&self) -> Option<&VecDeque<LogLine>> {
        self.selected_host()
            .and_then(|host| self.host_logs.get(host))
    }

    pub fn current_log_scroll(&self, viewport_height: u16) -> u16 {
        let max_scroll = self.max_log_scroll(viewport_height);
        let scroll = if self.follow_logs {
            max_scroll
        } else {
            self.log_scroll.min(max_scroll)
        };

        scroll.min(u16::MAX as usize) as u16
    }

    pub fn scroll_logs_up(&mut self, viewport_height: u16, lines: usize) {
        let max_scroll = self.max_log_scroll(viewport_height);
        if max_scroll == 0 {
            self.reset_log_scroll();
            return;
        }

        let current = self.current_log_scroll(viewport_height) as usize;
        self.log_scroll = current.saturating_sub(lines);
        self.follow_logs = false;
    }

    pub fn scroll_logs_down(&mut self, viewport_height: u16, lines: usize) {
        let max_scroll = self.max_log_scroll(viewport_height);
        if max_scroll == 0 {
            self.reset_log_scroll();
            return;
        }

        let current = self.current_log_scroll(viewport_height) as usize;
        let next = current.saturating_add(lines).min(max_scroll);
        self.log_scroll = next;
        self.follow_logs = next == max_scroll;
    }

    pub fn scroll_logs_to_top(&mut self, viewport_height: u16) {
        if self.max_log_scroll(viewport_height) == 0 {
            self.reset_log_scroll();
            return;
        }

        self.log_scroll = 0;
        self.follow_logs = false;
    }

    pub fn scroll_logs_to_bottom(&mut self) {
        self.log_scroll = 0;
        self.follow_logs = true;
    }

    fn reset_log_scroll(&mut self) {
        self.scroll_logs_to_bottom();
    }

    fn max_log_scroll(&self, viewport_height: u16) -> usize {
        self.selected_host_logs()
            .map(|logs| logs.len().saturating_sub(viewport_height as usize))
            .unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::App;
    use crate::types::{AppEvent, LogLine, LogStream};

    fn log_line(text: &str) -> LogLine {
        LogLine {
            stream: LogStream::Stdout,
            text: text.to_string(),
        }
    }

    fn app_with_logs(count: usize) -> App {
        let mut app = App::new();
        app.hosts = vec!["host-a".to_string()];

        for i in 0..count {
            app.apply_event(AppEvent::Log {
                host: "host-a".to_string(),
                line: log_line(&format!("line-{i}")),
            });
        }

        app
    }

    #[test]
    fn follows_bottom_by_default() {
        let app = app_with_logs(8);

        assert!(app.follow_logs);
        assert_eq!(app.current_log_scroll(4), 4);
    }

    #[test]
    fn scroll_up_and_down_updates_follow_mode() {
        let mut app = app_with_logs(10);

        app.scroll_logs_up(4, 2);
        assert!(!app.follow_logs);
        assert_eq!(app.current_log_scroll(4), 4);

        app.scroll_logs_down(4, 2);
        assert!(app.follow_logs);
        assert_eq!(app.current_log_scroll(4), 6);
    }

    #[test]
    fn trimming_preserves_viewport_when_scrolled_up() {
        let mut app = app_with_logs(1000);

        app.scroll_logs_up(5, 3);
        assert_eq!(app.current_log_scroll(5), 992);

        app.apply_event(AppEvent::Log {
            host: "host-a".to_string(),
            line: log_line("new-line"),
        });

        assert_eq!(app.current_log_scroll(5), 991);
        assert!(!app.follow_logs);
    }
}
