use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::Line,
    widgets::{Clear, Paragraph},
    Frame,
};

pub mod components;
pub mod panels;

use crate::ui::state::AppState;
use crate::version;

pub use components::render_diff_side_by_side;
use components::{
    calculate_layout, draw_solid_border, draw_solid_hline, GIT_HERO_ASCII, GIT_HERO_CREDIT,
    GIT_HERO_TAGLINE,
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

    let (outer, inner) = calculate_layout(area);
    if outer.width < 20 || outer.height < 8 {
        return; // Too small to render anything useful.
    }

    draw_solid_border(f, outer, &s.theme);
    draw_banner(f, area, &s.theme);

    // Reserve the bottom 2 rows of `inner` for the footer.
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

/// Render the ASCII art banner, tagline, version badge and credit line
/// above the main panel. Hidden on small terminals.
fn draw_banner(f: &mut Frame, area: Rect, theme: &crate::theme::Theme) {
    let show_banner = area.height >= 26 && area.width >= 75;
    if !show_banner {
        return;
    }
    let ascii_y_start = area.y + 1;

    // ASCII art (centered)
    for (i, line) in GIT_HERO_ASCII.iter().enumerate() {
        let lw = line.chars().count() as u16;
        let pad = (area.width.saturating_sub(lw)) / 2;
        f.render_widget(
            Paragraph::new(*line).style(
                Style::default()
                    .fg(theme.primary)
                    .bg(theme.background)
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

    let tag_y = ascii_y_start + GIT_HERO_ASCII.len() as u16 + 1;
    centered_line(f, area, tag_y, GIT_HERO_TAGLINE, theme.dimmed, Modifier::ITALIC);
    centered_line(f, area, tag_y + 1, &version::full(), theme.accent, Modifier::BOLD);
    centered_line(f, area, tag_y + 2, GIT_HERO_CREDIT, theme.dimmed, Modifier::ITALIC);
}

/// Helper for `draw_banner`: draw `text` centered on row `y` of `area`.
fn centered_line(
    f: &mut Frame,
    area: Rect,
    y: u16,
    text: &str,
    fg: ratatui::style::Color,
    modifier: Modifier,
) {
    let w = text.chars().count() as u16;
    let pad = area.width.saturating_sub(w) / 2;
    f.render_widget(
        Paragraph::new(text).style(
            Style::default()
                .fg(fg)
                .bg(ratatui::style::Color::Reset)
                .add_modifier(modifier),
        ),
        Rect {
            x: area.x + pad,
            y,
            width: w,
            height: 1,
        },
    );
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
        let _: fn(&mut Frame, Rect, &crate::theme::Theme) = draw_banner;
    }
}
