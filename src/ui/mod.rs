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
use std::time::{Duration, Instant};

use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers, MouseButton, MouseEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

/// Launch the TUI application
pub fn run_tui(debug: bool) -> Result<(), Box<dyn std::error::Error>> {
    if debug { crate::log::log_debug("TUI: Enabling raw mode"); }
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    if debug { crate::log::log_debug("TUI: Entering alternate screen"); }
    execute!(
        stdout,
        EnterAlternateScreen,
        crossterm::event::EnableMouseCapture
    )?;
    let backend = CrosstermBackend::new(stdout);
    if debug { crate::log::log_debug("TUI: Creating terminal"); }
    let mut terminal = Terminal::new(backend)?;

    if debug { crate::log::log_debug("TUI: Creating AppState"); }
    let mut state = AppState::new();
    // Check for updates before entering the event loop. This is a
    // synchronous git ls-remote call that takes ~300-800ms on a good
    // network; if it fails (no network, no tags), it's silently ignored.
    if debug { crate::log::log_debug("TUI: Checking for updates"); }
    state.check_for_updates();
    let (tx, rx) = std::sync::mpsc::channel::<state::TuiMessage>();
    state.tx = Some(tx);
    
    if debug { crate::log::log_debug(&format!("TUI: AppState created, is_git_repo={}, files={}, commits={}", state.is_git_repo, state.files.len(), state.commits.len())); }

    let mut frame_count: u64 = 0;
    let mut last_check = Instant::now();

    loop {
        frame_count += 1;

        if debug && frame_count == 1 { crate::log::log_debug("TUI: First draw starting"); }
        terminal.draw(|f| {
            draw_ui(f, &mut state);
        })?;
        if debug && frame_count == 1 { crate::log::log_debug("TUI: First draw completed"); }

        // Periodic git change detection (every 2 seconds) - only if not running a command
        if !state.console_running && last_check.elapsed() >= Duration::from_secs(2) {
            state.check_git_changes();
            last_check = Instant::now();
        }

        // Poll background thread messages
        while let Ok(msg) = rx.try_recv() {
            match msg {
                state::TuiMessage::ConsoleOutput(out) => {
                    state.console_output.push_str(&out);
                    // Scroll console to end
                    let all_lines: Vec<&str> = state.console_output.split('\n').collect();
                    let ch = (terminal.size()?.height * 35 / 100).clamp(6, 20);
                    let visible_h = ch.saturating_sub(2) as usize;
                    state.console_scroll = all_lines.len().saturating_sub(visible_h);
                }
                state::TuiMessage::UpdateAvailable(_version) => {
                    // Version check completed. Since we already called
                    // `check_for_updates()` synchronously at startup,
                    // this branch handles the corner case where the
                    // check was started by a background thread.
                    // Currently unused; the sync check in AppState::new
                    // already sets show_update_modal.
                }
                state::TuiMessage::CommandFinished(res) => {
                    state.console_running = false;
                    match res {
                        Ok(_) => {
                            state.status_message = "Command completed successfully.".to_string();
                        }
                        Err(e) => {
                            state.status_message = format!("Error: {}", e);
                        }
                    }
                    state.refresh_git_status();
                }
            }
        }

        // Poll for askpass prompt from helper process
        let prompt_path = std::env::temp_dir().join(format!("git-hero-askpass-prompt-{}.txt", state.session_id));
        if prompt_path.exists()
            && let Ok(prompt) = std::fs::read_to_string(&prompt_path)
        {
            let _ = std::fs::remove_file(&prompt_path);
            state.show_credentials_modal = true;
            state.credentials_prompt = prompt;
            state.credentials_input.clear();
            state.credentials_cursor = 0;
            let lower = state.credentials_prompt.to_lowercase();
            state.credentials_mask = lower.contains("password") || lower.contains("passphrase") || lower.contains("token") || lower.contains("clave");
        }

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == crossterm::event::KeyEventKind::Press {
                    if key.code == KeyCode::Char('c')
                        && key.modifiers.contains(KeyModifiers::CONTROL)
                    {
                        break;
                    }
                    if !handle_key_event(key, &mut state) {
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
