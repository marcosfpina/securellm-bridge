//! Logs panel component

use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem},
    Frame,
};

pub struct LogsPanel {
    logs: Vec<LogEntry>,
}

#[derive(Clone)]
struct LogEntry {
    level: String,
    message: String,
}

impl LogsPanel {
    pub fn new() -> Self {
        Self {
            logs: vec![
                LogEntry {
                    level: "INFO".to_string(),
                    message: "TUI initialized".to_string(),
                },
            ],
        }
    }

    pub fn add_log(&mut self, level: &str, message: &str) {
        self.logs.push(LogEntry {
            level: level.to_string(),
            message: message.to_string(),
        });

        // Keep last 100 logs
        if self.logs.len() > 100 {
            self.logs.remove(0);
        }
    }

    pub fn render(&self, f: &mut Frame, area: Rect) {
        let items: Vec<ListItem> = self
            .logs
            .iter()
            .rev() // Most recent first
            .take(20)
            .map(|log| {
                let level_style = match log.level.as_str() {
                    "ERROR" => Style::default().fg(Color::Red),
                    "WARN" => Style::default().fg(Color::Yellow),
                    "INFO" => Style::default().fg(Color::Blue),
                    "DEBUG" => Style::default().fg(Color::Gray),
                    _ => Style::default(),
                };

                let content = Line::from(vec![
                    Span::styled(format!("[{}] ", log.level), level_style),
                    Span::raw(&log.message),
                ]);

                ListItem::new(content)
            })
            .collect();

        let list = List::new(items).block(
            Block::default()
                .borders(Borders::ALL)
                .title("📜 Logs")
                .style(Style::default()),
        );

        f.render_widget(list, area);
    }
}

impl Default for LogsPanel {
    fn default() -> Self {
        Self::new()
    }
}
