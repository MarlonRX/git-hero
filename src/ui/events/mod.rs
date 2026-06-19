pub mod keyboard;
pub mod mouse;

use std::io::Stdout;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{backend::CrosstermBackend, layout::Rect, Terminal};

use crate::ui::state::AppState;
use keyboard::*;
use mouse::*;
use crate::ui::rendering::components::calculate_layout;

/// Returns false if the app should quit
pub fn handle_key_event(key: KeyEvent, s: &mut AppState) -> bool {
    let code = key.code;

    // ── Credentials Modal ─────────────────────────────────────────
    if s.show_credentials_modal {
        handle_credentials_key(key, s);
        return true;
    }

    // ── Commit Modal ──────────────────────────────────────────────
    if s.show_commit_modal {
        handle_commit_modal_key(key, s);
        return true;
    }

    // ── Confirm Push Modal ────────────────────────────────────────
    if s.show_confirm_push {
        handle_confirm_push_key(code, s);
        return true;
    }

    // ── Confirm Pull Modal ────────────────────────────────────────
    if s.show_confirm_pull {
        handle_confirm_pull_key(code, s);
        return true;
    }

    // ── Update Available Modal ───────────────────────────────────
    if s.show_update_modal {
        handle_update_modal_key(code, s);
        return true;
    }

    // ── Confirm Remove Modal ──────────────────────────────────────
    if s.show_confirm_remove {
        handle_confirm_remove_key(code, s);
        return true;
    }

    // ── Setup Wizard ─────────────────────────────────────────────
    if s.setup_step > 0 {
        handle_setup_key(code, s);
        return true;
    }

    // ── Mini Console ─────────────────────────────────────────────
    if s.console_visible {
        match code {
            KeyCode::Esc => { s.console_visible = false; return true; }
            KeyCode::Up | KeyCode::Char('k') => {
                s.console_scroll = s.console_scroll.saturating_sub(1);
                return true;
            }
            KeyCode::Down | KeyCode::Char('j') => {
                s.console_scroll += 1;
                return true;
            }
            KeyCode::PageUp => {
                s.console_scroll = s.console_scroll.saturating_sub(10);
                return true;
            }
            KeyCode::PageDown => {
                s.console_scroll += 10;
                return true;
            }
            _ => {} // allow other keys to fall through and close modals etc.
        }
    }

    // ── Help Modal ───────────────────────────────────────────────
    if s.show_help_modal {
        s.show_help_modal = false;
        return true;
    }

    // ── Docs Modal ───────────────────────────────────────────────
    if s.show_docs_modal {
        s.show_docs_modal = false;
        return true;
    }

    // ── Theme Modal ──────────────────────────────────────────────
    if s.show_theme_modal {
        handle_theme_modal_key(code, s);
        return true;
    }

    // ── Init Wizard ──────────────────────────────────────────────
    if s.init_wizard_active {
        handle_init_wizard_key(code, s);
        return true;
    }

    // ── Command Input Mode ───────────────────────────────────────
    if s.show_input {
        handle_input_key(code, s);
        return true;
    }

    // ── Normal Mode (not a repo) ─────────────────────────────────
    if !s.is_git_repo {
        return handle_no_repo_key(code, s);
    }

    // ── Normal Mode (inside repo) ────────────────────────────────
    handle_repo_key(code, s)
}

pub fn handle_mouse_click(
    col: u16,
    row: u16,
    s: &mut AppState,
    terminal: &Terminal<CrosstermBackend<Stdout>>,
) {
    let size = terminal.size().unwrap_or_default();
    let area = Rect { x: 0, y: 0, width: size.width, height: size.height };
    let (_outer, inner) = calculate_layout(area);

    // Close input
    if s.show_input {
        let iy = inner.y + inner.height - 1;
        let sl = s.suggestions.len() as u16;
        let clicked_input = row == iy && col >= inner.x && col < inner.x + inner.width;
        let clicked_sug = sl > 0 && row >= iy.saturating_sub(sl) - 1 && row < iy && col >= inner.x;
        if !clicked_input && !clicked_sug {
            s.show_input = false; s.input_value.clear(); s.input_cursor_pos = 0; s.suggestions.clear();
            return;
        }
    }

    // Setup wizard
    if s.setup_step > 0 {
        mouse_setup(col, row, s, area); return;
    }
    // Help modal
    if s.show_help_modal { s.show_help_modal = false; return; }
    // Docs modal
    if s.show_docs_modal { s.show_docs_modal = false; return; }
    // Theme modal
    if s.show_theme_modal { mouse_theme(col, row, s, area); return; }
    // Init wizard
    if s.init_wizard_active { mouse_init_wizard(col, row, s, inner); return; }
    // No repo panel
    if !s.is_git_repo { mouse_no_repo(col, row, s, inner); return; }
    // Confirm-remove modal: any click outside the modal dismisses it.
    if s.show_confirm_remove { s.show_confirm_remove = false; return; }
    // Dashboard clicks
    mouse_dashboard(col, row, s, inner);
}

/// Handle mouse wheel scroll - scrolls the panel under the cursor
pub fn handle_mouse_scroll(
    scroll_up: bool,
    col: u16,
    row: u16,
    s: &mut AppState,
    terminal: &Terminal<CrosstermBackend<Stdout>>,
) {
    let size = terminal.size().unwrap_or_default();
    let area = Rect { x: 0, y: 0, width: size.width, height: size.height };
    let (_outer, inner) = calculate_layout(area);

    // Don't scroll if modals are open
    if s.show_theme_modal || s.show_help_modal || s.show_docs_modal || s.setup_step > 0 || s.init_wizard_active {
        return;
    }

    let sidebar_w = (inner.width / 4).max(20);
    let split_x = inner.x + sidebar_w;
    let header_h: u16 = 2;
    let content_top = inner.y + header_h;

    // Determine which panel the mouse is over
    if col >= split_x && col < inner.x + inner.width {
        // Right panel (diff or commits)
        let right_height = inner.height.saturating_sub(header_h);
        let diff_height = (right_height * 65 / 100).max(3);
        let split_y = content_top + diff_height;

        if row >= content_top && row < split_y {
            // Diff panel - scroll diff
            if scroll_up {
                if s.diff_scroll_offset >= 3 { s.diff_scroll_offset -= 3; } else { s.diff_scroll_offset = 0; }
            } else {
                s.diff_scroll_offset += 3;
            }
        } else if row >= split_y && row < inner.y + inner.height {
            // Commits panel - scroll commits or commit detail
            if s.show_commit_detail {
                if scroll_up {
                    if s.commit_detail_scroll >= 3 { s.commit_detail_scroll -= 3; } else { s.commit_detail_scroll = 0; }
                } else {
                    s.commit_detail_scroll += 3;
                }
            } else if scroll_up {
                if s.commit_scroll_offset >= 3 { s.commit_scroll_offset -= 3; } else { s.commit_scroll_offset = 0; }
            } else {
                s.commit_scroll_offset += 3;
            }
        }
    } else if col >= inner.x && col < split_x {
        // Left sidebar - scroll files list
        let files_area_top = content_top + 3; // After STATUS block
        if row >= files_area_top {
            // Could add file list scroll here if needed in the future
            // For now, just ignore sidebar scroll
        }
    }
}
