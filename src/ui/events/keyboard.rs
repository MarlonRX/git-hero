use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::config::{load_config, save_config, Config};
use crate::git;
use crate::i18n::translate;
use crate::theme::{get_theme_by_name, get_themes};
use crate::ui::state::AppState;

pub fn handle_setup_key(code: KeyCode, s: &mut AppState) {
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
                    skipped_version: None,
                });
                s.refresh_git_status();
                s.status_message = translate(&s.language, "welcome_message").into_owned();
            }
            _ => {}
        },
        _ => {}
    }
}

pub fn handle_theme_modal_key(code: KeyCode, s: &mut AppState) {
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
            let existing_skipped = crate::config::load_config()
                .ok()
                .and_then(|c| c.skipped_version);
            let _ = save_config(&Config {
                language: s.language.clone(),
                nerd_font: s.nerd_font,
                theme: s.theme.name.to_string(),
                skipped_version: existing_skipped,
            });
            s.status_message = format!("Theme changed to: {}", s.theme.name);
        }
        _ => {}
    }
}

pub fn handle_init_wizard_key(code: KeyCode, s: &mut AppState) {
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

pub fn handle_input_key(code: KeyCode, s: &mut AppState) {
    match code {
        KeyCode::Esc => {
            s.show_input = false; s.input_value.clear(); s.input_cursor_pos = 0; s.suggestions.clear();
        }
        KeyCode::Tab => {
            if !s.suggestions.is_empty() {
                s.input_value = s.suggestions[s.active_sug].to_string();
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
            if !s.suggestions.is_empty() && s.input_value != s.suggestions[s.active_sug] {
                s.input_value = s.suggestions[s.active_sug].to_string();
                s.input_cursor_pos = s.input_value.len();
                s.update_suggestions();
            } else {
                let cmd = s.input_value.clone();
                s.show_input = false; s.input_value.clear(); s.input_cursor_pos = 0; s.suggestions.clear();
                s.execute_command(&cmd);
            }
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

pub fn handle_no_repo_key(code: KeyCode, s: &mut AppState) -> bool {
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

pub fn handle_repo_key(code: KeyCode, s: &mut AppState) -> bool {
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
            if s.focus_pane == "files" && !s.flat_entries.is_empty() {
                s.flat_idx = (s.flat_idx + s.flat_entries.len() - 1) % s.flat_entries.len();
                if let Some(entry) = s.flat_entries.get(s.flat_idx) {
                    let fi = entry.file_idx;
                    s.selected_file_idx = fi;
                }
                s.update_diff_content(); s.diff_scroll_offset = 0;
            } else if s.focus_pane == "commits" && !s.commits.is_empty() {
                s.selected_commit_idx = (s.selected_commit_idx + s.commits.len() - 1) % s.commits.len();
                s.commit_scroll_offset = 0;
                s.commit_detail_scroll = 0;
                s.update_diff_content(); s.diff_scroll_offset = 0;
            } else if s.focus_pane == "diff" && s.diff_scroll_offset > 0 {
                s.diff_scroll_offset -= 1;
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if s.focus_pane == "files" && !s.flat_entries.is_empty() {
                s.flat_idx = (s.flat_idx + 1) % s.flat_entries.len();
                if let Some(entry) = s.flat_entries.get(s.flat_idx) {
                    let fi = entry.file_idx;
                    s.selected_file_idx = fi;
                }
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
            if s.focus_pane == "files"
                && !s.flat_entries.is_empty()
                && let Some(entry) = s.flat_entries.get(s.flat_idx)
            {
                let fi = entry.file_idx;
                s.toggle_stage_file(fi);
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
            } else if s.focus_pane == "files"
                && !s.flat_entries.is_empty()
                && let Some(entry) = s.flat_entries.get(s.flat_idx)
            {
                let fi = entry.file_idx;
                s.toggle_stage_file(fi);
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
            // Copy current diff to clipboard (if available).
            // Phase 3.7: replaced `child.stdin.as_mut().unwrap()` with a
            // `match` so a closed pipe (e.g. pbcopy not installed) is
            // reported to the user instead of panicking.
            if s.active_diff.is_empty() {
                s.status_message = "Nothing to copy".to_string();
            } else {
                s.status_message = match copy_to_clipboard(&s.active_diff) {
                    Ok(()) => "Diff copied to clipboard!".to_string(),
                    Err(e) => format!("Copy failed: {e}"),
                };
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
            } else if s.diff_scroll_offset >= 5 { s.diff_scroll_offset -= 5; } else { s.diff_scroll_offset = 0; }
        }
        _ => {}
    }
    true
}

pub fn handle_confirm_push_key(code: KeyCode, s: &mut AppState) {
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

pub fn handle_confirm_pull_key(code: KeyCode, s: &mut AppState) {
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

/// Phase 3.4: keyboard handler for the new "are you sure?" modal that
/// guards `/remove-repo`. Same pattern as the push/pull handlers, but
/// executes the destructive action inline (there is no async command).
pub fn handle_confirm_remove_key(code: KeyCode, s: &mut AppState) {
    match code {
        KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter => {
            s.show_confirm_remove = false;
            match git::git_remove_repo() {
                Ok(()) => {
                    s.refresh_git_status();
                    s.status_message =
                        translate(&s.language, "status_remove_ok").into_owned();
                }
                Err(e) => {
                    s.status_message = format!("Error removing repo: {e}");
                }
            }
        }
        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
            s.show_confirm_remove = false;
            s.status_message = "Repository removal cancelled.".to_string();
        }
        _ => {}
    }
}

/// Keyboard handler for the update-available modal. Three choices:
/// - 1 / y / Enter → open the releases page in the browser
/// - 2 / n → dismiss (remind later)
/// - 3 / s / Esc → skip this version permanently
pub fn handle_update_modal_key(code: KeyCode, s: &mut AppState) {
    match code {
        KeyCode::Char('1') | KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter => {
            s.show_update_modal = false;
            // Open the GitHub releases page in the default browser.
            let url = "https://github.com/MarlonRX/git-hero/releases/latest";
            #[cfg(target_os = "macos")]
            let _ = std::process::Command::new("open").arg(url).spawn();
            #[cfg(target_os = "linux")]
            let _ = std::process::Command::new("xdg-open").arg(url).spawn();
            #[cfg(not(any(target_os = "macos", target_os = "linux")))]
            let _ = std::process::Command::new("xdg-open").arg(url).spawn();
            s.status_message = format!("Opening {} ...", url);
        }
        KeyCode::Char('2') | KeyCode::Char('n') | KeyCode::Char('N') => {
            s.show_update_modal = false;
            s.status_message = "Update dismissed. You can check later.".to_string();
        }
        KeyCode::Char('3') | KeyCode::Char('s') | KeyCode::Char('S') | KeyCode::Esc => {
            s.show_update_modal = false;
            // Save the skipped version so we don't prompt again.
            let mut cfg = load_config().unwrap_or(Config {
                language: s.language.clone(),
                nerd_font: s.nerd_font,
                theme: s.theme.name.to_string(),
                skipped_version: None,
            });
            cfg.skipped_version = Some(s.latest_version.clone());
            let _ = save_config(&cfg);
            s.status_message = format!(
                "Update v{} will not be shown again.",
                s.latest_version
            );
        }
        _ => {}
    }
}

pub fn handle_credentials_key(key: KeyEvent, s: &mut AppState) {
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

/// Build a wrapped view of the commit message for keyboard navigation.
/// Extracted so we don't duplicate the import of `commit_modal_editor_width`
/// inside the hot path.
fn build_commit_view(s: &AppState) -> Vec<(usize, usize, String, usize)> {
    crate::ui::modals::build_wrapped_view(&s.commit_message_lines, crate::ui::modals::commit_modal_editor_width())
}

pub fn handle_commit_modal_key(key: KeyEvent, s: &mut AppState) {
    let code = key.code;
    let mods = key.modifiers;
    
    // Shift+Enter: new line
    if code == KeyCode::Enter && mods.contains(KeyModifiers::SHIFT) {
        let current_line = &s.commit_message_lines[s.commit_cursor_row];
        let col = s.commit_cursor_col.min(current_line.len());
        let (before, after) = (current_line[..col].to_string(), current_line[col..].to_string());
        s.commit_message_lines[s.commit_cursor_row] = before;
        s.commit_message_lines.insert(s.commit_cursor_row + 1, after);
        s.commit_cursor_row += 1;
        s.commit_cursor_col = 0;
        return;
    }
    
    // Confirm commit: Enter only (avoid accidental commit when typing 'y')
    if code == KeyCode::Enter {
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
            s.status_message = translate(&s.language, "status_commit_success").into_owned();
            s.show_commit_modal = false;
        }
        return;
    }

    match code {
        KeyCode::Esc => {
            s.show_commit_modal = false;
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
        KeyCode::Home => {
            s.commit_cursor_col = 0;
        }
        KeyCode::End => {
            s.commit_cursor_col = s.commit_message_lines[s.commit_cursor_row].len();
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
        // Up/Down navigate between visual wrapped lines, not just
        // logical (Shift+Enter separated) lines. This is what users
        // expect from a text editor — when a long logical line wraps
        // to multiple visual rows, pressing ↑ moves up one visual row.
        KeyCode::Up => {
            let view = build_commit_view(s);
            if view.is_empty() {
                return;
            }
            let current_pos = crate::ui::modals::find_cursor_in_view(
                &view, s.commit_cursor_row, s.commit_cursor_col,
            );
            let (wrapped_row, wrapped_col) = current_pos.unwrap_or((0, 0));

            if wrapped_row > 0 {
                // Move up one visual line, keeping horizontal position.
                if let Some((row, col)) = crate::ui::modals::wrapped_to_logical(
                    &view, wrapped_row - 1, wrapped_col,
                ) {
                    s.commit_cursor_row = row;
                    s.commit_cursor_col = col;
                }
            } else if s.commit_cursor_row > 0 {
                // At the first visual line of a non-first logical line:
                // jump to the end of the previous logical line.
                s.commit_cursor_row -= 1;
                s.commit_cursor_col = s.commit_message_lines[s.commit_cursor_row].len();
            }
        }
        KeyCode::Down => {
            let view = build_commit_view(s);
            if view.is_empty() {
                return;
            }
            let current_pos = crate::ui::modals::find_cursor_in_view(
                &view, s.commit_cursor_row, s.commit_cursor_col,
            );
            let (wrapped_row, wrapped_col) = current_pos.unwrap_or((0, 0));

            if wrapped_row + 1 < view.len() {
                // Move down one visual line, keeping horizontal position.
                if let Some((row, col)) = crate::ui::modals::wrapped_to_logical(
                    &view, wrapped_row + 1, wrapped_col,
                ) {
                    s.commit_cursor_row = row;
                    s.commit_cursor_col = col;
                }
            } else if s.commit_cursor_row + 1 < s.commit_message_lines.len() {
                // At the last visual line, but there's a next logical line.
                s.commit_cursor_row += 1;
                let next_len = s.commit_message_lines[s.commit_cursor_row].len();
                if s.commit_cursor_col > next_len {
                    s.commit_cursor_col = next_len;
                }
            }
        }
        // PageUp / PageDown scroll the wrapped view without moving the
        // input cursor — useful when the message is long enough to need
        // scrolling past the cursor's own line.
        KeyCode::PageUp => {
            s.commit_modal_scroll = s.commit_modal_scroll.saturating_sub(5);
        }
        KeyCode::PageDown => {
            s.commit_modal_scroll = s.commit_modal_scroll.saturating_add(5);
        }
        KeyCode::Char(c) => {
            let current_line = &mut s.commit_message_lines[s.commit_cursor_row];
            current_line.insert(s.commit_cursor_col, c);
            s.commit_cursor_col += 1;
        }
        _ => {}
    }
}

/// Pipe `text` into the platform's clipboard helper (`pbcopy` on macOS,
/// `xclip` on Linux). Returns `Err` if the helper is missing, can't be
/// spawned, or the pipe is closed before the write completes.
///
/// Phase 3.7: this used to be inline with `child.stdin.as_mut().unwrap()`
/// inside an `and_then`, which panicked when the pipe was unexpectedly
/// closed (e.g. pbcopy not installed, broken xclip). Reporting the error is
/// strictly better than crashing the TUI.
fn copy_to_clipboard(text: &str) -> std::io::Result<()> {
    use std::io::Write;
    use std::process::{Command, Stdio};

    #[cfg(target_os = "macos")]
    let mut cmd = Command::new("pbcopy");
    #[cfg(target_os = "macos")]
    {
        cmd.stdin(Stdio::piped());
    }

    #[cfg(target_os = "linux")]
    let mut cmd = Command::new("xclip");
    #[cfg(target_os = "linux")]
    {
        cmd.arg("-selection").arg("clipboard").stdin(Stdio::piped());
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        let _ = text; // Silence unused warning.
        return Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "clipboard copy not supported on this platform",
        ));
    }

    let mut child = cmd.spawn()?;
    if let Some(stdin) = child.stdin.as_mut() {
        stdin.write_all(text.as_bytes())?;
    } else {
        return Err(std::io::Error::other(
            "clipboard helper closed its stdin before we could write",
        ));
    }
    let status = child.wait()?;
    if status.success() {
        Ok(())
    } else {
        Err(std::io::Error::other(format!(
            "clipboard helper exited with {status}"
        )))
    }
}
