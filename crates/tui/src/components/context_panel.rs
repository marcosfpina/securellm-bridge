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
        use crate::themes::catppuccin::*;

        let content = vec![
            Line::from(vec![
                Span::styled("◆ ", Style::default().fg(SECONDARY)),
                Span::styled("Tokens: ", Style::default().fg(FG_MUTED)),
                Span::styled(
                    format!("{}", self.total_tokens),
                    Style::default()
                        .fg(GRADIENT_BLUE)
                        .add_modifier(ratatui::style::Modifier::BOLD),
                ),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("◆ ", Style::default().fg(SUCCESS)),
                Span::styled("Compression", Style::default().fg(FG_MUTED)),
                Span::raw(": "),
                Span::styled(
                    format!("{:.1}x", self.compression_ratio),
                    Style::default()
                        .fg(GRADIENT_EMERALD)
                        .add_modifier(ratatui::style::Modifier::BOLD),
                ),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("◆ ", Style::default().fg(WARNING)),
                Span::styled("Cache Hits: ", Style::default().fg(FG_MUTED)),
                Span::styled(
                    format!("{}", self.cache_hits),
                    Style::default()
                        .fg(GRADIENT_ORANGE)
                        .add_modifier(ratatui::style::Modifier::BOLD),
                ),
            ]),
        ];

        let paragraph = Paragraph::new(content).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(BORDER))
                .title(vec![
                    Span::styled("📊 ", Style::default().fg(PRIMARY)),
                    Span::styled(
                        "Context",
                        Style::default()
                            .fg(FG_PRIMARY)
                            .add_modifier(ratatui::style::Modifier::BOLD),
                    ),
                ])
                .style(Style::default().bg(BG_CARD)),
        );

        f.render_widget(paragraph, area);
    }
}

impl Default for ContextPanel {
    fn default() -> Self {
        Self::new()
    }
}
