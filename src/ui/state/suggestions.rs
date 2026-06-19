//! Command and directory auto-completion for the command bar.
//!
//! Suggestions are produced lazily and kept in sync with the input value.
//! Command suggestions are `&'static str` (no allocation); directory
//! suggestions are `String` because paths are runtime data.

use std::borrow::Cow;
use std::fs;
use std::path::Path;

use crate::ui::state::AppState;

impl AppState {
    pub fn update_suggestions(&mut self) {
        let val = self.input_value.clone();
        if val.starts_with("/cd ") {
            self.suggestions = get_directory_suggestions(&val);
        } else if val.starts_with('/') {
            self.suggestions = get_command_suggestions(&val);
        } else {
            self.suggestions.clear();
        }
        if !self.suggestions.is_empty() && self.active_sug >= self.suggestions.len() {
            self.active_sug = 0;
        }
    }
}

/// Expand a leading `~` to the user's home directory.
pub fn expand_path(path: &str) -> Cow<'_, str> {
    if let Some(stripped) = path.strip_prefix('~')
        && let Some(home) = dirs::home_dir()
    {
        return Cow::Owned(format!("{}{}", home.to_string_lossy(), stripped));
    }
    Cow::Borrowed(path)
}

/// All slash-commands the app supports, in display order.
/// `&'static str` lets us filter and return them with zero allocations.
const COMMANDS: &[&str] = &[
    "/fetch",
    "/pull",
    "/push",
    "/commit ",
    "/stage-all",
    "/unstage-all",
    "/undo-commit",
    "/remove-repo",
    "/remote ",
    "/branch ",
    "/branches",
    "/switch ",
    "/config ",
    "/config-global ",
    "/stash",
    "/stash-pop",
    "/language ",
    "/cd ",
    "/themes",
    "/help",
    "/docs",
    "/quit",
];

pub fn get_command_suggestions(input: &str) -> Vec<Cow<'static, str>> {
    if input.is_empty() || input == "/" {
        return COMMANDS.iter().map(|c| Cow::Borrowed(*c)).collect();
    }
    COMMANDS
        .iter()
        .filter(|c| c.starts_with(input))
        .map(|c| Cow::Borrowed(*c))
        .collect()
}

pub fn get_directory_suggestions(input: &str) -> Vec<Cow<'static, str>> {
    let Some(path_arg) = input.strip_prefix("/cd ") else {
        return Vec::new();
    };
    let resolved_path = expand_path(path_arg);

    let (search_dir, prefix) = if path_arg.is_empty() {
        (".", "")
    } else if path_arg == "~" || path_arg.ends_with('/') || path_arg.ends_with('\\') {
        (resolved_path.as_ref(), "")
    } else {
        let path = Path::new(resolved_path.as_ref());
        let parent = path.parent().and_then(|p| p.to_str()).unwrap_or(".");
        let file_name = path.file_name().and_then(|f| f.to_str()).unwrap_or("");
        (parent, file_name)
    };

    let prefix_lower = prefix.to_ascii_lowercase();
    let mut suggestions: Vec<Cow<'static, str>> = Vec::new();
    if let Ok(entries) = fs::read_dir(search_dir) {
        for entry in entries.flatten() {
            // Skip non-directories early — we only suggest dirs to `/cd`.
            let Ok(file_type) = entry.file_type() else { continue };
            if !file_type.is_dir() {
                continue;
            }
            let name = entry.file_name();
            let name_bytes = name.as_encoded_bytes();
            if !prefix.is_empty() && !ascii_starts_with_ignore_case(name_bytes, prefix_lower.as_bytes()) {
                continue;
            }
            // Rebuild the base path the user has already typed.
            let base_path = if prefix.is_empty() {
                path_arg.to_string()
            } else {
                path_arg[..path_arg.len() - prefix.len()].to_string()
            };
            let needs_sep = !base_path.is_empty()
                && !base_path.ends_with('/')
                && !base_path.ends_with('\\');
            let name_str = name.to_string_lossy();
            let mut suggestion = String::with_capacity(
                base_path.len() + name_str.len() + 3, // "/cd " + base + "/" + name
            );
            suggestion.push_str("/cd ");
            suggestion.push_str(&base_path);
            if needs_sep {
                suggestion.push('/');
            }
            suggestion.push_str(&name_str);
            suggestion.push('/');
            suggestions.push(Cow::Owned(suggestion));
            if suggestions.len() >= 5 {
                break;
            }
        }
    }
    suggestions
}

/// ASCII case-insensitive prefix check. No allocation, no Unicode folding —
/// directory names on the filesystems we care about are ASCII.
#[inline]
fn ascii_starts_with_ignore_case(haystack: &[u8], needle_lower: &[u8]) -> bool {
    if needle_lower.is_empty() {
        return true;
    }
    if haystack.len() < needle_lower.len() {
        return false;
    }
    haystack[..needle_lower.len()].iter().zip(needle_lower).all(|(h, n)| {
        h.to_ascii_lowercase() == *n
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_suggestions_empty_returns_all() {
        let all = get_command_suggestions("");
        assert_eq!(all.len(), COMMANDS.len());
    }

    #[test]
    fn command_suggestions_slash_returns_all() {
        let all = get_command_suggestions("/");
        assert_eq!(all.len(), COMMANDS.len());
    }

    #[test]
    fn command_suggestions_filters() {
        let r = get_command_suggestions("/co");
        assert!(r.iter().any(|c| c == "/commit "));
        assert!(!r.iter().any(|c| c == "/fetch"));
    }

    #[test]
    fn command_suggestions_returns_borrowed() {
        // No allocations for known commands.
        for s in get_command_suggestions("/f") {
            assert!(matches!(s, Cow::Borrowed(_)));
        }
    }

    #[test]
    fn ascii_starts_with_basic() {
        assert!(ascii_starts_with_ignore_case(b"README.md", b"read"));
        assert!(ascii_starts_with_ignore_case(b"README.md", b"readme"));
        assert!(!ascii_starts_with_ignore_case(b"src", b"re"));
        assert!(ascii_starts_with_ignore_case(b"src", b""));
    }

    #[test]
    fn expand_path_no_tilde() {
        assert!(matches!(expand_path("foo"), Cow::Borrowed("foo")));
    }

    #[test]
    fn expand_path_with_tilde_uses_home() {
        if dirs::home_dir().is_some() {
            let r = expand_path("~/foo");
            assert!(matches!(r, Cow::Owned(_)));
            assert!(r.ends_with("/foo"));
        }
    }
}
