//! TUI Application for SecureLLM Bridge
//!
//! Terminal User Interface with multi-panel layout for chat,
//! task management, context metrics, and logging.

use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame, Terminal,
};
use std::io;

mod app;
mod components;
mod input_mode;

pub use app::TuiApp;
pub use input_mode::InputMode;

use components::{ChatPanel, ContextPanel, LogsPanel, StatusBar, TaskPanel};

/// Run the TUI application
pub async fn run() -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app
    let mut app = TuiApp::new()?;

    // Main loop
    let result = run_app(&mut terminal, &mut app).await;

    // Cleanup
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

async fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut TuiApp,
) -> Result<()> {
    loop {
        terminal.draw(|f| render_ui(f, app))?;

        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if handle_input(app, key).await? {
                    break;
                }
            }
        }

        // Update app state
        app.update().await?;
    }

    Ok(())
}

fn render_ui(f: &mut Frame, app: &TuiApp) {
    let size = f.area();

    // Main layout: [Top Area] [Status Bar]
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(3)])
        .split(size);

    // Top area: [Left Panel] [Right Panel]
    let top_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(chunks[0]);

    // Left panel: [Chat] [Context Metrics]
    let left_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
        .split(top_chunks[0]);

    // Right panel: [Tasks] [Logs]
    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(top_chunks[1]);

    // Render components
    app.chat_panel.render(f, left_chunks[0]);
    app.context_panel.render(f, left_chunks[1]);
    app.task_panel.render(f, right_chunks[0]);
    app.logs_panel.render(f, right_chunks[1]);
    app.status_bar.render(f, chunks[1], app);
}

async fn handle_input(app: &mut TuiApp, key: event::KeyEvent) -> Result<bool> {
    // Global shortcuts
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        match key.code {
            KeyCode::Char('c') => return Ok(true), // Quit
            KeyCode::Char('q') => return Ok(true),
            _ => {}
        }
    }

    match app.input_mode {
        InputMode::Normal => handle_normal_mode(app, key).await?,
        InputMode::Insert => handle_insert_mode(app, key).await?,
        InputMode::Command => handle_command_mode(app, key).await?,
        InputMode::Voice => handle_voice_mode(app, key).await?,
    }

    Ok(false)
}

async fn handle_normal_mode(app: &mut TuiApp, key: event::KeyEvent) -> Result<()> {
    match key.code {
        KeyCode::Char('q') => {
            // Quit confirmation could be added here
        }
        KeyCode::Char('i') => {
            app.input_mode = InputMode::Insert;
        }
        KeyCode::Char(':') => {
            app.input_mode = InputMode::Command;
        }
        KeyCode::Char('v') => {
            app.toggle_voice().await?;
        }
        KeyCode::Tab => {
            app.cycle_focus();
        }
        _ => {}
    }
    Ok(())
}

async fn handle_insert_mode(app: &mut TuiApp, key: event::KeyEvent) -> Result<()> {
    match key.code {
        KeyCode::Esc => {
            app.input_mode = InputMode::Normal;
        }
        KeyCode::Enter => {
            app.send_message().await?;
        }
        KeyCode::Char(c) => {
            app.input_buffer.push(c);
        }
        KeyCode::Backspace => {
            app.input_buffer.pop();
        }
        _ => {}
    }
    Ok(())
}

async fn handle_command_mode(_app: &mut TuiApp, key: event::KeyEvent) -> Result<()> {
    match key.code {
        KeyCode::Esc => {
            // Exit command mode
        }
        _ => {}
    }
    Ok(())
}

async fn handle_voice_mode(app: &mut TuiApp, key: event::KeyEvent) -> Result<()> {
    match key.code {
        KeyCode::Char('v') | KeyCode::Esc => {
            app.toggle_voice().await?;
        }
        _ => {}
    }
    Ok(())
}
