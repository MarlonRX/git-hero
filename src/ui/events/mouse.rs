use ratatui::layout::Rect;

use crate::git;
use crate::theme::get_themes;
use crate::ui::state::AppState;

pub fn mouse_setup(col: u16, row: u16, s: &mut AppState, area: Rect) {
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

pub fn mouse_theme(col: u16, row: u16, s: &mut AppState, area: Rect) {
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

pub fn mouse_init_wizard(_col: u16, row: u16, s: &mut AppState, inner: Rect) {
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

pub fn mouse_no_repo(_col: u16, row: u16, s: &mut AppState, inner: Rect) {
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

pub fn mouse_dashboard(col: u16, row: u16, s: &mut AppState, inner: Rect) {
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
            // SHORTCUTS panel → focus diff
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
