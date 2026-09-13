# Changelog

All notable changes to Git Hero are documented here.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.4.0] — 2026-09-12

### Added

- **Repository Browser (`/repos`, key `g`)**: the repository-manager core.
  A type-ahead dropdown in the sidebar's lower sector (where the shortcuts
  block used to live) that **recursively** scans the projects folder —
  depth-bounded (4 levels), skipping hidden and vendor/build directories
  (`node_modules`, `target`, `vendor`, …) and pruning at each repo root —
  and shows branch, ahead/behind, dirty-file count and last-commit age per
  repo, sorted by recent activity. Typing filters with zero git calls;
  Enter **or click** jumps into the repo (via `/cd`); `Ctrl+R` re-scans;
  `Esc` closes. Header line counts how many repos have uncommitted changes.
  Cost is two `git` processes per repo, only on open/re-scan — never per
  frame.
- **Browser as the no-repo landing screen**: opening `gith` in a directory
  that is not a repository (or `/cd`-ing to one, or after `/remove-repo`)
  now auto-opens the Repository Browser on the left and keeps the classic
  "Initialize Git repository here / Change Directory" options on the right.
  `Esc` hands the keyboard back to those options; clicking a row enters it.
- **Location made obvious**: the current folder/repo name is now a bold
  chip with background in the header status line; in the browser, repo
  names render as bold surface chips and the repository you are currently
  inside is tagged `● here`.
- **Persistent keybind strip**: the footer gained a third row of
  `[key] action` chips for the current context (repo / browser / no-repo /
  typing). Letter shortcuts are now always on screen and the strip degrades
  to an ellipsis on narrow terminals instead of hiding the keys.
- **Ignored smoke test** `repos::scan_recursively_finds_nested_repos_and_skips_vendor`
  (`cargo test -- --ignored`) proves the recursive scan end-to-end with real
  `git init` (nested repo found, `node_modules` decoy skipped, ordering and
  dirty count asserted).
- Status-bar version badge now uses `version::full()`: debug and dirty-tree
  builds are honestly marked (`v0.4.0-dev (hash)*`) instead of shipping-looking.

### Changed

- **Exactly two `git` invocations per refresh** (was three, originally six):
  `status --branch --porcelain=v2` now doubles as the "is this a repo?" probe
  via `GitError::NotARepository`; the separate `rev-parse` call is gone.
- **i18n is compile-time `phf` static maps** (was `OnceLock<HashMap>`): zero
  construction cost, and `translate()` stays zero-allocation for known keys.
- **CLI/TUI i18n unified**: `cli.rs` no longer contains any `if lang == "es"`
  pair — every user-facing string in both modes resolves through the same
  dictionary. TUI literals (theme/remove/cancel/status messages, footer,
  input placeholder, modal hints) migrated to i18n keys as well.
- `/repos` documented in `/help`, `/docs` and the shortcut tables.

### Fixed

- **FILES panel had no scrolling**: with more changed files than the panel
  height the list simply ran past the bottom and the selection went
  invisible. The list now windows around the selection (same follow model
  as the repo browser) and the panel title shows the position
  (`FILES (12/57)`) while overflowing. The COMMITS list got the same
  treatment; the old manual `commit_scroll_offset` (which could desync from
  the selection) is gone — PageUp/PageDown and the mouse wheel move the
  *selection* everywhere, and the view follows. Mouse wheel over the
  FILES/COMMITS/browser panels now scrolls them (the sidebar wheel used to
  be a no-op).
- `/` reliably opens the command bar from anywhere (repo mode, no-repo mode
  and from inside the repo browser). Command input is routed above the
  browser, so no letter shortcut or filter keystroke can fire while you are
  typing a written command; the footer strip also switches to
  Enter/Tab/Esc hints while typing.
- **Panic on non-ASCII commit subjects**: the commits panel truncated subjects
  with byte-slicing (`&subj[..n]`); any accent/emoji/CJK in a subject could
  crash the TUI. Truncation is now character-based (`truncate_subject`, unit
  tested). Same class of bug fixed in the fallback diff renderer.
- Console message drain in the event loop: `terminal.size()` was queried and
  the whole console output re-split **per message** (O(n²) on long output,
  and a `?` that could kill the TUI). Now once per capped batch (64 msgs).
- `cargo clippy --all-targets -- -D warnings` is clean under Rust 1.98
  (new lints fixed: `chunks_exact_to_as_chunks`, `collapsible_if`,
  `collapsible_match`, `needless_return`, dead code in `version.rs`).
- Whole tree formatted with `cargo fmt` (CI fmt gate was failing).

### Security

- Verified `/remove-repo` behavior against the refactor plan §1.3.1: the
  destructive delete is reachable **only** through the `show_confirm_remove`
  modal (y/N), and both `/help` and `/docs` say "asks confirmation". Locked
  in with regression tests (`modals::tests`), including a static check that
  `cmd_remove_repo` never calls `git_remove_repo` directly.

## [0.2.1] — 2025-07-23

### Fixed

- **Files incorrectly appearing as staged (selected) at startup**: in porcelain
  v2 mode, the index-status field (X) used `.` to mean "unmodified", but the
  parser only normalised the worktree field (Y). Every unstaged file showed
  `[✓]`. Both fields are now normalised correctly. ([#3])

- **Commit modal cursor stuck on first visual line**: `find_cursor_in_view`
  used `cursor_col <= start_col + line_len`, which matched the first wrapped
  segment for any column from 0 to `max_width`. The cursor never appeared on
  wrapped lines beyond the first one, and auto-scroll never triggered. Changed
  to strict `<`.

- **Commit modal word-wrapping collapsed spaces**: the old `wrap_line` did
  word-based wrapping that collapsed multiple consecutive spaces into one and
  dropped leading/trailing whitespace. Replaced with `wrap_line_hard` that
  preserves every character and breaks at `max_width` columns.

- **Commit modal navigation limited to logical lines**: `Up`/`Down` only
  moved between Shift+Enter-separated logical lines, not between visual
  wrapped segments. Now uses the wrapped view to navigate one visual line
  at a time, as expected from a text editor.

- **`Home` / `End` keys not handled in commit modal**: `Home` now jumps
  to column 0, `End` jumps to the end of the current logical line.

## [0.2.0] — 2025-06-19

### Added

- **Startup version check**: on launch, `git-hero` checks GitHub tags for a
  newer release. If found, a modal offers "Open download page", "Remind later",
  or "Don't show again for this version". The skip choice persists in config
  (`skipped_version`). ([#1])

- **`/language <en|es>` command**: switch the UI language at runtime. Persists
  to config and takes effect immediately — no restart needed. ([#2])

- **`Command` enum + `parse()` function**: replaces the 300-line if-else chain
  in `execute_command` with a typed dispatch table. 25 variants, 38 unit tests.
  Adding a new command = 1 variant + 1 parse arm + 1 test.
  ([4.1-4.2])

- **`status_snapshot()` / `log_snapshot()`**: single `git status --branch
  --porcelain=v2` call replaces 6+ separate git invocations in
  `refresh_git_status`. Second single `git log` call computes pushed-per-commit
  from the `ahead` count directly — no more hanging. ([2.1-2.4])

- **Diff cache by key**: `active_diff` computation is now cached by
  `DiffKey::File { path, staged, status }` or `DiffKey::Commit(hash)`,
  skipping git entirely when the user hasn't changed selection during an
  auto-refresh tick. ([2.5])

- **Theme-aware diff line cache**: diff re-renders when theme changes even
  if `active_diff` content is identical. ([2.8])

- **`i18n::trf()` helper**: look up a translation key and substitute `{}`
  placeholders in one call. 50+ new i18n keys for both English and Spanish.
  ([4.3])

- **`Command::HELP` constant**: single source of truth for the command list,
  used by the help modal. ([4.4])

- **`has_active_modal()` method**: encapsulates the OR of 8 modal bools.
  ([4.8])

- **`GitError` enum** (with `thiserror`): typed error handling for all git
  operations. Transitional `From<String>` shim preserves backwards
  compatibility. ([1.8, 1.11])

- **`log.rs` module with cached file handle**: debug log file is opened once
  instead of on every `log_debug()` call. ([1.10])

- **`icons.rs` module with `phf` tables**: O(1) compile-time lookup tables
  for icon codepoints (Nerd Font + ASCII fallback). ([1.5])

- **CI workflow**: `cargo check`, `cargo clippy -- -D warnings`,
  `cargo fmt --check`, `cargo test` on every push. ([5.7])

### Changed

- **`/remove-repo` now requires confirmation**: red-bordered modal warns
  about the irreversible destruction. The docs modal used to say
  "(no confirm!)" — that is no longer true. ([3.2-3.5])

- **`askpass` poll interval reduced**: 50ms → 500ms (90% fewer wakeups).
  Timeout reduced from 5 min to 2.5 min. ([3.6])

- **Clipboard copy without `unwrap()`**: the inline `and_then` with
  `child.stdin.as_mut().unwrap()` is now a dedicated `copy_to_clipboard()`
  function that returns a proper `io::Result`, shown to the user as
  `"Copy failed: …"`. ([3.7])

- **`cli.rs` propagates errors**: 6× `std::process::exit(1)` replaced by
  `return Err(…)`, allowing unit tests to exercise failure paths. ([3.8])

- **`run_git_verbose` better error**: when stderr is empty but the exit code
  is non-zero, the message is `"git failed with exit code N (no output)"`
  instead of `"Unknown error"`. ([3.9])

- **`draw_ui` split into 7 functions**: background, banner, footer, command
  bar, modal routing, and console are now named section drawers. Main
  function is 60 lines (down from 292). ([4.8])

- **Shortcut lines in static memory**: keyboard shortcut labels moved from
  a per-frame `vec!` to a `static SHORTCUT_LINES: &[&str]`. ([4.10])

- **Commit list O(n²) → O(n)**: replaced `s.commits.iter().position(…)` per
  visible commit with an offset-based index lookup. ([4.11])

- **`count_commits` returns `u64`**: avoids overflow on repos with >2³¹
  commits (unlikely but type-correct). ([1.7])

- **`stderr_to_err` accepts `&[u8]`**: takes a borrow instead of `Vec<u8>`,
  uses `Cow<'static, str>` for the result. ([1.9])

- **i18n dictionaries moved to `OnceLock<HashMap>`**: dictionaries are built
  once on first use instead of on every call. 40× reduction in allocations
  per `translate()` call. ([1.1-1.2])

- **Command suggestions are `Cow<'static, str>`**: `/cd` suggestions are
  `Cow::Owned`, slash-command suggestions are `Cow::Borrowed` from a static
  array. Zero allocations per keystroke for the command list. ([1.3-1.4])

- **`FlatEntryKind` removed**: was an enum with a single variant. Both the
  enum and the wrapper struct `FlatEntry` are gone; `flat_entries` is now
  `Vec<usize>`. ([1.6])

### Fixed

- **`remove-repo` safety**: the docs modal noted "(no confirm!)". Now it
  requires explicit `y`/`Enter` confirmation. ([3.2-3.5])

- **Clippy warnings**: codebase is `cargo clippy --all-targets -- -D
  warnings` clean (33 warnings fixed). ([5.0])

### Removed

- `get_changed_files()` (dead code, replaced by `status_snapshot`)
- `get_recent_commits()` (dead code, replaced by `log_snapshot`)
- `FlatEntryKind` (enum with single variant)
- `chrono_lite()` (inlined)

### Performance

| Metric | Before | After |
|---|---|---|
| Allocations per `translate()` | ~41 (dict rebuild + result) | 0-1 (Cow borrowed or owned) |
| Allocations per keystroke in command bar | 21 Strings | 0 (static slice filter) |
| Git processes in `refresh_git_status` | 6+ | 2 |
| Git processes per auto-refresh tick (no selection change) | 2-3 | 0 |
| O(n²) in commit list with 1000 commits | 1M ops/frame | 10 ops/frame |
| Askpass process wakeups | 6000 × 50ms = 300/s | 300 × 500ms = 2/s |
| File handle re-opens in debug mode | 60/s (1/log call) | 1 (cached) |
| Clippy warnings | 33 | 0 |
| Unit tests | 2 (version.rs only) | ~90 |

### Test coverage

| Module | Tests |
|---|---|
| `command.rs` | 38 |
| `git.rs` (parsers) | 14 |
| `suggestions.rs` | 7 |
| `icons.rs` | 4 |
| `i18n.rs` | 5 |
| `git_error.rs` | 6 |
| `theme.rs` | 5 |
| `config.rs` | 2 |
| `log.rs` | 2 |
| `version.rs` | 2 |
| `cli.rs` | 1 |
| **Total** | **~85** |

### Dependencies

- **Added**: `phf 0.11` (compile-time maps), `thiserror 1.0` (error derive)

[0.2.0]: https://github.com/MarlonRX/git-hero/releases/tag/v0.2.0
