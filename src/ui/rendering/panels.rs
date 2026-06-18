use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Clear, List, ListItem, Paragraph, BorderType},
    Frame,
};

use crate::ui::state::{AppState, FlatEntryKind, GitCommit};
use super::components::{draw_solid_border, draw_solid_hline, draw_continuous_border, soften, short_path};

pub fn draw_no_repo_panel(f: &mut Frame, s: &mut AppState, body: Rect) {
    f.render_widget(Paragraph::new("").style(Style::default().bg(s.theme.background)), body);

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
        Style::default().fg(s.theme.warning).add_modifier(Modifier::BOLD),
        s.theme.primary,
        s.theme.background,
        BorderType::Rounded,
    );

    let inner_pa = Rect { x: pa.x + 1, y: pa.y + 1, width: pa.width.saturating_sub(2), height: pa.height.saturating_sub(2) };
    f.render_widget(Paragraph::new("").style(Style::default().bg(s.theme.background)), inner_pa);

    let cy = pa.y + 3;

    f.render_widget(
        Paragraph::new("This directory is not inside a Git repository.")
            .alignment(Alignment::Center)
            .style(Style::default().fg(s.theme.warning).bg(s.theme.background).add_modifier(Modifier::BOLD)),
        Rect { x: pa.x + 2, y: cy, width: pa.width - 4, height: 1 },
    );

    f.render_widget(
        Paragraph::new(format!(" \u{1F4C1} Current path:  {}", short_path(&s.cwd)))
            .alignment(Alignment::Center)
            .style(Style::default().fg(s.theme.foreground).bg(s.theme.background)),
        Rect { x: pa.x + 2, y: cy + 2, width: pa.width - 4, height: 1 },
    );

    f.render_widget(
        Paragraph::new("\u{2501} Options \u{2501}")
            .alignment(Alignment::Center)
            .style(Style::default().fg(s.theme.accent).bg(s.theme.background).add_modifier(Modifier::BOLD)),
        Rect { x: pa.x + 2, y: cy + 5, width: pa.width - 4, height: 1 },
    );

    let opt1 = "\u{2776} Initialize Git repository here";
    let opt2 = "\u{2777} Change Directory (/cd <path>)";

    let o1s = if s.init_cursor == 0 {
        Style::default().bg(s.theme.primary).fg(s.theme.background).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(s.theme.foreground).bg(s.theme.surface)
    };
    let o2s = if s.init_cursor == 1 {
        Style::default().bg(s.theme.primary).fg(s.theme.background).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(s.theme.foreground).bg(s.theme.surface)
    };

    f.render_widget(
        Paragraph::new(format!("  {}  ", opt1)).alignment(Alignment::Center).style(o1s),
        Rect { x: pa.x + 4, y: cy + 7, width: pa.width - 8, height: 1 },
    );
    f.render_widget(
        Paragraph::new(format!("  {}  ", opt2)).alignment(Alignment::Center).style(o2s),
        Rect { x: pa.x + 4, y: cy + 9, width: pa.width - 8, height: 1 },
    );

    f.render_widget(
        Paragraph::new("Arrow keys / Enter / Click to select")
            .alignment(Alignment::Center)
            .style(Style::default().fg(s.theme.dimmed).bg(s.theme.background)),
        Rect { x: pa.x + 2, y: cy + 12, width: pa.width - 4, height: 1 },
    );
}

pub fn draw_init_wizard(f: &mut Frame, s: &mut AppState, body: Rect) {
    f.render_widget(Paragraph::new("").style(Style::default().bg(s.theme.background)), body);

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
        Style::default().fg(s.theme.accent).add_modifier(Modifier::BOLD),
        s.theme.primary,
        s.theme.background,
        BorderType::Rounded,
    );

    let inner = Rect { x: pa.x + 1, y: pa.y + 1, width: pa.width.saturating_sub(2), height: pa.height.saturating_sub(2) };
    f.render_widget(Paragraph::new("").style(Style::default().bg(s.theme.background)), inner);

    let cy = pa.y + 2;
    f.render_widget(
        Paragraph::new(lines.join("\n"))
            .style(Style::default().fg(s.theme.foreground).bg(s.theme.background)),
        Rect { x: pa.x + 2, y: cy, width: pa.width - 4, height: pa.height - 4 },
    );
}

fn wizard_content(s: &AppState) -> (String, Vec<String>) {
    let mut title = " Git Init - Step 1/3 ".to_string();
    let mut lines = Vec::new();

    match s.init_wizard_step {
        1 => {
            lines.push("Choose default branch name:".to_string());
            lines.push(String::new());
            for (i, o) in ["main", "master", "Custom (type name below)"].iter().enumerate() {
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
            let remote = if s.init_remote_url.is_empty() { "None" } else { &s.init_remote_url };
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
    f.render_widget(Paragraph::new("").style(Style::default().bg(s.theme.background)), body);

    let header_h: u16 = 2;

    let header_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(20),
            Constraint::Min(10),
            Constraint::Length(25),
        ])
        .split(Rect { x: body.x + 1, y: body.y, width: body.width.saturating_sub(2), height: 1 });

    let branch_text = format!("{} {}", s.get_icon_str("branch"), s.branch);
    f.render_widget(
        Paragraph::new(branch_text)
            .style(Style::default().fg(s.theme.primary).bg(s.theme.background).add_modifier(Modifier::BOLD)),
        header_layout[0],
    );

    let mut dir = s.cwd.clone();
    if let Some(home) = dirs::home_dir() {
        let home_str = home.to_string_lossy().to_string();
        if dir.starts_with(&home_str) {
            dir = dir.replacen(&home_str, "~", 1);
        }
    }
    let path_max_w = (header_layout[1].width as usize).saturating_sub(5);
    if dir.len() > path_max_w && path_max_w > 5 {
        dir = format!("...{}", &dir[dir.len().saturating_sub(path_max_w - 3)..]);
    }
    f.render_widget(
        Paragraph::new(format!("{} {}", s.get_icon_str("dir"), dir))
            .style(Style::default().fg(s.theme.foreground).bg(s.theme.background)),
        header_layout[1],
    );

    let remote_text = format!("{} {}", s.get_icon_str("fetch"), s.remote);
    f.render_widget(
        Paragraph::new(remote_text)
            .style(Style::default().fg(s.theme.dimmed).bg(s.theme.background)),
        header_layout[2],
    );

    draw_solid_hline(f, body.x, body.y + 1, body.width, s.theme.border);

    if body.width < 50 {
        draw_compact(f, s, body, header_h);
        return;
    }

    let content = Rect {
        x: body.x,
        y: body.y + header_h,
        width: body.width,
        height: body.height.saturating_sub(header_h),
    };

    let sidebar_w = (content.width / 4).max(20);
    let main = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(sidebar_w), Constraint::Min(20)])
        .split(content);

    let sidebar = main[0];
    let right = main[1];

    let side_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(3), Constraint::Length(9)])
        .split(sidebar);

    let info_area = side_chunks[0];
    draw_continuous_border(
        f,
        info_area,
        " STATUS ",
        Style::default().fg(s.theme.primary).add_modifier(Modifier::BOLD),
        s.theme.border,
        s.theme.background,
        BorderType::Plain,
    );
    let info_inner = Rect {
        x: info_area.x + 1,
        y: info_area.y + 1,
        width: info_area.width.saturating_sub(2),
        height: info_area.height.saturating_sub(2),
    };

    let behind_s = if s.behind > 0 { s.theme.warning } else { s.theme.dimmed };
    let ahead_s = if s.ahead > 0 { s.theme.success } else { s.theme.dimmed };

    f.render_widget(
        Paragraph::new(vec![
            Line::from(Span::styled(
                format!("{} Behind: {} commits", s.get_icon_str("commit"), s.behind),
                Style::default().fg(behind_s).bg(s.theme.background),
            )),
            Line::from(Span::styled(
                format!("{} Ahead:  {} commits", s.get_icon_str("commit"), s.ahead),
                Style::default().fg(ahead_s).bg(s.theme.background),
            )),
        ]),
        info_inner,
    );

    let files_area = side_chunks[1];
    let files_title = format!(" FILES ({}) ", s.files.len());
    let border_color = if s.focus_pane == "files" { s.theme.primary } else { s.theme.border };
    let title_style = if s.focus_pane == "files" {
        Style::default().fg(s.theme.primary).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(s.theme.foreground).add_modifier(Modifier::BOLD)
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
        let clean = if s.language == "es" { "\u{2713} Working directory clean" } else { "\u{2713} Working directory clean" };
        f.render_widget(
            Paragraph::new(clean).style(Style::default().fg(s.theme.success).bg(s.theme.background).add_modifier(Modifier::BOLD)),
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
            indicators.push(format!("{}{}", s.get_icon_str("untracked"), untracked_count));
        }
        
        let indicator_text = if indicators.is_empty() {
            String::new()
        } else {
            format!(" [{}] ", indicators.join(" "))
        };
        
        let items: Vec<ListItem> = s.flat_entries.iter().enumerate().map(|(i, entry)| {
            let pre = if i == s.flat_idx && s.focus_pane == "files" { "\u{25B6} " } else { "  " };
            let FlatEntryKind::File(fi) = entry.kind;

            let f = &s.files[fi];
            let fg = if f.staged { s.theme.success } else if f.status == "??" { s.theme.dimmed } else { s.theme.warning };
            let cb = if f.staged { "[\u{2713}]" } else { "[ ]" };
            
            let (icon, icon_color) = match f.status.as_str() {
                "A" => (s.get_icon_str("add"), s.theme.success),
                "D" => (s.get_icon_str("del"), s.theme.warning),
                "??" => (s.get_icon_str("untracked"), s.theme.dimmed),
                "M" | "MM" => (s.get_icon_str("mod"), s.theme.accent),
                _ => (s.get_icon_str("mod"), fg),
            };
            
            let style = if i == s.flat_idx && s.focus_pane == "files" {
                Style::default().bg(s.theme.highlight).fg(s.theme.on_highlight).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(fg).bg(s.theme.background)
            };
            
            let line = Line::from(vec![
                Span::raw(pre),
                Span::styled(cb, Style::default().fg(if f.staged { s.theme.success } else { s.theme.dimmed }).bg(s.theme.background)),
                Span::raw(" "),
                Span::styled(icon, Style::default().fg(icon_color).bg(s.theme.background)),
                Span::raw(" "),
                Span::styled(&f.path, style),
            ]);
            
            ListItem::new(line)
        }).collect();
        
        let all_items = if !indicator_text.is_empty() {
            let mut all = vec![ListItem::new(
                Line::from(Span::styled(
                    indicator_text.trim(),
                    Style::default().fg(s.theme.dimmed).bg(s.theme.background).add_modifier(Modifier::BOLD)
                ))
            )];
            all.extend(items);
            all
        } else {
            items
        };
        
        f.render_widget(List::new(all_items).style(Style::default().bg(s.theme.background)), files_inner);
    }

    let shortcuts_area = side_chunks[2];
    draw_continuous_border(
        f,
        shortcuts_area,
        " ⌨ SHORTCUTS ",
        Style::default().fg(s.theme.accent).add_modifier(Modifier::BOLD),
        s.theme.border,
        s.theme.background,
        BorderType::Plain,
    );
    let shortcuts_inner = Rect {
        x: shortcuts_area.x + 1,
        y: shortcuts_area.y + 1,
        width: shortcuts_area.width.saturating_sub(2),
        height: shortcuts_area.height.saturating_sub(2),
    };

    let lines = vec![
        Line::from(Span::styled(" a Stage all  u Unstage all", Style::default().fg(s.theme.dimmed).bg(s.theme.background))),
        Line::from(Span::styled(" c Commit     r Undo commit", Style::default().fg(s.theme.dimmed).bg(s.theme.background))),
        Line::from(Span::styled(" p Push       f Fetch", Style::default().fg(s.theme.dimmed).bg(s.theme.background))),
        Line::from(Span::styled(" l Pull       s Stash", Style::default().fg(s.theme.dimmed).bg(s.theme.background))),
        Line::from(Span::styled(" d Stash pop  n New branch", Style::default().fg(s.theme.dimmed).bg(s.theme.background))),
        Line::from(Span::styled(" b Branches   o Remote", Style::default().fg(s.theme.dimmed).bg(s.theme.background))),
        Line::from(Span::styled(" t Theme      Spc Stage", Style::default().fg(s.theme.dimmed).bg(s.theme.background))),
        Line::from(Span::styled(" Enter Detail y Copy diff", Style::default().fg(s.theme.dimmed).bg(s.theme.background))),
        Line::from(Span::styled(" Scroll: PgUp/PgDn or Mouse  ? Help q Quit", Style::default().fg(s.theme.dimmed).bg(s.theme.background))),
    ];
    f.render_widget(Paragraph::new(lines).style(Style::default().bg(s.theme.background)), shortcuts_inner);

    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(65), Constraint::Percentage(35)])
        .split(right);

    let diff_area = right_chunks[0];
    let mut diff_title = " DIFF ".to_string();
    if s.focus_pane == "commits" && !s.commits.is_empty() && s.selected_commit_idx < s.commits.len() {
        diff_title = format!(" COMMIT: {} ", s.commits[s.selected_commit_idx].hash);
    } else if !s.files.is_empty() && s.selected_file_idx < s.files.len() {
        let file_label = &s.files[s.selected_file_idx].path;
        diff_title = format!(" {} DIFF: {} {} ", 
            if s.focus_pane == "diff" { "\u{25BC}" } else { "" },
            file_label,
            if s.focus_pane == "diff" { "\u{25BC}" } else { "" },
        );
    }
    let diff_title_style = if s.focus_pane == "diff" {
        Style::default().fg(s.theme.primary).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(s.theme.accent).add_modifier(Modifier::BOLD)
    };
    let border_color = if s.focus_pane == "diff" { s.theme.primary } else { s.theme.border };
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
    let visible: Vec<Line> = dlines.iter()
        .skip(s.diff_scroll_offset)
        .take(diff_inner.height as usize)
        .cloned()
        .collect();
    f.render_widget(Paragraph::new(visible).style(Style::default().bg(s.theme.background)), diff_inner);

    let commit_area = right_chunks[1];
    let commits_title = if s.show_commit_detail {
        " COMMIT DETAILS (Enter to close) ".to_string()
    } else {
        " COMMITS (Enter for details) ".to_string()
    };
    let border_color = if s.focus_pane == "commits" { s.theme.primary } else { s.theme.border };
    let title_style = if s.focus_pane == "commits" {
        Style::default().fg(s.theme.primary).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(s.theme.foreground).add_modifier(Modifier::BOLD)
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
        let commit_theme = s.theme.clone();
        let c_warn = soften(commit_theme.warning, commit_theme.background, 0.25);
        let c_succ = soften(commit_theme.success, commit_theme.background, 0.25);
        let detail_lines: Vec<Line> = s.commit_detail_diff.split('\n')
            .skip(s.commit_detail_scroll)
            .take(commits_inner.height as usize)
            .map(|line| {
                if line.starts_with("commit ") {
                    Line::from(Span::styled(line.to_string(), Style::default().fg(commit_theme.accent).bg(commit_theme.background).add_modifier(Modifier::BOLD)))
                } else if line.starts_with("@@") {
                    Line::from(Span::styled(line.to_string(), Style::default().fg(commit_theme.primary).bg(commit_theme.surface).add_modifier(Modifier::BOLD)))
                } else if line.starts_with('+') && !line.starts_with("+++") {
                    let rest = if line.len() > 1 { line[1..].to_string() } else { String::new() };
                    Line::from(vec![
                        Span::styled("+".to_string(), Style::default().fg(commit_theme.foreground).bg(c_succ).add_modifier(Modifier::BOLD)),
                        Span::styled(rest, Style::default().fg(commit_theme.foreground).bg(c_succ)),
                    ])
                } else if line.starts_with('-') && !line.starts_with("---") {
                    let rest = if line.len() > 1 { line[1..].to_string() } else { String::new() };
                    Line::from(vec![
                        Span::styled("-".to_string(), Style::default().fg(commit_theme.foreground).bg(c_warn).add_modifier(Modifier::BOLD)),
                        Span::styled(rest, Style::default().fg(commit_theme.foreground).bg(c_warn)),
                    ])
                } else if line.starts_with("Author:") || line.starts_with("Date:") || line.starts_with("---") || line.starts_with("+++") {
                    Line::from(Span::styled(line.to_string(), Style::default().fg(commit_theme.dimmed).bg(commit_theme.background)))
                } else if line.trim().is_empty() {
                    Line::from(Span::raw(""))
                } else {
                    Line::from(Span::styled(line.to_string(), Style::default().fg(commit_theme.foreground).bg(commit_theme.background)))
                }
            })
            .collect();
        f.render_widget(Paragraph::new(detail_lines).style(Style::default().bg(s.theme.background)), commits_inner);
    } else {
        let visible_commits: Vec<&GitCommit> = s.commits.iter()
            .skip(s.commit_scroll_offset)
            .take(commits_inner.height as usize)
            .collect();
        
        let items: Vec<ListItem> = visible_commits.iter().enumerate().map(|(i, c)| {
            let actual_idx = s.commits.iter().position(|commit| commit.hash == c.hash).unwrap_or(i);
            let pre = if actual_idx == s.selected_commit_idx && s.focus_pane == "commits" { "\u{25B6} " } else { "  " };
            let mut subj = c.subject.clone();
            let sw = right.width.saturating_sub(36) as usize;
            if subj.len() > sw.max(5) { subj = format!("{}...", &subj[..sw.max(5) - 3]); }

            let push_icon = if c.pushed { "\u{2713}" } else { "\u{21C8}" };
            let push_style = if c.pushed {
                Style::default().fg(s.theme.success)
            } else {
                Style::default().fg(s.theme.warning).add_modifier(Modifier::BOLD)
            };

            let item_s = if actual_idx == s.selected_commit_idx && s.focus_pane == "commits" {
                Style::default().bg(s.theme.highlight).fg(s.theme.on_highlight).add_modifier(Modifier::BOLD)
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
                    Span::styled(format!("  ({})", c.date), Style::default().fg(s.theme.dimmed)),
                    Span::raw(format!("  {}", subj)),
                    Span::styled(format!("  {}", push_icon), push_style),
                ])
            };
            ListItem::new(line).style(item_s)
        }).collect();
        f.render_widget(List::new(items).style(Style::default().bg(s.theme.background)), commits_inner);
    }
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
        Paragraph::new(lines).style(Style::default().fg(s.theme.foreground).bg(s.theme.background)),
        pa,
    );
}

pub fn draw_console(f: &mut Frame, area: Rect, s: &mut AppState) {
    let ch = (area.height * 35 / 100).min(20).max(6);
    let cw = (area.width * 80 / 100).min(100);
    let cx = (area.width.saturating_sub(cw)) / 2;
    let cy = area.height.saturating_sub(ch + 1);
    let car = Rect { x: cx, y: cy, width: cw, height: ch + 1 };

    let fill_bg = s.theme.surface;
    for row in car.y..car.y + car.height {
        f.render_widget(
            Paragraph::new(" ".repeat(car.width as usize))
                .style(Style::default().bg(fill_bg)),
            Rect { x: car.x, y: row, width: car.width, height: 1 },
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
            Rect { x: tx, y: car.y, width: tw, height: 1 },
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
            Paragraph::new("(no output)")
                .style(Style::default().fg(s.theme.dimmed).bg(fill_bg)),
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
            } else if line.starts_with("error:") || line.starts_with("fatal:") || line.starts_with("Error") {
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
