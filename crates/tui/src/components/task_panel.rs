//! Task panel component

use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem},
    Frame,
};
use securellm_task_manager::{Task, TaskManager, TaskState};

pub struct TaskPanel {
    tasks: Vec<Task>,
}

impl TaskPanel {
    pub fn new() -> Self {
        Self { tasks: Vec::new() }
    }

    pub fn update(&mut self, task_manager: &TaskManager) {
        self.tasks = task_manager.list(None);
    }

    pub fn render(&self, f: &mut Frame, area: Rect) {
        use crate::themes::catppuccin::*;

        let items: Vec<ListItem> = self
            .tasks
            .iter()
            .map(|task| {
                let (state_icon, state_color) = match &task.state {
                    TaskState::Pending => ("⏸", FG_MUTED),
                    TaskState::Running { .. } => ("▶", SUCCESS),
                    TaskState::Completed { .. } => ("✓", GRADIENT_EMERALD),
                    TaskState::Failed { .. } => ("✗", ERROR),
                    TaskState::Cancelled { .. } => ("○", WARNING),
                };

                let progress_bar = Self::render_progress_bar(task.progress, 10);

                let content = Line::from(vec![
                    Span::styled(
                        state_icon,
                        Style::default()
                            .fg(state_color)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(" "),
                    Span::styled(&task.name, Style::default().fg(FG_PRIMARY)),
                    Span::raw(" "),
                    Span::styled(progress_bar, Style::default().fg(SECONDARY)),
                ]);

                ListItem::new(content)
            })
            .collect();

        let list = List::new(items).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(BORDER))
                .title(vec![
                    Span::styled("📋 ", Style::default().fg(SECONDARY)),
                    Span::styled(
                        "Tasks",
                        Style::default().fg(FG_PRIMARY).add_modifier(Modifier::BOLD),
                    ),
                ])
                .style(Style::default().bg(BG_CARD)),
        );

        f.render_widget(list, area);
    }

    fn render_progress_bar(progress: f32, width: usize) -> String {
        let filled = ((progress * width as f32) as usize).min(width);
        let empty = width - filled;
        format!("[{}{}]", "█".repeat(filled), "░".repeat(empty))
    }
}

impl Default for TaskPanel {
    fn default() -> Self {
        Self::new()
    }
}
