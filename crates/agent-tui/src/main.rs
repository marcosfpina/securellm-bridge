//! Agent TUI - Floating overlay for agentic shell interaction
//!
//! Minimal, borderless interface that:
//! - Appears via Super+K (Hyprland keybind)
//! - Takes natural language input
//! - Generates shell commands via LLM
//! - Injects into Zellij session
//! - Disappears after execution

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
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame, Terminal,
};
use std::io;
use std::process::Command;
use std::fs;

// Catppuccin Mocha
const ACCENT: Color = Color::Rgb(137, 180, 250);    // Blue
const DIM: Color = Color::Rgb(108, 112, 134);       // Overlay0
const BRIGHT: Color = Color::Rgb(205, 214, 244);    // Text
const SUCCESS: Color = Color::Rgb(166, 227, 161);   // Green
const ERROR: Color = Color::Rgb(243, 139, 168);     // Red

/// Load context directly from Zellij pane
fn load_shell_context() -> String {
    // 1. Get current pane content (dump-screen)
    let output = Command::new("zellij")
        .args(["action", "dump-screen", "--full"])
        .output();
        
    let screen_content = match output {
        Ok(out) if out.status.success() => String::from_utf8_lossy(&out.stdout).to_string(),
        _ => String::from("[No Zellij context available]"),
    };
    
    // Take last 20 lines to avoid too much context
    let context_lines: Vec<&str> = screen_content.lines().rev().take(20).collect();
    let recent_context: Vec<&str> = context_lines.into_iter().rev().collect();
    
    format!(
        "Terminal content (last 20 lines):\n---\n{}\n---\nCurrent directory: {}", 
        recent_context.join("\n"),
        std::env::current_dir().unwrap_or_default().display()
    )
}

struct AgentOverlay {
    input: String,
    output: String,
    status: Status,
    should_quit: bool,
}

#[derive(Clone, Copy, PartialEq)]
enum Status {
    Idle,
    Processing,
    Ready,
    Error,
}

impl AgentOverlay {
    fn new() -> Self {
        Self {
            input: String::new(),
            output: String::new(),
            status: Status::Idle,
            should_quit: false,
        }
    }

    async fn process_input(&mut self) -> Result<()> {
        if self.input.is_empty() {
            return Ok(());
        }

        self.status = Status::Processing;
        self.output = "Thinking...".to_string();

        // Call LLM
        use securellm_core::{LLMProvider, Request, Message, MessageRole, MessageContent};
        use securellm_providers::llamacpp::LlamaCppProvider;

        match LlamaCppProvider::new(8081, "llamacppturbo") {
            Ok(provider) => {
                let context = load_shell_context();
                let system_prompt = format!(
                    "You are a shell command generator. Given a natural language request, output ONLY the shell command to execute. No explanations, no markdown, just the raw command.\n\n{}\n\nRespond with the exact command to run.",
                    context
                );

                let request = Request::new("llamacpp", "llamacppturbo")
                    .add_message(Message {
                        role: MessageRole::System,
                        content: MessageContent::Text(system_prompt),
                        name: None,
                        metadata: None,
                    })
                    .add_message(Message {
                        role: MessageRole::User,
                        content: MessageContent::Text(self.input.clone()),
                        name: None,
                        metadata: None,
                    });

                match provider.send_request(request).await {
                    Ok(response) => {
                        if let Ok(text) = response.text() {
                            self.output = text.trim().to_string();
                            self.status = Status::Ready;
                        }
                    }
                    Err(e) => {
                        self.output = format!("Error: {}", e);
                        self.status = Status::Error;
                    }
                }
            }
            Err(e) => {
                self.output = format!("LLM Error: {}", e);
                self.status = Status::Error;
            }
        }

        Ok(())
    }

    fn inject_to_zellij(&self) -> Result<()> {
        if self.output.is_empty() || self.status != Status::Ready {
            return Ok(());
        }

        // Check if Zellij is running
        let zellij_check = Command::new("zellij")
            .args(["list-sessions"])
            .output();

        match zellij_check {
            Ok(output) if output.status.success() => {
                let sessions = String::from_utf8_lossy(&output.stdout);
                if sessions.trim().is_empty() {
                    // No Zellij session, just print command
                    eprintln!("No Zellij session. Command: {}", self.output);
                    return Ok(());
                }

                // Inject command into active Zellij pane
                let cmd = format!("{}\n", self.output);
                let result = Command::new("zellij")
                    .args(["action", "write-chars", &cmd])
                    .status();

                if let Err(e) = result {
                    eprintln!("Zellij injection failed: {}", e);
                }
            }
            _ => {
                // Zellij not available, copy to clipboard as fallback
                let _ = Command::new("wl-copy")
                    .arg(&self.output)
                    .status();
                eprintln!("Copied to clipboard: {}", self.output);
            }
        }

        Ok(())
    }
}

/// Check if we're running inside Zellij
fn is_inside_zellij() -> bool {
    std::env::var("ZELLIJ").is_ok()
}

/// Get current Zellij session name
fn get_zellij_session() -> Option<String> {
    std::env::var("ZELLIJ_SESSION_NAME").ok()
}

fn render(f: &mut Frame, app: &AgentOverlay) {
    let area = f.area();

    // Clear with transparent bg
    let bg = Block::default().style(Style::default().bg(Color::Reset));
    f.render_widget(bg, area);

    // Inner area with padding
    let inner = Rect {
        x: area.x + 4,
        y: area.y + 2,
        width: area.width.saturating_sub(8),
        height: area.height.saturating_sub(4),
    };

    // Layout: [Prompt] [Input] [Separator] [Output] [Status]
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),  // Prompt
            Constraint::Length(3),  // Input
            Constraint::Length(1),  // Separator
            Constraint::Min(3),     // Output
            Constraint::Length(1),  // Status
        ])
        .split(inner);

    // Prompt line
    let prompt = Paragraph::new(Line::from(vec![
        Span::styled("❯ ", Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)),
        Span::styled("What do you want to do?", Style::default().fg(DIM)),
    ]));
    f.render_widget(prompt, chunks[0]);

    // Input (no borders)
    let input_style = Style::default().fg(BRIGHT).add_modifier(Modifier::BOLD);
    let input = Paragraph::new(app.input.as_str())
        .style(input_style)
        .block(Block::default().borders(Borders::NONE));
    f.render_widget(input, chunks[1]);

    // Cursor
    f.set_cursor_position((
        chunks[1].x + app.input.len() as u16,
        chunks[1].y,
    ));

    // Separator
    let sep_char = match app.status {
        Status::Idle => "─",
        Status::Processing => "◆",
        Status::Ready => "✓",
        Status::Error => "✗",
    };
    let sep_color = match app.status {
        Status::Idle => DIM,
        Status::Processing => ACCENT,
        Status::Ready => SUCCESS,
        Status::Error => ERROR,
    };
    let separator = Paragraph::new(sep_char.repeat(inner.width as usize))
        .style(Style::default().fg(sep_color));
    f.render_widget(separator, chunks[2]);

    // Output
    let output_style = match app.status {
        Status::Ready => Style::default().fg(SUCCESS),
        Status::Error => Style::default().fg(ERROR),
        _ => Style::default().fg(DIM),
    };
    let output = Paragraph::new(app.output.as_str())
        .style(output_style)
        .wrap(Wrap { trim: true });
    f.render_widget(output, chunks[3]);

    // Status bar
    let status_text = match app.status {
        Status::Idle => "[Enter] Send  [Esc] Quit",
        Status::Processing => "Processing...",
        Status::Ready => "[Enter] Execute  [Esc] Cancel",
        Status::Error => "[Enter] Retry  [Esc] Quit",
    };
    let status = Paragraph::new(status_text)
        .style(Style::default().fg(DIM));
    f.render_widget(status, chunks[4]);
}

async fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut AgentOverlay,
) -> Result<()> {
    loop {
        terminal.draw(|f| render(f, app))?;

        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Esc => {
                        app.should_quit = true;
                        break;
                    }
                    KeyCode::Enter => {
                        if app.status == Status::Ready {
                            // Inject and quit
                            app.inject_to_zellij()?;
                            app.should_quit = true;
                            break;
                        } else {
                            // Process input
                            app.process_input().await?;
                        }
                    }
                    KeyCode::Char(c) => {
                        if app.status != Status::Processing {
                            app.input.push(c);
                            app.status = Status::Idle;
                        }
                    }
                    KeyCode::Backspace => {
                        app.input.pop();
                        app.status = Status::Idle;
                    }
                    _ => {}
                }
            }
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Run app
    let mut app = AgentOverlay::new();
    let result = run_app(&mut terminal, &mut app).await;

    // Cleanup
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}
