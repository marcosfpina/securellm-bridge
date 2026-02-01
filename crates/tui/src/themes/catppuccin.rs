//! Catppuccin theme colors

use ratatui::style::Color;

// Catppuccin Mocha (Dark theme)
pub const BG_BASE: Color = Color::Rgb(30, 30, 46); // #1e1e2e
pub const BG_SURFACE: Color = Color::Rgb(49, 50, 68); // #313244
pub const BG_OVERLAY: Color = Color::Rgb(108, 112, 134); // #6c7086
pub const BG_CARD: Color = BG_SURFACE; // Alias for components

pub const FG_TEXT: Color = Color::Rgb(205, 214, 244); // #cdd6f4
pub const FG_MUTED: Color = Color::Rgb(108, 112, 134); // #6c7086
pub const FG_PRIMARY: Color = FG_TEXT; // Alias

pub const PRIMARY: Color = ACCENT; // Alias for accent
pub const ACCENT: Color = Color::Rgb(137, 180, 250); // #89b4fa (Blue)
pub const SECONDARY: Color = Color::Rgb(203, 166, 247); // #cba6f7 (Mauve)

pub const SUCCESS: Color = Color::Rgb(166, 227, 161); // #a6e3a1 (Green)
pub const WARNING: Color = Color::Rgb(249, 226, 175); // #f9e2af (Yellow)
pub const ERROR: Color = Color::Rgb(243, 139, 168); // #f38ba8 (Red)

pub const BORDER: Color = Color::Rgb(88, 91, 112); // #585b70
pub const BORDER_FOCUSED: Color = ACCENT;

// Gradients (using Catppuccin colors)
pub const GRADIENT_BLUE: Color = Color::Rgb(137, 180, 250); // Blue
pub const GRADIENT_PURPLE: Color = Color::Rgb(203, 166, 247); // Mauve
pub const GRADIENT_PINK: Color = Color::Rgb(245, 194, 231); // Pink
pub const GRADIENT_EMERALD: Color = Color::Rgb(166, 227, 161); // Green
pub const GRADIENT_ORANGE: Color = Color::Rgb(250, 179, 135); // Peach
