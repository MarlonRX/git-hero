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
use super::state::{AppState, GitCommit};

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

/// Parsea el diff y lo convierte en líneas lado-a-lado (antes | después)
/// Blend a color with background at given opacity (0.0 = bg only, 1.0 = full color)
fn soften(color: ratatui::style::Color, bg: ratatui::style::Color, opacity: f32) -> ratatui::style::Color {
    use ratatui::style::Color;
    let to_rgb = |c: Color| -> (u8, u8, u8) {
        match c {
            Color::Rgb(r, g, b) => (r, g, b),
            Color::Red => (255, 60, 60),
            Color::Green => (60, 255, 60),
            Color::Yellow => (255, 255, 0),
            Color::Blue => (0, 0, 255),
            Color::Black => (0, 0, 0),
            Color::White => (255, 255, 255),
            _ => (128, 128, 128),
        }
    };
    let (br, bg_c, bb) = to_rgb(bg);
    let (cr, cg, cb) = to_rgb(color);
    let r = (br as f32 * (1.0 - opacity) + cr as f32 * opacity) as u8;
    let g = (bg_c as f32 * (1.0 - opacity) + cg as f32 * opacity) as u8;
    let b = (bb as f32 * (1.0 - opacity) + cb as f32 * opacity) as u8;
    Color::Rgb(r, g, b)
}

/// Robust side-by-side diff renderer. Same simple pattern: split + take + map.
/// No pairing logic, no accumulation, guaranteed termination.
pub fn render_diff_side_by_side(diff: &str, width: u16, theme: &Theme) -> Vec<Line<'static>> {
    let half = (width.saturating_sub(3) / 2) as usize; // -3 for " │ " separator
    if half < 10 { return render_diff_lines(diff, theme); } // fallback for tiny widths
    
    let warn_bg = soften(theme.warning, theme.background, 0.25);
    let succ_bg = soften(theme.success, theme.background, 0.25);
    
    diff.split('\n')
        .take(200)
        .map(|line| {
            // ── Full-width headers (truncated to fit) ──
            let max_w = width as usize;
            let trunc = |s: &str| -> String { s.chars().take(max_w).collect() };
            
            if line.starts_with("diff --git") || line.starts_with("index ") || line.starts_with("commit ") {
                return Line::from(Span::styled(
                    trunc(line),
                    Style::default().fg(theme.accent).bg(theme.background).add_modifier(Modifier::BOLD)
                ));
            }
            if line.starts_with("@@") {
                return Line::from(Span::styled(
                    format!("{:<width$}", trunc(line), width = max_w),
                    Style::default().fg(theme.primary).bg(theme.surface).add_modifier(Modifier::BOLD)
                ));
            }
            if line.starts_with("Author:") || line.starts_with("Date:") {
                return Line::from(Span::styled(
                    trunc(line),
                    Style::default().fg(theme.dimmed).bg(theme.background)
                ));
            }
            if line.trim().is_empty() {
                return Line::from(Span::raw(""));
            }
            
            // ── Side-by-side columns: truncate then pad to fit ──
            let fit = |s: &str| -> String {
                let truncated: String = s.chars().take(half).collect();
                format!("{:<width$}", truncated, width = half)
            };
            
            if line.starts_with('-') && !line.starts_with("---") {
                // Removed → LEFT column only
                let text = if line.len() > 1 { &line[1..] } else { "" };
                Line::from(vec![
                    Span::styled(fit(text), Style::default().fg(theme.foreground).bg(warn_bg)),
                    Span::styled(" │ ".to_string(), Style::default().fg(theme.border).bg(theme.background)),
                    Span::styled(fit(""), Style::default().bg(theme.background)),
                ])
            } else if line.starts_with('+') && !line.starts_with("+++") {
                // Added → RIGHT column only
                let text = if line.len() > 1 { &line[1..] } else { "" };
                Line::from(vec![
                    Span::styled(fit(""), Style::default().bg(theme.background)),
                    Span::styled(" │ ".to_string(), Style::default().fg(theme.border).bg(theme.background)),
                    Span::styled(fit(text), Style::default().fg(theme.foreground).bg(succ_bg)),
                ])
            } else if line.starts_with("---") || line.starts_with("+++") {
                // File paths → both sides with accent
                // Guard against short lines to avoid panic on &line[4..]
                let text = if line.len() > 4 { &line[4..] } else { "" };
                let col = fit(text);
                Line::from(vec![
                    Span::styled(col.clone(), Style::default().fg(theme.dimmed).bg(theme.background)),
                    Span::styled(" │ ".to_string(), Style::default().fg(theme.border).bg(theme.background)),
                    Span::styled(col, Style::default().fg(theme.dimmed).bg(theme.background)),
                ])
            } else if line.starts_with(' ') {
                // Context → both columns
                let text = if line.len() > 1 { &line[1..] } else { "" };
                let col = fit(text);
                Line::from(vec![
                    Span::styled(col.clone(), Style::default().fg(theme.foreground).bg(theme.background)),
                    Span::styled(" │ ".to_string(), Style::default().fg(theme.border).bg(theme.background)),
                    Span::styled(col, Style::default().fg(theme.foreground).bg(theme.background)),
                ])
            } else {
                // Unknown → simple line
                Line::from(Span::styled(line.to_string(), Style::default().fg(theme.foreground).bg(theme.background)))
            }
        })
        .collect()
}

/// Simple line-by-line diff renderer. Reliable, fast, no hangs.
pub fn render_diff_lines(diff: &str, theme: &Theme) -> Vec<Line<'static>> {
    let warn_bg = soften(theme.warning, theme.background, 0.25);
    let succ_bg = soften(theme.success, theme.background, 0.25);
    
    // Take at most 200 lines to absolutely guarantee no hangs
    diff.split('\n')
        .take(200)
        .map(|line| {
            if line.starts_with("diff --git") || line.starts_with("index ") || line.starts_with("commit ") {
                let truncated = if line.len() > 120 { &line[..120] } else { line };
                Line::from(Span::styled(
                    truncated.to_string(),
                    Style::default().fg(theme.accent).bg(theme.background).add_modifier(Modifier::BOLD)
                ))
            } else if line.starts_with("@@") {
                Line::from(Span::styled(line.to_string(),
                    Style::default().fg(theme.primary).bg(theme.surface).add_modifier(Modifier::BOLD)))
            } else if line.starts_with('+') && !line.starts_with("+++") {
                let rest = if line.len() > 1 { line[1..].to_string() } else { String::new() };
                Line::from(vec![
                    Span::styled("+".to_string(), Style::default().fg(theme.foreground).bg(succ_bg).add_modifier(Modifier::BOLD)),
                    Span::styled(rest, Style::default().fg(theme.foreground).bg(succ_bg)),
                ])
            } else if line.starts_with('-') && !line.starts_with("---") {
                let rest = if line.len() > 1 { line[1..].to_string() } else { String::new() };
                Line::from(vec![
                    Span::styled("-".to_string(), Style::default().fg(theme.foreground).bg(warn_bg).add_modifier(Modifier::BOLD)),
                    Span::styled(rest, Style::default().fg(theme.foreground).bg(warn_bg)),
                ])
            } else if line.starts_with("Author:") || line.starts_with("Date:") || line.starts_with("---") || line.starts_with("+++") {
                Line::from(Span::styled(line.to_string(), Style::default().fg(theme.dimmed).bg(theme.background)))
            } else if line.trim().is_empty() {
                Line::from(Span::raw(""))
            } else {
                Line::from(Span::styled(line.to_string(), Style::default().fg(theme.foreground).bg(theme.background)))
            }
        })
        .collect()
}

// ═════════════════════════════════════════════════════════════════════════
//  GIT HERO ASCII ART
// ═════════════════════════════════════════════════════════════════════════

/// Compact ASCII art for the Git Hero brand — 5 lines, ~46 chars wide.
/// Uses Unicode box-drawing glyphs for a clean figlet-style look.
const GIT_HERO_ASCII: &[&str] = &[
    "  ███████ ██╗ ████████╗     ██╗  ██╗███████╗██████╗  ██████╗ ",
    " ██╔════╝ ██║ ╚══██╔══╝     ██║  ██║██╔════╝██╔══██╗██╔═══██╗",
    " ██║  ███╗██║    ██║        ███████║█████╗  ██████╔╝██║   ██║",
    " ██║   ██║██║    ██║        ██╔══██║██╔══╝  ██╔══██╗██║   ██║",
    " ╚██████╔╝██║    ██║        ██║  ██║███████╗██║  ██║╚██████╔╝",
    "  ╚═════╝ ╚═╝    ╚═╝        ╚═╝  ╚═╝╚══════╝╚═╝  ╚═╝ ╚═════╝ ",
];

const GIT_HERO_TAGLINE: &str = "Your terminal git companion";
const GIT_HERO_CREDIT: &str = "developed by abvilabs";

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

    // 2. Centered 80% layout — shifted down to leave room for ASCII banner
    let target_w = (area.width as f32 * 0.80) as u16;
    let target_h = (area.height as f32 * 0.78) as u16; // slightly shorter to accommodate banner

    // Reserve space for ASCII banner + tagline + version + credit + gap
    let banner_reserve: u16 = if area.width >= 50 && area.height >= 22 {
        GIT_HERO_ASCII.len() as u16 + 5 // ascii(5) + tagline(1) + version(1) + credit(1) + gap(1)
    } else {
        0
    };

    let outer = Rect {
        x: area.x + (area.width.saturating_sub(target_w)) / 2,
        y: area.y + banner_reserve + (area.height.saturating_sub(banner_reserve + target_h)) / 2,
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

    // 5. ASCII art banner ABOVE the main panel (outside the border)
    let ascii_h = if area.width >= 50 && area.height >= 22 {
        GIT_HERO_ASCII.len() as u16 + 3 // +1 for tagline, +1 for version, +1 for credit
    } else {
        0
    };

    if ascii_h > 0 {
        // Start ASCII art with generous top margin for breathing room
        let ascii_y_start = area.y + 3;

        // Render each line of the ASCII art, centered horizontally on full screen
        for (i, line) in GIT_HERO_ASCII.iter().enumerate() {
            let lw = line.chars().count() as u16;
            let pad = (area.width.saturating_sub(lw)) / 2;
            f.render_widget(
                Paragraph::new(*line)
                    .style(
                        Style::default()
                            .fg(s.theme.primary)
                            .bg(s.theme.background)
                            .add_modifier(Modifier::BOLD),
                    ),
                Rect {
                    x: area.x + pad,
                    y: ascii_y_start + i as u16,
                    width: lw,
                    height: 1,
                },
            );
        }

        // Tagline below the ASCII art (with a blank line gap)
        let tag_y = ascii_y_start + GIT_HERO_ASCII.len() as u16 + 1;
        let tag_w = GIT_HERO_TAGLINE.chars().count() as u16;
        let tag_pad = (area.width.saturating_sub(tag_w)) / 2;
        f.render_widget(
            Paragraph::new(GIT_HERO_TAGLINE)
                .style(
                    Style::default()
                        .fg(s.theme.dimmed)
                        .bg(s.theme.background)
                        .add_modifier(Modifier::ITALIC),
                ),
            Rect {
                x: area.x + tag_pad,
                y: tag_y,
                width: tag_w,
                height: 1,
            },
        );

        // Version badge below tagline — pulled from Cargo.toml + git at build time
        let ver_text = crate::version::full();
        let ver_w = ver_text.chars().count() as u16;
        let ver_pad = (area.width.saturating_sub(ver_w)) / 2;
        let ver_y = tag_y + 1;
        f.render_widget(
            Paragraph::new(ver_text)
                .style(
                    Style::default()
                        .fg(s.theme.accent)
                        .bg(s.theme.background)
                        .add_modifier(Modifier::BOLD),
                ),
            Rect {
                x: area.x + ver_pad,
                y: ver_y,
                width: ver_w,
                height: 1,
            },
        );

        // Credit line below the version badge
        let cred_w = GIT_HERO_CREDIT.chars().count() as u16;
        let cred_pad = (area.width.saturating_sub(cred_w)) / 2;
        f.render_widget(
            Paragraph::new(GIT_HERO_CREDIT)
                .style(
                    Style::default()
                        .fg(s.theme.dimmed)
                        .bg(s.theme.background)
                        .add_modifier(Modifier::ITALIC),
                ),
            Rect {
                x: area.x + cred_pad,
                y: ver_y + 1,
                width: cred_w,
                height: 1,
            },
        );
    }

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
    } else if s.show_commit_modal {
        super::modals::draw_commit_modal(f, s);
    } else if s.show_confirm_push {
        super::modals::draw_confirm_push_modal(f, s);
    } else if s.show_confirm_pull {
        super::modals::draw_confirm_pull_modal(f, s);
    } else if s.show_credentials_modal {
        super::modals::draw_credentials_modal(f, s);
    }

    // 11. Mini Console (overlay at bottom)
    if s.console_visible {
        draw_console(f, area, s);
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
        // Count file types for indicators
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
        
        // Build indicator line
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
        
        // Render files with enhanced indicators
        let items: Vec<ListItem> = s.files.iter().enumerate().map(|(i, f)| {
            let fg = if f.staged { s.theme.success } else if f.status == "??" { s.theme.dimmed } else { s.theme.warning };
            let cb = if f.staged { "[\u{2713}]" } else { "[ ]" };
            
            // Enhanced icon based on status
            let (icon, icon_color) = match f.status.as_str() {
                "A" => (s.get_icon_str("add"), s.theme.success),
                "D" => (s.get_icon_str("del"), s.theme.warning),
                "??" => (s.get_icon_str("untracked"), s.theme.dimmed),
                "M" | "MM" => (s.get_icon_str("mod"), s.theme.accent),
                _ => (s.get_icon_str("mod"), fg),
            };
            
            let pre = if i == s.selected_file_idx && s.focus_pane == "files" { "\u{25B6} " } else { "  " };

            let style = if i == s.selected_file_idx && s.focus_pane == "files" {
                Style::default().bg(s.theme.highlight).fg(s.theme.on_highlight).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(fg).bg(s.theme.background)
            };
            
            // Create multi-colored line for better readability
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
        
        // Add indicator text as first item if we have space
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
        Line::from(Span::styled(" Enter Detail y Copy diff", Style::default().fg(s.theme.dimmed).bg(s.theme.background))),
        Line::from(Span::styled(" Scroll: PgUp/PgDn or Mouse  ? Help q Quit", Style::default().fg(s.theme.dimmed).bg(s.theme.background))),
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
        diff_title = format!(" {} DIFF: {} {} ", 
            if s.focus_pane == "diff" { "▼" } else { "" },
            s.files[s.selected_file_idx].path,
            if s.focus_pane == "diff" { "▼" } else { "" },
        );
    }
    let diff_title_style = if s.focus_pane == "diff" {
        Style::default().fg(s.theme.primary).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(s.theme.accent).add_modifier(Modifier::BOLD)
    };
    let diff_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Plain)
        .border_style(Style::default().fg(s.theme.border))
        .title(diff_title)
        .title_style(diff_title_style);
    f.render_widget(diff_block.clone(), diff_area);
    let diff_inner = diff_block.inner(diff_area);

    // Side-by-side diff rendering - robust, no hangs. Scrolled via diff_scroll_offset
    let dlines = s.get_cached_diff_lines(diff_inner.width);
    let visible: Vec<Line> = dlines.iter()
        .skip(s.diff_scroll_offset)
        .take(diff_inner.height as usize)
        .cloned()
        .collect();
    f.render_widget(Paragraph::new(visible).style(Style::default().bg(s.theme.background)), diff_inner);

    // Commits block - recuadro fijo con borde
    let commit_area = right_chunks[1];
    let commits_title = if s.show_commit_detail {
        " COMMIT DETAILS (Enter to close) ".to_string()
    } else {
        " COMMITS (Enter for details) ".to_string()
    };
    let commits_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Plain)
        .border_style(Style::default().fg(s.theme.border))
        .title(commits_title)
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
    } else if s.show_commit_detail && !s.commit_detail_diff.is_empty() {
        // Show detailed commit view with diff (with scroll)
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
        // Normal commit list view (with scroll)
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

// ═════════════════════════════════════════════════════════════════════════
//  MINI CONSOLE — Shows git command output
// ═════════════════════════════════════════════════════════════════════════

fn draw_console(f: &mut Frame, area: Rect, s: &mut AppState) {
    // Console takes bottom 35% of screen, max 20 lines
    let ch = (area.height * 35 / 100).min(20).max(6);
    let cw = (area.width * 80 / 100).min(100);
    let cx = (area.width.saturating_sub(cw)) / 2;
    let cy = area.height.saturating_sub(ch + 1);
    let car = Rect { x: cx, y: cy, width: cw, height: ch + 1 };

    // Fill entire console area with solid surface color (not transparent)
    let fill_bg = s.theme.surface;
    for row in car.y..car.y + car.height {
        f.render_widget(
            Paragraph::new(" ".repeat(car.width as usize))
                .style(Style::default().bg(fill_bg)),
            Rect { x: car.x, y: row, width: car.width, height: 1 },
        );
    }

    // Solid border (matching main UI style)
    draw_solid_border(f, car, &s.theme);

    // Title bar on top border
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

    // Content area (inside border)
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

    // Split output into lines and apply scroll
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
