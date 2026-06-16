// ── UI Module ───────────────────────────────────────────────────────────
// Git Hero TUI - Refactored into focused submodules for maintainability
//
// ui/mod.rs       → Module hub + run_tui() event loop
// ui/state.rs     → AppState, GitFile, GitCommit, business logic
// ui/rendering.rs → draw_ui(), draw_dashboard(), panel drawing
// ui/modals.rs    → Modal windows (setup, theme, help)
// ui/events.rs    → Keyboard & mouse event handlers

pub mod state;
pub mod rendering;
pub mod modals;
pub mod events;

pub use state::AppState;
pub use rendering::draw_ui;
pub use events::{handle_key_event, handle_mouse_click, handle_mouse_scroll};

use std::io;
use std::time::Duration;

use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers, MouseButton, MouseEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

/// Launch the TUI application
pub fn run_tui(debug: bool) -> Result<(), Box<dyn std::error::Error>> {
    if debug { crate::log_debug("TUI: Enabling raw mode"); }
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    if debug { crate::log_debug("TUI: Entering alternate screen"); }
    execute!(
        stdout,
        EnterAlternateScreen,
        crossterm::event::EnableMouseCapture
    )?;
    let backend = CrosstermBackend::new(stdout);
    if debug { crate::log_debug("TUI: Creating terminal"); }
    let mut terminal = Terminal::new(backend)?;

    if debug { crate::log_debug("TUI: Creating AppState"); }
    let mut state = AppState::new();
    if debug { crate::log_debug(&format!("TUI: AppState created, is_git_repo={}, files={}, commits={}", state.is_git_repo, state.files.len(), state.commits.len())); }

    let mut frame_count: u64 = 0;
    
    loop {
        frame_count += 1;
        
        if debug && frame_count == 1 { crate::log_debug("TUI: First draw starting"); }
        terminal.draw(|f| {
            draw_ui(f, &mut state);
        })?;
        if debug && frame_count == 1 { crate::log_debug("TUI: First draw completed"); }

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == crossterm::event::KeyEventKind::Press {
                    if key.code == KeyCode::Char('c')
                        && key.modifiers.contains(KeyModifiers::CONTROL)
                    {
                        break;
                    }
                    if !handle_key_event(key.code, &mut state) {
                        break;
                    }
                }
            } else if let Event::Mouse(mouse) = event::read()? {
                match mouse.kind {
                    MouseEventKind::Down(MouseButton::Left) => {
                        handle_mouse_click(mouse.column, mouse.row, &mut state, &terminal);
                    }
                    MouseEventKind::ScrollUp => {
                        handle_mouse_scroll(true, mouse.column, mouse.row, &mut state, &terminal);
                    }
                    MouseEventKind::ScrollDown => {
                        handle_mouse_scroll(false, mouse.column, mouse.row, &mut state, &terminal);
                    }
                    _ => {}
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        crossterm::event::DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    Ok(())
}
