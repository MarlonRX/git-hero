// ── Rendering Engine ──────────────────────────────────────────────────
// Main UI layout, dashboard, and panel rendering with theme-aware styles

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, Paragraph},
    Frame,
};

use crate::theme::Theme;
use super::state::AppState;

/// Replace home directory with ~ for readable display
fn short_path(path: &str) -> String {
    if let Some(home) = dirs::home_dir() {
        let home_str = home.to_string_lossy();
        if let Some(rest) = path.strip_prefix(home_str.as_ref()) {
            return format!("~{}", rest);
        }
    }
    path.to_string()
}

/// Dibuja un border sólido con caracteres de bloque █
/// Compatible con TODOS los emuladores (Warp, iTerm, VSCode, Kitty, etc.)
pub fn draw_solid_border(f: &mut Frame, area: Rect, theme: &Theme) {
    let s = Style::default().bg(theme.border).fg(theme.border);
    let w = area.width as usize;

    // Top & bottom
    f.render_widget(
        Paragraph::new("\u{2588}".repeat(w)).style(s),
        Rect { x: area.x, y: area.y, width: area.width, height: 1 },
    );
    f.render_widget(
        Paragraph::new("\u{2588}".repeat(w)).style(s),
        Rect { x: area.x, y: area.y + area.height - 1, width: area.width, height: 1 },
    );
    // Left & right
    for row in 1..area.height.saturating_sub(1) {
        let y = area.y + row;
        f.render_widget(Paragraph::new("\u{2588}").style(s), Rect { x: area.x, y, width: 1, height: 1 });
        f.render_widget(
            Paragraph::new("\u{2588}").style(s),
            Rect { x: area.x + area.width - 1, y, width: 1, height: 1 },
        );
    }
}

/// Dibuja una línea horizontal de bloques sólidos
pub fn draw_solid_hline(f: &mut Frame, x: u16, y: u16, width: u16, color: ratatui::style::Color) {
    let s = Style::default().bg(color).fg(color);
    f.render_widget(
        Paragraph::new("\u{2588}".repeat(width as usize)).style(s),
        Rect { x, y, width, height: 1 },
    );
}

// ═════════════════════════════════════════════════════════════════════════
//  MAIN UI
// ═════════════════════════════════════════════════════════════════════════

pub fn draw_ui(f: &mut Frame, s: &mut AppState) {
    let area = f.area();

    // Setup wizard override
    if s.setup_step > 0 {
        super::modals::draw_setup_wizard(f, s);
        return;
    }

    // 1. Full theme background
    f.render_widget(Paragraph::new("").style(Style::default().bg(s.theme.background)), area);

    // 2. Centered 80% layout
    let target_w = (area.width as f32 * 0.80) as u16;
    let target_h = (area.height as f32 * 0.85) as u16;

    let outer = Rect {
        x: area.x + (area.width.saturating_sub(target_w)) / 2,
        y: area.y + (area.height.saturating_sub(target_h)) / 2,
        width: target_w.max(40),
        height: target_h.max(10),
    };
    if outer.width < 20 || outer.height < 8 {
        return;
    }

    // 3. Outer border (solid blocks)
    draw_solid_border(f, outer, &s.theme);

    // 4. Inner area
    let inner = Rect {
        x: outer.x + 1,
        y: outer.y + 1,
        width: outer.width.saturating_sub(2),
        height: outer.height.saturating_sub(2),
    };

    // 5. Header — text directly on the border line (no badges, no fill gaps)
    let header_text = format!("[ GIT HERO {} ]", s.get_icon_str("commit"));
    f.render_widget(
        Paragraph::new(header_text)
            .style(Style::default().fg(s.theme.primary).bg(s.theme.border).add_modifier(Modifier::BOLD)),
        Rect { x: outer.x + 2, y: outer.y, width: 20, height: 1 },
    );

    // Theme name on the right side of top border
    let badge_text = format!("[ {} ]", s.theme.name);
    let badge_w = badge_text.chars().count() as u16;
    let badge_x = outer.x + outer.width.saturating_sub(badge_w + 2);
    f.render_widget(
        Paragraph::new(badge_text)
            .style(Style::default().fg(s.theme.accent).bg(s.theme.border).add_modifier(Modifier::BOLD)),
        Rect { x: badge_x, y: outer.y, width: badge_w, height: 1 },
    );

    // 6. Footer reservation
    let footer_h: u16 = 2;
    let body = Rect {
        x: inner.x,
        y: inner.y,
        width: inner.width,
        height: inner.height.saturating_sub(footer_h),
    };
    let footer = Rect {
        x: inner.x,
        y: inner.y + body.height,
        width: inner.width,
        height: footer_h,
    };

    // 7. Content routing
    if !s.is_git_repo && !s.init_wizard_active {
        draw_no_repo_panel(f, s, body);
    } else if s.init_wizard_active {
        draw_init_wizard(f, s, body);
    } else {
        draw_dashboard(f, s, body);
    }

    // 8. Footer
    draw_solid_hline(f, footer.x, footer.y, footer.width, s.theme.border);

    // Status
    let status_icon = if s.fetching {
        format!(" {}", s.get_icon_str("fetch"))
    } else {
        String::new()
    };
    let status_str = format!(" {} {}", status_icon, s.status_message);
    let status_style = if s.fetching {
        Style::default().fg(s.theme.warning).bg(s.theme.background)
    } else {
        Style::default().fg(s.theme.success).bg(s.theme.background).add_modifier(Modifier::BOLD)
    };
    f.render_widget(
        Paragraph::new(status_str).style(status_style),
        Rect { x: footer.x + 1, y: footer.y + 1, width: footer.width / 2, height: 1 },
    );

    // Keyboard legend (right-aligned, minimal)
    let legend = if s.language == "es" {
        "? Ayuda | q Salir"
    } else {
        "? Help | q Quit"
    };
    let llen = legend.chars().count() as u16;
    f.render_widget(
        Paragraph::new(legend).style(Style::default().fg(s.theme.dimmed).bg(s.theme.background)),
        Rect { x: footer.x + footer.width.saturating_sub(llen + 1), y: footer.y + 1, width: llen, height: 1 },
    );

    // 9. Command bar (overlay on bottom border)
    if s.show_input && !s.show_theme_modal && !s.show_help_modal && !s.show_docs_modal {
        let iy = outer.y + outer.height - 1;
        let ia = Rect { x: outer.x + 1, y: iy, width: outer.width.saturating_sub(2), height: 1 };

        // Input background (uses primary to stand out)
        f.render_widget(
            Paragraph::new(" ".repeat(ia.width as usize))
                .style(Style::default().bg(s.theme.primary)),
            ia,
        );

        // Prompt arrow
        f.render_widget(
            Paragraph::new(" \u{276F} ")
                .style(Style::default().fg(s.theme.accent).bg(s.theme.primary).add_modifier(Modifier::BOLD)),
            Rect { x: ia.x, y: iy, width: 3, height: 1 },
        );

        // Input text
        let display = if s.input_value.is_empty() {
            Span::styled("Type a command...", Style::default().fg(s.theme.dimmed).bg(s.theme.primary))
        } else {
            Span::styled(
                &s.input_value,
                Style::default().fg(s.theme.background).bg(s.theme.primary).add_modifier(Modifier::BOLD),
            )
        };
        let ta = Rect { x: ia.x + 3, y: iy, width: ia.width.saturating_sub(4), height: 1 };
        f.render_widget(Paragraph::new(Line::from(vec![display])), ta);

        // Cursor
        let cx = ta.x + s.input_cursor_pos as u16;
        if cx < ta.x + ta.width {
            f.render_widget(
                Paragraph::new(" ").style(Style::default().bg(s.theme.accent)),
                Rect { x: cx, y: iy, width: 1, height: 1 },
            );
        }

        // Suggestions
        if !s.suggestions.is_empty() {
            let sh = s.suggestions.len() as u16;
            let sa = Rect { x: outer.x + 1, y: iy.saturating_sub(sh + 1), width: 42, height: sh };
            f.render_widget(Clear, sa);
            let items: Vec<ListItem> = s.suggestions.iter().enumerate().map(|(i, sug)| {
                let sty = if i == s.active_sug {
                    Style::default().bg(s.theme.primary).fg(s.theme.background).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(s.theme.foreground).bg(s.theme.background)
                };
                let pre = if i == s.active_sug { " \u{25B6} " } else { "   " };
                ListItem::new(format!("{}{}", pre, sug)).style(sty)
            }).collect();
            let sl = List::new(items).block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(s.theme.dimmed)),
            );
            f.render_widget(sl, Rect { x: sa.x, y: sa.y.saturating_sub(1), width: sa.width, height: sa.height + 2 });
        }
    }

    // 10. Floating modals
    if s.show_theme_modal {
        super::modals::draw_theme_modal(f, s);
    } else if s.show_help_modal {
        super::modals::draw_help_modal(f, s);
    } else if s.show_docs_modal {
        super::modals::draw_docs_modal(f, s);
    }
}

// ═════════════════════════════════════════════════════════════════════════
//  NO-REPO PANEL
// ═════════════════════════════════════════════════════════════════════════

pub fn draw_no_repo_panel(f: &mut Frame, s: &mut AppState, body: Rect) {
    f.render_widget(Paragraph::new("").style(Style::default().bg(s.theme.background)), body);

    let pa = Rect {
        x: body.x + 1,
        y: body.y + 1,
        width: body.width.saturating_sub(2),
        height: body.height.saturating_sub(2),
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(s.theme.primary))
        .title(" \u{26A0} No Git Repository Detected ")
        .title_style(Style::default().fg(s.theme.warning).add_modifier(Modifier::BOLD));
    f.render_widget(Clear, pa);
    f.render_widget(block, pa);

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

// ═════════════════════════════════════════════════════════════════════════
//  INIT WIZARD PANEL
// ═════════════════════════════════════════════════════════════════════════

pub fn draw_init_wizard(f: &mut Frame, s: &mut AppState, body: Rect) {
    f.render_widget(Paragraph::new("").style(Style::default().bg(s.theme.background)), body);

    let pa = Rect {
        x: body.x + 1,
        y: body.y + 1,
        width: body.width.saturating_sub(2),
        height: body.height.saturating_sub(2),
    };

    let (title, lines) = wizard_content(s);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(s.theme.primary))
        .title(title)
        .title_style(Style::default().fg(s.theme.accent).add_modifier(Modifier::BOLD));
    f.render_widget(block, pa);

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

// ═════════════════════════════════════════════════════════════════════════
//  MAIN DASHBOARD - Recuadros fijos con textos planos encima
// ═════════════════════════════════════════════════════════════════════════

pub fn draw_dashboard(f: &mut Frame, s: &mut AppState, body: Rect) {
    // Full theme background
    f.render_widget(Paragraph::new("").style(Style::default().bg(s.theme.background)), body);

    // ── Header row ──
    let header_h: u16 = 2;

    // Branch as plain text (no badge background)
    let branch_text = format!("{} {}", s.get_icon_str("branch"), s.branch);
    f.render_widget(
        Paragraph::new(branch_text)
            .style(Style::default().fg(s.theme.primary).bg(s.theme.background).add_modifier(Modifier::BOLD)),
        Rect { x: body.x + 1, y: body.y, width: 20, height: 1 },
    );

    let mut dir = s.cwd.clone();
    // Replace home directory with ~ for readability
    if let Some(home) = dirs::home_dir() {
        let home_str = home.to_string_lossy().to_string();
        if dir.starts_with(&home_str) {
            dir = dir.replacen(&home_str, "~", 1);
        }
    }
    if dir.len() > 35 {
        dir = format!("...{}", &dir[dir.len() - 32..]);
    }
    f.render_widget(
        Paragraph::new(format!("{} {}", s.get_icon_str("dir"), dir))
            .style(Style::default().fg(s.theme.foreground).bg(s.theme.background)),
        Rect { x: body.x + 22, y: body.y, width: 38, height: 1 },
    );

    let remote_text = format!("{} {}", s.get_icon_str("fetch"), s.remote);
    f.render_widget(
        Paragraph::new(remote_text)
            .style(Style::default().fg(s.theme.dimmed).bg(s.theme.background)),
        Rect { x: body.x + 60, y: body.y, width: 28, height: 1 },
    );

    // Sub-header separator line (thin, subtle)
    f.render_widget(
        Paragraph::new("\u{2500}".repeat(body.width as usize))
            .style(Style::default().fg(s.theme.border).bg(s.theme.background)),
        Rect { x: body.x, y: body.y + 1, width: body.width, height: 1 },
    );

    if body.width < 50 {
        draw_compact(f, s, body, header_h);
        return;
    }

    // Content area
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

    // ── SIDEBAR ────────────────────────────────────────────────────
    let side_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(3), Constraint::Length(9)])
        .split(sidebar);

    // Info block - recuadro fijo con borde
    let info_area = side_chunks[0];
    let info_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Plain)
        .border_style(Style::default().fg(s.theme.border))
        .title(" STATUS ")
        .title_style(Style::default().fg(s.theme.primary).add_modifier(Modifier::BOLD));
    f.render_widget(info_block.clone(), info_area);
    let info_inner = info_block.inner(info_area);

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

    // Files block - recuadro fijo con borde
    let files_area = side_chunks[1];
    let files_title = format!(" FILES ({}) ", s.files.len());
    let files_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Plain)
        .border_style(Style::default().fg(s.theme.border))
        .title(files_title)
        .title_style(if s.focus_pane == "files" {
            Style::default().fg(s.theme.primary).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(s.theme.foreground).add_modifier(Modifier::BOLD)
        });
    f.render_widget(files_block.clone(), files_area);
    let files_inner = files_block.inner(files_area);

    if s.files.is_empty() {
        let clean = if s.language == "es" { "\u{2713} Working directory clean" } else { "\u{2713} Working directory clean" };
        f.render_widget(
            Paragraph::new(clean).style(Style::default().fg(s.theme.success).bg(s.theme.background).add_modifier(Modifier::BOLD)),
            files_inner,
        );
    } else {
        let items: Vec<ListItem> = s.files.iter().enumerate().map(|(i, f)| {
            let fg = if f.staged { s.theme.success } else if f.status == "??" { s.theme.dimmed } else { s.theme.warning };
            let cb = if f.staged { "[\u{2713}]" } else { "[ ]" };
            let icon = match f.status.as_str() {
                "A" => s.get_icon_str("add"),
                "D" => s.get_icon_str("del"),
                "??" => s.get_icon_str("untracked"),
                _ => s.get_icon_str("mod"),
            };
            let pre = if i == s.selected_file_idx && s.focus_pane == "files" { "\u{25B6} " } else { "  " };

            let style = if i == s.selected_file_idx && s.focus_pane == "files" {
                Style::default().bg(s.theme.highlight).fg(s.theme.on_highlight).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(fg).bg(s.theme.background)
            };
            ListItem::new(format!("{}{} {} {}", pre, cb, icon, f.path)).style(style)
        }).collect();
        f.render_widget(List::new(items).style(Style::default().bg(s.theme.background)), files_inner);
    }

    // ── SHORTCUTS ─────────────────────────────────────────────────
    let shortcuts_area = side_chunks[2];
    let shortcuts_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Plain)
        .border_style(Style::default().fg(s.theme.border))
        .title(" \u{2328} SHORTCUTS ");
    f.render_widget(shortcuts_block.clone(), shortcuts_area);
    let shortcuts_inner = shortcuts_block.inner(shortcuts_area);

    let lines = vec![
        Line::from(Span::styled(" a Stage all  u Unstage all", Style::default().fg(s.theme.dimmed).bg(s.theme.background))),
        Line::from(Span::styled(" c Commit     r Undo commit", Style::default().fg(s.theme.dimmed).bg(s.theme.background))),
        Line::from(Span::styled(" p Push       f Fetch", Style::default().fg(s.theme.dimmed).bg(s.theme.background))),
        Line::from(Span::styled(" l Pull       s Stash", Style::default().fg(s.theme.dimmed).bg(s.theme.background))),
        Line::from(Span::styled(" d Stash pop  n New branch", Style::default().fg(s.theme.dimmed).bg(s.theme.background))),
        Line::from(Span::styled(" b Branches   o Remote", Style::default().fg(s.theme.dimmed).bg(s.theme.background))),
        Line::from(Span::styled(" t Theme      Spc Stage", Style::default().fg(s.theme.dimmed).bg(s.theme.background))),
        Line::from(Span::styled(" Tab Focus    ? Help  q Quit", Style::default().fg(s.theme.dimmed).bg(s.theme.background))),
    ];
    f.render_widget(Paragraph::new(lines).style(Style::default().bg(s.theme.background)), shortcuts_inner);

    // ── RIGHT PANEL ───────────────────────────────────────────────
    // Diff gets more space (65%) than commits (35%)
    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(65), Constraint::Percentage(35)])
        .split(right);

    // Diff block - recuadro fijo con borde
    let diff_area = right_chunks[0];
    let mut diff_title = " DIFF ".to_string();
    if s.focus_pane == "commits" && !s.commits.is_empty() && s.selected_commit_idx < s.commits.len() {
        diff_title = format!(" COMMIT: {} ", s.commits[s.selected_commit_idx].hash);
    } else if !s.files.is_empty() && s.selected_file_idx < s.files.len() {
        diff_title = format!(" DIFF: {} ", s.files[s.selected_file_idx].path);
    }
    let diff_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Plain)
        .border_style(Style::default().fg(s.theme.border))
        .title(diff_title)
        .title_style(Style::default().fg(s.theme.accent).add_modifier(Modifier::BOLD));
    f.render_widget(diff_block.clone(), diff_area);
    let diff_inner = diff_block.inner(diff_area);

    let dlines: Vec<Line> = s.active_diff.split('\n')
        .skip(s.diff_scroll_offset)
        .take(diff_inner.height as usize)
        .map(|line| {
            let fg = if line.starts_with('+') && !line.starts_with("+++") {
                s.theme.success
            } else if line.starts_with('-') && !line.starts_with("---") {
                s.theme.warning
            } else if line.starts_with("@@") {
                s.theme.primary
            } else if line.starts_with("commit ") || line.starts_with("diff ") || line.starts_with("Author:") || line.starts_with("Date:") {
                s.theme.accent
            } else {
                s.theme.foreground
            };
            Line::from(Span::styled(line, Style::default().fg(fg).bg(s.theme.background)))
        })
        .collect();
    f.render_widget(Paragraph::new(dlines).style(Style::default().bg(s.theme.background)), diff_inner);

    // Commits block - recuadro fijo con borde
    let commit_area = right_chunks[1];
    let commits_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Plain)
        .border_style(Style::default().fg(s.theme.border))
        .title(" COMMITS ")
        .title_style(if s.focus_pane == "commits" {
            Style::default().fg(s.theme.primary).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(s.theme.foreground).add_modifier(Modifier::BOLD)
        });
    f.render_widget(commits_block.clone(), commit_area);
    let commits_inner = commits_block.inner(commit_area);

    if s.commits.is_empty() {
        f.render_widget(
            Paragraph::new("No commits found.")
                .style(Style::default().fg(s.theme.dimmed).bg(s.theme.background)),
            commits_inner,
        );
    } else {
        let items: Vec<ListItem> = s.commits.iter().enumerate().map(|(i, c)| {
            let pre = if i == s.selected_commit_idx && s.focus_pane == "commits" { "\u{25B6} " } else { "  " };
            let mut subj = c.subject.clone();
            let sw = right.width.saturating_sub(30) as usize;
            if subj.len() > sw.max(5) { subj = format!("{}...", &subj[..sw.max(5) - 3]); }

            let item_s = if i == s.selected_commit_idx && s.focus_pane == "commits" {
                Style::default().bg(s.theme.highlight).fg(s.theme.on_highlight).add_modifier(Modifier::BOLD)
            } else {
                Style::default().bg(s.theme.background)
            };

            let line = if i == s.selected_commit_idx && s.focus_pane == "commits" {
                Line::from(vec![
                    Span::raw(format!("{}{}", pre, c.hash)),
                    Span::raw(format!("  ({})", c.date)),
                    Span::raw(format!("  {}", subj)),
                ])
            } else {
                Line::from(vec![
                    Span::raw(pre),
                    Span::styled(&c.hash, Style::default().fg(s.theme.accent)),
                    Span::styled(format!("  ({})", c.date), Style::default().fg(s.theme.dimmed)),
                    Span::raw(format!("  {}", subj)),
                ])
            };
            ListItem::new(line).style(item_s)
        }).collect();
        f.render_widget(List::new(items).style(Style::default().bg(s.theme.background)), commits_inner);
    }
}

// ═════════════════════════════════════════════════════════════════════════
//  COMPACT FALLBACK
// ═════════════════════════════════════════════════════════════════════════

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
