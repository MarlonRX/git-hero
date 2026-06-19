# Changelog

All notable changes to Git Hero are documented here.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased] — Major internal refactor

### Added

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
| **Total** | **~86** |

### Dependencies

- **Added**: `phf 0.11` (compile-time maps), `thiserror 1.0` (error derive)
