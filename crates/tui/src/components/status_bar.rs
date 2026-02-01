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
        use crate::themes::catppuccin::*;

        let mode_color = match app.input_mode {
            crate::InputMode::Normal => SECONDARY,    // Blue
            crate::InputMode::Insert => SUCCESS,      // Green
            crate::InputMode::Command => WARNING,     // Orange
            crate::InputMode::Voice => GRADIENT_PINK, // Pink
        };

        let content = vec![
            Line::from(vec![
                Span::styled(
                    format!(" {} ", app.input_mode.as_str()),
                    Style::default()
                        .fg(BG_BASE)
                        .bg(mode_color)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw("  "),
                Span::styled("◆ ", Style::default().fg(PRIMARY)),
                Span::styled("Provider: ", Style::default().fg(FG_MUTED)),
                Span::styled(
                    "LlamaCpp",
                    Style::default()
                        .fg(GRADIENT_PURPLE)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw("  "),
                Span::styled("│", Style::default().fg(BORDER)),
                Span::raw("  "),
                Span::styled("◆ ", Style::default().fg(SECONDARY)),
                Span::styled("Model: ", Style::default().fg(FG_MUTED)),
                Span::styled(
                    "llamacppturbo:8081",
                    Style::default()
                        .fg(GRADIENT_BLUE)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw("  "),
                Span::styled("│", Style::default().fg(BORDER)),
                Span::raw("  "),
                Span::styled("◆ ", Style::default().fg(SUCCESS)),
                Span::styled("Tokens: ", Style::default().fg(FG_MUTED)),
                Span::styled(
                    "1.2K",
                    Style::default()
                        .fg(GRADIENT_EMERALD)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::raw(" "),
                Span::styled("▸ ", Style::default().fg(PRIMARY)),
                if !app.input_buffer.is_empty() {
                    Span::styled(&app.input_buffer, Style::default().fg(FG_PRIMARY))
                } else {
                    Span::styled(
                        "Press 'i' to insert • 'v' for voice • ':' for commands • 'q' to quit",
                        Style::default().fg(FG_MUTED).add_modifier(Modifier::ITALIC),
                    )
                },
            ]),
        ];

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(BORDER))
            .style(Style::default().bg(BG_CARD));

        let paragraph = Paragraph::new(content).block(block);

        f.render_widget(paragraph, area);
    }
}

impl Default for StatusBar {
    fn default() -> Self {
        Self::new()
    }
}
