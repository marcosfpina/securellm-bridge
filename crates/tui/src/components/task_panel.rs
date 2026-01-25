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
        let items: Vec<ListItem> = self
            .tasks
            .iter()
            .map(|task| {
                let (state_icon, state_color) = match &task.state {
                    TaskState::Pending => ("⏸", Color::Gray),
                    TaskState::Running { .. } => ("▶", Color::Blue),
                    TaskState::Completed { .. } => ("✓", Color::Green),
                    TaskState::Failed { .. } => ("✗", Color::Red),
                    TaskState::Cancelled { .. } => ("○", Color::Yellow),
                };

                let progress_bar = Self::render_progress_bar(task.progress, 10);

                let content = Line::from(vec![
                    Span::styled(state_icon, Style::default().fg(state_color)),
                    Span::raw(" "),
                    Span::raw(&task.name),
                    Span::raw(" "),
                    Span::styled(progress_bar, Style::default().fg(Color::Cyan)),
                ]);

                ListItem::new(content)
            })
            .collect();

        let list = List::new(items).block(
            Block::default()
                .borders(Borders::ALL)
                .title("📋 Tasks")
                .style(Style::default()),
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
