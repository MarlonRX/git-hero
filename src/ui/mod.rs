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
pub use events::{handle_key_event, handle_mouse_click};

use std::io;
use std::time::Duration;

use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers, MouseButton, MouseEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

/// Launch the TUI application
pub fn run_tui() -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(
        stdout,
        EnterAlternateScreen,
        crossterm::event::EnableMouseCapture
    )?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut state = AppState::new();

    loop {
        terminal.draw(|f| draw_ui(f, &mut state))?;

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
                if mouse.kind == MouseEventKind::Down(MouseButton::Left) {
                    handle_mouse_click(mouse.column, mouse.row, &mut state, &terminal);
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
