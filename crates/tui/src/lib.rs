//! TUI Application for SecureLLM Bridge
//!
//! Zellij-style multiplexed interface with tabs, splits, and modern aesthetics

use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Tabs},
    Frame, Terminal,
};
use std::io;

mod app;
mod components;
mod input_mode;
mod multiplex;
mod themes;

pub use app::TuiApp;
pub use input_mode::InputMode;
pub use multiplex::{Pane, TabBar};
pub use themes::catppuccin::*;

use components::{StatusBar, TabBarWidget};

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

    // Main layout: [Tab Bar] [Main Area] [Status Bar]
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Tab bar
            Constraint::Min(0),    // Main content
            Constraint::Length(3), // Status bar
        ])
        .split(size);

    // Render tab bar
    let tab_names: Vec<String> = app.tab_bar.tabs.iter().map(|t| t.name.clone()).collect();
    TabBarWidget::render(f, main_chunks[0], &tab_names, app.tab_bar.active_index);

    // Top area: [Left] [Middle] [Right]
    let top_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(40),
            Constraint::Percentage(30),
            Constraint::Percentage(30),
        ])
        .split(main_chunks[1]);

    // Left panel: [Chat] [Context]
    let left_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
        .split(top_chunks[0]);

    // Right panel: [Tasks] [Logs]
    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(top_chunks[2]);

    // Render components
    app.chat_panel.render(f, left_chunks[0]);
    app.context_panel.render(f, left_chunks[1]);

    // Show tool panel if agent mode is enabled
    if app.agent_mode {
        app.tool_panel.render(f, top_chunks[1]);
    }

    app.task_panel.render(f, right_chunks[0]);
    app.logs_panel.render(f, right_chunks[1]);
    app.status_bar.render(f, main_chunks[2], app);
}

async fn handle_input(app: &mut TuiApp, key: event::KeyEvent) -> Result<bool> {
    // Global shortcuts
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        match key.code {
            KeyCode::Char('c') => return Ok(true), // Quit
            KeyCode::Char('q') => return Ok(true),
            KeyCode::Char('t') => {
                // Cycle tabs
                app.tab_bar.next_tab();
                return Ok(false);
            }
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
        KeyCode::Char('a') => {
            app.toggle_agent_mode();
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
