use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::Line,
    widgets::{BorderType, Clear, Paragraph},
};

pub mod components;
pub mod panels;

use crate::i18n::translate;
use crate::ui::state::AppState;
use crate::version;

pub use components::render_diff_side_by_side;
use components::{
    GIT_HERO_ASCII, calculate_layout_scaled, draw_continuous_border, draw_solid_border,
    draw_solid_hline,
};

/// Header rows occupied by the logo/status block inside the inner frame.
pub const HEADER_H: u16 = 9;
/// Footer rows (separator + status line + persistent keybind strip).
pub const FOOTER_H: u16 = 3;

/// The body rectangle between header and footer for a given inner frame.
/// Shared by `draw_ui` and the mouse handlers so hit-testing geometry can
/// never drift from what is actually painted.
pub fn body_area(inner: Rect) -> Rect {
    Rect {
        x: inner.x,
        y: inner.y + HEADER_H,
        width: inner.width,
        height: inner.height.saturating_sub(HEADER_H + FOOTER_H),
    }
}

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
    let header_h: u16 = HEADER_H;
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
        Style::default()
            .fg(s.theme.primary)
            .add_modifier(Modifier::BOLD),
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
    draw_status_line(
        f,
        inner.x + 1,
        inner.y + 7,
        inner.width.saturating_sub(2),
        s,
    );

    let footer_h: u16 = FOOTER_H;
    let body = body_area(inner);
    let footer = Rect {
        x: inner.x,
        y: body.y + body.height,
        width: inner.width,
        height: footer_h,
    };

    // Route the body to the right panel.
    if !s.is_git_repo && !s.init_wizard_active {
        if s.show_repo_overview {
            // Outside a repo: browser gets its own left column so the
            // no-repo panel (init / /cd suggestions) stays visible.
            let area = panels::browser_area(body, s.repo_view.len(), true);
            let right = Rect {
                x: body.x + area.width,
                y: body.y,
                width: body.width - area.width,
                height: body.height,
            };
            panels::draw_repo_browser(f, area, s);
            panels::draw_no_repo_panel(f, s, right);
        } else {
            panels::draw_no_repo_panel(f, s, body);
        }
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
            Rect {
                x: area.x + 2,
                y: area.y + i as u16,
                width,
                height: 1,
            },
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

    // Version badge. `full()` == `short()` on release builds but adds
    // `-dev`/dirty/hash markers locally, so screenshots from dev runs are
    // honestly distinguishable from shipped binaries.
    left_spans.push(ratatui::text::Span::styled(
        format!(" {} ", version::full()),
        Style::default()
            .fg(s.theme.background)
            .bg(s.theme.accent)
            .add_modifier(Modifier::BOLD),
    ));
    left_spans.push(ratatui::text::Span::raw("  "));

    // Branch icon + name
    left_spans.push(ratatui::text::Span::styled(
        format!("{} ", s.get_icon_str("branch")),
        Style::default().fg(s.theme.primary),
    ));
    left_spans.push(ratatui::text::Span::styled(
        s.branch.clone(),
        Style::default()
            .fg(s.theme.primary)
            .add_modifier(Modifier::BOLD),
    ));

    // Behind / ahead badges
    if s.behind > 0 {
        left_spans.push(ratatui::text::Span::raw("  "));
        left_spans.push(ratatui::text::Span::styled(
            format!(" ↓{} ", s.behind),
            Style::default()
                .fg(s.theme.background)
                .bg(s.theme.warning)
                .add_modifier(Modifier::BOLD),
        ));
    }
    if s.ahead > 0 {
        left_spans.push(ratatui::text::Span::raw("  "));
        left_spans.push(ratatui::text::Span::styled(
            format!(" ↑{} ", s.ahead),
            Style::default()
                .fg(s.theme.background)
                .bg(s.theme.success)
                .add_modifier(Modifier::BOLD),
        ));
    }

    let left_line = ratatui::text::Line::from(left_spans);
    f.render_widget(
        Paragraph::new(left_line),
        Rect {
            x,
            y,
            width,
            height: 1,
        },
    );

    // Right side: where the user is — folder/repo NAME as a bold chip on a
    // surface background (unmistakable at a glance), then the full path
    // dimmed behind it.
    let base_name = std::path::Path::new(s.cwd.as_str())
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| s.cwd.clone());
    let dir_spans = vec![
        ratatui::text::Span::styled(
            format!(" {} {} ", s.get_icon_str("dir"), base_name),
            Style::default()
                .fg(s.theme.primary)
                .bg(s.theme.surface)
                .add_modifier(Modifier::BOLD),
        ),
        ratatui::text::Span::styled(
            format!(" {dir}"),
            Style::default()
                .fg(s.theme.dimmed)
                .add_modifier(Modifier::ITALIC),
        ),
    ];
    let dir_line = ratatui::text::Line::from(dir_spans);
    let dir_width: u16 = dir_line.width() as u16;
    if dir_width + 4 < width {
        f.render_widget(
            Paragraph::new(dir_line),
            Rect {
                x: x + width - dir_width,
                y,
                width: dir_width,
                height: 1,
            },
        );
    }
}

/// Footer: separator, status message, and the persistent keybind strip.
/// The strip shows `[key] action` chips for the current context (repo /
/// browser / no-repo), degrading to a `…` as the terminal narrows, so the
/// letter commands are always in front of the user.
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
            width: footer.width.saturating_sub(2),
            height: 1,
        },
    );

    // Row 3: the keybind strip.
    let legend_key = if s.show_input {
        // While typing a command the strip itself changes shape: what
        // matters now is Tab/Enter/Esc, not the letter shortcuts.
        "legend_typing"
    } else if s.is_git_repo {
        "legend_repo"
    } else if s.show_repo_overview {
        "legend_browser"
    } else {
        "legend_norepo"
    };
    let raw = translate(&s.language, legend_key);
    let parts = parse_legend(&raw);
    let line = legend_line(&parts, footer.width.saturating_sub(2), &s.theme);
    f.render_widget(
        Paragraph::new(line).style(Style::default().bg(s.theme.background)),
        Rect {
            x: footer.x + 1,
            y: footer.y + 2,
            width: footer.width.saturating_sub(2),
            height: 1,
        },
    );
}

/// Parse the `k:label|k:label|…` legend payloads from the i18n dict.
pub(crate) fn parse_legend(raw: &str) -> Vec<(&str, &str)> {
    raw.split('|')
        .filter_map(|part| {
            let mut kv = part.splitn(2, ':');
            let key = kv.next()?.trim();
            let label = kv.next()?.trim();
            (!key.is_empty() && !label.is_empty()).then_some((key, label))
        })
        .collect()
}

/// Build a single styled line of `[k] label` chips that fits `width`,
/// dropping the tail (with a `…` marker) when it cannot.
pub(crate) fn legend_line(
    parts: &[(&str, &str)],
    width: u16,
    theme: &crate::theme::Theme,
) -> ratatui::text::Line<'static> {
    use ratatui::text::Span;
    let key_style = Style::default()
        .fg(theme.background)
        .bg(theme.accent)
        .add_modifier(Modifier::BOLD);
    let label_style = Style::default().fg(theme.dimmed).bg(theme.background);
    let mut spans: Vec<Span<'static>> = Vec::new();
    let mut used: u16 = 0;
    let mut truncated = false;
    for (key, label) in parts {
        // " [k] label" costs: 1 sep + [ + key + ] + 1 space + label.
        let cost = 1 + (key.chars().count() as u16) + 2 + 1 + (label.chars().count() as u16);
        if used + cost > width {
            truncated = true;
            break;
        }
        spans.push(Span::styled(format!(" [{key}]"), key_style));
        spans.push(Span::styled(format!(" {label}"), label_style));
        used += cost;
    }
    if truncated {
        let dots = 1;
        if used + dots <= width && !spans.is_empty() {
            spans.pop(); // last label makes room for the ellipsis
            spans.push(Span::styled(" \u{2026}", label_style));
        }
    }
    ratatui::text::Line::from(spans)
}

/// Input bar that overlays the bottom border. Hidden while a modal is
/// open so the user can't accidentally type into the input while
/// answering a confirmation.
fn draw_command_bar(f: &mut Frame, outer: Rect, s: &AppState) {
    if !s.show_input || s.show_theme_modal || s.show_help_modal || s.show_docs_modal {
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
            translate(&s.language, "input_placeholder").into_owned(),
            Style::default().fg(s.theme.dimmed).bg(s.theme.primary),
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
            let pre = if i == s.active_sug {
                " \u{25B6} "
            } else {
                "   "
            };
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

    #[test]
    fn parse_legend_splits_key_value_pairs() {
        assert_eq!(
            parse_legend("c:commit|q:quit"),
            vec![("c", "commit"), ("q", "quit")]
        );
    }

    #[test]
    fn parse_legend_skips_malformed_parts() {
        let parts = parse_legend("a:stage|novalue|:emptykey||b:pop");
        assert_eq!(parts, vec![("a", "stage"), ("b", "pop")]);
    }

    #[test]
    fn all_legend_keys_parse_to_pairs_in_both_languages() {
        for key in [
            "legend_repo",
            "legend_norepo",
            "legend_browser",
            "legend_typing",
        ] {
            for lang in ["en", "es"] {
                let raw = crate::i18n::translate(lang, key);
                let parts = parse_legend(&raw);
                assert!(
                    parts.len() >= 3,
                    "legend {key} ({lang}) parsed to too few parts: {raw:?}"
                );
            }
        }
    }

    #[test]
    fn legend_line_fits_width_and_marks_truncation() {
        let theme = crate::theme::get_theme_by_name("Nord");
        let parts = parse_legend("a:apple|b:banana|c:cherry");
        // Wide enough for all three.
        let wide = legend_line(&parts, 80, &theme);
        let wide_text: String = wide.spans.iter().map(|s| s.content.as_ref()).collect();
        assert!(wide_text.contains("apple") && wide_text.contains("cherry"));
        // Room for one chip + ellipsis on the middle ones.
        let narrow = legend_line(&parts, 20, &theme);
        let last = narrow.spans.last().unwrap().content.clone();
        assert!(
            last.contains('\u{2026}'),
            "expected ellipsis, got {narrow:?}"
        );
        // Zero width produces no spans and must not panic.
        let none = legend_line(&parts, 0, &theme);
        assert!(none.spans.is_empty());
    }
}
