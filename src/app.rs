use std::collections::VecDeque;

use crate::types::{AppEvent, LogLine, RebuildAction};

pub struct App {
    pub hosts: Vec<String>,
    pub selected_host_index: usize,
    pub logs: VecDeque<LogLine>,
    pub running_action: Option<(String, RebuildAction)>,
    pub status_msg: String,
    pub error_msg: Option<String>,
    pub should_quit: bool,
}

impl App {
    pub fn new() -> Self {
        Self {
            hosts: Vec::new(),
            selected_host_index: 0,
            logs: VecDeque::new(),
            running_action: None,
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
            AppEvent::Log(line) => {
                self.logs.push_back(line);
                if self.logs.len() > 1000 {
                    self.logs.pop_front();
                }
            }
            AppEvent::CommandStarted { host, action } => {
                self.running_action = Some((host, action));
                self.logs.clear();
                self.status_msg = "Running...".to_string();
            }
            AppEvent::CommandFinished { success, .. } => {
                self.running_action = None;
                if success {
                    self.status_msg = "Success".to_string();
                } else {
                    self.status_msg = "Failed".to_string();
                }
            }
            AppEvent::CommandErrored { error, .. } => {
                self.running_action = None;
                self.error_msg = Some(error);
                self.status_msg = "Failed to start".to_string();
            }
        }
    }

    pub fn can_start_action(&self) -> bool {
        self.running_action.is_none() && !self.hosts.is_empty()
    }
}
