use crate::git;
use crate::theme::get_themes;
use crate::ui::rendering::panels::{browser_scroll, sidebar_split};
use crate::ui::state::AppState;
use ratatui::layout::Rect;

pub fn mouse_setup(col: u16, row: u16, s: &mut AppState, area: Rect) {
    let mw = 60;
    let mh = 14;
    let mx = (area.width.saturating_sub(mw)) / 2;
    let my = (area.height.saturating_sub(mh)) / 2;
    if col >= mx + 2 && col < mx + mw - 2 && row >= my + 2 && row < my + mh - 2 {
        let idx = row.saturating_sub(my + 2) as usize;
        match s.setup_step {
            1 | 2 => {
                if idx < 2 {
                    s.setup_cursor = idx;
                }
            }
            3 => {
                let themes = get_themes();
                let start = s.setup_cursor.saturating_sub(3);
                let clicked = start + idx;
                if clicked < themes.len() {
                    s.setup_cursor = clicked;
                }
            }
            _ => {}
        }
    }
}

pub fn mouse_theme(col: u16, row: u16, s: &mut AppState, area: Rect) {
    let themes = get_themes();
    let mw = 50;
    let mh = (themes.len() as u16 + 6).min(area.height - 2);
    let mx = (area.width.saturating_sub(mw)) / 2;
    let my = (area.height.saturating_sub(mh)) / 2;
    if col >= mx + 2 && col < mx + mw - 2 && row >= my + 2 && row < my + mh - 3 {
        let start = s.theme_cursor.saturating_sub(4);
        let idx = start + row.saturating_sub(my + 2) as usize;
        if idx < themes.len() {
            s.theme_cursor = idx;
            s.theme = themes[idx];
            s.update_diff_content();
        }
    }
}

pub fn mouse_init_wizard(_col: u16, row: u16, s: &mut AppState, inner: Rect) {
    let py = inner.y + 1;
    let clicked = row.saturating_sub(py + 2);
    match s.init_wizard_step {
        1 => {
            if (3..=5).contains(&clicked) {
                let idx = clicked as usize - 3;
                s.init_cursor = idx;
                if idx == 0 {
                    s.init_branch_name = "main".into();
                    s.init_wizard_step = 2;
                    s.input_value.clear();
                } else if idx == 1 {
                    s.init_branch_name = "master".into();
                    s.init_wizard_step = 2;
                    s.input_value.clear();
                }
            }
        }
        3 if clicked == 7 => {
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
        _ => {}
    }
}

pub fn mouse_no_repo(_col: u16, row: u16, s: &mut AppState, inner: Rect) {
    let py = inner.y + 1;
    let cy = py + 3;
    if row == cy + 7 {
        s.init_cursor = 0;
        s.init_wizard_active = true;
        s.init_wizard_step = 1;
        s.init_branch_name = "main".into();
        s.init_remote_url.clear();
        s.status_message = "Select main branch name.".to_string();
    } else if row == cy + 9 {
        s.init_cursor = 1;
        s.show_input = true;
        s.input_value = "/cd ".into();
        s.input_cursor_pos = 4;
        s.update_suggestions();
    }
}

/// Dashboard clicks. `body` MUST be the same rect `draw_ui` hands to
/// `draw_dashboard` (i.e. `body_area(inner)`); all geometry comes from the
/// same helpers the renderer uses (`sidebar_split`, `browser_scroll`), so
/// click rows always line up with painted rows.
pub fn mouse_dashboard(col: u16, row: u16, s: &mut AppState, body: Rect) {
    if body.width < 50 {
        return; // compact layout has no interactive rows
    }
    let (files_col, _browser) = sidebar_split(body, s.show_repo_overview, s.repo_view.len());
    let split_x = body.x + files_col.width;

    if col < split_x {
        // FILES panel (browser clicks are handled before we get here).
        let list_top = files_col.y + 1; // first row inside the border
        let rows_h = files_col.height.saturating_sub(2) as usize;
        let has_indicator = s
            .files
            .iter()
            .any(|f| matches!(f.status.as_str(), "A" | "M" | "MM" | "D" | "??"));
        if !s.flat_entries.is_empty() && row >= list_top {
            let total = s.flat_entries.len() + usize::from(has_indicator);
            let sel_row = s.flat_idx + usize::from(has_indicator);
            let start = browser_scroll(total, rows_h, sel_row);
            let clicked = (row - list_top) as usize;
            if clicked < total {
                let mut global = start + clicked;
                if has_indicator {
                    if global == 0 {
                        return; // clicked the indicator line
                    }
                    global -= 1;
                }
                if let Some(entry) = s.flat_entries.get(global) {
                    let fi = entry.file_idx;
                    s.focus_pane = "files".into();
                    s.flat_idx = global.min(s.flat_entries.len() - 1);
                    s.diff_scroll_offset = 0;
                    s.selected_file_idx = fi;
                    if col > files_col.x && col <= files_col.x + 6 {
                        // The checkbox column toggles stage.
                        s.toggle_stage_file(fi);
                    } else {
                        s.update_diff_content();
                    }
                }
                return;
            }
        }
        s.focus_pane = "files".into();
        return;
    }

    // Right side: DIFF over COMMITS, matching draw_dashboard's split
    // (50/50 when commits are focused, 70/30 otherwise).
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
        s.focus_pane = "diff".into();
    } else {
        let list_top = commits_top + 1; // inside the border row
        let rows_h = right.height.saturating_sub(diff_h + 2) as usize;
        let clicked = row.saturating_sub(list_top) as usize;
        if !s.show_commit_detail {
            let start = browser_scroll(s.commits.len(), rows_h, s.selected_commit_idx);
            let idx = start + clicked;
            if idx < s.commits.len() {
                s.focus_pane = "commits".into();
                s.selected_commit_idx = idx;
                s.diff_scroll_offset = 0;
                s.update_diff_content();
                return;
            }
        }
        s.focus_pane = "commits".into();
    }
}
