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
        use crate::themes::catppuccin::*;

        let items: Vec<ListItem> = self
            .logs
            .iter()
            .rev() // Most recent first
            .take(20)
            .map(|log| {
                let (icon, level_style) = match log.level.as_str() {
                    "ERROR" => ("✗", Style::default().fg(ERROR)),
                    "WARN" => ("⚠", Style::default().fg(WARNING)),
                    "INFO" => ("◆", Style::default().fg(SECONDARY)),
                    "DEBUG" => ("○", Style::default().fg(FG_MUTED)),
                    _ => ("·", Style::default().fg(FG_MUTED)),
                };

                let content = Line::from(vec![
                    Span::styled(icon, level_style.clone().add_modifier(ratatui::style::Modifier::BOLD)),
                    Span::raw(" "),
                    Span::styled(format!("{}", log.level), level_style.add_modifier(ratatui::style::Modifier::DIM)),
                    Span::raw(" "),
                    Span::styled(&log.message, Style::default().fg(FG_PRIMARY)),
                ]);

                ListItem::new(content)
            })
            .collect();

        let list = List::new(items).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(BORDER))
                .title(vec![
                    Span::styled("📜 ", Style::default().fg(WARNING)),
                    Span::styled("Logs", Style::default().fg(FG_PRIMARY).add_modifier(ratatui::style::Modifier::BOLD)),
                ])
                .style(Style::default().bg(BG_CARD)),
        );

        f.render_widget(list, area);
    }
}

impl Default for LogsPanel {
    fn default() -> Self {
        Self::new()
    }
}
