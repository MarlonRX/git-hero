pub mod keyboard;
pub mod mouse;

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{Terminal, backend::CrosstermBackend, layout::Rect};
use std::io::Stdout;

use crate::ui::rendering::body_area;
use crate::ui::rendering::components::calculate_layout_scaled;
use crate::ui::rendering::panels::{
    browser_area, browser_rows_rect, browser_scroll, sidebar_split,
};
use crate::ui::state::AppState;
use keyboard::*;
use mouse::*;

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
            KeyCode::Esc => {
                s.console_visible = false;
                return true;
            }
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

    // ── Repo browser (sidebar dropdown). Placed AFTER command input so
    // pressing `/` opens the command bar and typing a written command
    // never leaks into the browser filter or fires letter shortcuts. ─
    if s.show_repo_overview && !s.init_wizard_active {
        handle_repo_overview_key(key, s);
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
    let area = Rect {
        x: 0,
        y: 0,
        width: size.width,
        height: size.height,
    };
    let (_outer, inner) = calculate_layout_scaled(area);
    let body = body_area(inner);

    // Close input
    if s.show_input {
        let iy = inner.y + inner.height - 1;
        let sl = s.suggestions.len() as u16;
        let clicked_input = row == iy && col >= inner.x && col < inner.x + inner.width;
        let clicked_sug = sl > 0 && row >= iy.saturating_sub(sl) - 1 && row < iy && col >= inner.x;
        if !clicked_input && !clicked_sug {
            s.show_input = false;
            s.input_value.clear();
            s.input_cursor_pos = 0;
            s.suggestions.clear();
            return;
        }
    }

    // Setup wizard
    if s.setup_step > 0 {
        mouse_setup(col, row, s, area);
        return;
    }
    // Help modal
    if s.show_help_modal {
        s.show_help_modal = false;
        return;
    }
    // Docs modal
    if s.show_docs_modal {
        s.show_docs_modal = false;
        return;
    }
    // Theme modal
    if s.show_theme_modal {
        mouse_theme(col, row, s, area);
        return;
    }
    // Init wizard
    if s.init_wizard_active {
        mouse_init_wizard(col, row, s, body);
        return;
    }
    // Repo browser (sidebar dropdown): clicking a row opens that repo;
    // clicks inside the browser's own rect are swallowed; clicks above it
    // fall through to the FILES panel. Geometry helpers are shared with
    // draw_dashboard so hit-testing can never drift from what is painted.
    if s.show_repo_overview {
        let bar = browser_area(body, s.repo_view.len(), !s.is_git_repo);
        let in_col = col > bar.x && col < bar.x + bar.width.saturating_sub(1);
        let in_rows = row >= bar.y && row < bar.y + bar.height;
        if in_col && in_rows {
            let (rows_top, rows_h) = browser_rows_rect(bar);
            let rel = row.saturating_sub(rows_top) as usize;
            if rel < rows_h {
                let start = browser_scroll(s.repo_view.len(), rows_h, s.repo_cursor);
                let pos = start + rel;
                if pos < s.repo_view.len() {
                    s.repo_cursor = pos;
                    s.open_selected_repo();
                }
            }
            return;
        }
    }
    // No repo panel
    if !s.is_git_repo {
        mouse_no_repo(col, row, s, body);
        return;
    }
    // Confirm-remove modal: any click outside the modal dismisses it.
    if s.show_confirm_remove {
        s.show_confirm_remove = false;
        return;
    }
    // Dashboard clicks
    mouse_dashboard(col, row, s, body);
}

/// Handle mouse wheel scroll - scrolls the panel under the cursor.
///
/// For the three list panels (files, commits, repo browser) the wheel
/// moves the *selection* and the panel's auto-windowed view follows —
/// there is no free-floating viewport that can hide the cursor. Only the
/// diff/detail text panes scroll as viewports.
pub fn handle_mouse_scroll(
    scroll_up: bool,
    col: u16,
    row: u16,
    s: &mut AppState,
    terminal: &Terminal<CrosstermBackend<Stdout>>,
) {
    let size = terminal.size().unwrap_or_default();
    let area = Rect {
        x: 0,
        y: 0,
        width: size.width,
        height: size.height,
    };
    let (_outer, inner) = calculate_layout_scaled(area);

    // Don't scroll if modals are open
    if s.show_theme_modal
        || s.show_help_modal
        || s.show_docs_modal
        || s.setup_step > 0
        || s.init_wizard_active
    {
        return;
    }

    let body = body_area(inner);
    let (files_col, browser) = sidebar_split(body, s.show_repo_overview, s.repo_view.len());
    let split_x = body.x + files_col.width;

    // Repo browser: the wheel moves its cursor through the filtered view.
    // Only consume the event when the pointer is actually over the browser
    // rect; otherwise it should reach the FILES list above it.
    if s.show_repo_overview
        && !s.init_wizard_active
        && let Some(b) = browser
    {
        let in_col = col > b.x && col < b.x + b.width;
        let in_rows = row >= b.y && row < b.y + b.height;
        if in_col && in_rows {
            let (rows_top, rows_h) = browser_rows_rect(b);
            let rel = row.saturating_sub(rows_top) as usize;
            if rel < rows_h && !s.repo_view.is_empty() {
                s.repo_cursor = if scroll_up {
                    s.repo_cursor.saturating_sub(3)
                } else {
                    (s.repo_cursor + 3).min(s.repo_view.len() - 1)
                };
            }
            return;
        }
    }

    if col >= split_x {
        // Right panel (diff over commits), same split as draw_dashboard.
        let right = Rect {
            x: split_x,
            y: body.y,
            width: body.width - files_col.width,
            height: body.height,
        };
        let diff_pct: u16 = if s.focus_pane == "commits" { 50 } else { 70 };
        let diff_h = (right.height * diff_pct / 100).max(3);
        let commits_top = right.y + diff_h;

        if row < commits_top {
            // Diff panel - scroll diff
            if scroll_up {
                s.diff_scroll_offset = s.diff_scroll_offset.saturating_sub(3);
            } else {
                s.diff_scroll_offset += 3;
            }
        } else if row < body.y + body.height {
            // Commits panel
            if s.show_commit_detail {
                if scroll_up {
                    s.commit_detail_scroll = s.commit_detail_scroll.saturating_sub(3);
                } else {
                    s.commit_detail_scroll += 3;
                }
            } else if !s.commits.is_empty() {
                s.selected_commit_idx = if scroll_up {
                    s.selected_commit_idx.saturating_sub(3)
                } else {
                    (s.selected_commit_idx + 3).min(s.commits.len() - 1)
                };
                s.update_diff_content();
                s.diff_scroll_offset = 0;
            }
        }
    } else if col >= body.x && !s.flat_entries.is_empty() {
        // Left sidebar - the wheel now actually scrolls the files list by
        // moving the selection (the old code ignored sidebar wheel).
        s.flat_idx = if scroll_up {
            s.flat_idx.saturating_sub(3)
        } else {
            (s.flat_idx + 3).min(s.flat_entries.len() - 1)
        };
        if let Some(entry) = s.flat_entries.get(s.flat_idx) {
            s.selected_file_idx = entry.file_idx;
        }
        s.update_diff_content();
        s.diff_scroll_offset = 0;
    }
}
