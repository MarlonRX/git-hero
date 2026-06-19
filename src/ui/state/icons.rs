//! Compile-time icon tables for the TUI.
//!
//! Two [`phf::Map`]s (nerd-font glyphs and ASCII fallbacks) keyed by semantic
//! names like `"branch"`, `"dir"`, `"commit"`. Lookups are O(1) and the maps
//! are built once at compile time — zero allocations, zero runtime parsing.

use phf::Map;

/// Nerd-Font codepoints (private-use area). Requires a Nerd-Font terminal.
const ICONS_NERD: Map<&'static str, &'static str> = phf::phf_map! {
    "branch"    => "\u{E725}",
    "dir"       => "\u{F07C}",
    "fetch"     => "\u{F021}",
    "commit"    => "\u{F417}",
    "mod"       => "\u{F448}",
    "add"       => "\u{F067}",
    "del"       => "\u{F057}",
    "untracked" => "\u{F128}",
    "ok"        => "\u{2714}",
    "warn"      => "\u{26A0}",
};

/// ASCII / Unicode fallbacks for terminals without Nerd Fonts.
const ICONS_ASCII: Map<&'static str, &'static str> = phf::phf_map! {
    "branch"    => "\u{2A}",
    "dir"       => "\u{2F}",
    "fetch"     => "\u{2193}",
    "commit"    => "\u{23}",
    "mod"       => "\u{2192}",
    "add"       => "\u{2B}",
    "del"       => "\u{2212}",
    "untracked" => "\u{3F}",
    "ok"        => "\u{2713}",
    "warn"      => "\u{21}",
};

/// Look up an icon by semantic key. Returns `""` for unknown keys (caller
/// can decide whether to render a placeholder or skip the slot).
#[inline]
pub fn lookup(nerd_font: bool, key: &str) -> &'static str {
    let table = if nerd_font { &ICONS_NERD } else { &ICONS_ASCII };
    table.get(key).copied().unwrap_or("")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nerd_lookups_match_expected_codepoints() {
        assert_eq!(lookup(true, "branch"), "\u{E725}");
        assert_eq!(lookup(true, "dir"), "\u{F07C}");
        assert_eq!(lookup(true, "ok"), "\u{2714}");
    }

    #[test]
    fn ascii_lookups_match_expected_codepoints() {
        assert_eq!(lookup(false, "branch"), "\u{2A}");
        assert_eq!(lookup(false, "dir"), "\u{2F}");
        assert_eq!(lookup(false, "ok"), "\u{2713}");
    }

    #[test]
    fn unknown_key_returns_empty_string() {
        assert_eq!(lookup(true, "no_such_icon"), "");
        assert_eq!(lookup(false, ""), "");
    }

    #[test]
    fn tables_have_the_same_keys() {
        for key in ICONS_NERD.keys() {
            assert!(ICONS_ASCII.contains_key(key), "ASCII table is missing key: {key}");
        }
    }
}
