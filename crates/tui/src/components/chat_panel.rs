//! Chat panel component

use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};

pub struct ChatPanel {
    messages: Vec<ChatMessage>,
}

#[derive(Clone)]
struct ChatMessage {
    role: String,
    content: String,
}

impl ChatPanel {
    pub fn new() -> Self {
        Self {
            messages: vec![
                ChatMessage {
                    role: "system".to_string(),
                    content: "Welcome to SecureLLM Bridge TUI. Press 'i' to enter insert mode, 'v' for voice, ':' for commands.".to_string(),
                }
            ],
        }
    }

    pub fn add_message(&mut self, role: &str, content: &str) {
        self.messages.push(ChatMessage {
            role: role.to_string(),
            content: content.to_string(),
        });
    }

    pub fn render(&self, f: &mut Frame, area: Rect) {
        let items: Vec<ListItem> = self
            .messages
            .iter()
            .map(|msg| {
                let role_style = match msg.role.as_str() {
                    "user" => Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                    "assistant" => Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
                    "system" => Style::default().fg(Color::Yellow),
                    _ => Style::default(),
                };

                let content = Line::from(vec![
                    Span::styled(format!("[{}] ", msg.role), role_style),
                    Span::raw(&msg.content),
                ]);

                ListItem::new(content)
            })
            .collect();

        let list = List::new(items).block(
            Block::default()
                .borders(Borders::ALL)
                .title("💬 Chat")
                .style(Style::default()),
        );

        f.render_widget(list, area);
    }
}

impl Default for ChatPanel {
    fn default() -> Self {
        Self::new()
    }
}
