//! Input mode state

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    /// Normal mode - vim-like navigation
    Normal,

    /// Insert mode - typing in chat
    Insert,

    /// Command mode - executing commands
    Command,

    /// Voice mode - recording audio
    Voice,
}

impl Default for InputMode {
    fn default() -> Self {
        InputMode::Normal
    }
}

impl InputMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            InputMode::Normal => "NORMAL",
            InputMode::Insert => "INSERT",
            InputMode::Command => "COMMAND",
            InputMode::Voice => "VOICE",
        }
    }
}
