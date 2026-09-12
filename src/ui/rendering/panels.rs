use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{BorderType, Clear, List, ListItem, Paragraph},
};

use crate::i18n::{translate, trf};
use crate::ui::state::repos::RepoEntry;
use crate::ui::state::{AppState, GitCommit};

use super::components::{draw_continuous_border, draw_solid_border, short_path, soften};

pub fn draw_no_repo_panel(f: &mut Frame, s: &mut AppState, body: Rect) {
    f.render_widget(
        Paragraph::new("").style(Style::default().bg(s.theme.background)),
        body,
    );

    let pa = Rect {
        x: body.x + 1,
        y: body.y + 1,
        width: body.width.saturating_sub(2),
        height: body.height.saturating_sub(2),
    };

    f.render_widget(Clear, pa);
    draw_continuous_border(
        f,
        pa,
        " ⚠ No Git Repository Detected ",
        Style::default()
            .fg(s.theme.warning)
            .add_modifier(Modifier::BOLD),
        s.theme.primary,
        s.theme.background,
        BorderType::Rounded,
    );

    let inner_pa = Rect {
        x: pa.x + 1,
        y: pa.y + 1,
        width: pa.width.saturating_sub(2),
        height: pa.height.saturating_sub(2),
    };
    f.render_widget(
        Paragraph::new("").style(Style::default().bg(s.theme.background)),
        inner_pa,
    );

    let cy = pa.y + 3;

    f.render_widget(
        Paragraph::new("This directory is not inside a Git repository.")
            .alignment(Alignment::Center)
            .style(
                Style::default()
                    .fg(s.theme.warning)
                    .bg(s.theme.background)
                    .add_modifier(Modifier::BOLD),
            ),
        Rect {
            x: pa.x + 2,
            y: cy,
            width: pa.width - 4,
            height: 1,
        },
    );

    f.render_widget(
        Paragraph::new(format!(" \u{1F4C1} Current path:  {}", short_path(&s.cwd)))
            .alignment(Alignment::Center)
            .style(
                Style::default()
                    .fg(s.theme.foreground)
                    .bg(s.theme.background),
            ),
        Rect {
            x: pa.x + 2,
            y: cy + 2,
            width: pa.width - 4,
            height: 1,
        },
    );

    f.render_widget(
        Paragraph::new("\u{2501} Options \u{2501}")
            .alignment(Alignment::Center)
            .style(
                Style::default()
                    .fg(s.theme.accent)
                    .bg(s.theme.background)
                    .add_modifier(Modifier::BOLD),
            ),
        Rect {
            x: pa.x + 2,
            y: cy + 5,
            width: pa.width - 4,
            height: 1,
        },
    );

    let opt1 = "\u{2776} Initialize Git repository here";
    let opt2 = "\u{2777} Change Directory (/cd <path>)";

    let o1s = if s.init_cursor == 0 {
        Style::default()
            .bg(s.theme.primary)
            .fg(s.theme.background)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(s.theme.foreground).bg(s.theme.surface)
    };
    let o2s = if s.init_cursor == 1 {
        Style::default()
            .bg(s.theme.primary)
            .fg(s.theme.background)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(s.theme.foreground).bg(s.theme.surface)
    };

    f.render_widget(
        Paragraph::new(format!("  {}  ", opt1))
            .alignment(Alignment::Center)
            .style(o1s),
        Rect {
            x: pa.x + 4,
            y: cy + 7,
            width: pa.width - 8,
            height: 1,
        },
    );
    f.render_widget(
        Paragraph::new(format!("  {}  ", opt2))
            .alignment(Alignment::Center)
            .style(o2s),
        Rect {
            x: pa.x + 4,
            y: cy + 9,
            width: pa.width - 8,
            height: 1,
        },
    );

    let options_hint = if s.show_repo_overview {
        translate(&s.language, "no_repo_browser_hint").into_owned()
    } else {
        "Arrow keys / Enter / Click to select".to_string()
    };
    f.render_widget(
        Paragraph::new(options_hint)
            .alignment(Alignment::Center)
            .style(Style::default().fg(s.theme.dimmed).bg(s.theme.background)),
        Rect {
            x: pa.x + 2,
            y: cy + 12,
            width: pa.width - 4,
            height: 1,
        },
    );
}

pub fn draw_init_wizard(f: &mut Frame, s: &mut AppState, body: Rect) {
    f.render_widget(
        Paragraph::new("").style(Style::default().bg(s.theme.background)),
        body,
    );

    let pa = Rect {
        x: body.x + 1,
        y: body.y + 1,
        width: body.width.saturating_sub(2),
        height: body.height.saturating_sub(2),
    };

    let (title, lines) = wizard_content(s);

    draw_continuous_border(
        f,
        pa,
        &title,
        Style::default()
            .fg(s.theme.accent)
            .add_modifier(Modifier::BOLD),
        s.theme.primary,
        s.theme.background,
        BorderType::Rounded,
    );

    let inner = Rect {
        x: pa.x + 1,
        y: pa.y + 1,
        width: pa.width.saturating_sub(2),
        height: pa.height.saturating_sub(2),
    };
    f.render_widget(
        Paragraph::new("").style(Style::default().bg(s.theme.background)),
        inner,
    );

    let cy = pa.y + 2;
    f.render_widget(
        Paragraph::new(lines.join("\n")).style(
            Style::default()
                .fg(s.theme.foreground)
                .bg(s.theme.background),
        ),
        Rect {
            x: pa.x + 2,
            y: cy,
            width: pa.width - 4,
            height: pa.height - 4,
        },
    );
}

fn wizard_content(s: &AppState) -> (String, Vec<String>) {
    let mut title = " Git Init - Step 1/3 ".to_string();
    let mut lines = Vec::new();

    match s.init_wizard_step {
        1 => {
            lines.push("Choose default branch name:".to_string());
            lines.push(String::new());
            for (i, o) in ["main", "master", "Custom (type name below)"]
                .iter()
                .enumerate()
            {
                if i == s.init_cursor {
                    lines.push(format!("   \u{25B6} [{}] {}", i + 1, o));
                } else {
                    lines.push(format!("     [{}] {}", i + 1, o));
                }
            }
            if s.init_cursor == 2 {
                lines.push(String::new());
                lines.push(format!("   Branch name: {}", s.input_value));
            }
        }
        2 => {
            title = " Git Init - Step 2/3 ".to_string();
            lines.push("Enter remote repository URL (optional):".to_string());
            lines.push(String::new());
            lines.push(format!("   URL: {}", s.input_value));
            lines.push(String::new());
            lines.push("Press Enter to continue or leave empty to skip.".to_string());
        }
        3 => {
            title = " Git Init - Step 3/3 ".to_string();
            let remote = if s.init_remote_url.is_empty() {
                "None"
            } else {
                &s.init_remote_url
            };
            lines.push("Review initialization details:".to_string());
            lines.push(String::new());
            lines.push(format!("   \u{1F4C1} Path:   {}", short_path(&s.cwd)));
            lines.push(format!("   \u{2B50} Branch: {}", s.init_branch_name));
            lines.push(format!("   \u{1F310} Remote: {}", remote));
            lines.push(String::new());
            if s.init_cursor == 0 {
                lines.push("   \u{25B6} [1] Initialize Repository".to_string());
                lines.push("     [2] Cancel".to_string());
            } else {
                lines.push("     [1] Initialize Repository".to_string());
                lines.push("   \u{25B6} [2] Cancel".to_string());
            }
        }
        _ => {}
    }
    (title, lines)
}

pub fn draw_dashboard(f: &mut Frame, s: &mut AppState, body: Rect) {
    f.render_widget(
        Paragraph::new("").style(Style::default().bg(s.theme.background)),
        body,
    );

    if body.width < 50 {
        draw_compact(f, s, body, 0);
        return;
    }

    let content = body;

    // Sidebar: FILES on top; when the repo browser is open it takes the
    // bottom sector of the same column (where the shortcuts block used to
    // live) and the column widens — the dropdown is the focus then.
    let (files_area, browser_area) =
        sidebar_split(content, s.show_repo_overview, s.repo_view.len());
    let sidebar_w = files_area.width;
    let main = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(sidebar_w), Constraint::Min(20)])
        .split(content);

    let sidebar = main[0];
    let right = main[1];
    let files_area = Rect {
        x: sidebar.x,
        y: sidebar.y,
        width: sidebar.width,
        height: files_area.height,
    };
    let files_title = format!(" FILES ({}) ", s.files.len());
    let border_color = if s.focus_pane == "files" {
        s.theme.primary
    } else {
        s.theme.border
    };
    let title_style = if s.focus_pane == "files" {
        Style::default()
            .fg(s.theme.primary)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
            .fg(s.theme.foreground)
            .add_modifier(Modifier::BOLD)
    };
    draw_continuous_border(
        f,
        files_area,
        &files_title,
        title_style,
        border_color,
        s.theme.background,
        BorderType::Plain,
    );
    let files_inner = Rect {
        x: files_area.x + 1,
        y: files_area.y + 1,
        width: files_area.width.saturating_sub(2),
        height: files_area.height.saturating_sub(2),
    };

    if s.files.is_empty() {
        let clean = "\u{2713} Working directory clean";
        f.render_widget(
            Paragraph::new(clean).style(
                Style::default()
                    .fg(s.theme.success)
                    .bg(s.theme.background)
                    .add_modifier(Modifier::BOLD),
            ),
            files_inner,
        );
    } else {
        let mut added_count = 0;
        let mut modified_count = 0;
        let mut deleted_count = 0;
        let mut untracked_count = 0;

        for file in &s.files {
            match file.status.as_str() {
                "A" => added_count += 1,
                "M" | "MM" => modified_count += 1,
                "D" => deleted_count += 1,
                "??" => untracked_count += 1,
                _ => {}
            }
        }

        let mut indicators = Vec::new();
        if added_count > 0 {
            indicators.push(format!("{}{}", s.get_icon_str("add"), added_count));
        }
        if modified_count > 0 {
            indicators.push(format!("{}{}", s.get_icon_str("mod"), modified_count));
        }
        if deleted_count > 0 {
            indicators.push(format!("{}{}", s.get_icon_str("del"), deleted_count));
        }
        if untracked_count > 0 {
            indicators.push(format!(
                "{}{}",
                s.get_icon_str("untracked"),
                untracked_count
            ));
        }

        let indicator_text = if indicators.is_empty() {
            String::new()
        } else {
            format!(" [{}] ", indicators.join(" "))
        };

        let items: Vec<ListItem> = s
            .flat_entries
            .iter()
            .enumerate()
            .map(|(i, entry)| {
                let pre = if i == s.flat_idx && s.focus_pane == "files" {
                    "\u{25B6} "
                } else {
                    "  "
                };
                let fi = entry.file_idx;

                let f = &s.files[fi];
                let fg = if f.staged {
                    s.theme.success
                } else if f.status == "??" {
                    s.theme.dimmed
                } else {
                    s.theme.warning
                };
                let cb = if f.staged { "[\u{2713}]" } else { "[ ]" };

                let (icon, icon_color) = match f.status.as_str() {
                    "A" => (s.get_icon_str("add"), s.theme.success),
                    "D" => (s.get_icon_str("del"), s.theme.warning),
                    "??" => (s.get_icon_str("untracked"), s.theme.dimmed),
                    "M" | "MM" => (s.get_icon_str("mod"), s.theme.accent),
                    _ => (s.get_icon_str("mod"), fg),
                };

                let style = if i == s.flat_idx && s.focus_pane == "files" {
                    Style::default()
                        .bg(s.theme.highlight)
                        .fg(s.theme.on_highlight)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(fg).bg(s.theme.background)
                };

                let line = Line::from(vec![
                    Span::raw(pre),
                    Span::styled(
                        cb,
                        Style::default()
                            .fg(if f.staged {
                                s.theme.success
                            } else {
                                s.theme.dimmed
                            })
                            .bg(s.theme.background),
                    ),
                    Span::raw(" "),
                    Span::styled(icon, Style::default().fg(icon_color).bg(s.theme.background)),
                    Span::raw(" "),
                    Span::styled(&f.path, style),
                ]);

                ListItem::new(line)
            })
            .collect();

        let all_items = if !indicator_text.is_empty() {
            let mut all = vec![ListItem::new(Line::from(Span::styled(
                indicator_text.trim(),
                Style::default()
                    .fg(s.theme.dimmed)
                    .bg(s.theme.background)
                    .add_modifier(Modifier::BOLD),
            )))];
            all.extend(items);
            all
        } else {
            items
        };

        f.render_widget(
            List::new(all_items).style(Style::default().bg(s.theme.background)),
            files_inner,
        );
    }

    if let Some(browser) = browser_area {
        draw_repo_browser(f, browser, s);
    }

    // ── Right panel: DIFF + COMMITS ────────────────────────────────
    let right_constraints: Vec<Constraint> = if s.focus_pane == "commits" {
        vec![Constraint::Percentage(50), Constraint::Percentage(50)]
    } else {
        vec![Constraint::Percentage(70), Constraint::Percentage(30)]
    };
    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(right_constraints)
        .split(right);

    let diff_area = right_chunks[0];
    let mut diff_title = " DIFF ".to_string();
    if s.focus_pane == "commits" && !s.commits.is_empty() && s.selected_commit_idx < s.commits.len()
    {
        diff_title = " FILES CHANGED ".to_string();
    } else if !s.files.is_empty() && s.selected_file_idx < s.files.len() {
        let file_label = &s.files[s.selected_file_idx].path;
        diff_title = format!(
            " {} DIFF: {} {} ",
            if s.focus_pane == "diff" {
                "\u{25BC}"
            } else {
                ""
            },
            file_label,
            if s.focus_pane == "diff" {
                "\u{25BC}"
            } else {
                ""
            },
        );
    }
    let diff_title_style = if s.focus_pane == "diff" {
        Style::default()
            .fg(s.theme.primary)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
            .fg(s.theme.accent)
            .add_modifier(Modifier::BOLD)
    };
    let border_color = if s.focus_pane == "diff" {
        s.theme.primary
    } else {
        s.theme.border
    };
    draw_continuous_border(
        f,
        diff_area,
        &diff_title,
        diff_title_style,
        border_color,
        s.theme.background,
        BorderType::Plain,
    );
    let diff_inner = Rect {
        x: diff_area.x + 1,
        y: diff_area.y + 1,
        width: diff_area.width.saturating_sub(2),
        height: diff_area.height.saturating_sub(2),
    };

    let dlines = s.get_cached_diff_lines(diff_inner.width);
    let visible: Vec<Line> = dlines
        .iter()
        .skip(s.diff_scroll_offset)
        .take(diff_inner.height as usize)
        .cloned()
        .collect();
    f.render_widget(
        Paragraph::new(visible).style(Style::default().bg(s.theme.background)),
        diff_inner,
    );

    let commit_area = right_chunks[1];
    let commits_title = if s.show_commit_detail {
        " COMMIT DETAILS (Enter to close) ".to_string()
    } else {
        " COMMITS (Enter for details) ".to_string()
    };
    let border_color = if s.focus_pane == "commits" {
        s.theme.primary
    } else {
        s.theme.border
    };
    let title_style = if s.focus_pane == "commits" {
        Style::default()
            .fg(s.theme.primary)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
            .fg(s.theme.foreground)
            .add_modifier(Modifier::BOLD)
    };
    draw_continuous_border(
        f,
        commit_area,
        &commits_title,
        title_style,
        border_color,
        s.theme.background,
        BorderType::Plain,
    );
    let commits_inner = Rect {
        x: commit_area.x + 1,
        y: commit_area.y + 1,
        width: commit_area.width.saturating_sub(2),
        height: commit_area.height.saturating_sub(2),
    };

    if s.commits.is_empty() {
        f.render_widget(
            Paragraph::new("No commits found.")
                .style(Style::default().fg(s.theme.dimmed).bg(s.theme.background)),
            commits_inner,
        );
    } else if s.show_commit_detail && !s.commit_detail_diff.is_empty() {
        let commit_theme = s.theme;
        let c_warn = soften(commit_theme.warning, commit_theme.background, 0.25);
        let c_succ = soften(commit_theme.success, commit_theme.background, 0.25);
        let detail_lines: Vec<Line> = s
            .commit_detail_diff
            .split('\n')
            .skip(s.commit_detail_scroll)
            .take(commits_inner.height as usize)
            .map(|line| {
                if line.starts_with("commit ") {
                    Line::from(Span::styled(
                        line.to_string(),
                        Style::default()
                            .fg(commit_theme.accent)
                            .bg(commit_theme.background)
                            .add_modifier(Modifier::BOLD),
                    ))
                } else if line.starts_with("@@") {
                    Line::from(Span::styled(
                        line.to_string(),
                        Style::default()
                            .fg(commit_theme.primary)
                            .bg(commit_theme.surface)
                            .add_modifier(Modifier::BOLD),
                    ))
                } else if line.starts_with('+') && !line.starts_with("+++") {
                    let rest = if line.len() > 1 {
                        line[1..].to_string()
                    } else {
                        String::new()
                    };
                    Line::from(vec![
                        Span::styled(
                            "+".to_string(),
                            Style::default()
                                .fg(commit_theme.foreground)
                                .bg(c_succ)
                                .add_modifier(Modifier::BOLD),
                        ),
                        Span::styled(
                            rest,
                            Style::default().fg(commit_theme.foreground).bg(c_succ),
                        ),
                    ])
                } else if line.starts_with('-') && !line.starts_with("---") {
                    let rest = if line.len() > 1 {
                        line[1..].to_string()
                    } else {
                        String::new()
                    };
                    Line::from(vec![
                        Span::styled(
                            "-".to_string(),
                            Style::default()
                                .fg(commit_theme.foreground)
                                .bg(c_warn)
                                .add_modifier(Modifier::BOLD),
                        ),
                        Span::styled(
                            rest,
                            Style::default().fg(commit_theme.foreground).bg(c_warn),
                        ),
                    ])
                } else if line.starts_with("Author:")
                    || line.starts_with("Date:")
                    || line.starts_with("---")
                    || line.starts_with("+++")
                {
                    Line::from(Span::styled(
                        line.to_string(),
                        Style::default()
                            .fg(commit_theme.dimmed)
                            .bg(commit_theme.background),
                    ))
                } else if line.trim().is_empty() {
                    Line::from(Span::raw(""))
                } else {
                    Line::from(Span::styled(
                        line.to_string(),
                        Style::default()
                            .fg(commit_theme.foreground)
                            .bg(commit_theme.background),
                    ))
                }
            })
            .collect();
        f.render_widget(
            Paragraph::new(detail_lines).style(Style::default().bg(s.theme.background)),
            commits_inner,
        );
    } else {
        // Phase 4.11: O(n) slice + O(1) per-item index lookup. The previous
        // version did `s.commits.iter().position(|c| c.hash == c.hash)` per
        // visible commit, which is O(n²) for repos with many commits.
        let start = s.commit_scroll_offset.min(s.commits.len());
        let end = (start + commits_inner.height as usize).min(s.commits.len());
        let visible: &[GitCommit] = &s.commits[start..end];

        let items: Vec<ListItem> = visible
            .iter()
            .enumerate()
            .map(|(i, c)| {
                let actual_idx = start + i;
                let pre = if actual_idx == s.selected_commit_idx && s.focus_pane == "commits" {
                    "\u{25B6} "
                } else {
                    "  "
                };
                let subj = truncate_subject(&c.subject, right.width.saturating_sub(36) as usize);

                let push_icon = if c.pushed { "\u{2713}" } else { "\u{21C8}" };
                let push_style = if c.pushed {
                    Style::default().fg(s.theme.success)
                } else {
                    Style::default()
                        .fg(s.theme.warning)
                        .add_modifier(Modifier::BOLD)
                };

                let item_s = if actual_idx == s.selected_commit_idx && s.focus_pane == "commits" {
                    Style::default()
                        .bg(s.theme.highlight)
                        .fg(s.theme.on_highlight)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().bg(s.theme.background)
                };

                let line = if actual_idx == s.selected_commit_idx && s.focus_pane == "commits" {
                    Line::from(vec![
                        Span::raw(format!("{}{}", pre, c.hash)),
                        Span::raw(format!("  ({})", c.date)),
                        Span::raw(format!("  {}", subj)),
                        Span::raw(format!("  {}", push_icon)),
                    ])
                } else {
                    Line::from(vec![
                        Span::raw(pre),
                        Span::styled(&c.hash, Style::default().fg(s.theme.accent)),
                        Span::styled(
                            format!("  ({})", c.date),
                            Style::default().fg(s.theme.dimmed),
                        ),
                        Span::raw(format!("  {}", subj)),
                        Span::styled(format!("  {}", push_icon), push_style),
                    ])
                };
                ListItem::new(line).style(item_s)
            })
            .collect();
        f.render_widget(
            List::new(items).style(Style::default().bg(s.theme.background)),
            commits_inner,
        );
    }
}

/// Sidebar geometry: `(files_area, browser_area)`. Kept as a pure helper so
/// the mouse handlers hit-test exactly what `draw_dashboard` paints.
/// With the browser closed nothing changes; with it open the column widens
/// (clamped) and the bottom sector becomes the repo dropdown.
pub fn sidebar_split(body: Rect, browser_open: bool, view_rows: usize) -> (Rect, Option<Rect>) {
    if !browser_open {
        let w = (body.width / 4).max(20).min(body.width);
        let files = Rect {
            x: body.x,
            y: body.y,
            width: w,
            height: body.height,
        };
        return (files, None);
    }
    let browser = browser_area(body, view_rows, false);
    let files = Rect {
        x: body.x,
        y: body.y,
        width: browser.width,
        height: body.height - browser.height,
    };
    (files, Some(browser))
}

/// Geometry of the repo browser dropdown. `full_height` is used when there
/// is no repository to show above it (the no-repo screen splits the body
/// browser-left / panel-right instead of stacking them).
pub fn browser_area(body: Rect, view_rows: usize, full_height: bool) -> Rect {
    let w = (body.width / 2).clamp(38, 58).min(body.width);
    if full_height {
        return Rect {
            x: body.x,
            y: body.y,
            width: w,
            height: body.height,
        };
    }
    // chrome: 2 border rows + summary + filter + hint = 5, +1 breathing
    let want = (view_rows as u16).clamp(3, 14) + 6;
    let bh = want
        .max(9)
        .min(body.height.saturating_sub(6).max(9))
        .min(body.height);
    Rect {
        x: body.x,
        y: body.y + body.height - bh,
        width: w,
        height: bh,
    }
}

/// Visible row window inside a browser rect: `(rows_top, rows_h)` in screen
/// coordinates. Single source of truth for drawing and click hit-testing.
pub fn browser_rows_rect(area: Rect) -> (u16, usize) {
    // inner.y = area.y + 1; row 0 = summary, row 1 = filter, rows start +2
    (area.y + 3, area.height.saturating_sub(5) as usize)
}

/// First visible row index for the browser's scroll window.
pub fn browser_scroll(view_len: usize, rows_h: usize, cursor: usize) -> usize {
    if rows_h == 0 || view_len <= rows_h {
        return 0;
    }
    cursor.saturating_sub(rows_h / 2).min(view_len - rows_h)
}

/// The repo browser dropdown: bordered panel with a dirty-summary line, a
/// type-to-filter line, matching rows (Enter/click opens) and a keybinding
/// hint. All data is pre-computed by `state::repos::scan_repos` — drawing
/// performs zero git calls.
pub fn draw_repo_browser(f: &mut Frame, area: Rect, s: &AppState) {
    draw_continuous_border(
        f,
        area,
        &translate(&s.language, "repos_browser_title"),
        Style::default()
            .fg(s.theme.primary)
            .add_modifier(Modifier::BOLD),
        s.theme.primary,
        s.theme.background,
        BorderType::Plain,
    );
    let inner = Rect {
        x: area.x + 1,
        y: area.y + 1,
        width: area.width.saturating_sub(2),
        height: area.height.saturating_sub(2),
    };
    if inner.width < 6 || inner.height < 4 {
        return;
    }

    let w = inner.width as usize;
    let mut lines: Vec<Line> = Vec::with_capacity(inner.height as usize);

    // Line 1: "X of Y repos have uncommitted changes".
    lines.push(Line::from(vec![Span::styled(
        trf(
            &s.language,
            "repo_overview_summary",
            &[&s.repo_dirty_count.to_string(), &s.repos.len().to_string()],
        ),
        Style::default()
            .fg(if s.repo_dirty_count > 0 {
                s.theme.warning
            } else {
                s.theme.success
            })
            .bg(s.theme.background)
            .add_modifier(Modifier::BOLD),
    )]));

    // Line 2: the live filter.
    lines.push(Line::from(vec![
        Span::styled(
            format!("\u{2315} {}", s.repo_filter),
            Style::default()
                .fg(s.theme.accent)
                .bg(s.theme.background)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            "\u{2588}",
            Style::default().fg(s.theme.accent).bg(s.theme.background),
        ),
        Span::styled(
            format!(" {}/{}", s.repo_view.len(), s.repos.len()),
            Style::default().fg(s.theme.dimmed).bg(s.theme.background),
        ),
    ]));

    // Rows (scroll window around the cursor). Geometry shared with the
    // mouse hit-testing via `browser_rows_rect`.
    let (_rows_top, rows_h) = browser_rows_rect(area);
    if s.repo_view.is_empty() {
        let msg = if s.repos.is_empty() {
            translate(&s.language, "repo_overview_empty").into_owned()
        } else {
            translate(&s.language, "repos_no_match").into_owned()
        };
        lines.push(Line::from(Span::styled(
            msg,
            Style::default().fg(s.theme.dimmed).bg(s.theme.background),
        )));
    } else {
        let start = browser_scroll(s.repo_view.len(), rows_h, s.repo_cursor);
        for (pos, &idx) in s.repo_view.iter().enumerate().skip(start).take(rows_h) {
            if lines.len() >= (inner.height as usize).saturating_sub(1) {
                break;
            }
            lines.push(repo_browser_row(s, pos, &s.repos[idx], w));
        }
    }

    // Hint pinned to the last inner row.
    while lines.len() < (inner.height as usize).saturating_sub(1) {
        lines.push(Line::from(""));
    }
    lines.push(Line::from(Span::styled(
        translate(&s.language, "repo_overview_help").into_owned(),
        Style::default().fg(s.theme.dimmed).bg(s.theme.background),
    )));

    f.render_widget(
        Paragraph::new(lines).style(Style::default().bg(s.theme.background)),
        inner,
    );
}

/// One browser row. Columns degrade on narrow sidebars: relative path
/// (+ branch/sync when there's room), dirty chip, activity age.
fn repo_browser_row(s: &AppState, view_pos: usize, e: &RepoEntry, width: usize) -> Line<'static> {
    let selected = view_pos == s.repo_cursor;
    let here = is_within(&e.path, &s.cwd);
    let base = if selected {
        Style::default()
            .bg(s.theme.highlight)
            .fg(s.theme.on_highlight)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
            .bg(s.theme.background)
            .fg(s.theme.foreground)
    };
    let pre = if selected { "\u{25B6} " } else { "  " };
    let wide = width >= 40;
    let (chip, chip_style) = if selected {
        (format!("{:<5}", format!("{} \u{2731}", e.dirty)), base)
    } else if e.dirty > 0 {
        (
            format!("{:<5}", format!("{} \u{2731}", e.dirty)),
            base.fg(s.theme.warning).add_modifier(Modifier::BOLD),
        )
    } else {
        ("  \u{2713}  ".to_string(), base.fg(s.theme.success))
    };
    let name_w = if wide { 18 } else { 24 };

    // Name chip: bold label on a surface-colored background — the folder /
    // repo name is the primary thing a row communicates. The repository
    // the user is currently inside is additionally tagged `● here`.
    let name_chip = Style::default()
        .fg(if selected {
            s.theme.on_highlight
        } else if here {
            s.theme.success
        } else {
            s.theme.foreground
        })
        .bg(if selected {
            s.theme.highlight
        } else {
            s.theme.surface
        })
        .add_modifier(Modifier::BOLD);

    let mut spans = vec![
        Span::styled(pre, base),
        Span::styled(clip(&e.rel, name_w), name_chip),
    ];
    if here {
        let label = format!(" \u{25CF}{}", translate(&s.language, "repos_here"));
        spans.push(Span::styled(
            label,
            base.fg(s.theme.success).add_modifier(Modifier::BOLD),
        ));
    }
    if wide {
        let sync = format!("\u{2191}{}\u{2193}{}", e.ahead, e.behind);
        let sync_style = if selected {
            base
        } else if e.behind > 0 {
            base.fg(s.theme.warning)
        } else if e.ahead > 0 {
            base.fg(s.theme.success)
        } else {
            base.fg(s.theme.dimmed)
        };
        spans.push(Span::styled(
            format!(" {}", clip(&e.branch, 10)),
            base.fg(s.theme.primary),
        ));
        spans.push(Span::styled(format!(" {sync:<5}"), sync_style));
    } else if !here {
        spans.push(Span::raw("  "));
    }
    if wide || !here {
        spans.push(Span::styled(chip, chip_style));
        spans.push(Span::styled(
            e.age.clone(),
            base.fg(s.theme.dimmed).add_modifier(Modifier::BOLD),
        ));
    }
    Line::from(spans)
}

/// True when `dir` is `repo` itself or lives inside it. Compares raw bytes
/// (case-insensitive for Windows paths) and requires a separator boundary,
/// so `/a/bc` never counts as inside `/a/b` and a multi-byte char can never
/// be split.
pub fn is_within(repo: &str, dir: &str) -> bool {
    let r = repo.trim_end_matches(['/', '\\']);
    let (rb, db) = (r.as_bytes(), dir.as_bytes());
    if db.len() < rb.len() || !db[..rb.len()].eq_ignore_ascii_case(rb) {
        return false;
    }
    match db.get(rb.len()) {
        None => true,
        Some(&c) => c == b'/' || c == b'\\',
    }
}

/// Left-align and clip to `max` chars (char-boundary safe, `…` marker).
fn clip(text: &str, max: usize) -> String {
    if text.chars().count() <= max {
        format!("{text:<max$}")
    } else {
        let mut out: String = text.chars().take(max.saturating_sub(1)).collect();
        out.push('\u{2026}');
        out
    }
}

/// Truncate `subject` to at most `max` characters, replacing the tail with
/// an ellipsis when shortened. Character-boundary safe: the previous
/// implementation byte-sliced (`&subj[..n]`), which panicked the whole TUI
/// on any commit subject containing multi-byte UTF-8 (accents, emoji,
/// CJK) — a routine occurrence for a repo manager.
fn truncate_subject(subject: &str, max: usize) -> String {
    const MIN_BUDGET: usize = 5;
    let budget = max.max(MIN_BUDGET);
    if subject.chars().count() <= budget {
        return subject.to_string();
    }
    let mut out: String = subject.chars().take(budget - 3).collect();
    out.push_str("...");
    out
}

fn draw_compact(f: &mut Frame, s: &mut AppState, body: Rect, header_h: u16) {
    let pa = Rect {
        x: body.x,
        y: body.y + header_h + 1,
        width: body.width,
        height: body.height.saturating_sub(header_h + 1),
    };
    let lines = vec![
        Line::from(format!("Dir:    {}", short_path(&s.cwd))),
        Line::from(format!("Branch: {}", s.branch)),
        Line::from(format!("Remote: {}", s.remote)),
        Line::from(format!("Behind: {} commits", s.behind)),
        Line::from(format!("Ahead:  {} commits", s.ahead)),
        Line::from(format!("Files:  {} modified files", s.files.len())),
    ];
    f.render_widget(
        Paragraph::new(lines).style(
            Style::default()
                .fg(s.theme.foreground)
                .bg(s.theme.background),
        ),
        pa,
    );
}

pub fn draw_console(f: &mut Frame, area: Rect, s: &mut AppState) {
    let ch = (area.height * 35 / 100).clamp(6, 20);
    let cw = (area.width * 80 / 100).min(100);
    let cx = (area.width.saturating_sub(cw)) / 2;
    let cy = area.height.saturating_sub(ch + 1);
    let car = Rect {
        x: cx,
        y: cy,
        width: cw,
        height: ch + 1,
    };

    let fill_bg = s.theme.surface;
    for row in car.y..car.y + car.height {
        f.render_widget(
            Paragraph::new(" ".repeat(car.width as usize)).style(Style::default().bg(fill_bg)),
            Rect {
                x: car.x,
                y: row,
                width: car.width,
                height: 1,
            },
        );
    }

    draw_solid_border(f, car, &s.theme);

    let title = if s.console_running {
        " ⏳ Console (running...) Esc=close ↑↓=scroll "
    } else {
        " ⏹ Console Esc=close ↑↓=scroll "
    };
    let tw = title.chars().count() as u16;
    if tw < car.width.saturating_sub(2) {
        let tx = car.x + (car.width - tw) / 2;
        f.render_widget(
            Paragraph::new(title).style(
                Style::default()
                    .fg(fill_bg)
                    .bg(s.theme.primary)
                    .add_modifier(Modifier::BOLD),
            ),
            Rect {
                x: tx,
                y: car.y,
                width: tw,
                height: 1,
            },
        );
    }

    let inner = Rect {
        x: car.x + 1,
        y: car.y + 1,
        width: car.width.saturating_sub(2),
        height: car.height.saturating_sub(2),
    };

    if s.console_output.is_empty() {
        f.render_widget(
            Paragraph::new("(no output)").style(Style::default().fg(s.theme.dimmed).bg(fill_bg)),
            inner,
        );
        return;
    }

    let all_lines: Vec<&str> = s.console_output.split('\n').collect();
    let visible_h = inner.height as usize;
    let max_scroll = all_lines.len().saturating_sub(visible_h);
    if s.console_scroll > max_scroll {
        s.console_scroll = max_scroll;
    }

    let visible: Vec<&str> = all_lines
        .iter()
        .skip(s.console_scroll)
        .take(visible_h)
        .copied()
        .collect();

    let styled: Vec<Line> = visible
        .iter()
        .map(|line| {
            if line.starts_with('$') {
                Line::from(Span::styled(
                    *line,
                    Style::default()
                        .fg(s.theme.accent)
                        .bg(fill_bg)
                        .add_modifier(Modifier::BOLD),
                ))
            } else if line.starts_with("✓") {
                Line::from(Span::styled(
                    *line,
                    Style::default()
                        .fg(s.theme.success)
                        .bg(fill_bg)
                        .add_modifier(Modifier::BOLD),
                ))
            } else if line.starts_with("error:")
                || line.starts_with("fatal:")
                || line.starts_with("Error")
            {
                Line::from(Span::styled(
                    *line,
                    Style::default().fg(s.theme.warning).bg(fill_bg),
                ))
            } else {
                Line::from(Span::styled(
                    *line,
                    Style::default().fg(s.theme.foreground).bg(fill_bg),
                ))
            }
        })
        .collect();

    f.render_widget(Paragraph::new(styled), inner);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncate_subject_keeps_short_text_intact() {
        assert_eq!(truncate_subject("init", 50), "init");
        assert_eq!(truncate_subject("exactly5", 8), "exactly5");
    }

    #[test]
    fn truncate_subject_ascii_long_text_gets_ellipsis() {
        let s = truncate_subject(&"x".repeat(40), 10);
        assert_eq!(s, "xxxxxxx...");
    }

    #[test]
    fn truncate_subject_multibyte_does_not_panic() {
        // Regression: byte-slicing panicked on subjects like "añadir ✨ función".
        let s = truncate_subject("añadir ✨ función con tildes", 10);
        assert_eq!(s.chars().count(), 10);
        assert!(s.ends_with("..."));
    }

    #[test]
    fn truncate_subject_respects_min_budget() {
        // max < 5 clamps to 5, so "ab..." is always representable.
        let s = truncate_subject("abcdefg", 1);
        assert_eq!(s, "ab...");
    }

    #[test]
    fn clip_pads_short_and_elides_long() {
        assert_eq!(clip("ab", 5), "ab   ");
        assert_eq!(clip("abcdef", 5), "abcd\u{2026}");
        // Char-boundary safe with multibyte content.
        assert_eq!(clip("café-verde-más", 6), "caf\u{00e9}-\u{2026}");
    }

    #[test]
    fn browser_scroll_no_op_when_it_fits() {
        assert_eq!(browser_scroll(5, 10, 4), 0);
        assert_eq!(browser_scroll(3, 0, 2), 0);
    }

    #[test]
    fn browser_scroll_keeps_cursor_visible() {
        // 30 rows, 10 visible: bottom cursor clamps to the last window.
        assert_eq!(browser_scroll(30, 10, 29), 20);
        // Middle cursor centers.
        assert_eq!(browser_scroll(30, 10, 15), 10);
        // Top cursor pins to zero.
        assert_eq!(browser_scroll(30, 10, 2), 0);
    }

    #[test]
    fn sidebar_split_without_browser_keeps_quarter_width() {
        let body = Rect {
            x: 1,
            y: 2,
            width: 100,
            height: 40,
        };
        let (files, browser) = sidebar_split(body, false, 0);
        assert!(browser.is_none());
        assert_eq!(files.width, 25);
        assert_eq!(files.height, 40);
    }

    #[test]
    fn sidebar_split_with_browser_stacks_bottom_sector() {
        let body = Rect {
            x: 0,
            y: 0,
            width: 100,
            height: 40,
        };
        let (files, Some(browser)) = sidebar_split(body, true, 8) else {
            panic!("browser area expected");
        };
        assert_eq!(files.width, browser.width);
        assert_eq!(files.height + browser.height, body.height);
        assert_eq!(browser.y, body.y + files.height);
        // 8 rows + 6 chrome = 14 requested.
        assert_eq!(browser.height, 14);
    }

    #[test]
    fn browser_area_full_height_when_no_repo() {
        let body = Rect {
            x: 0,
            y: 0,
            width: 100,
            height: 40,
        };
        let area = browser_area(body, 30, true);
        assert_eq!(area.y, 0);
        assert_eq!(area.height, 40);
        assert!(area.width >= 38 && area.width <= 58);
    }

    #[test]
    fn is_within_matches_same_dir_and_children() {
        assert!(is_within("/a/b", "/a/b"));
        assert!(is_within("/a/b/", "/a/b"));
        assert!(is_within("/a/b", "/a/b/src"));
        assert!(is_within("/a/b", "/a/b/src/deep"));
        // Windows-style separators + case-insensitivity.
        assert!(is_within("C:\\Users\\me\\Proj", "c:\\users\\me\\proj\\src"));
    }

    #[test]
    fn is_within_rejects_lookalikes_and_parents() {
        assert!(!is_within("/a/b", "/a/bc")); // sibling, not child
        assert!(!is_within("/a/b", "/a")); // parent
        assert!(!is_within("/a/b", "/x/a/b")); // unrelated prefix
        assert!(!is_within("C:\\a", "C:\\ab"));
        // Multi-byte boundary: must not panic nor match mid-codepoint.
        assert!(!is_within("/a/b", "/a/bé"));
        assert!(is_within("/café", "/café/src"));
    }
}
