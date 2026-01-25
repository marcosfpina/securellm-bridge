//! Context metrics panel

use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use securellm_context_manager::ContextManager;

pub struct ContextPanel {
    total_tokens: usize,
    compression_ratio: f32,
    cache_hits: usize,
}

impl ContextPanel {
    pub fn new() -> Self {
        Self {
            total_tokens: 0,
            compression_ratio: 1.0,
            cache_hits: 0,
        }
    }

    pub fn update(&mut self, _context_manager: &ContextManager) {
        // TODO: Get actual metrics from context manager
        self.total_tokens = 1234;
        self.compression_ratio = 3.5;
        self.cache_hits = 42;
    }

    pub fn render(&self, f: &mut Frame, area: Rect) {
        let content = vec![
            Line::from(vec![
                Span::styled("Tokens: ", Style::default().fg(Color::Gray)),
                Span::styled(
                    format!("{}", self.total_tokens),
                    Style::default().fg(Color::Cyan),
                ),
            ]),
            Line::from(vec![
                Span::styled("Compression: ", Style::default().fg(Color::Gray)),
                Span::styled(
                    format!("{:.1}x", self.compression_ratio),
                    Style::default().fg(Color::Green),
                ),
            ]),
            Line::from(vec![
                Span::styled("Cache Hits: ", Style::default().fg(Color::Gray)),
                Span::styled(
                    format!("{}", self.cache_hits),
                    Style::default().fg(Color::Yellow),
                ),
            ]),
        ];

        let paragraph = Paragraph::new(content).block(
            Block::default()
                .borders(Borders::ALL)
                .title("📊 Context Metrics"),
        );

        f.render_widget(paragraph, area);
    }
}

impl Default for ContextPanel {
    fn default() -> Self {
        Self::new()
    }
}
