// ── Modal Windows ──────────────────────────────────────────────────
// Setup wizard, theme selector, and help overlay
// Uses solid █ borders (same style as main UI layout)

use ratatui::{
    layout::{Alignment, Rect},
    style::{Modifier, Style},
    widgets::Paragraph,
    Frame,
};

use crate::i18n::translate;
use crate::theme::get_themes;
use crate::ui::state::AppState;

// ── Solid Border Helper ───────────────────────────────────────────

/// Draw a modal with solid █ borders (same as main UI layout).
/// Fills interior with theme background. Returns the inner content area.
fn draw_modal_frame(
    f: &mut Frame,
    modal: Rect,
    bg: ratatui::style::Color,
    border: ratatui::style::Color,
) -> Rect {
    let os = Style::default().fg(border).bg(bg);

    // 1. Fill entire modal area with background first
    for row in modal.y..modal.y + modal.height {
        f.render_widget(
            Paragraph::new(" ".repeat(modal.width as usize))
                .style(Style::default().bg(bg)),
            Rect { x: modal.x, y: row, width: modal.width, height: 1 },
        );
    }

    // 2. Draw outer rounded border
    // Top: ╭ ─ ... ─ ╮
    let top_str = format!("╭{}╮", "─".repeat(modal.width.saturating_sub(2) as usize));
    f.render_widget(
        Paragraph::new(top_str).style(os),
        Rect { x: modal.x, y: modal.y, width: modal.width, height: 1 },
    );
    // Bottom: ╰ ─ ... ─ ╯
    let bottom_str = format!("╰{}╯", "─".repeat(modal.width.saturating_sub(2) as usize));
    f.render_widget(
        Paragraph::new(bottom_str).style(os),
        Rect { x: modal.x, y: modal.y + modal.height - 1, width: modal.width, height: 1 },
    );
    // Sides: │
    for row in (modal.y + 1)..(modal.y + modal.height - 1) {
        f.render_widget(Paragraph::new("│").style(os), Rect { x: modal.x, y: row, width: 1, height: 1 });
        f.render_widget(Paragraph::new("│").style(os), Rect { x: modal.x + modal.width - 1, y: row, width: 1, height: 1 });
    }

    // Inner content area (inside the border)
    Rect {
        x: modal.x + 1,
        y: modal.y + 1,
        width: modal.width.saturating_sub(2),
        height: modal.height.saturating_sub(2),
    }
}

/// Draw a title badge centered on the top border
fn draw_modal_title(
    f: &mut Frame,
    modal: Rect,
    title: &str,
    fg: ratatui::style::Color,
    bg: ratatui::style::Color,
) {
    use unicode_width::UnicodeWidthStr;

    // Determine maximum allowed visual width for the entire formatted title.
    // Leave at least 4 cells on both sides (including corners) to prevent overflow/clash.
    let max_formatted_w = (modal.width as usize).saturating_sub(8);
    if max_formatted_w < 6 {
        return; // Modal is too narrow to display any title
    }

    let clean_title = title.trim();
    let formatted_title = format!("┤ {} ├", clean_title);
    let visual_w = formatted_title.width();

    let final_title = if visual_w > max_formatted_w {
        // Truncate clean_title to fit within budget.
        // "┤ " (2) + " ├" (2) + "..." (3) = 7 cells reserved.
        let text_budget = max_formatted_w.saturating_sub(7);
        let mut truncated = String::new();
        let mut curr_w = 0;
        for c in clean_title.chars() {
            use unicode_width::UnicodeWidthChar;
            let w = c.width().unwrap_or(0);
            if curr_w + w > text_budget {
                break;
            }
            truncated.push(c);
            curr_w += w;
        }
        format!("┤ {}... ├", truncated.trim_end())
    } else {
        formatted_title
    };

    let tw = final_title.width() as u16;
    let tx = modal.x + (modal.width.saturating_sub(tw)) / 2;
    f.render_widget(
        Paragraph::new(final_title).style(
            Style::default().fg(fg).bg(bg).add_modifier(Modifier::BOLD),
        ),
        Rect { x: tx, y: modal.y, width: tw, height: 1 },
    );
}

// ── Setup Wizard ───────────────────────────────────────────────────

pub fn draw_setup_wizard(f: &mut Frame, s: &mut AppState) {
    let area = f.area();
    let mw = 60u16;
    let mh = 14u16;
    let mx = (area.width.saturating_sub(mw)) / 2;
    let my = (area.height.saturating_sub(mh)) / 2;
    let modal = Rect { x: mx, y: my, width: mw, height: mh };

    let inner = draw_modal_frame(f, modal, s.theme.surface, s.theme.primary);

    let (title, lines) = setup_content(s);
    draw_modal_title(f, modal, &title, s.theme.surface, s.theme.primary);

    let cy = inner.y + 1;
    f.render_widget(
        Paragraph::new(lines.join("\n"))
            .style(Style::default().fg(s.theme.foreground).bg(s.theme.surface)),
        Rect { x: inner.x + 1, y: cy, width: inner.width - 2, height: inner.height - 2 },
    );

    let help = translate(&s.language, "setup_help");
    f.render_widget(
        Paragraph::new(help).alignment(Alignment::Center)
            .style(Style::default().fg(s.theme.primary).bg(s.theme.surface)),
        Rect { x: inner.x, y: inner.y + inner.height - 1, width: inner.width, height: 1 },
    );
}

fn setup_content(s: &AppState) -> (String, Vec<String>) {
    let mut title = " Language Setup ".to_string();
    let mut lines = Vec::new();
    match s.setup_step {
        1 => {
            lines.push("Select Language / Selecciona Idioma:".to_string());
            lines.push(String::new());
            for (i, o) in ["English", "Espa\u{00F1}ol"].iter().enumerate() {
                lines.push(if i == s.setup_cursor {
                    format!(" \u{25B6} {}", o)
                } else {
                    format!("   {}", o)
                });
            }
        }
        2 => {
            title = " Icons Setup ".to_string();
            lines.push("Select Icon Set:".to_string());
            lines.push(String::new());
            for (i, o) in ["Nerd Fonts (with icons)", "Standard ASCII (plain text)"].iter().enumerate() {
                lines.push(if i == s.setup_cursor {
                    format!(" \u{25B6} {}", o)
                } else {
                    format!("   {}", o)
                });
            }
        }
        3 => {
            title = " Theme Setup ".to_string();
            lines.push("Select Initial Theme:".to_string());
            lines.push(String::new());
            let themes = get_themes();
            let start = if s.setup_cursor > 3 { s.setup_cursor - 3 } else { 0 };
            let end = (start + 5).min(themes.len());
            for i in start..end {
                let t = &themes[i];
                lines.push(if i == s.setup_cursor {
                    format!(" \u{25B6} {:<20} \u{25A0}", t.name)
                } else {
                    format!("   {}", t.name)
                });
            }
        }
        _ => {}
    }
    (title, lines)
}

// ── Theme Selector Modal ───────────────────────────────────────────

pub fn draw_theme_modal(f: &mut Frame, s: &mut AppState) {
    let area = f.area();
    let themes = get_themes();
    let mw = 50u16;
    let mh = (themes.len() as u16 + 6).min(area.height - 2);
    let mx = (area.width.saturating_sub(mw)) / 2;
    let my = (area.height.saturating_sub(mh)) / 2;
    let modal = Rect { x: mx, y: my, width: mw, height: mh };

    let inner = draw_modal_frame(f, modal, s.theme.surface, s.theme.primary);
    draw_modal_title(f, modal, " \u{1F3A8} Select Visual Theme ", s.theme.surface, s.theme.accent);

    let mut lines = Vec::new();
    lines.push(format!("{}:", translate(&s.language, "theme_title")));
    lines.push(String::new());

    let start = if s.theme_cursor > 4 { s.theme_cursor - 4 } else { 0 };
    let end = (start + 9).min(themes.len());

    for i in start..end {
        let t = &themes[i];
        if i == s.theme_cursor {
            lines.push(format!(" \u{25B6} {:<20} \u{25A0}", t.name));
        } else {
            lines.push(format!("   {}", t.name));
        }
    }

    let cy = inner.y + 1;
    f.render_widget(
        Paragraph::new(lines.join("\n"))
            .style(Style::default().fg(s.theme.foreground).bg(s.theme.surface)),
        Rect { x: inner.x + 1, y: cy, width: inner.width - 2, height: inner.height - 2 },
    );

    let help = translate(&s.language, "theme_help");
    f.render_widget(
        Paragraph::new(help).alignment(Alignment::Center)
            .style(Style::default().fg(s.theme.primary).bg(s.theme.surface)),
        Rect { x: inner.x, y: inner.y + inner.height - 1, width: inner.width, height: 1 },
    );
}

// ── Help Modal ─────────────────────────────────────────────────────

pub fn draw_help_modal(f: &mut Frame, s: &mut AppState) {
    let area = f.area();
    let mw = 62u16;
    let mh = 20u16;
    let mx = (area.width.saturating_sub(mw)) / 2;
    let my = (area.height.saturating_sub(mh)) / 2;
    let modal = Rect { x: mx, y: my, width: mw, height: mh };

    let inner = draw_modal_frame(f, modal, s.theme.surface, s.theme.primary);
    draw_modal_title(f, modal, " \u{2753} Keyboard Shortcuts & Commands ", s.theme.surface, s.theme.accent);

    let lines: Vec<&str> = if s.language == "es" {
        vec![
            "\u{2699} NAVEGACI\u{00D3}N \u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}",
            "  j/k/\u{2191}/\u{2193}  Mover cursor  | Tab  Cambiar foco",
            "  Space      Stage/unstage archivo individual",
            "",
            "\u{2699} ACCIONES \u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}",
            "  a    Stage all cambios  |  u   Unstage all",
            "  c    Commit (\u{2710})   |  r   Undo last commit",
            "  p    Push               |  f   Fetch",
            "  l    Pull               |  s   Stash / d  Pop",
            "  b    List branches      |  n   New branch",
            "  o    Change remote      |  t   Select theme",
            "",
            "\u{2699} COMANDOS \u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}",
            "  /config <k> <v>  Set config local",
            "  /config-global <k> <v>  Set global",
            "  /branch <name>   Create & switch",
            "  /remove-repo     Delete .git dir",
            "  /undo-commit     Reset HEAD\u{2192}1 (soft)",
            "",
            "  ? / Esc  Close   |  q  Quit   |  /  Cmd bar",
        ]
    } else {
        vec![
            "\u{2699} NAVIGATION \u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}",
            "  j/k/\u{2191}/\u{2193}  Move cursor   | Tab  Switch focus",
            "  Space      Stage/unstage individual file",
            "",
            "\u{2699} ACTIONS \u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}",
            "  a    Stage all changes  |  u   Unstage all",
            "  c    Commit (\u{2710})   |  r   Undo last commit",
            "  p    Push               |  f   Fetch",
            "  l    Pull               |  s   Stash / d  Pop",
            "  b    List branches      |  n   New branch",
            "  o    Change remote      |  t   Select theme",
            "",
            "\u{2699} COMMANDS \u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}",
            "  /config <k> <v>  Set config locally",
            "  /config-global <k> <v>  Set globally",
            "  /branch <name>   Create & switch",
            "  /remove-repo     Delete .git dir",
            "  /undo-commit     Reset HEAD\u{2192}1 (soft)",
            "",
            "  ? / Esc  Close   |  q  Quit   |  /  Cmd bar",
        ]
    };

    let cy = inner.y + 1;
    f.render_widget(
        Paragraph::new(lines.join("\n"))
            .style(Style::default().fg(s.theme.foreground).bg(s.theme.surface)),
        Rect { x: inner.x + 1, y: cy, width: inner.width - 2, height: inner.height - 2 },
    );

    f.render_widget(
        Paragraph::new("Press any key to close.")
            .alignment(Alignment::Center)
            .style(Style::default().fg(s.theme.primary).bg(s.theme.surface)),
        Rect { x: inner.x, y: inner.y + inner.height - 1, width: inner.width, height: 1 },
    );
}

// ── Docs Modal ─────────────────────────────────────────────────────

pub fn draw_docs_modal(f: &mut Frame, s: &mut AppState) {
    let area = f.area();
    let mw = 68u16;
    let mh = 22u16;
    let mx = (area.width.saturating_sub(mw)) / 2;
    let my = (area.height.saturating_sub(mh)) / 2;
    let modal = Rect { x: mx, y: my, width: mw, height: mh };

    let inner = draw_modal_frame(f, modal, s.theme.surface, s.theme.primary);
    draw_modal_title(f, modal, " \u{1F4D6} Detailed Shortcut Reference ", s.theme.surface, s.theme.accent);

    let lines: Vec<&str> = if s.language == "es" {
        vec![
            "\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}",
            "",
            "\u{2699} Change Management",
            "  Space (select file)  Toggle stage/unstage a single file",
            "  a  /  /stage-all      Stage all changes (git add .)",
            "  u  /  /unstage-all    Unstage everything (git reset HEAD)",
            "  c  /  /commit <msg>   Create a commit; auto-stages if needed",
            "  r  /  /undo-commit    Undo last commit (soft reset)",
            "",
            "\u{2699} Branches & Remote",
            "  b  /  /branches       List all branches",
            "  n  /  /branch <name>  Create new branch and switch to it",
            "  /switch <name>        Switch to an existing branch",
            "  o  /  /remote <url>   Change the 'origin' remote URL",
            "",
            "\u{2699} Sync (fetch/pull/push)",
            "  f  /  /fetch          Download remote metadata (no merge)",
            "  l  /  /pull           Download + merge remote changes",
            "  p  /  /push           Push local commits to remote",
            "",
            "\u{2699} Stash",
            "  s  /  /stash          Stash local changes away",
            "  d  /  /stash-pop      Restore changes from the stash",
            "",
            "\u{2699} Configuration",
            "  /config <k> [v]       Read or set local repo config",
            "  /config-global <k><v> Set global config (~/.gitconfig)",
            "",
            "\u{2699} Other",
            "  t  /  /themes        Open visual theme picker",
            "  /remove-repo         Delete .git directory (no confirm!)",
            "  /docs               This detailed reference",
            "  ?  /  /help          Quick shortcut help",
        ]
    } else {
        vec![
            "\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}",
            "",
            "\u{2699} Change Management",
            "  Space (select file)  Toggle stage/unstage a single file",
            "  a  /  /stage-all      Stage all changes (git add .)",
            "  u  /  /unstage-all    Unstage everything (git reset HEAD)",
            "  c  /  /commit <msg>   Create a commit; auto-stages if needed",
            "  r  /  /undo-commit    Undo last commit (soft reset)",
            "",
            "\u{2699} Branches & Remote",
            "  b  /  /branches       List all branches",
            "  n  /  /branch <name>  Create new branch and switch to it",
            "  /switch <name>        Switch to an existing branch",
            "  o  /  /remote <url>   Change the 'origin' remote URL",
            "",
            "\u{2699} Sync (fetch/pull/push)",
            "  f  /  /fetch          Download remote metadata (no merge)",
            "  l  /  /pull           Download + merge remote changes",
            "  p  /  /push           Push local commits to remote",
            "",
            "\u{2699} Stash",
            "  s  /  /stash          Stash local changes away",
            "  d  /  /stash-pop      Restore changes from the stash",
            "",
            "\u{2699} Configuration",
            "  /config <k> [v]       Read or set local repo config",
            "  /config-global <k><v> Set global config (~/.gitconfig)",
            "",
            "\u{2699} Other",
            "  t  /  /themes        Open visual theme picker",
            "  /remove-repo         Delete .git directory (no confirm!)",
            "  /docs               This detailed reference",
            "  ?  /  /help          Quick shortcut help",
        ]
    };

    let cy = inner.y;
    f.render_widget(
        Paragraph::new(lines.join("\n"))
            .style(Style::default().fg(s.theme.foreground).bg(s.theme.surface)),
        Rect { x: inner.x + 1, y: cy, width: inner.width - 2, height: inner.height },
    );

    f.render_widget(
        Paragraph::new("Press any key to close.")
            .alignment(Alignment::Center)
            .style(Style::default().fg(s.theme.primary).bg(s.theme.surface)),
        Rect { x: inner.x, y: inner.y + inner.height - 1, width: inner.width, height: 1 },
    );
}

// ── Push Confirmation Modal ───────────────────────────────────────

pub fn draw_confirm_push_modal(f: &mut Frame, s: &mut AppState) {
    let area = f.area();
    let mw = 50u16;
    let mh = 8u16;
    let mx = (area.width.saturating_sub(mw)) / 2;
    let my = (area.height.saturating_sub(mh)) / 2;
    let modal = Rect { x: mx, y: my, width: mw, height: mh };

    let inner = draw_modal_frame(f, modal, s.theme.surface, s.theme.primary);
    draw_modal_title(f, modal, " 🚀 Push Confirmation ", s.theme.surface, s.theme.accent);

    let text = format!("Are you sure you want to push to remote?\n\nTarget: {}/{}", s.remote, s.branch);
    
    f.render_widget(
        Paragraph::new(text)
            .alignment(Alignment::Center)
            .style(Style::default().fg(s.theme.foreground).bg(s.theme.surface)),
        Rect { x: inner.x + 1, y: inner.y + 1, width: inner.width - 2, height: inner.height - 2 },
    );

    let help = "Press [y] / [Enter] to Confirm | [n] / [Esc] to Cancel";
    f.render_widget(
        Paragraph::new(help).alignment(Alignment::Center)
            .style(Style::default().fg(s.theme.primary).bg(s.theme.surface)),
        Rect { x: inner.x, y: inner.y + inner.height - 1, width: inner.width, height: 1 },
    );
}

// ── Pull Confirmation Modal ───────────────────────────────────────

pub fn draw_confirm_pull_modal(f: &mut Frame, s: &mut AppState) {
    let area = f.area();
    let mw = 50u16;
    let mh = 8u16;
    let mx = (area.width.saturating_sub(mw)) / 2;
    let my = (area.height.saturating_sub(mh)) / 2;
    let modal = Rect { x: mx, y: my, width: mw, height: mh };

    let inner = draw_modal_frame(f, modal, s.theme.surface, s.theme.primary);
    draw_modal_title(f, modal, " 📥 Pull Confirmation ", s.theme.surface, s.theme.accent);

    let text = format!("Are you sure you want to pull from remote?\n\nSource: {}/{}", s.remote, s.branch);
    
    f.render_widget(
        Paragraph::new(text)
            .alignment(Alignment::Center)
            .style(Style::default().fg(s.theme.foreground).bg(s.theme.surface)),
        Rect { x: inner.x + 1, y: inner.y + 1, width: inner.width - 2, height: inner.height - 2 },
    );

    let help = "Press [y] / [Enter] to Confirm | [n] / [Esc] to Cancel";
    f.render_widget(
        Paragraph::new(help).alignment(Alignment::Center)
            .style(Style::default().fg(s.theme.primary).bg(s.theme.surface)),
        Rect { x: inner.x, y: inner.y + inner.height - 1, width: inner.width, height: 1 },
    );
}

// ── Credentials Input Modal ───────────────────────────────────────

pub fn draw_credentials_modal(f: &mut Frame, s: &mut AppState) {
    let area = f.area();
    let mw = 60u16;
    let mh = 10u16;
    let mx = (area.width.saturating_sub(mw)) / 2;
    let my = (area.height.saturating_sub(mh)) / 2;
    let modal = Rect { x: mx, y: my, width: mw, height: mh };

    let inner = draw_modal_frame(f, modal, s.theme.surface, s.theme.primary);
    draw_modal_title(f, modal, " 🔑 Remote Credentials Required ", s.theme.surface, s.theme.primary);

    let prompt_text = format!("Prompt: {}", s.credentials_prompt);
    f.render_widget(
        Paragraph::new(prompt_text)
            .style(Style::default().fg(s.theme.foreground).bg(s.theme.surface)),
        Rect { x: inner.x + 2, y: inner.y + 1, width: inner.width - 4, height: 2 },
    );

    let input_y = inner.y + 3;
    let input_w = inner.width - 4;
    f.render_widget(
        Paragraph::new(" ".repeat(input_w as usize))
            .style(Style::default().bg(s.theme.surface)),
        Rect { x: inner.x + 2, y: input_y, width: input_w, height: 1 },
    );

    let displayed_input = if s.credentials_mask {
        "*".repeat(s.credentials_input.len())
    } else {
        s.credentials_input.clone()
    };
    f.render_widget(
        Paragraph::new(displayed_input)
            .style(Style::default().fg(s.theme.accent).bg(s.theme.surface)),
        Rect { x: inner.x + 3, y: input_y, width: input_w - 2, height: 1 },
    );

    let cx = inner.x + 3 + s.credentials_cursor as u16;
    if cx < inner.x + 2 + input_w {
        f.render_widget(
            Paragraph::new(" ").style(Style::default().bg(s.theme.accent)),
            Rect { x: cx, y: input_y, width: 1, height: 1 },
        );
    }

    let help = "Enter: Submit | Esc: Cancel";
    f.render_widget(
        Paragraph::new(help).alignment(Alignment::Center)
            .style(Style::default().fg(s.theme.primary).bg(s.theme.surface)),
        Rect { x: inner.x, y: inner.y + inner.height - 1, width: inner.width, height: 1 },
    );
}

// ── Commit Message Editor Modal ─────────────────────────

pub fn draw_commit_modal(f: &mut Frame, s: &mut AppState) {
    let area = f.area();
    let mw = 70u16;
    let mh = 16u16;
    let mx = (area.width.saturating_sub(mw)) / 2;
    let my = (area.height.saturating_sub(mh)) / 2;
    let modal = Rect { x: mx, y: my, width: mw, height: mh };

    let inner = draw_modal_frame(f, modal, s.theme.surface, s.theme.primary);
    draw_modal_title(f, modal, " 📝 Commit Message ", s.theme.surface, s.theme.accent);

    let max_lines = inner.height.saturating_sub(2) as usize;
    if s.commit_cursor_row >= s.commit_modal_scroll + max_lines {
        s.commit_modal_scroll = s.commit_cursor_row - max_lines + 1;
    } else if s.commit_cursor_row < s.commit_modal_scroll {
        s.commit_modal_scroll = s.commit_cursor_row;
    }

    let text_area = Rect {
        x: inner.x + 2,
        y: inner.y + 1,
        width: inner.width - 4,
        height: inner.height - 2,
    };
    
    for row in text_area.y..text_area.y + text_area.height {
        f.render_widget(
            Paragraph::new(" ".repeat(text_area.width as usize))
                .style(Style::default().bg(s.theme.surface)),
            Rect { x: text_area.x, y: row, width: text_area.width, height: 1 },
        );
    }

    let visible_lines: Vec<String> = s.commit_message_lines
        .iter()
        .skip(s.commit_modal_scroll)
        .take(max_lines)
        .cloned()
        .collect();

    for (i, line) in visible_lines.iter().enumerate() {
        f.render_widget(
            Paragraph::new(line.clone())
                .style(Style::default().fg(s.theme.foreground).bg(s.theme.surface)),
            Rect {
                x: text_area.x + 1,
                y: text_area.y + i as u16,
                width: text_area.width - 2,
                height: 1,
            },
        );
    }

    let relative_row = s.commit_cursor_row.saturating_sub(s.commit_modal_scroll);
    if relative_row < max_lines {
        let cx = text_area.x + 1 + s.commit_cursor_col as u16;
        let cy = text_area.y + relative_row as u16;
        if cx < text_area.x + text_area.width - 1 {
            f.render_widget(
                Paragraph::new(" ").style(Style::default().bg(s.theme.accent)),
                Rect { x: cx, y: cy, width: 1, height: 1 },
            );
        }
    }

    let help = "Enter: New line | Ctrl+Enter or Ctrl+S: Confirm | Esc: Cancel";
    f.render_widget(
        Paragraph::new(help).alignment(Alignment::Center)
            .style(Style::default().fg(s.theme.primary).bg(s.theme.surface)),
        Rect { x: inner.x, y: inner.y + inner.height - 1, width: inner.width, height: 1 },
    );
}
