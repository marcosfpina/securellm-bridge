//! TUI Application state

use anyhow::Result;
use securellm_context_manager::ContextManager;
use securellm_task_manager::TaskManager;
use securellm_voice_agents::VoiceAgent;

use crate::components::*;
use crate::InputMode;

pub struct TuiApp {
    pub input_mode: InputMode,
    pub input_buffer: String,
    pub chat_panel: ChatPanel,
    pub task_panel: TaskPanel,
    pub context_panel: ContextPanel,
    pub logs_panel: LogsPanel,
    pub status_bar: StatusBar,
    pub focused_panel: FocusedPanel,
    
    // Backend services
    task_manager: TaskManager,
    context_manager: ContextManager,
    voice_agent: Option<VoiceAgent>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusedPanel {
    Chat,
    Tasks,
    Context,
    Logs,
}

impl TuiApp {
    pub fn new() -> Result<Self> {
        let task_manager = TaskManager::new();
        let context_manager = ContextManager::new()?;
        let voice_agent = VoiceAgent::new().ok(); // Optional

        Ok(Self {
            input_mode: InputMode::Normal,
            input_buffer: String::new(),
            chat_panel: ChatPanel::new(),
            task_panel: TaskPanel::new(),
            context_panel: ContextPanel::new(),
            logs_panel: LogsPanel::new(),
            status_bar: StatusBar::new(),
            focused_panel: FocusedPanel::Chat,
            task_manager,
            context_manager,
            voice_agent,
        })
    }

    pub async fn update(&mut self) -> Result<()> {
        // Update panel states
        self.task_panel.update(&self.task_manager);
        self.context_panel.update(&self.context_manager);
        Ok(())
    }

    pub async fn send_message(&mut self) -> Result<()> {
        if !self.input_buffer.is_empty() {
            let message = self.input_buffer.clone();
            self.chat_panel.add_message("user", &message);
            self.input_buffer.clear();
            
            // TODO: Send to LLM provider
            // For now, just echo
            self.chat_panel.add_message("assistant", &format!("Echo: {}", message));
        }
        Ok(())
    }

    pub async fn toggle_voice(&mut self) -> Result<()> {
        if self.input_mode == InputMode::Voice {
            // Stop recording
            self.input_mode = InputMode::Normal;
            // TODO: Process audio
        } else {
            // Start recording
            self.input_mode = InputMode::Voice;
            // TODO: Start audio capture
        }
        Ok(())
    }

    pub fn cycle_focus(&mut self) {
        self.focused_panel = match self.focused_panel {
            FocusedPanel::Chat => FocusedPanel::Tasks,
            FocusedPanel::Tasks => FocusedPanel::Context,
            FocusedPanel::Context => FocusedPanel::Logs,
            FocusedPanel::Logs => FocusedPanel::Chat,
        };
    }
}
