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

pub use components::render_diff_side_by_side;
use components::{
    calculate_layout, draw_solid_border, draw_solid_hline, GIT_HERO_ASCII, GIT_HERO_CREDIT,
    GIT_HERO_TAGLINE,
};

pub fn draw_ui(f: &mut Frame, s: &mut AppState) {
    let area = f.area();

    // Setup wizard override
    if s.setup_step > 0 {
        super::modals::draw_setup_wizard(f, s);
        return;
    }

    // 1. Full theme background
    f.render_widget(Paragraph::new("").style(Style::default().bg(s.theme.background)), area);

    // 2. Responsive Sizing & Layout (Media-query style)
    let (outer, inner) = calculate_layout(area);
    if outer.width < 20 || outer.height < 8 {
        return;
    }

    // 3. Outer border (solid blocks)
    draw_solid_border(f, outer, &s.theme);

    // 4. Inner area (already calculated as inner)
    // 5. ASCII art banner ABOVE the main panel (outside the border)
    let show_banner = area.height >= 26 && area.width >= 75;
    let ascii_h = if show_banner {
        GIT_HERO_ASCII.len() as u16 + 3 // +1 for tagline, +1 for version, +1 for credit
    } else {
        0
    };

    if ascii_h > 0 {
        // Start ASCII art with minimal top margin
        let ascii_y_start = area.y + 1;

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

        // Version badge below tagline
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
        panels::draw_no_repo_panel(f, s, body);
    } else if s.init_wizard_active {
        panels::draw_init_wizard(f, s, body);
    } else {
        panels::draw_dashboard(f, s, body);
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

        // Input background
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
            ratatui::text::Span::styled("Type a command...", Style::default().fg(s.theme.dimmed).bg(s.theme.primary))
        } else {
            ratatui::text::Span::styled(
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
            let items: Vec<ratatui::widgets::ListItem> = s.suggestions.iter().enumerate().map(|(i, sug)| {
                let sty = if i == s.active_sug {
                    Style::default().bg(s.theme.primary).fg(s.theme.background).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(s.theme.foreground).bg(s.theme.background)
                };
                let pre = if i == s.active_sug { " \u{25B6} " } else { "   " };
                ratatui::widgets::ListItem::new(format!("{}{}", pre, sug)).style(sty)
            }).collect();
            let sl = ratatui::widgets::List::new(items).block(
                ratatui::widgets::Block::default()
                    .borders(ratatui::widgets::Borders::ALL)
                    .border_type(ratatui::widgets::BorderType::Rounded)
                    .border_style(Style::default().fg(s.theme.dimmed)),
            );
            f.render_widget(sl, Rect { x: sa.x, y: sa.y.saturating_sub(1), width: sa.width, height: sa.height + 2 });
        }
    }

    // 10. Floating modals (with dimming backdrop to emphasize focus)
    let has_active_modal = s.show_theme_modal
        || s.show_help_modal
        || s.show_docs_modal
        || s.show_commit_modal
        || s.show_confirm_push
        || s.show_confirm_pull
        || s.show_credentials_modal;

    if has_active_modal {
        components::apply_dim_overlay(f, &s.theme);
    }

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

    // 11. Mini Console (overlay at bottom) – hidden when a modal is active
    if s.console_visible && !has_active_modal {
        panels::draw_console(f, area, s);
    }
}
