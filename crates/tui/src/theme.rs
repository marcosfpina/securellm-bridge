//! Modern color theme for the TUI
//! Inspired by ai-assistant-frontend's premium dark design

use ratatui::style::Color;

/// Base background color - deep blue-black
pub const BG_BASE: Color = Color::Rgb(10, 14, 23);

/// Card/panel background - slightly lighter
pub const BG_CARD: Color = Color::Rgb(15, 20, 31);

/// Elevated surface (hover, focus)
pub const BG_ELEVATED: Color = Color::Rgb(20, 26, 40);

/// Foreground text - almost white
pub const FG_PRIMARY: Color = Color::Rgb(248, 250, 252);

/// Muted text - for secondary info
pub const FG_MUTED: Color = Color::Rgb(148, 163, 184);

/// Border color
pub const BORDER: Color = Color::Rgb(30, 41, 59);

/// Primary accent - vibrant purple
pub const PRIMARY: Color = Color::Rgb(168, 85, 247);

/// Secondary accent - blue
pub const SECONDARY: Color = Color::Rgb(59, 130, 246);

/// Success/accent - emerald green
pub const SUCCESS: Color = Color::Rgb(34, 197, 94);

/// Warning - orange
pub const WARNING: Color = Color::Rgb(251, 146, 60);

/// Error/destructive - red
pub const ERROR: Color = Color::Rgb(239, 68, 68);

/// Gradient colors for visual interest
pub const GRADIENT_PURPLE: Color = Color::Rgb(168, 85, 247);
pub const GRADIENT_BLUE: Color = Color::Rgb(59, 130, 246);
pub const GRADIENT_TEAL: Color = Color::Rgb(20, 184, 166);
pub const GRADIENT_EMERALD: Color = Color::Rgb(34, 197, 94);
pub const GRADIENT_ORANGE: Color = Color::Rgb(251, 146, 60);
pub const GRADIENT_PINK: Color = Color::Rgb(236, 72, 153);

/// Get color for severity level
pub fn severity_color(level: &str) -> Color {
    match level.to_uppercase().as_str() {
        "ERROR" | "CRITICAL" => ERROR,
        "WARN" | "WARNING" => WARNING,
        "INFO" => SECONDARY,
        "DEBUG" => FG_MUTED,
        "SUCCESS" => SUCCESS,
        _ => FG_PRIMARY,
    }
}

/// Get gradient color for index (cycles through gradient palette)
pub fn gradient_color(index: usize) -> Color {
    let colors = [
        GRADIENT_PURPLE,
        GRADIENT_BLUE,
        GRADIENT_TEAL,
        GRADIENT_EMERALD,
        GRADIENT_ORANGE,
        GRADIENT_PINK,
    ];
    colors[index % colors.len()]
}

/// Status indicator colors
pub fn status_color(status: &str) -> Color {
    match status.to_lowercase().as_str() {
        "running" | "active" => SUCCESS,
        "pending" | "waiting" => WARNING,
        "failed" | "error" => ERROR,
        "completed" | "done" => SECONDARY,
        _ => FG_MUTED,
    }
}
