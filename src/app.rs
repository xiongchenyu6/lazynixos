use std::collections::{HashMap, VecDeque};

use crate::types::{AppEvent, LogLine, RebuildAction};

pub struct App {
    pub hosts: Vec<String>,
    pub selected_host_index: usize,
    pub host_logs: HashMap<String, VecDeque<LogLine>>,
    pub running_actions: HashMap<String, RebuildAction>,
    pub status_msg: String,
    pub error_msg: Option<String>,
    pub should_quit: bool,
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
            should_quit: false,
        }
    }

    pub fn selected_host(&self) -> Option<&String> {
        self.hosts.get(self.selected_host_index)
    }

    pub fn move_up(&mut self) {
        if self.selected_host_index > 0 {
            self.selected_host_index -= 1;
        }
    }

    pub fn move_down(&mut self) {
        if !self.hosts.is_empty() && self.selected_host_index < self.hosts.len() - 1 {
            self.selected_host_index += 1;
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
                let logs = self.host_logs.entry(host).or_default();
                logs.push_back(line);
                if logs.len() > 1000 {
                    logs.pop_front();
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
}
