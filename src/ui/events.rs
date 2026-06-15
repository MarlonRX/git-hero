// ── Event Handlers ─────────────────────────────────────────────────────
// Keyboard and mouse input handling

use std::io::Stdout;

use crossterm::event::KeyCode;
use ratatui::{backend::CrosstermBackend, layout::Rect, Terminal};

use crate::config::{save_config, Config};
use crate::git;
use crate::i18n::translate;
use crate::theme::{get_theme_by_name, get_themes};
use crate::ui::state::AppState;

/// Returns false if the app should quit
pub fn handle_key_event(code: KeyCode, s: &mut AppState) -> bool {
    // ── Setup Wizard ─────────────────────────────────────────────
    if s.setup_step > 0 {
        handle_setup_key(code, s);
        return true;
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

// ── Sub-handlers ───────────────────────────────────────────────────

fn handle_setup_key(code: KeyCode, s: &mut AppState) {
    match code {
        KeyCode::Up | KeyCode::Char('k') => {
            let max = if s.setup_step == 3 {
                get_themes().len().saturating_sub(1)
            } else {
                1
            };
            s.setup_cursor = (s.setup_cursor + max) % (max + 1);
            if s.setup_step == 3 {
                s.theme = get_themes()[s.setup_cursor];
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            let limit = if s.setup_step == 3 { get_themes().len() } else { 2 };
            s.setup_cursor = (s.setup_cursor + 1) % limit;
            if s.setup_step == 3 {
                s.theme = get_themes()[s.setup_cursor];
            }
        }
        KeyCode::Enter => match s.setup_step {
            1 => {
                s.language = if s.setup_cursor == 0 { "en".into() } else { "es".into() };
                s.setup_step = 2;
                s.setup_cursor = 0;
            }
            2 => {
                s.nerd_font = s.setup_cursor == 0;
                s.setup_step = 3;
                s.setup_cursor = 0;
            }
            3 => {
                s.theme = get_themes()[s.setup_cursor];
                s.setup_step = 0;
                s.focus_pane = "files".into();
                let _ = save_config(&Config {
                    language: s.language.clone(),
                    nerd_font: s.nerd_font,
                    theme: s.theme.name.to_string(),
                });
                s.refresh_git_status();
                s.status_message = translate(&s.language, "welcome_message");
            }
            _ => {}
        },
        _ => {}
    }
}

fn handle_theme_modal_key(code: KeyCode, s: &mut AppState) {
    let themes = get_themes();
    match code {
        KeyCode::Esc => {
            s.theme = get_theme_by_name(&s.saved_theme);
            s.show_theme_modal = false;
        }
        KeyCode::Up | KeyCode::Char('k') => {
            s.theme_cursor = (s.theme_cursor + themes.len() - 1) % themes.len();
            s.theme = themes[s.theme_cursor];
            s.update_diff_content();
        }
        KeyCode::Down | KeyCode::Char('j') => {
            s.theme_cursor = (s.theme_cursor + 1) % themes.len();
            s.theme = themes[s.theme_cursor];
            s.update_diff_content();
        }
        KeyCode::Enter => {
            s.show_theme_modal = false;
            let _ = save_config(&Config {
                language: s.language.clone(),
                nerd_font: s.nerd_font,
                theme: s.theme.name.to_string(),
            });
            s.status_message = format!("Theme changed to: {}", s.theme.name);
        }
        _ => {}
    }
}

fn handle_init_wizard_key(code: KeyCode, s: &mut AppState) {
    // Text input sub-mode
    if (s.init_wizard_step == 1 && s.init_cursor == 2) || s.init_wizard_step == 2 {
        match code {
            KeyCode::Esc => {
                s.init_wizard_active = false;
                s.status_message = "Git initialization cancelled.".to_string();
            }
            KeyCode::Enter => {
                if s.init_wizard_step == 1 {
                    if s.input_value.is_empty() {
                        s.status_message = "Branch name cannot be empty.".to_string();
                        return;
                    }
                    s.init_branch_name = s.input_value.clone();
                    s.init_wizard_step = 2;
                    s.input_value.clear();
                    s.input_cursor_pos = 0;
                } else {
                    s.init_remote_url = s.input_value.clone();
                    s.init_wizard_step = 3;
                    s.init_cursor = 0;
                    s.input_value.clear();
                    s.input_cursor_pos = 0;
                }
            }
            KeyCode::Backspace => {
                if s.input_cursor_pos > 0 {
                    s.input_value.remove(s.input_cursor_pos - 1);
                    s.input_cursor_pos -= 1;
                }
            }
            KeyCode::Left => {
                if s.input_cursor_pos > 0 { s.input_cursor_pos -= 1; }
            }
            KeyCode::Right => {
                if s.input_cursor_pos < s.input_value.len() { s.input_cursor_pos += 1; }
            }
            KeyCode::Char(c) => {
                s.input_value.insert(s.input_cursor_pos, c);
                s.input_cursor_pos += 1;
            }
            _ => {}
        }
        return;
    }

    // Selection mode
    match code {
        KeyCode::Esc => {
            s.init_wizard_active = false;
            s.status_message = "Git initialization cancelled.".to_string();
        }
        KeyCode::Up | KeyCode::Char('k') => {
            if s.init_wizard_step == 1 {
                s.init_cursor = (s.init_cursor + 2) % 3;
            } else if s.init_wizard_step == 3 {
                s.init_cursor = (s.init_cursor + 1) % 2;
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if s.init_wizard_step == 1 {
                s.init_cursor = (s.init_cursor + 1) % 3;
            } else if s.init_wizard_step == 3 {
                s.init_cursor = (s.init_cursor + 1) % 2;
            }
        }
        KeyCode::Enter => {
            if s.init_wizard_step == 1 {
                match s.init_cursor {
                    0 => { s.init_branch_name = "main".into(); s.init_wizard_step = 2; s.input_value.clear(); s.input_cursor_pos = 0; }
                    1 => { s.init_branch_name = "master".into(); s.init_wizard_step = 2; s.input_value.clear(); s.input_cursor_pos = 0; }
                    _ => { s.input_value.clear(); s.input_cursor_pos = 0; }
                }
            } else if s.init_wizard_step == 3 && s.init_cursor == 0 {
                s.init_wizard_active = false;
                let _ = git::run_git(&["init"]);
                let _ = git::run_git(&["checkout", "-b", &s.init_branch_name]);
                if !s.init_remote_url.is_empty() {
                    let _ = git::run_git(&["remote", "add", "origin", &s.init_remote_url]);
                }
                let _ = git::git_add_all();
                let _ = git::git_commit("Initial commit");
                s.refresh_git_status();
                s.status_message = "Git repository initialized successfully!".to_string();
            }
        }
        _ => {}
    }
}

fn handle_input_key(code: KeyCode, s: &mut AppState) {
    match code {
        KeyCode::Esc => {
            s.show_input = false; s.input_value.clear(); s.input_cursor_pos = 0; s.suggestions.clear();
        }
        KeyCode::Tab => {
            if !s.suggestions.is_empty() {
                s.input_value = s.suggestions[s.active_sug].clone();
                s.input_cursor_pos = s.input_value.len();
                s.update_suggestions();
            }
        }
        KeyCode::Up => {
            if !s.suggestions.is_empty() {
                s.active_sug = (s.active_sug + s.suggestions.len() - 1) % s.suggestions.len();
            }
        }
        KeyCode::Down => {
            if !s.suggestions.is_empty() {
                s.active_sug = (s.active_sug + 1) % s.suggestions.len();
            }
        }
        KeyCode::Enter => {
            let cmd = s.input_value.clone();
            s.show_input = false; s.input_value.clear(); s.input_cursor_pos = 0; s.suggestions.clear();
            s.execute_command(&cmd);
        }
        KeyCode::Backspace => {
            if s.input_cursor_pos > 0 {
                s.input_value.remove(s.input_cursor_pos - 1);
                s.input_cursor_pos -= 1;
                s.update_suggestions();
            }
        }
        KeyCode::Left => { if s.input_cursor_pos > 0 { s.input_cursor_pos -= 1; } }
        KeyCode::Right => { if s.input_cursor_pos < s.input_value.len() { s.input_cursor_pos += 1; } }
        KeyCode::Char(c) => {
            s.input_value.insert(s.input_cursor_pos, c);
            s.input_cursor_pos += 1;
            s.update_suggestions();
        }
        _ => {}
    }
}

fn handle_no_repo_key(code: KeyCode, s: &mut AppState) -> bool {
    match code {
        KeyCode::Up | KeyCode::Down | KeyCode::Char('j') | KeyCode::Char('k') => {
            s.init_cursor = 1 - s.init_cursor;
        }
        KeyCode::Enter => {
            if s.init_cursor == 0 {
                s.init_wizard_active = true; s.init_wizard_step = 1; s.init_cursor = 0;
                s.init_branch_name = "main".into(); s.init_remote_url.clear();
                s.status_message = "Select main branch name.".to_string();
            } else {
                s.show_input = true; s.input_value = "/cd ".into(); s.input_cursor_pos = 4;
                s.update_suggestions();
            }
        }
        KeyCode::Char('t') | KeyCode::Char('T') => s.execute_command("/themes"),
        KeyCode::Char('q') | KeyCode::Char('Q') => return false,
        _ => {}
    }
    true
}

fn handle_repo_key(code: KeyCode, s: &mut AppState) -> bool {
    match code {
        KeyCode::Tab => {
            s.focus_pane = if s.focus_pane == "files" { "commits".into() } else { "files".into() };
            s.diff_scroll_offset = 0;
            s.update_diff_content();
        }
        KeyCode::Up | KeyCode::Char('k') => {
            if s.focus_pane == "files" && !s.files.is_empty() {
                s.selected_file_idx = (s.selected_file_idx + s.files.len() - 1) % s.files.len();
                s.update_diff_content(); s.diff_scroll_offset = 0;
            } else if s.focus_pane == "commits" && !s.commits.is_empty() {
                s.selected_commit_idx = (s.selected_commit_idx + s.commits.len() - 1) % s.commits.len();
                s.update_diff_content(); s.diff_scroll_offset = 0;
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if s.focus_pane == "files" && !s.files.is_empty() {
                s.selected_file_idx = (s.selected_file_idx + 1) % s.files.len();
                s.update_diff_content(); s.diff_scroll_offset = 0;
            } else if s.focus_pane == "commits" && !s.commits.is_empty() {
                s.selected_commit_idx = (s.selected_commit_idx + 1) % s.commits.len();
                s.update_diff_content(); s.diff_scroll_offset = 0;
            }
        }
        KeyCode::Char(' ') | KeyCode::Enter => {
            if s.focus_pane == "files" && !s.files.is_empty() {
                s.toggle_stage_file(s.selected_file_idx);
            }
        }
        // ── Extended shortcuts ──────────────────────────────────
        KeyCode::Char('a') | KeyCode::Char('A') => s.execute_command("/stage-all"),
        KeyCode::Char('u') | KeyCode::Char('U') => s.execute_command("/unstage-all"),
        KeyCode::Char('r') | KeyCode::Char('R') => s.execute_command("/undo-commit"),
        KeyCode::Char('s') | KeyCode::Char('S') => s.execute_command("/stash"),
        KeyCode::Char('d') | KeyCode::Char('D') => s.execute_command("/stash-pop"),
        KeyCode::Char('b') | KeyCode::Char('B') => s.execute_command("/branches"),
        KeyCode::Char('n') | KeyCode::Char('N') => { s.show_input = true; s.input_value = "/branch ".into(); s.input_cursor_pos = 8; s.update_suggestions(); }
        KeyCode::Char('o') | KeyCode::Char('O') => { s.show_input = true; s.input_value = "/remote ".into(); s.input_cursor_pos = 8; }
        // ── Original shortcuts ──────────────────────────────────
        KeyCode::Char('c') | KeyCode::Char('C') => { s.show_input = true; s.input_value = "/commit ".into(); s.input_cursor_pos = 8; }
        KeyCode::Char('p') | KeyCode::Char('P') => s.execute_command("/push"),
        KeyCode::Char('f') | KeyCode::Char('F') => s.execute_command("/fetch"),
        KeyCode::Char('l') | KeyCode::Char('L') => s.execute_command("/pull"),
        KeyCode::Char('t') | KeyCode::Char('T') => s.execute_command("/themes"),
        KeyCode::Char('?') | KeyCode::Char('h') | KeyCode::Char('H') => { s.show_help_modal = true; }
        // Capital d is already handled above with stash-pop
        KeyCode::Char('q') | KeyCode::Char('Q') => return false,
        KeyCode::Char('/') => { s.show_input = true; s.input_value = "/".into(); s.input_cursor_pos = 1; s.update_suggestions(); }
        KeyCode::PageDown => { s.diff_scroll_offset += 5; }
        KeyCode::PageUp => {
            if s.diff_scroll_offset >= 5 { s.diff_scroll_offset -= 5; } else { s.diff_scroll_offset = 0; }
        }
        _ => {}
    }
    true
}

// ── Mouse Handler ──────────────────────────────────────────────────

pub fn handle_mouse_click(
    col: u16,
    row: u16,
    s: &mut AppState,
    terminal: &Terminal<CrosstermBackend<Stdout>>,
) {
    let size = terminal.size().unwrap_or_default();
    let area = Rect { x: 0, y: 0, width: size.width, height: size.height };
    let target_w = (area.width as f32 * 0.80) as u16;
    let target_h = (area.height as f32 * 0.85) as u16;
    let outer = Rect {
        x: area.x + (area.width.saturating_sub(target_w)) / 2,
        y: area.y + (area.height.saturating_sub(target_h)) / 2,
        width: target_w.max(40),
        height: target_h.max(10),
    };
    let inner = Rect {
        x: outer.x + 1,
        y: outer.y + 1,
        width: outer.width.saturating_sub(2),
        height: outer.height.saturating_sub(2),
    };

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
    // Dashboard clicks
    mouse_dashboard(col, row, s, inner);
}

fn mouse_setup(col: u16, row: u16, s: &mut AppState, area: Rect) {
    let mw = 60; let mh = 14;
    let mx = (area.width.saturating_sub(mw)) / 2;
    let my = (area.height.saturating_sub(mh)) / 2;
    if col >= mx + 2 && col < mx + mw - 2 && row >= my + 2 && row < my + mh - 2 {
        let idx = row.saturating_sub(my + 2) as usize;
        match s.setup_step {
            1 | 2 => { if idx < 2 { s.setup_cursor = idx; } }
            3 => {
                let themes = get_themes();
                let start = if s.setup_cursor > 3 { s.setup_cursor - 3 } else { 0 };
                let clicked = start + idx;
                if clicked < themes.len() { s.setup_cursor = clicked; }
            }
            _ => {}
        }
    }
}

fn mouse_theme(col: u16, row: u16, s: &mut AppState, area: Rect) {
    let themes = get_themes();
    let mw = 50; let mh = (themes.len() as u16 + 6).min(area.height - 2);
    let mx = (area.width.saturating_sub(mw)) / 2;
    let my = (area.height.saturating_sub(mh)) / 2;
    if col >= mx + 2 && col < mx + mw - 2 && row >= my + 2 && row < my + mh - 3 {
        let start = if s.theme_cursor > 4 { s.theme_cursor - 4 } else { 0 };
        let idx = start + row.saturating_sub(my + 2) as usize;
        if idx < themes.len() {
            s.theme_cursor = idx; s.theme = themes[idx]; s.update_diff_content();
        }
    }
}

fn mouse_init_wizard(_col: u16, row: u16, s: &mut AppState, inner: Rect) {
    let py = inner.y + 1;
    let clicked = row.saturating_sub(py + 2);
    match s.init_wizard_step {
        1 => {
            if clicked >= 3 && clicked <= 5 {
                let idx = clicked as usize - 3;
                s.init_cursor = idx;
                if idx == 0 { s.init_branch_name = "main".into(); s.init_wizard_step = 2; s.input_value.clear(); }
                else if idx == 1 { s.init_branch_name = "master".into(); s.init_wizard_step = 2; s.input_value.clear(); }
            }
        }
        3 => {
            if clicked == 7 {
                s.init_wizard_active = false;
                let _ = git::run_git(&["init"]);
                let _ = git::run_git(&["checkout", "-b", &s.init_branch_name]);
                if !s.init_remote_url.is_empty() {
                    let _ = git::run_git(&["remote", "add", "origin", &s.init_remote_url]);
                }
                let _ = git::git_add_all();
                let _ = git::git_commit("Initial commit");
                s.refresh_git_status();
                s.status_message = "Git repository initialized successfully!".to_string();
            }
        }
        _ => {}
    }
}

fn mouse_no_repo(_col: u16, row: u16, s: &mut AppState, inner: Rect) {
    let py = inner.y + 1;
    let cy = py + 3;
    if row == cy + 7 {
        s.init_cursor = 0; s.init_wizard_active = true; s.init_wizard_step = 1;
        s.init_branch_name = "main".into(); s.init_remote_url.clear();
        s.status_message = "Select main branch name.".to_string();
    } else if row == cy + 9 {
        s.init_cursor = 1; s.show_input = true; s.input_value = "/cd ".into();
        s.input_cursor_pos = 4; s.update_suggestions();
    }
}

fn mouse_dashboard(col: u16, row: u16, s: &mut AppState, inner: Rect) {
    let sidebar_w = (inner.width / 4).max(20);
    let split_x = inner.x + sidebar_w;
    let header_h: u16 = 2;
    let files_start_y = inner.y + header_h + 3;

    if col >= inner.x && col < split_x {
        let clicked = row.saturating_sub(files_start_y) as usize;
        if clicked < s.files.len() {
            s.focus_pane = "files".into(); s.selected_file_idx = clicked; s.diff_scroll_offset = 0;
            if col >= inner.x + 2 && col <= inner.x + 6 {
                s.toggle_stage_file(clicked);
            } else { s.update_diff_content(); }
        }
    } else if col > split_x && col < inner.x + inner.width {
        let split_y = inner.y + header_h + ((inner.height.saturating_sub(header_h)) * 65 / 100);
        let commit_start = split_y + 1;
        let clicked = row.saturating_sub(commit_start) as usize;
        if clicked < s.commits.len() {
            s.focus_pane = "commits".into(); s.selected_commit_idx = clicked; s.diff_scroll_offset = 0;
            s.update_diff_content();
        }
    }
}
