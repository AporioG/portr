use crate::scanner::{scan_all_sockets, SocketEntry};
use crate::services::ServiceRegistry;
use ratatui::widgets::TableState;

#[derive(PartialEq)]
pub enum InputMode {
    Normal,
    Filtering,
}

pub struct App {
    pub sockets: Vec<SocketEntry>,
    pub table_state: TableState,
    pub filter: String,
    pub input_mode: InputMode,
    pub status_message: String,
    pub service_registry: ServiceRegistry,
}

impl App {
    pub fn new() -> Self {
        let registry = ServiceRegistry::load();
        let mut app = Self {
            sockets: Vec::new(),
            table_state: TableState::default(),
            filter: String::new(),
            input_mode: InputMode::Normal,
            status_message: "Ready".to_string(),
            service_registry: registry,
        };
        app.refresh();
        if !app.filtered_sockets().is_empty() {
            app.table_state.select(Some(0));
        }
        app
    }

    pub fn refresh(&mut self) {
        match scan_all_sockets(&self.service_registry) {
            Ok(list) => {
                self.sockets = list;
                let count = self.filtered_sockets().len();
                if let Some(selected) = self.table_state.selected() {
                    if count == 0 {
                        self.table_state.select(None);
                    } else if selected >= count {
                        self.table_state.select(Some(count - 1));
                    }
                } else if count > 0 {
                    self.table_state.select(Some(0));
                }
            }
            Err(e) => {
                self.status_message = format!("Scan error: {}", e);
            }
        }
    }

    pub fn filtered_sockets(&self) -> Vec<&SocketEntry> {
        if self.filter.is_empty() {
            return self.sockets.iter().collect();
        }
        let q = self.filter.to_lowercase();
        self.sockets
            .iter()
            .filter(|s| {
                s.proto.as_str().to_lowercase().contains(&q)
                    || s.local.to_lowercase().contains(&q)
                    || s.remote.to_lowercase().contains(&q)
                    || s.state.to_lowercase().contains(&q)
                    || s.service
                        .as_deref()
                        .unwrap_or("")
                        .to_lowercase()
                        .contains(&q)
                    || s.process_name
                        .as_deref()
                        .unwrap_or("")
                        .to_lowercase()
                        .contains(&q)
                    || s.exe
                        .as_deref()
                        .unwrap_or("")
                        .to_lowercase()
                        .contains(&q)
                    || s.cmdline
                        .as_deref()
                        .unwrap_or("")
                        .to_lowercase()
                        .contains(&q)
                    || s.pid
                        .map(|p| p.to_string())
                        .unwrap_or_default()
                        .contains(&q)
            })
            .collect()
    }

    pub fn next(&mut self) {
        let total = self.filtered_sockets().len();
        if total == 0 {
            return;
        }
        let i = match self.table_state.selected() {
            Some(i) => {
                if i + 1 >= total {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.table_state.select(Some(i));
    }

    pub fn previous(&mut self) {
        let total = self.filtered_sockets().len();
        if total == 0 {
            return;
        }
        let i = match self.table_state.selected() {
            Some(i) => {
                if i == 0 {
                    total - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.table_state.select(Some(i));
    }

    pub fn kill_selected(&mut self, signal: libc::c_int) {
        let filtered = self.filtered_sockets();
        let selected_idx = match self.table_state.selected() {
            Some(i) => i,
            None => {
                self.status_message = "No process selected".to_string();
                return;
            }
        };

        if let Some(entry) = filtered.get(selected_idx) {
            if let Some(pid) = entry.pid {
                let res = unsafe { libc::kill(pid as libc::pid_t, signal) };
                if res == 0 {
                    let sig_name = if signal == libc::SIGKILL { "SIGKILL" } else { "SIGTERM" };
                    self.status_message = format!("Sent {} to PID {}", sig_name, pid);
                    self.refresh();
                } else {
                    let err = std::io::Error::last_os_error();
                    self.status_message = format!("Error killing PID {}: {}", pid, err);
                }
            } else {
                self.status_message = "Cannot kill: PID unknown".to_string();
            }
        }
    }
}
