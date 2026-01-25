//! Status bar component

use crate::app::TuiApp;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub struct StatusBar;

impl StatusBar {
    pub fn new() -> Self {
        Self
    }

    pub fn render(&self, f: &mut Frame, area: Rect, app: &TuiApp) {
        let mode_color = match app.input_mode {
            crate::InputMode::Normal => Color::Blue,
            crate::InputMode::Insert => Color::Green,
            crate::InputMode::Command => Color::Yellow,
            crate::InputMode::Voice => Color::Red,
        };

        let content = vec![
            Line::from(vec![
                Span::styled(
                    format!(" {} ", app.input_mode.as_str()),
                    Style::default()
                        .fg(Color::Black)
                        .bg(mode_color)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" "),
                Span::styled("Provider: ", Style::default().fg(Color::Gray)),
                Span::styled("DeepSeek", Style::default().fg(Color::Cyan)),
                Span::raw(" │ "),
                Span::styled("Model: ", Style::default().fg(Color::Gray)),
                Span::styled("deepseek-chat", Style::default().fg(Color::Cyan)),
                Span::raw(" │ "),
                Span::styled("Tokens: ", Style::default().fg(Color::Gray)),
                Span::styled("1.2K", Style::default().fg(Color::Green)),
            ]),
            Line::from(vec![
                Span::raw(" "),
                if !app.input_buffer.is_empty() {
                    Span::styled(&app.input_buffer, Style::default().fg(Color::White))
                } else {
                    Span::styled("Press 'i' to insert, 'v' for voice, ':' for commands, 'q' to quit", Style::default().fg(Color::DarkGray))
                },
            ]),
        ];

        let paragraph = Paragraph::new(content).block(Block::default().borders(Borders::ALL));

        f.render_widget(paragraph, area);
    }
}

impl Default for StatusBar {
    fn default() -> Self {
        Self::new()
    }
}
