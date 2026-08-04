use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::Line,
    widgets::{BorderType, Clear, Paragraph},
    Frame,
};

pub mod components;
pub mod panels;

use crate::ui::state::AppState;
use crate::version;

pub use components::render_diff_side_by_side;
use components::{
    calculate_layout_scaled, draw_continuous_border, draw_solid_border, draw_solid_hline,
    GIT_HERO_ASCII,
};

/// Top-level entry point. The setup wizard short-circuits everything
/// else; otherwise we compose `draw_background`, `draw_banner`, the
/// main panel routing, the footer, the command bar, the active modal
/// and the mini-console — in that order so later layers paint on top.
pub fn draw_ui(f: &mut Frame, s: &mut AppState) {
    let area = f.area();

    // Setup wizard is a full-screen overlay that preempts everything.
    if s.setup_step > 0 {
        super::modals::draw_setup_wizard(f, s);
        return;
    }

    draw_background(f, area, &s.theme);

    let (outer, inner) = calculate_layout_scaled(area);
    if outer.width < 20 || outer.height < 8 {
        return; // Too small to render anything useful.
    }

    draw_solid_border(f, outer, &s.theme);

    // Header: bordered container with logo + status (9 rows: 1 top + 6 logo + 1 status + 1 bottom)
    let header_h: u16 = 9;
    let header_area = Rect {
        x: inner.x,
        y: inner.y,
        width: inner.width,
        height: header_h,
    };
    draw_continuous_border(
        f,
        header_area,
        "",
        Style::default().fg(s.theme.primary).add_modifier(Modifier::BOLD),
        s.theme.border,
        s.theme.background,
        BorderType::Plain,
    );
    // Logo area is inside the bordered container (skip top border row)
    let logo_area = Rect {
        x: inner.x + 1,
        y: inner.y + 1,
        width: inner.width.saturating_sub(2),
        height: 6,
    };
    draw_banner(f, logo_area, s);
    // Status line at the bottom of the bordered container (before bottom border)
    draw_status_line(f, inner.x + 1, inner.y + 7, inner.width.saturating_sub(2), s);

    let footer_h: u16 = 2;
    let body = Rect {
        x: inner.x,
        y: inner.y + header_h,
        width: inner.width,
        height: inner.height.saturating_sub(header_h + footer_h),
    };
    let footer = Rect {
        x: inner.x,
        y: body.y + body.height,
        width: inner.width,
        height: footer_h,
    };

    // Route the body to the right panel.
    if !s.is_git_repo && !s.init_wizard_active {
        panels::draw_no_repo_panel(f, s, body);
    } else if s.init_wizard_active {
        panels::draw_init_wizard(f, s, body);
    } else {
        panels::draw_dashboard(f, s, body);
    }

    draw_footer(f, footer, s);
    draw_command_bar(f, outer, s);
    draw_active_modal(f, s);
    if s.console_visible && !s.has_active_modal() {
        panels::draw_console(f, area, s);
    }
}

// ── Section drawers ───────────────────────────────────────────────

/// Fill the entire frame with the theme background. Always the first
/// draw so subsequent layers paint on top of a known color.
fn draw_background(f: &mut Frame, area: Rect, theme: &crate::theme::Theme) {
    f.render_widget(
        Paragraph::new("").style(Style::default().bg(theme.background)),
        area,
    );
}

/// Render the ASCII typography inside the panel header.
fn draw_banner(f: &mut Frame, area: Rect, s: &AppState) {
    if area.width < 20 || area.height < 6 {
        return;
    }
    // Rows 0-5: ASCII logo
    for (i, line) in GIT_HERO_ASCII.iter().enumerate() {
        let width = line.chars().count() as u16;
        if width + 3 > area.width {
            break;
        }
        f.render_widget(
            Paragraph::new(*line).style(
                Style::default()
                    .fg(s.theme.primary)
                    .bg(s.theme.background)
                    .add_modifier(Modifier::BOLD),
            ),
            Rect { x: area.x + 2, y: area.y + i as u16, width, height: 1 },
        );
    }
}

/// Render the status line (version, branch, behind/ahead, path) below the logo.
fn draw_status_line(f: &mut Frame, x: u16, y: u16, width: u16, s: &AppState) {
    let mut dir = s.cwd.clone();
    if let Some(home) = dirs::home_dir() {
        let home_str = home.to_string_lossy().to_string();
        if dir.starts_with(&home_str) {
            dir = dir.replacen(&home_str, "~", 1);
        }
    }

    // Build left side spans: version · branch icon+name · behind/ahead
    let mut left_spans: Vec<ratatui::text::Span> = Vec::new();

    // Version badge
    left_spans.push(ratatui::text::Span::styled(
        format!(" {} ", version::short()),
        Style::default().fg(s.theme.background).bg(s.theme.accent).add_modifier(Modifier::BOLD),
    ));
    left_spans.push(ratatui::text::Span::raw("  "));

    // Branch icon + name
    left_spans.push(ratatui::text::Span::styled(
        format!("{} ", s.get_icon_str("branch")),
        Style::default().fg(s.theme.primary),
    ));
    left_spans.push(ratatui::text::Span::styled(
        s.branch.clone(),
        Style::default().fg(s.theme.primary).add_modifier(Modifier::BOLD),
    ));

    // Behind / ahead badges
    if s.behind > 0 {
        left_spans.push(ratatui::text::Span::raw("  "));
        left_spans.push(ratatui::text::Span::styled(
            format!(" ↓{} ", s.behind),
            Style::default().fg(s.theme.background).bg(s.theme.warning).add_modifier(Modifier::BOLD),
        ));
    }
    if s.ahead > 0 {
        left_spans.push(ratatui::text::Span::raw("  "));
        left_spans.push(ratatui::text::Span::styled(
            format!(" ↑{} ", s.ahead),
            Style::default().fg(s.theme.background).bg(s.theme.success).add_modifier(Modifier::BOLD),
        ));
    }

    let left_line = ratatui::text::Line::from(left_spans);
    f.render_widget(
        Paragraph::new(left_line),
        Rect { x, y, width, height: 1 },
    );

    // Right side: directory path with icon
    let dir_spans = vec![
        ratatui::text::Span::styled(
            format!("{} ", s.get_icon_str("dir")),
            Style::default().fg(s.theme.accent),
        ),
        ratatui::text::Span::styled(
            dir,
            Style::default().fg(s.theme.foreground).add_modifier(Modifier::ITALIC),
        ),
    ];
    let dir_line = ratatui::text::Line::from(dir_spans);
    let dir_width: u16 = dir_line.width() as u16;
    if dir_width + 4 < width {
        f.render_widget(
            Paragraph::new(dir_line),
            Rect { x: x + width - dir_width, y, width: dir_width, height: 1 },
        );
    }
}

/// Bottom 2-row footer: thin separator line, status message on the
/// left, keybind legend on the right.
fn draw_footer(f: &mut Frame, footer: Rect, s: &AppState) {
    draw_solid_hline(f, footer.x, footer.y, footer.width, s.theme.border);

    let status_icon = if s.fetching {
        format!(" {}", s.get_icon_str("fetch"))
    } else {
        String::new()
    };
    let status_str = format!(" {} {}", status_icon, s.status_message);
    let status_style = if s.fetching {
        Style::default().fg(s.theme.warning).bg(s.theme.background)
    } else {
        Style::default()
            .fg(s.theme.success)
            .bg(s.theme.background)
            .add_modifier(Modifier::BOLD)
    };
    f.render_widget(
        Paragraph::new(status_str).style(status_style),
        Rect {
            x: footer.x + 1,
            y: footer.y + 1,
            width: footer.width / 2,
            height: 1,
        },
    );

    let legend = if s.language == "es" {
        "? Ayuda | q Salir"
    } else {
        "? Help | q Quit"
    };
    let llen = legend.chars().count() as u16;
    f.render_widget(
        Paragraph::new(legend)
            .style(Style::default().fg(s.theme.dimmed).bg(s.theme.background)),
        Rect {
            x: footer.x + footer.width.saturating_sub(llen + 1),
            y: footer.y + 1,
            width: llen,
            height: 1,
        },
    );
}

/// Input bar that overlays the bottom border. Hidden while a modal is
/// open so the user can't accidentally type into the input while
/// answering a confirmation.
fn draw_command_bar(f: &mut Frame, outer: Rect, s: &AppState) {
    if !s.show_input
        || s.show_theme_modal
        || s.show_help_modal
        || s.show_docs_modal
    {
        return;
    }
    let iy = outer.y + outer.height - 1;
    let ia = Rect {
        x: outer.x + 1,
        y: iy,
        width: outer.width.saturating_sub(2),
        height: 1,
    };

    // Background fill
    f.render_widget(
        Paragraph::new(" ".repeat(ia.width as usize)).style(Style::default().bg(s.theme.primary)),
        ia,
    );
    // Prompt arrow
    f.render_widget(
        Paragraph::new(" \u{276F} ").style(
            Style::default()
                .fg(s.theme.accent)
                .bg(s.theme.primary)
                .add_modifier(Modifier::BOLD),
        ),
        Rect {
            x: ia.x,
            y: iy,
            width: 3,
            height: 1,
        },
    );
    // Text
    let display = if s.input_value.is_empty() {
        ratatui::text::Span::styled(
            "Type a command...",
            Style::default()
                .fg(s.theme.dimmed)
                .bg(s.theme.primary),
        )
    } else {
        ratatui::text::Span::styled(
            &s.input_value,
            Style::default()
                .fg(s.theme.background)
                .bg(s.theme.primary)
                .add_modifier(Modifier::BOLD),
        )
    };
    let ta = Rect {
        x: ia.x + 3,
        y: iy,
        width: ia.width.saturating_sub(4),
        height: 1,
    };
    f.render_widget(Paragraph::new(Line::from(vec![display])), ta);
    // Cursor
    let cx = ta.x + s.input_cursor_pos as u16;
    if cx < ta.x + ta.width {
        f.render_widget(
            Paragraph::new(" ").style(Style::default().bg(s.theme.accent)),
            Rect {
                x: cx,
                y: iy,
                width: 1,
                height: 1,
            },
        );
    }

    // Suggestion list
    if s.suggestions.is_empty() {
        return;
    }
    let sh = s.suggestions.len() as u16;
    let sa = Rect {
        x: outer.x + 1,
        y: iy.saturating_sub(sh + 1),
        width: 42,
        height: sh,
    };
    f.render_widget(Clear, sa);
    let items: Vec<ratatui::widgets::ListItem> = s
        .suggestions
        .iter()
        .enumerate()
        .map(|(i, sug)| {
            let sty = if i == s.active_sug {
                Style::default()
                    .bg(s.theme.primary)
                    .fg(s.theme.background)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
                    .fg(s.theme.foreground)
                    .bg(s.theme.background)
            };
            let pre = if i == s.active_sug { " \u{25B6} " } else { "   " };
            ratatui::widgets::ListItem::new(format!("{}{}", pre, sug)).style(sty)
        })
        .collect();
    let sl = ratatui::widgets::List::new(items).block(
        ratatui::widgets::Block::default()
            .borders(ratatui::widgets::Borders::ALL)
            .border_type(ratatui::widgets::BorderType::Rounded)
            .border_style(Style::default().fg(s.theme.dimmed)),
    );
    f.render_widget(
        sl,
        Rect {
            x: sa.x,
            y: sa.y.saturating_sub(1),
            width: sa.width,
            height: sa.height + 2,
        },
    );
}

/// Paint a dimming overlay (so the modal stands out) and the modal
/// itself. The if-else chain is intentional — modals are exclusive:
/// showing two at once is a bug, not a feature, so we want the first
/// match to win.
fn draw_active_modal(f: &mut Frame, s: &mut AppState) {
    if !s.has_active_modal() {
        return;
    }
    components::apply_dim_overlay(f, &s.theme);

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
    } else if s.show_confirm_remove {
        super::modals::draw_confirm_remove_modal(f, s);
    } else if s.show_credentials_modal {
        super::modals::draw_credentials_modal(f, s);
    } else if s.show_update_modal {
        super::modals::draw_update_modal(f, s);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Compile-only smoke test: the section-drawer functions exist with
    // the expected signatures. The functions themselves need a real
    // `Frame` to test, which lives in the ratatui test harness (not
    // worth pulling in just for that).
    #[allow(dead_code)]
    fn _signature_check() {
        fn _takes_frame_and_rect(_f: &mut Frame, _r: Rect) {}
        let _: fn(&mut Frame, Rect, &crate::theme::Theme) = draw_background;
        let _: fn(&mut Frame, Rect, &crate::ui::state::AppState) = draw_banner;
        let _: fn(&mut Frame, u16, u16, u16, &crate::ui::state::AppState) = draw_status_line;
    }
}
