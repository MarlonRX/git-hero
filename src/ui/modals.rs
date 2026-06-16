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
    let bs = Style::default().bg(border).fg(border);

    // Fill entire area with background first
    for row in modal.y..modal.y + modal.height {
        f.render_widget(
            Paragraph::new(" ".repeat(modal.width as usize))
                .style(Style::default().bg(bg)),
            Rect { x: modal.x, y: row, width: modal.width, height: 1 },
        );
    }

    // Top border
    f.render_widget(
        Paragraph::new("█".repeat(modal.width as usize)).style(bs),
        Rect { x: modal.x, y: modal.y, width: modal.width, height: 1 },
    );
    // Bottom border
    f.render_widget(
        Paragraph::new("█".repeat(modal.width as usize)).style(bs),
        Rect { x: modal.x, y: modal.y + modal.height - 1, width: modal.width, height: 1 },
    );
    // Left + Right borders
    for row in (modal.y + 1)..(modal.y + modal.height - 1) {
        f.render_widget(
            Paragraph::new("█").style(bs),
            Rect { x: modal.x, y: row, width: 1, height: 1 },
        );
        f.render_widget(
            Paragraph::new("█").style(bs),
            Rect { x: modal.x + modal.width - 1, y: row, width: 1, height: 1 },
        );
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
    let tw = title.len() as u16;
    if tw >= modal.width - 2 { return; }
    let tx = modal.x + (modal.width - tw) / 2;
    f.render_widget(
        Paragraph::new(title).style(
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

    let inner = draw_modal_frame(f, modal, s.theme.background, s.theme.border);

    let (title, lines) = setup_content(s);
    draw_modal_title(f, modal, &title, s.theme.background, s.theme.primary);

    let cy = inner.y + 1;
    f.render_widget(
        Paragraph::new(lines.join("\n"))
            .style(Style::default().fg(s.theme.foreground).bg(s.theme.background)),
        Rect { x: inner.x + 1, y: cy, width: inner.width - 2, height: inner.height - 2 },
    );

    let help = translate(&s.language, "setup_help");
    f.render_widget(
        Paragraph::new(help).alignment(Alignment::Center)
            .style(Style::default().fg(s.theme.dimmed).bg(s.theme.background)),
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

    let inner = draw_modal_frame(f, modal, s.theme.background, s.theme.border);
    draw_modal_title(f, modal, " \u{1F3A8} Select Visual Theme ", s.theme.background, s.theme.accent);

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
            .style(Style::default().fg(s.theme.foreground).bg(s.theme.background)),
        Rect { x: inner.x + 1, y: cy, width: inner.width - 2, height: inner.height - 2 },
    );

    let help = translate(&s.language, "theme_help");
    f.render_widget(
        Paragraph::new(help).alignment(Alignment::Center)
            .style(Style::default().fg(s.theme.dimmed).bg(s.theme.background)),
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

    let inner = draw_modal_frame(f, modal, s.theme.background, s.theme.border);
    draw_modal_title(f, modal, " \u{2753} Keyboard Shortcuts & Commands ", s.theme.background, s.theme.accent);

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
            .style(Style::default().fg(s.theme.foreground).bg(s.theme.background)),
        Rect { x: inner.x + 1, y: cy, width: inner.width - 2, height: inner.height - 2 },
    );

    f.render_widget(
        Paragraph::new("Press any key to close.")
            .alignment(Alignment::Center)
            .style(Style::default().fg(s.theme.dimmed).bg(s.theme.background)),
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

    let inner = draw_modal_frame(f, modal, s.theme.background, s.theme.border);
    draw_modal_title(f, modal, " \u{1F4D6} Detailed Shortcut Reference ", s.theme.background, s.theme.accent);

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
            .style(Style::default().fg(s.theme.foreground).bg(s.theme.background)),
        Rect { x: inner.x + 1, y: cy, width: inner.width - 2, height: inner.height },
    );

    f.render_widget(
        Paragraph::new("Press any key to close.")
            .alignment(Alignment::Center)
            .style(Style::default().fg(s.theme.dimmed).bg(s.theme.background)),
        Rect { x: inner.x, y: inner.y + inner.height - 1, width: inner.width, height: 1 },
    );
}
