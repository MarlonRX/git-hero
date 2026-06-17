// ── Event Handlers ─────────────────────────────────────────────────────
// Keyboard and mouse input handling

use std::io::Stdout;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{backend::CrosstermBackend, layout::Rect, Terminal};

use crate::config::{save_config, Config};
use crate::git;
use crate::i18n::translate;
use crate::theme::{get_theme_by_name, get_themes};
use crate::ui::state::AppState;

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
            // Cycle focus: files → diff → commits → files
            s.focus_pane = match s.focus_pane.as_str() {
                "files" => "diff".into(),
                "diff" => "commits".into(),
                _ => "files".into(),
            };
            s.diff_scroll_offset = 0;
        }
        KeyCode::Up | KeyCode::Char('k') => {
            if s.focus_pane == "files" && !s.files.is_empty() {
                s.selected_file_idx = (s.selected_file_idx + s.files.len() - 1) % s.files.len();
                s.update_diff_content(); s.diff_scroll_offset = 0;
            } else if s.focus_pane == "commits" && !s.commits.is_empty() {
                s.selected_commit_idx = (s.selected_commit_idx + s.commits.len() - 1) % s.commits.len();
                s.commit_scroll_offset = 0;
                s.commit_detail_scroll = 0;
                s.update_diff_content(); s.diff_scroll_offset = 0;
            } else if s.focus_pane == "diff" {
                if s.diff_scroll_offset > 0 { s.diff_scroll_offset -= 1; }
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if s.focus_pane == "files" && !s.files.is_empty() {
                s.selected_file_idx = (s.selected_file_idx + 1) % s.files.len();
                s.update_diff_content(); s.diff_scroll_offset = 0;
            } else if s.focus_pane == "commits" && !s.commits.is_empty() {
                s.selected_commit_idx = (s.selected_commit_idx + 1) % s.commits.len();
                s.commit_scroll_offset = 0;
                s.commit_detail_scroll = 0;
                s.update_diff_content(); s.diff_scroll_offset = 0;
            } else if s.focus_pane == "diff" {
                s.diff_scroll_offset += 1;
            }
        }
        KeyCode::Char(' ') => {
            if s.focus_pane == "files" && !s.files.is_empty() {
                s.toggle_stage_file(s.selected_file_idx);
            }
        }
        KeyCode::Enter => {
            if s.focus_pane == "commits" && !s.commits.is_empty() {
                // Toggle commit detail view
                if s.show_commit_detail {
                    // Close detail view
                    s.show_commit_detail = false;
                    s.commit_detail_diff.clear();
                    s.commit_detail_scroll = 0;
                    s.update_diff_content();
                } else {
                    // Show commit details
                    s.commit_detail_scroll = 0;
                    let commit_hash = &s.commits[s.selected_commit_idx].hash;
                    match git::git_diff_commit(commit_hash) {
                        Ok(diff) => {
                            s.show_commit_detail = true;
                            s.commit_detail_diff = diff;
                        }
                        Err(e) => {
                            s.status_message = format!("Error loading commit: {}", e);
                        }
                    }
                }
            } else if s.focus_pane == "files" && !s.files.is_empty() {
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
        KeyCode::Char('c') | KeyCode::Char('C') => {
            s.show_commit_modal = true;
            s.commit_message_lines = vec![String::new()];
            s.commit_cursor_row = 0;
            s.commit_cursor_col = 0;
            s.commit_modal_scroll = 0;
        }
        KeyCode::Char('p') | KeyCode::Char('P') => s.execute_command("/push"),
        KeyCode::Char('f') | KeyCode::Char('F') => s.execute_command("/fetch"),
        KeyCode::Char('l') | KeyCode::Char('L') => s.execute_command("/pull"),
        KeyCode::Char('t') | KeyCode::Char('T') => s.execute_command("/themes"),
        KeyCode::Char('y') | KeyCode::Char('Y') => {
            // Copy current diff to clipboard (if available)
            if !s.active_diff.is_empty() {
                #[cfg(target_os = "macos")]
                {
                    use std::process::Command;
                    let _ = Command::new("pbcopy")
                        .stdin(std::process::Stdio::piped())
                        .spawn()
                        .and_then(|mut child| {
                            use std::io::Write;
                            child.stdin.as_mut().unwrap().write_all(s.active_diff.as_bytes())?;
                            child.wait()
                        });
                    s.status_message = "Diff copied to clipboard!".to_string();
                }
                #[cfg(target_os = "linux")]
                {
                    use std::process::Command;
                    let _ = Command::new("xclip")
                        .arg("-selection")
                        .arg("clipboard")
                        .stdin(std::process::Stdio::piped())
                        .spawn()
                        .and_then(|mut child| {
                            use std::io::Write;
                            child.stdin.as_mut().unwrap().write_all(s.active_diff.as_bytes())?;
                            child.wait()
                        });
                    s.status_message = "Diff copied to clipboard!".to_string();
                }
                #[cfg(not(any(target_os = "macos", target_os = "linux")))]
                {
                    s.status_message = "Copy not supported on this platform".to_string();
                }
            } else {
                s.status_message = "Nothing to copy".to_string();
            }
        }
        KeyCode::Char('?') | KeyCode::Char('h') | KeyCode::Char('H') => { s.show_help_modal = true; }
        // Capital d is already handled above with stash-pop
        KeyCode::Char('q') | KeyCode::Char('Q') => return false,
        KeyCode::Char('/') => { s.show_input = true; s.input_value = "/".into(); s.input_cursor_pos = 1; s.update_suggestions(); }
        // ── Scroll support ────────────────────────────────────────
        KeyCode::PageDown => {
            if s.focus_pane == "commits" && s.show_commit_detail {
                s.commit_detail_scroll += 5;
            } else if s.focus_pane == "commits" {
                s.commit_scroll_offset += 5;
            } else {
                s.diff_scroll_offset += 5;
            }
        }
        KeyCode::PageUp => {
            if s.focus_pane == "commits" && s.show_commit_detail {
                if s.commit_detail_scroll >= 5 { s.commit_detail_scroll -= 5; } else { s.commit_detail_scroll = 0; }
            } else if s.focus_pane == "commits" {
                if s.commit_scroll_offset >= 5 { s.commit_scroll_offset -= 5; } else { s.commit_scroll_offset = 0; }
            } else {
                if s.diff_scroll_offset >= 5 { s.diff_scroll_offset -= 5; } else { s.diff_scroll_offset = 0; }
            }
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
    
    // Sidebar click zones
    let info_end_y = inner.y + header_h + 3;   // STATUS panel (3 rows)
    let shortcuts_start_y = inner.y + inner.height.saturating_sub(9); // SHORTCUTS panel (9 rows)
    
    if col >= inner.x && col < split_x {
        // Clicked in sidebar
        if row < info_end_y {
            // STATUS panel → focus files
            s.focus_pane = "files".into();
        } else if row >= shortcuts_start_y {
            // SHORTCUTS panel → could be useful, focus diff for now
            s.focus_pane = "diff".into();
        } else {
            // FILES panel → select file
            let clicked = row.saturating_sub(info_end_y) as usize;
            if clicked < s.files.len() {
                s.focus_pane = "files".into();
                s.selected_file_idx = clicked;
                s.diff_scroll_offset = 0;
                if col >= inner.x + 2 && col <= inner.x + 6 {
                    s.toggle_stage_file(clicked);
                } else {
                    s.update_diff_content();
                }
            } else {
                // Clicked empty space in files → still focus files
                s.focus_pane = "files".into();
            }
        }
    } else if col >= split_x && col < inner.x + inner.width {
        // Right panel
        let right_height = inner.height.saturating_sub(header_h);
        let diff_height = (right_height * 65 / 100).max(3);
        let split_y = inner.y + header_h + diff_height;
        
        if row >= inner.y + header_h && row < split_y {
            // DIFF panel → focus diff
            s.focus_pane = "diff".into();
        } else if row >= split_y && row < inner.y + inner.height {
            // COMMITS panel → select commit
            let commit_start = split_y + 1;
            let clicked = row.saturating_sub(commit_start) as usize;
            if clicked < s.commits.len() {
                s.focus_pane = "commits".into();
                s.selected_commit_idx = clicked;
                s.diff_scroll_offset = 0;
                s.update_diff_content();
            } else {
                // Clicked empty space in commits → still focus commits
                s.focus_pane = "commits".into();
            }
        }
    }
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
            } else {
                if scroll_up {
                    if s.commit_scroll_offset >= 3 { s.commit_scroll_offset -= 3; } else { s.commit_scroll_offset = 0; }
                } else {
                    s.commit_scroll_offset += 3;
                }
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

// ── New Modal Key Handlers ─────────────────────────────────────────

fn handle_confirm_push_key(code: KeyCode, s: &mut AppState) {
    match code {
        KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter => {
            s.show_confirm_push = false;
            s.run_push();
        }
        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
            s.show_confirm_push = false;
        }
        _ => {}
    }
}

fn handle_confirm_pull_key(code: KeyCode, s: &mut AppState) {
    match code {
        KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter => {
            s.show_confirm_pull = false;
            s.run_pull();
        }
        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
            s.show_confirm_pull = false;
        }
        _ => {}
    }
}

fn handle_credentials_key(key: KeyEvent, s: &mut AppState) {
    match key.code {
        KeyCode::Enter => {
            let session_id = &s.session_id;
            let response_path = std::env::temp_dir().join(format!("git-hero-askpass-response-{}.txt", session_id));
            let _ = std::fs::write(&response_path, &s.credentials_input);
            s.show_credentials_modal = false;
            s.credentials_input.clear();
            s.credentials_cursor = 0;
        }
        KeyCode::Esc => {
            let session_id = &s.session_id;
            let response_path = std::env::temp_dir().join(format!("git-hero-askpass-response-{}.txt", session_id));
            let _ = std::fs::write(&response_path, "");
            s.show_credentials_modal = false;
            s.credentials_input.clear();
            s.credentials_cursor = 0;
        }
        KeyCode::Char(c) => {
            s.credentials_input.insert(s.credentials_cursor, c);
            s.credentials_cursor += 1;
        }
        KeyCode::Backspace => {
            if s.credentials_cursor > 0 {
                s.credentials_cursor -= 1;
                s.credentials_input.remove(s.credentials_cursor);
            }
        }
        KeyCode::Delete => {
            if s.credentials_cursor < s.credentials_input.len() {
                s.credentials_input.remove(s.credentials_cursor);
            }
        }
        KeyCode::Left => {
            s.credentials_cursor = s.credentials_cursor.saturating_sub(1);
        }
        KeyCode::Right => {
            if s.credentials_cursor < s.credentials_input.len() {
                s.credentials_cursor += 1;
            }
        }
        _ => {}
    }
}

fn handle_commit_modal_key(key: KeyEvent, s: &mut AppState) {
    let code = key.code;
    let mods = key.modifiers;
    
    // Confirm commit: Ctrl+Enter, Ctrl+S, or Ctrl+D
    if (code == KeyCode::Enter && mods.contains(KeyModifiers::CONTROL))
        || (code == KeyCode::Char('s') && mods.contains(KeyModifiers::CONTROL))
        || (code == KeyCode::Char('d') && mods.contains(KeyModifiers::CONTROL))
    {
        let msg = s.commit_message_lines.join("\n");
        let msg = msg.trim();
        if msg.is_empty() {
            s.status_message = "Error: commit message empty.".to_string();
            return;
        }
        let has_staged = s.files.iter().any(|f| f.staged);
        if !has_staged {
            let _ = git::git_add_all();
        }
        if let Err(e) = git::git_commit(msg) {
            s.status_message = format!("Error committing: {}", e);
        } else {
            s.selected_file_idx = 0;
            s.selected_commit_idx = 0;
            s.diff_scroll_offset = 0;
            s.refresh_git_status();
            s.status_message = translate(&s.language, "status_commit_success");
            s.show_commit_modal = false;
        }
        return;
    }

    match code {
        KeyCode::Esc => {
            s.show_commit_modal = false;
        }
        KeyCode::Enter => {
            let current_line = &s.commit_message_lines[s.commit_cursor_row];
            let (before, after) = current_line.split_at(s.commit_cursor_col);
            let before_str = before.to_string();
            let after_str = after.to_string();
            
            s.commit_message_lines[s.commit_cursor_row] = before_str;
            s.commit_message_lines.insert(s.commit_cursor_row + 1, after_str);
            s.commit_cursor_row += 1;
            s.commit_cursor_col = 0;
        }
        KeyCode::Backspace => {
            if s.commit_cursor_col > 0 {
                let current_line = &mut s.commit_message_lines[s.commit_cursor_row];
                current_line.remove(s.commit_cursor_col - 1);
                s.commit_cursor_col -= 1;
            } else if s.commit_cursor_row > 0 {
                let current_line = s.commit_message_lines.remove(s.commit_cursor_row);
                s.commit_cursor_row -= 1;
                let prev_line = &mut s.commit_message_lines[s.commit_cursor_row];
                s.commit_cursor_col = prev_line.len();
                prev_line.push_str(&current_line);
            }
        }
        KeyCode::Delete => {
            let current_line_len = s.commit_message_lines[s.commit_cursor_row].len();
            if s.commit_cursor_col < current_line_len {
                let current_line = &mut s.commit_message_lines[s.commit_cursor_row];
                current_line.remove(s.commit_cursor_col);
            } else if s.commit_cursor_row + 1 < s.commit_message_lines.len() {
                let next_line = s.commit_message_lines.remove(s.commit_cursor_row + 1);
                let current_line = &mut s.commit_message_lines[s.commit_cursor_row];
                current_line.push_str(&next_line);
            }
        }
        KeyCode::Left => {
            if s.commit_cursor_col > 0 {
                s.commit_cursor_col -= 1;
            } else if s.commit_cursor_row > 0 {
                s.commit_cursor_row -= 1;
                s.commit_cursor_col = s.commit_message_lines[s.commit_cursor_row].len();
            }
        }
        KeyCode::Right => {
            let current_line_len = s.commit_message_lines[s.commit_cursor_row].len();
            if s.commit_cursor_col < current_line_len {
                s.commit_cursor_col += 1;
            } else if s.commit_cursor_row + 1 < s.commit_message_lines.len() {
                s.commit_cursor_row += 1;
                s.commit_cursor_col = 0;
            }
        }
        KeyCode::Up => {
            if s.commit_cursor_row > 0 {
                s.commit_cursor_row -= 1;
                let prev_len = s.commit_message_lines[s.commit_cursor_row].len();
                if s.commit_cursor_col > prev_len {
                    s.commit_cursor_col = prev_len;
                }
            }
        }
        KeyCode::Down => {
            if s.commit_cursor_row + 1 < s.commit_message_lines.len() {
                s.commit_cursor_row += 1;
                let next_len = s.commit_message_lines[s.commit_cursor_row].len();
                if s.commit_cursor_col > next_len {
                    s.commit_cursor_col = next_len;
                }
            }
        }
        KeyCode::Char(c) => {
            let current_line = &mut s.commit_message_lines[s.commit_cursor_row];
            current_line.insert(s.commit_cursor_col, c);
            s.commit_cursor_col += 1;
        }
        _ => {}
    }
}
