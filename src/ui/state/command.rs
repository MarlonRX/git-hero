//! Slash-command parser.
//!
//! Replaces the 300-line if-else chain that used to live inside
//! `AppState::execute_command`. The parser is a pure function: input in,
//! typed `Command` out. Behaviour is locked in by 30+ unit tests so the
//! TUI dispatch table can stay simple (`match cmd { ... }`).
//!
//! Adding a new command is a 3-step process:
//!   1. Add a variant to [`Command`]
//!   2. Add a parse arm in [`Command::parse`]
//!   3. Add a test in the `tests` module
//!
//! The parser is deliberately permissive: unknown commands become
//! `Command::Unknown(input)` (not an error) so the UI can show a
//! friendly "type /help" message. Truly malformed commands (e.g. `/cd`
//! with no path) still return [`ParseError`].

use std::borrow::Cow;

/// A slash command, parsed and ready to dispatch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    /// `/cd <path>` — change working directory
    Cd(String),
    /// `/fetch` — `git fetch`
    Fetch,
    /// `/pull` — show the pull confirmation modal
    Pull,
    /// `/push` — show the push confirmation modal
    Push,
    /// `/commit` — open the multi-line commit modal
    Commit,
    /// `/commit <msg>` — commit immediately with the given message
    CommitMessage(String),
    /// `/themes` — open the theme picker
    Themes,
    /// `/help` — show the quick help modal
    Help,
    /// `/docs` — show the detailed reference modal
    Docs,
    /// `/stage-all` — `git add .`
    StageAll,
    /// `/unstage-all` — `git reset HEAD`
    UnstageAll,
    /// `/undo-commit` — soft-reset the last commit
    UndoCommit,
    /// `/remove-repo` — show the destructive-action confirmation modal
    RemoveRepo,
    /// `/remote <url>` — set the `origin` remote URL
    SetRemote(String),
    /// `/branch` or `/branches` — list branches
    ListBranches,
    /// `/branch -d <name>` — delete a branch
    DeleteBranch(String),
    /// `/branch -s <name>` or `/switch <name>` — switch or create
    SwitchOrCreate(String),
    /// `/branch <name>` — create and switch
    CreateBranch(String),
    /// `/config` — list local config
    ListConfig,
    /// `/config <key> [value]` — local config
    ConfigLocal { key: String, value: Option<String> },
    /// `/config-global <key> [value]` — global config
    ConfigGlobal { key: String, value: Option<String> },
    /// `/stash` — stash local changes
    Stash,
    /// `/stash-pop` — pop the stash
    StashPop,
    /// `/quit` or `/exit` — leave the TUI
    Quit,
    /// Anything starting with `/` we don't recognise. The original input
    /// is preserved so the dispatcher can show "Unknown command: /foo".
    Unknown(String),
}

/// Reason a command couldn't be parsed. Reserved for genuinely malformed
/// input (empty after a prefix that requires an argument, etc.) — never
/// for "unknown command", which is `Command::Unknown`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    pub input: String,
    pub reason: Cow<'static, str>,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} (input: {})", self.reason, self.input)
    }
}

impl std::error::Error for ParseError {}

impl Command {
    /// Parse a slash command from a user-typed string. Whitespace at the
    /// edges is trimmed. Empty input and non-`/`-prefixed input are errors.
    pub fn parse(input: &str) -> Result<Self, ParseError> {
        let input = input.trim();
        if input.is_empty() {
            return Err(ParseError {
                input: input.to_string(),
                reason: Cow::Borrowed("empty input"),
            });
        }
        if !input.starts_with('/') {
            return Err(ParseError {
                input: input.to_string(),
                reason: Cow::Borrowed("command must start with '/'"),
            });
        }

        // ── Commands that take a path/argument ────────────────────
        if let Some(rest) = input.strip_prefix("/cd ") {
            return Ok(Command::Cd(rest.trim().to_string()));
        }
        if let Some(rest) = input.strip_prefix("/remote ") {
            let url = rest.trim();
            if url.is_empty() {
                return Err(ParseError {
                    input: input.to_string(),
                    reason: Cow::Borrowed("/remote requires a URL"),
                });
            }
            return Ok(Command::SetRemote(url.to_string()));
        }
        if let Some(rest) = input.strip_prefix("/commit ") {
            return Ok(Command::CommitMessage(rest.trim().to_string()));
        }

        // ── Branch management (the order matters: -d, -s, then bare) ──
        if let Some(rest) = input.strip_prefix("/branch -d ") {
            let name = rest.trim();
            if name.is_empty() {
                return Err(ParseError {
                    input: input.to_string(),
                    reason: Cow::Borrowed("/branch -d requires a name"),
                });
            }
            return Ok(Command::DeleteBranch(name.to_string()));
        }
        if let Some(rest) = input.strip_prefix("/branch -s ") {
            let name = rest.trim();
            if name.is_empty() {
                return Err(ParseError {
                    input: input.to_string(),
                    reason: Cow::Borrowed("/branch -s requires a name"),
                });
            }
            return Ok(Command::SwitchOrCreate(name.to_string()));
        }
        if let Some(rest) = input.strip_prefix("/switch ") {
            let name = rest.trim();
            if name.is_empty() {
                return Err(ParseError {
                    input: input.to_string(),
                    reason: Cow::Borrowed("/switch requires a name"),
                });
            }
            return Ok(Command::SwitchOrCreate(name.to_string()));
        }
        if let Some(rest) = input.strip_prefix("/branch ") {
            let name = rest.trim();
            // A name starting with `-` means a flag was intended
            // (`/branch -d` / `/branch -s`); the user forgot the name.
            // " -" inside the name catches things like `/branch foo -d`.
            if name.is_empty() || name.starts_with('-') || name.contains(" -") {
                return Err(ParseError {
                    input: input.to_string(),
                    reason: Cow::Borrowed("/branch requires a name"),
                });
            }
            return Ok(Command::CreateBranch(name.to_string()));
        }

        // ── Config (local and global share the parsing logic) ────
        if let Some(rest) = input.strip_prefix("/config-global ") {
            let (key, value) = split_key_value(rest.trim());
            if key.is_empty() {
                return Err(ParseError {
                    input: input.to_string(),
                    reason: Cow::Borrowed("/config-global requires a key"),
                });
            }
            return Ok(Command::ConfigGlobal {
                key: key.to_string(),
                value: value.map(String::from),
            });
        }
        if let Some(rest) = input.strip_prefix("/config ") {
            let (key, value) = split_key_value(rest.trim());
            if key.is_empty() {
                return Err(ParseError {
                    input: input.to_string(),
                    reason: Cow::Borrowed("/config requires a key"),
                });
            }
            return Ok(Command::ConfigLocal {
                key: key.to_string(),
                value: value.map(String::from),
            });
        }

        // ── Exact-match commands ─────────────────────────────────
        match input {
            "/cd" => return Err(ParseError {
                input: input.to_string(),
                reason: Cow::Borrowed("/cd requires a path"),
            }),
            "/remote" => return Err(ParseError {
                input: input.to_string(),
                reason: Cow::Borrowed("/remote requires a URL"),
            }),
            "/commit" => return Ok(Command::Commit),
            "/fetch" => return Ok(Command::Fetch),
            "/pull" => return Ok(Command::Pull),
            "/push" => return Ok(Command::Push),
            "/themes" => return Ok(Command::Themes),
            "/help" => return Ok(Command::Help),
            "/docs" => return Ok(Command::Docs),
            "/stage-all" => return Ok(Command::StageAll),
            "/unstage-all" => return Ok(Command::UnstageAll),
            "/undo-commit" => return Ok(Command::UndoCommit),
            "/remove-repo" => return Ok(Command::RemoveRepo),
            "/branch" | "/branches" => return Ok(Command::ListBranches),
            "/branch -d" | "/branch -s" => return Err(ParseError {
                input: input.to_string(),
                reason: Cow::Borrowed("/branch -d/-s requires a name"),
            }),
            "/switch" => return Err(ParseError {
                input: input.to_string(),
                reason: Cow::Borrowed("/switch requires a name"),
            }),
            "/config" => return Ok(Command::ListConfig),
            "/config-global" => return Err(ParseError {
                input: input.to_string(),
                reason: Cow::Borrowed("/config-global requires a key"),
            }),
            "/stash" => return Ok(Command::Stash),
            "/stash-pop" => return Ok(Command::StashPop),
            "/quit" | "/exit" | "/q" => return Ok(Command::Quit),
            _ => {}
        }

        // Recognised prefix but no matching arm → unknown.
        Ok(Command::Unknown(input.to_string()))
    }

    /// Static help text used by `/help` and `git-hero --help`. Single
    /// source of truth — keeping the help modal and the CLI's `--help`
    /// output in sync is now a compile-time invariant.
    pub const HELP: &'static [(&'static str, &'static str)] = &[
        ("/cd <path>", "Change working directory"),
        ("/fetch", "git fetch"),
        ("/pull", "git pull (with confirmation)"),
        ("/push", "git push (with confirmation)"),
        ("/commit [msg]", "Open modal, or commit immediately with a message"),
        ("/stage-all", "git add ."),
        ("/unstage-all", "git reset HEAD"),
        ("/undo-commit", "Soft-reset the last commit"),
        ("/remove-repo", "Delete .git directory (asks confirmation)"),
        ("/remote <url>", "Set the 'origin' remote URL"),
        ("/branch [name]", "List, create, or switch branches"),
        ("/switch <name>", "Switch to (or create) a branch"),
        ("/config <k> [v]", "Read or set local config"),
        ("/config-global <k> [v]", "Read or set global config"),
        ("/stash", "git stash"),
        ("/stash-pop", "git stash pop"),
        ("/themes", "Open the theme picker"),
        ("/help", "Quick help"),
        ("/docs", "Detailed reference"),
        ("/quit", "Leave the TUI"),
    ];
}

/// Split `<rest>` on the first space, returning `(key, Some(value))` or
/// `(key, None)` when there's no value.
fn split_key_value(rest: &str) -> (&str, Option<&str>) {
    match rest.find(' ') {
        Some(idx) => (&rest[..idx], Some(&rest[idx + 1..])),
        None => (rest, None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ok(input: &str) -> Command {
        Command::parse(input)
            .unwrap_or_else(|e| panic!("expected ok for {input:?}, got {e}"))
    }

    fn err(input: &str) {
        assert!(
            Command::parse(input).is_err(),
            "expected err for {input:?}"
        );
    }

    // ── Sanity ──
    #[test]
    fn empty_is_error() {
        err("");
        err("   ");
    }

    #[test]
    fn non_slash_is_error() {
        err("foo");
        err("git status");
    }

    #[test]
    fn leading_trailing_whitespace_ignored() {
        assert_eq!(ok("  /fetch  "), Command::Fetch);
    }

    // ── Exact matches ──
    #[test]
    fn fetch() {
        assert_eq!(ok("/fetch"), Command::Fetch);
    }
    #[test]
    fn pull_push() {
        assert_eq!(ok("/pull"), Command::Pull);
        assert_eq!(ok("/push"), Command::Push);
    }
    #[test]
    fn themes_help_docs() {
        assert_eq!(ok("/themes"), Command::Themes);
        assert_eq!(ok("/help"), Command::Help);
        assert_eq!(ok("/docs"), Command::Docs);
    }
    #[test]
    fn stage_unstage_undo_remove() {
        assert_eq!(ok("/stage-all"), Command::StageAll);
        assert_eq!(ok("/unstage-all"), Command::UnstageAll);
        assert_eq!(ok("/undo-commit"), Command::UndoCommit);
        assert_eq!(ok("/remove-repo"), Command::RemoveRepo);
    }
    #[test]
    fn commit_no_arg() {
        assert_eq!(ok("/commit"), Command::Commit);
    }
    #[test]
    fn commit_with_msg() {
        assert_eq!(
            ok("/commit fix the bug"),
            Command::CommitMessage("fix the bug".into())
        );
    }
    #[test]
    fn stash_and_pop() {
        assert_eq!(ok("/stash"), Command::Stash);
        assert_eq!(ok("/stash-pop"), Command::StashPop);
    }
    #[test]
    fn quit_aliases() {
        assert_eq!(ok("/quit"), Command::Quit);
        assert_eq!(ok("/exit"), Command::Quit);
        assert_eq!(ok("/q"), Command::Quit);
    }

    // ── With arguments ──
    #[test]
    fn cd_with_path() {
        assert_eq!(ok("/cd /tmp"), Command::Cd("/tmp".into()));
        assert_eq!(ok("/cd ~/projects"), Command::Cd("~/projects".into()));
    }
    #[test]
    fn cd_without_path_is_error() {
        err("/cd");
    }

    #[test]
    fn remote_with_url() {
        assert_eq!(
            ok("/remote git@github.com:a/b"),
            Command::SetRemote("git@github.com:a/b".into())
        );
    }
    #[test]
    fn remote_empty_is_error() {
        err("/remote");
        err("/remote ");
    }

    #[test]
    fn branch_list_aliases() {
        assert_eq!(ok("/branch"), Command::ListBranches);
        assert_eq!(ok("/branches"), Command::ListBranches);
    }
    #[test]
    fn branch_create() {
        assert_eq!(ok("/branch feature-x"), Command::CreateBranch("feature-x".into()));
    }
    #[test]
    fn branch_create_with_flag_is_error() {
        err("/branch -d");
        err("/branch -s");
        err("/branch foo -d"); // mixed form
    }
    #[test]
    fn branch_create_with_dash_name_is_error() {
        err("/branch -foo");
    }
    #[test]
    fn branch_delete() {
        assert_eq!(
            ok("/branch -d old-branch"),
            Command::DeleteBranch("old-branch".into())
        );
    }
    #[test]
    fn branch_switch_flag() {
        assert_eq!(
            ok("/branch -s main"),
            Command::SwitchOrCreate("main".into())
        );
    }
    #[test]
    fn switch_alias() {
        assert_eq!(ok("/switch main"), Command::SwitchOrCreate("main".into()));
        err("/switch");
    }

    #[test]
    fn config_list() {
        assert_eq!(ok("/config"), Command::ListConfig);
    }
    #[test]
    fn config_local_get() {
        assert_eq!(
            ok("/config user.name"),
            Command::ConfigLocal {
                key: "user.name".into(),
                value: None,
            }
        );
    }
    #[test]
    fn config_local_set() {
        assert_eq!(
            ok("/config user.email me@example.com"),
            Command::ConfigLocal {
                key: "user.email".into(),
                value: Some("me@example.com".into()),
            }
        );
    }
    #[test]
    fn config_global_get() {
        assert_eq!(
            ok("/config-global user.name"),
            Command::ConfigGlobal {
                key: "user.name".into(),
                value: None,
            }
        );
    }
    #[test]
    fn config_global_set() {
        assert_eq!(
            ok("/config-global core.editor vim"),
            Command::ConfigGlobal {
                key: "core.editor".into(),
                value: Some("vim".into()),
            }
        );
    }
    #[test]
    fn config_empty_key_is_error() {
        // "/config-global " (with trailing space) trims to the bare
        // command which is malformed, so the parser returns Err.
        err("/config-global ");
    }
    #[test]
    fn config_with_only_whitespace_trims_to_list() {
        // "/config " trims to "/config" which is ListConfig (read all),
        // not an error. User-hostile to make it an error.
        assert_eq!(ok("/config "), Command::ListConfig);
    }

    // ── Unknown commands ──
    #[test]
    fn unknown_slash_returns_unknown_variant() {
        assert_eq!(ok("/xyz"), Command::Unknown("/xyz".into()));
        assert_eq!(ok("/foo bar"), Command::Unknown("/foo bar".into()));
    }

    // ── HELP constant ──
    #[test]
    fn help_table_is_non_empty_and_includes_cd() {
        assert!(!Command::HELP.is_empty());
        assert!(Command::HELP.iter().any(|(cmd, _)| *cmd == "/cd <path>"));
    }
    #[test]
    fn help_table_has_unique_command_keys() {
        let mut seen = std::collections::HashSet::new();
        for (cmd, _) in Command::HELP {
            assert!(seen.insert(*cmd), "duplicate help entry: {cmd}");
        }
    }
}
