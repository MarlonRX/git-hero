use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Paragraph, BorderType},
    Frame,
};
use crate::theme::Theme;

/// Replace home directory with ~ for readable display
pub fn short_path(path: &str) -> String {
    if let Some(home) = dirs::home_dir() {
        let home_str = home.to_string_lossy();
        if let Some(rest) = path.strip_prefix(home_str.as_ref()) {
            return format!("~{}", rest);
        }
    }
    path.to_string()
}

/// Dibuja un border sólido con caracteres de bloque █
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

/// Blend a color with background at given opacity (0.0 = bg only, 1.0 = full color)
pub fn soften(color: ratatui::style::Color, bg: ratatui::style::Color, opacity: f32) -> ratatui::style::Color {
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
pub fn render_diff_side_by_side(diff: &str, width: u16, theme: &Theme) -> Vec<Line<'static>> {
    let half = (width.saturating_sub(3) / 2) as usize; // -3 for " ▎ " separator
    if half < 10 { return render_diff_lines(diff, theme); } // fallback for tiny widths
    
    let warn_bg = soften(theme.warning, theme.background, 0.25);
    let succ_bg = soften(theme.success, theme.background, 0.25);
    
    diff.split('\n')
        .take(200)
        .map(|line| {
            let fit = |s: &str| -> String {
                use unicode_width::UnicodeWidthChar;
                let clean = s.replace('\t', "    ");
                let mut res = String::new();
                let mut current_width = 0;
                for c in clean.chars() {
                    let w = c.width().unwrap_or(0);
                    if current_width + w > half {
                        break;
                    }
                    res.push(c);
                    current_width += w;
                }
                if current_width < half {
                    res.push_str(&" ".repeat(half - current_width));
                }
                res
            };

            if line.starts_with("diff --git") || line.starts_with("index ") || line.starts_with("commit ") {
                return Line::from(vec![
                    Span::styled(fit(line), Style::default().fg(theme.accent).bg(theme.background).add_modifier(Modifier::BOLD)),
                    Span::styled(" ▎ ".to_string(), Style::default().fg(theme.border).bg(theme.background)),
                    Span::styled(fit(""), Style::default().bg(theme.background)),
                ]);
            }
            if line.starts_with("@@") {
                return Line::from(vec![
                    Span::styled(fit(line), Style::default().fg(theme.primary).bg(theme.surface).add_modifier(Modifier::BOLD)),
                    Span::styled(" ▎ ".to_string(), Style::default().fg(theme.border).bg(theme.surface)),
                    Span::styled(fit(""), Style::default().bg(theme.surface)),
                ]);
            }
            if line.starts_with("Author:") || line.starts_with("Date:") {
                return Line::from(vec![
                    Span::styled(fit(line), Style::default().fg(theme.dimmed).bg(theme.background)),
                    Span::styled(" ▎ ".to_string(), Style::default().fg(theme.border).bg(theme.background)),
                    Span::styled(fit(""), Style::default().bg(theme.background)),
                ]);
            }
            if line.trim().is_empty() {
                return Line::from(vec![
                    Span::styled(fit(""), Style::default().bg(theme.background)),
                    Span::styled(" ▎ ".to_string(), Style::default().fg(theme.border).bg(theme.background)),
                    Span::styled(fit(""), Style::default().bg(theme.background)),
                ]);
            }
            
            if line.starts_with('-') && !line.starts_with("---") {
                let text = if line.len() > 1 { &line[1..] } else { "" };
                Line::from(vec![
                    Span::styled(fit(text), Style::default().fg(theme.foreground).bg(warn_bg)),
                    Span::styled(" ▎ ".to_string(), Style::default().fg(theme.border).bg(theme.background)),
                    Span::styled(fit(""), Style::default().bg(theme.background)),
                ])
            } else if line.starts_with('+') && !line.starts_with("+++") {
                let text = if line.len() > 1 { &line[1..] } else { "" };
                Line::from(vec![
                    Span::styled(fit(""), Style::default().bg(theme.background)),
                    Span::styled(" ▎ ".to_string(), Style::default().fg(theme.border).bg(theme.background)),
                    Span::styled(fit(text), Style::default().fg(theme.foreground).bg(succ_bg)),
                ])
            } else if let Some(text) = line.strip_prefix("--- ") {
                let col = fit(text);
                Line::from(vec![
                    Span::styled(col.clone(), Style::default().fg(theme.dimmed).bg(theme.background)),
                    Span::styled(" ▎ ".to_string(), Style::default().fg(theme.border).bg(theme.background)),
                    Span::styled(col, Style::default().fg(theme.dimmed).bg(theme.background)),
                ])
            } else if let Some(text) = line.strip_prefix("+++ ") {
                let col = fit(text);
                Line::from(vec![
                    Span::styled(col.clone(), Style::default().fg(theme.dimmed).bg(theme.background)),
                    Span::styled(" ▎ ".to_string(), Style::default().fg(theme.border).bg(theme.background)),
                    Span::styled(col, Style::default().fg(theme.dimmed).bg(theme.background)),
                ])
            } else if let Some(text) = line.strip_prefix(' ') {
                    let col = fit(text);
                    Line::from(vec![
                        Span::styled(col.clone(), Style::default().fg(theme.dimmed).bg(theme.background)),
                        Span::styled(" ▎ ".to_string(), Style::default().fg(theme.border).bg(theme.background)),
                        Span::styled(col, Style::default().fg(theme.dimmed).bg(theme.background)),
                    ])
            } else {
                Line::from(vec![
                    Span::styled(fit(line), Style::default().fg(theme.foreground).bg(theme.background)),
                    Span::styled(" ▎ ".to_string(), Style::default().fg(theme.border).bg(theme.background)),
                    Span::styled(fit(""), Style::default().bg(theme.background)),
                ])
            }
        })
        .collect()
}

/// Simple line-by-line diff renderer.
pub fn render_diff_lines(diff: &str, theme: &Theme) -> Vec<Line<'static>> {
    let warn_bg = soften(theme.warning, theme.background, 0.25);
    let succ_bg = soften(theme.success, theme.background, 0.25);
    
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

pub const GIT_HERO_ASCII: &[&str] = &[
    "  ███████ ██╗ ████████╗     ██╗  ██╗███████╗██████╗  ██████╗ ",
    " ██╔════╝ ██║ ╚══██╔══╝     ██║  ██║██╔════╝██╔══██╗██╔═══██╗",
    " ██║  ███╗██║    ██║        ███████║█████╗  ██████╔╝██║   ██║",
    " ██║   ██║██║    ██║        ██╔══██║██╔══╝  ██╔══██╗██║   ██║",
    " ╚██████╔╝██║    ██║        ██║  ██║███████╗██║  ██║╚██████╔╝",
    "  ╚═════╝ ╚═╝    ╚═╝        ╚═╝  ╚═╝╚══════╝╚═╝  ╚═╝ ╚═════╝ ",
];

pub const GIT_HERO_TAGLINE: &str = "Your terminal git companion";
pub const GIT_HERO_CREDIT: &str = "developed by abvilabs";

/// Calculates the responsive layout boundaries (outer, inner) for the main application box.
pub fn calculate_layout(area: Rect) -> (Rect, Rect) {
    let show_banner = area.height >= 26 && area.width >= 75;
    let banner_reserve: u16 = if show_banner { 11 } else { 0 };

    let outer_width = if show_banner {
        (area.width as f32 * 0.90) as u16
    } else {
        (area.width as f32 * 0.96) as u16
    }.max(40);

    let outer_height = if show_banner {
        area.height.saturating_sub(banner_reserve + 1)
    } else {
        area.height.saturating_sub(1)
    }.max(10);

    // Make sure we never overflow the area height
    let outer_height = outer_height.min(area.height.saturating_sub(banner_reserve + 1).max(10));

    let outer = Rect {
        x: area.x + (area.width.saturating_sub(outer_width)) / 2,
        y: area.y + banner_reserve + (area.height.saturating_sub(banner_reserve + outer_height)) / 2,
        width: outer_width,
        height: outer_height,
    };

    let inner = Rect {
        x: outer.x + 1,
        y: outer.y + 1,
        width: outer.width.saturating_sub(2),
        height: outer.height.saturating_sub(2),
    };

    (outer, inner)
}

/// Dibuja un borde continuo y delgado en un panel usando caracteres de bloque Unicode.
/// Esto evita que los bordes se vean discontinuos en emuladores de terminal con interlineados grandes.
pub fn draw_continuous_border(
    f: &mut Frame,
    area: Rect,
    title: &str,
    title_style: Style,
    color: ratatui::style::Color,
    bg: ratatui::style::Color,
    border_type: BorderType,
) {
    let s_vert = Style::default().fg(color).bg(bg);
    let s_horiz = Style::default().fg(color).bg(bg);

    // Get corner characters based on BorderType
    let (tl, tr, bl, br) = match border_type {
        BorderType::Rounded => ("╭", "╮", "╰", "╯"),
        _ => ("┌", "┐", "└", "┘"),
    };

    // Top border (using Horizontal line ─ U+2500, excluding corners)
    let top_w = area.width.saturating_sub(2);
    if top_w > 0 {
        let top_str = "─".repeat(top_w as usize);
        f.render_widget(Paragraph::new(top_str).style(s_horiz), Rect { x: area.x + 1, y: area.y, width: top_w, height: 1 });
    }

    // Bottom border (using Horizontal line ─ U+2500, excluding corners)
    let bottom_w = area.width.saturating_sub(2);
    if bottom_w > 0 {
        let bottom_str = "─".repeat(bottom_w as usize);
        f.render_widget(Paragraph::new(bottom_str).style(s_horiz), Rect { x: area.x + 1, y: area.y + area.height - 1, width: bottom_w, height: 1 });
    }

    // Left & Right borders (using ASCII vertical bar |)
    for row in 1..area.height.saturating_sub(1) {
        let y = area.y + row;
        f.render_widget(Paragraph::new("|").style(s_vert), Rect { x: area.x, y, width: 1, height: 1 });
        f.render_widget(Paragraph::new("|").style(s_vert), Rect { x: area.x + area.width - 1, y, width: 1, height: 1 });
    }

    // Corners
    if area.width > 0 && area.height > 0 {
        f.render_widget(Paragraph::new(tl).style(s_vert), Rect { x: area.x, y: area.y, width: 1, height: 1 });
        f.render_widget(Paragraph::new(tr).style(s_vert), Rect { x: area.x + area.width - 1, y: area.y, width: 1, height: 1 });
        f.render_widget(Paragraph::new(bl).style(s_vert), Rect { x: area.x, y: area.y + area.height - 1, width: 1, height: 1 });
        f.render_widget(Paragraph::new(br).style(s_vert), Rect { x: area.x + area.width - 1, y: area.y + area.height - 1, width: 1, height: 1 });
    }

    // Title overlay (centered or left-aligned on the top border)
    if !title.is_empty() {
        let tw = title.chars().count() as u16;
        if tw < area.width.saturating_sub(4) {
            let tx = area.x + 2; // pad 2 cells from left
            f.render_widget(
                Paragraph::new(title).style(title_style),
                Rect { x: tx, y: area.y, width: tw, height: 1 }
            );
        }
    }
}

/// Applies a dimming overlay across the entire frame buffer to emphasize focus on active modals.
pub fn apply_dim_overlay(f: &mut Frame, theme: &Theme) {
    let area = f.area();
    let buffer = f.buffer_mut();
    for y in area.y..area.y + area.height {
        for x in area.x..area.x + area.width {
            if let Some(cell) = buffer.cell_mut((x, y)) {
                cell.set_style(
                    ratatui::style::Style::default()
                        .fg(theme.dimmed)
                        .bg(theme.background)
                        .add_modifier(ratatui::style::Modifier::DIM),
                );
            }
        }
    }
}
