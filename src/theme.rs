use ratatui::style::Color;

#[derive(Clone, Copy)]
pub struct Theme {
    pub name: &'static str,
    pub background: Color,
    pub foreground: Color,
    pub primary: Color,
    pub border: Color,
    pub accent: Color,
    pub warning: Color,
    pub success: Color,
    pub dimmed: Color,
    /// Color más brillante para elementos seleccionados (glow/highlight)
    pub highlight: Color,
    /// Color sutil para fondos de paneles secundarios
    pub surface: Color,
    /// Color para texto sobre el highlight
    pub on_highlight: Color,
}

pub fn get_themes() -> Vec<Theme> {
    vec![
        Theme {
            name: "Catppuccin Mocha",
            background: Color::Rgb(0x1E, 0x1E, 0x2E),
            foreground: Color::Rgb(0xCD, 0xD6, 0xF4),
            primary: Color::Rgb(0x89, 0xB4, 0xFA),
            border: Color::Rgb(0x45, 0x45, 0x5A),
            accent: Color::Rgb(0xF9, 0xE2, 0xAF),
            warning: Color::Rgb(0xFA, 0xB3, 0x87),
            success: Color::Rgb(0xA6, 0xE3, 0xA1),
            dimmed: Color::Rgb(0x58, 0x5B, 0x70),
            highlight: Color::Rgb(0xC4, 0xA7, 0xE7),
            surface: Color::Rgb(0x25, 0x25, 0x35),
            on_highlight: Color::Rgb(0x1E, 0x1E, 0x2E),
        },
        Theme {
            name: "Tokyo Night",
            background: Color::Rgb(0x1A, 0x1B, 0x26),
            foreground: Color::Rgb(0xA9, 0xB1, 0xD6),
            primary: Color::Rgb(0x7A, 0xA2, 0xF7),
            border: Color::Rgb(0x36, 0x38, 0x4A),
            accent: Color::Rgb(0xE0, 0xAF, 0x68),
            warning: Color::Rgb(0xF7, 0x76, 0x8E),
            success: Color::Rgb(0x9E, 0xCE, 0x6A),
            dimmed: Color::Rgb(0x56, 0x5F, 0x89),
            highlight: Color::Rgb(0xBB, 0x9A, 0xF7),
            surface: Color::Rgb(0x20, 0x21, 0x30),
            on_highlight: Color::Rgb(0x1A, 0x1B, 0x26),
        },
        Theme {
            name: "Gruvbox Dark",
            background: Color::Rgb(0x28, 0x28, 0x28),
            foreground: Color::Rgb(0xEB, 0xDB, 0xB2),
            primary: Color::Rgb(0xFA, 0xBD, 0x2F),
            border: Color::Rgb(0x50, 0x50, 0x50),
            accent: Color::Rgb(0x8E, 0xC0, 0x7C),
            warning: Color::Rgb(0xFB, 0x49, 0x34),
            success: Color::Rgb(0xB8, 0xBB, 0x26),
            dimmed: Color::Rgb(0x66, 0x5C, 0x54),
            highlight: Color::Rgb(0xFE, 0x80, 0x19),
            surface: Color::Rgb(0x32, 0x32, 0x32),
            on_highlight: Color::Rgb(0x28, 0x28, 0x28),
        },
        Theme {
            name: "Nord",
            background: Color::Rgb(0x2E, 0x34, 0x40),
            foreground: Color::Rgb(0xD8, 0xDE, 0xE9),
            primary: Color::Rgb(0x88, 0xC0, 0xD0),
            border: Color::Rgb(0x4C, 0x56, 0x6A),
            accent: Color::Rgb(0xEB, 0xCB, 0x8B),
            warning: Color::Rgb(0xBF, 0x61, 0x6A),
            success: Color::Rgb(0xA3, 0xBE, 0x8C),
            dimmed: Color::Rgb(0x4C, 0x56, 0x6A),
            highlight: Color::Rgb(0x81, 0xA1, 0xC1),
            surface: Color::Rgb(0x38, 0x3E, 0x4A),
            on_highlight: Color::Rgb(0x2E, 0x34, 0x40),
        },
        Theme {
            name: "Rose Pine",
            background: Color::Rgb(0x19, 0x17, 0x24),
            foreground: Color::Rgb(0xE0, 0xDE, 0xF4),
            primary: Color::Rgb(0x9C, 0xCF, 0xD8),
            border: Color::Rgb(0x35, 0x33, 0x40),
            accent: Color::Rgb(0xF6, 0xC1, 0x77),
            warning: Color::Rgb(0xEB, 0x6F, 0x92),
            success: Color::Rgb(0x31, 0x74, 0x8F),
            dimmed: Color::Rgb(0x6E, 0x6A, 0x86),
            highlight: Color::Rgb(0xC4, 0xA7, 0xE7),
            surface: Color::Rgb(0x1F, 0x1D, 0x2A),
            on_highlight: Color::Rgb(0x19, 0x17, 0x24),
        },
        Theme {
            name: "Kanagawa",
            background: Color::Rgb(0x1F, 0x1F, 0x28),
            foreground: Color::Rgb(0xDC, 0xD7, 0xBA),
            primary: Color::Rgb(0x7E, 0x9C, 0xD8),
            border: Color::Rgb(0x40, 0x40, 0x50),
            accent: Color::Rgb(0xFF, 0x9E, 0x3B),
            warning: Color::Rgb(0xC3, 0x40, 0x43),
            success: Color::Rgb(0x76, 0x94, 0x6A),
            dimmed: Color::Rgb(0x54, 0x54, 0x6D),
            highlight: Color::Rgb(0xFF, 0x9E, 0x3B),
            surface: Color::Rgb(0x25, 0x25, 0x2E),
            on_highlight: Color::Rgb(0x1F, 0x1F, 0x28),
        },
        Theme {
            name: "Everforest",
            background: Color::Rgb(0x2D, 0x35, 0x3B),
            foreground: Color::Rgb(0xD3, 0xC6, 0xAA),
            primary: Color::Rgb(0xA7, 0xC0, 0x80),
            border: Color::Rgb(0x50, 0x58, 0x5E),
            accent: Color::Rgb(0x7F, 0xBB, 0xB3),
            warning: Color::Rgb(0xE6, 0x7E, 0x80),
            success: Color::Rgb(0x83, 0xC0, 0x92),
            dimmed: Color::Rgb(0x5C, 0x6A, 0x72),
            highlight: Color::Rgb(0xDB, 0xBC, 0x7F),
            surface: Color::Rgb(0x33, 0x3C, 0x43),
            on_highlight: Color::Rgb(0x2D, 0x35, 0x3B),
        },
        Theme {
            name: "Dracula",
            background: Color::Rgb(0x28, 0x2A, 0x36),
            foreground: Color::Rgb(0xF8, 0xF8, 0xF2),
            primary: Color::Rgb(0x8B, 0xE9, 0xFD),
            border: Color::Rgb(0x44, 0x46, 0x52),
            accent: Color::Rgb(0xFF, 0x79, 0xC6),
            warning: Color::Rgb(0xFF, 0x55, 0x55),
            success: Color::Rgb(0x50, 0xFA, 0x7B),
            dimmed: Color::Rgb(0x62, 0x72, 0xA4),
            highlight: Color::Rgb(0xBD, 0x93, 0xF9),
            surface: Color::Rgb(0x2E, 0x30, 0x3C),
            on_highlight: Color::Rgb(0x28, 0x2A, 0x36),
        },
        Theme {
            name: "One Dark",
            background: Color::Rgb(0x28, 0x2C, 0x34),
            foreground: Color::Rgb(0xAB, 0xB2, 0xBF),
            primary: Color::Rgb(0x61, 0xAF, 0xEF),
            border: Color::Rgb(0x40, 0x44, 0x4C),
            accent: Color::Rgb(0xE5, 0xC0, 0x7B),
            warning: Color::Rgb(0xE0, 0x6C, 0x75),
            success: Color::Rgb(0x98, 0xC3, 0x79),
            dimmed: Color::Rgb(0x5C, 0x63, 0x70),
            highlight: Color::Rgb(0xC6, 0x78, 0xDD),
            surface: Color::Rgb(0x2E, 0x32, 0x3A),
            on_highlight: Color::Rgb(0x28, 0x2C, 0x34),
        },
        Theme {
            name: "Nightfly",
            background: Color::Rgb(0x01, 0x16, 0x27),
            foreground: Color::Rgb(0xBD, 0xC1, 0xC6),
            primary: Color::Rgb(0x00, 0xA6, 0xF2),
            border: Color::Rgb(0x22, 0x2E, 0x3F),
            accent: Color::Rgb(0xEC, 0xC6, 0x2F),
            warning: Color::Rgb(0xFC, 0x51, 0x4E),
            success: Color::Rgb(0x21, 0xC7, 0xA8),
            dimmed: Color::Rgb(0x44, 0x4A, 0x73),
            highlight: Color::Rgb(0xAE, 0x81, 0xFF),
            surface: Color::Rgb(0x06, 0x1C, 0x2D),
            on_highlight: Color::Rgb(0x01, 0x16, 0x27),
        },
    ]
}

pub fn get_theme_by_name(name: &str) -> Theme {
    let themes = get_themes();
    for t in themes.iter() {
        if t.name == name {
            return *t;
        }
    }
    themes[1] // Tokyo Night default
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_theme_has_unique_name() {
        let themes = get_themes();
        let names: std::collections::HashSet<&str> = themes.iter().map(|t| t.name).collect();
        assert_eq!(names.len(), themes.len());
    }

    #[test]
    fn all_theme_names_are_non_empty() {
        for t in get_themes() {
            assert!(!t.name.is_empty(), "theme has empty name");
        }
    }

    #[test]
    fn name_lookup_finds_every_theme() {
        for t in get_themes() {
            let found = get_theme_by_name(t.name);
            assert_eq!(found.name, t.name, "failed to find theme '{}'", t.name);
        }
    }

    #[test]
    fn unknown_name_returns_tokyo_night() {
        let t = get_theme_by_name("BogusThemeXYZ");
        assert_eq!(t.name, "Tokyo Night");
    }

    #[test]
    fn theme_is_copy() {
        let themes = get_themes();
        let t1 = themes[0];
        let t2 = t1; // Copy
        assert_eq!(t1.name, t2.name);
    }
}
