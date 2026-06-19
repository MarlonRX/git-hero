use std::fs;
use std::time::SystemTime;

use crate::config::{load_config, Config};
use crate::git;
use crate::log;
use crate::theme::{get_theme_by_name, Theme};

pub mod command;
pub mod commands;
pub mod icons;
pub mod suggestions;

/// Identifies the *input* to `update_diff_content` so we can skip git
/// invocations when the user (or the auto-refresh loop) re-asks for the
/// same thing. Only the fields that influence the git output are part of
/// the key — theme and width are handled by the line cache.
#[derive(Debug, Clone, PartialEq, Eq)]
enum DiffKey {
    /// Showing the file list of a commit (focus = "commits").
    Commit(String),
    /// Showing the diff of a working-tree file (focus != "commits").
    File {
        path: String,
        /// Cached copy of `GitFile::staged` so the key invalidates correctly
        /// when the user stages/unstages.
        staged: bool,
        /// Status string (e.g. "M", "??", "MM"). Needed to choose between
        /// the `git diff` path and the "read file directly" path for
        /// untracked files.
        status: String,
    },
    /// No file, no commit selected — "working directory clean" or similar.
    Empty,
}

#[derive(Debug, Clone)]
pub enum TuiMessage {
    ConsoleOutput(String),
    CommandFinished(Result<(), String>),
    /// Version-check result: latest version string.
    /// Reserved for future async check — currently the sync check in
    /// `AppState::new()` sets `show_update_modal` directly.
    #[allow(dead_code)]
    UpdateAvailable(String),
}

#[derive(Clone)]
pub struct GitFile {
    pub path: String,
    pub staged: bool,
    pub status: String,
}

#[derive(Clone)]
pub struct FlatEntry {
    /// Index into `AppState::files`.
    pub file_idx: usize,
}

#[derive(Clone)]
pub struct GitCommit {
    pub hash: String,
    pub date: String,
    pub subject: String,
    pub pushed: bool,
}

pub struct AppState {
    // ── Git State ──────────────────────────────────────────────────
    pub cwd: String,
    pub is_git_repo: bool,
    pub branch: String,
    pub remote: String,
    pub behind: u64,
    pub ahead: u64,

    // ── Config State ───────────────────────────────────────────────
    pub theme: Theme,
    pub language: String,
    pub nerd_font: bool,
    pub status_message: String,
    pub fetching: bool,

    // ── Interactive Lists ──────────────────────────────────────────
    pub files: Vec<GitFile>,
    pub selected_file_idx: usize,
    pub flat_entries: Vec<FlatEntry>,
    pub flat_idx: usize,
    pub commits: Vec<GitCommit>,
    pub selected_commit_idx: usize,
    pub focus_pane: String,
    pub active_diff: String,
    pub diff_scroll_offset: usize,
    pub commit_scroll_offset: usize,  // Scroll offset for commits panel
    
    // ── Commit Detail View ────────────────────────────────────────
    pub show_commit_detail: bool,  // Show detailed commit info when clicking/entering on commit
    pub commit_detail_diff: String,  // Diff for the selected commit
    pub commit_detail_scroll: usize,  // Scroll for commit detail view
    
    // ── Diff Cache (performance) ─────────────────────────────────
    /// Last `active_diff` content rendered, used to invalidate the line cache.
    cached_diff_content: String,
    cached_diff_lines: Vec<ratatui::text::Line<'static>>,
    cached_diff_width: u16,  // Width used for side-by-side rendering
    /// Theme name used when `cached_diff_lines` was last rendered. Phase 2.8:
    /// re-render when the user switches theme even if `active_diff` is identical.
    cached_diff_theme_name: Option<&'static str>,
    /// Phase 2.5: cache of the raw git output keyed by what produced it.
    /// Avoids re-running `git diff` 2-3 times per auto-refresh tick when the
    /// user has not changed selection.
    diff_cache: Option<(DiffKey, String)>,

    // ── Command Input ──────────────────────────────────────────────
    pub input_value: String,
    pub input_cursor_pos: usize,
    pub show_input: bool,
    pub suggestions: Vec<std::borrow::Cow<'static, str>>,
    pub active_sug: usize,

    // ── Modals ─────────────────────────────────────────────────────
    pub show_theme_modal: bool,
    pub show_help_modal: bool,
    pub show_docs_modal: bool,
    pub theme_cursor: usize,
    pub saved_theme: String,

    // ── Mini Console ───────────────────────────────────────────────
    pub console_output: String,
    pub console_visible: bool,
    pub console_scroll: usize,
    pub console_running: bool,

    // ── Auto-refresh ───────────────────────────────────────────────
    pub last_git_mtime: Option<SystemTime>,

    // ── Wizards ────────────────────────────────────────────────────
    pub setup_step: usize,
    pub setup_cursor: usize,
    pub init_wizard_active: bool,
    pub init_wizard_step: usize,
    pub init_cursor: usize,
    pub init_branch_name: String,
    pub init_remote_url: String,

    // ── Commit Message Editor ─────────────────────────────
    pub show_commit_modal: bool,
    pub commit_message_lines: Vec<String>,
    pub commit_cursor_row: usize,
    pub commit_cursor_col: usize,
    pub commit_modal_scroll: usize,

    // ── Push / Pull / Remove Confirmations ─────────────────────────
    pub show_confirm_push: bool,
    pub show_confirm_pull: bool,
    /// Phase 3.3: required confirmation before `/remove-repo` deletes
    /// the `.git` directory. The original code had no confirmation —
    /// see `docs.rs`: "/remove-repo         Delete .git directory (no confirm!)".
    pub show_confirm_remove: bool,

    // ── Update notification ────────────────────────────────────────
    /// Set to `true` when `check_for_updates` finds a newer version.
    pub show_update_modal: bool,
    /// The latest version string (e.g. "0.2.0") to display in the modal.
    pub latest_version: String,

    // ── Credentials Handler ────────────────────────────────────────
    pub session_id: String,
    pub tx: Option<std::sync::mpsc::Sender<TuiMessage>>,
    pub show_credentials_modal: bool,
    pub credentials_prompt: String,
    pub credentials_input: String,
    pub credentials_cursor: usize,
    pub credentials_mask: bool,
}

impl AppState {
    pub fn new() -> Self {
        let config = load_config().unwrap_or(Config {
            language: "en".to_string(),
            nerd_font: false,
            theme: "Tokyo Night".to_string(),
            skipped_version: None,
        });

        let theme = get_theme_by_name(&config.theme);
        let cwd = std::env::current_dir()
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_else(|_| ".".to_string());

        let mut state = Self {
            cwd,
            is_git_repo: false,
            branch: String::new(),
            remote: String::new(),
            behind: 0,
            ahead: 0,
            theme,
            language: config.language.clone(),
            nerd_font: config.nerd_font,
            status_message: "Ready. Press ? for help.".to_string(),
            fetching: false,
            files: Vec::new(),
            selected_file_idx: 0,
            flat_entries: Vec::new(),
            flat_idx: 0,
            commits: Vec::new(),
            selected_commit_idx: 0,
            focus_pane: "files".to_string(),
            active_diff: String::new(),
            diff_scroll_offset: 0,
            commit_scroll_offset: 0,
            show_commit_detail: false,
            commit_detail_diff: String::new(),
            commit_detail_scroll: 0,
            cached_diff_content: String::new(),
            cached_diff_lines: Vec::new(),
            cached_diff_width: 0,
            cached_diff_theme_name: None,
            diff_cache: None,
            input_value: String::new(),
            input_cursor_pos: 0,
            show_input: false,
            suggestions: Vec::new(),
            active_sug: 0,
            show_theme_modal: false,
            show_help_modal: false,
            show_docs_modal: false,
            theme_cursor: 0,
            saved_theme: config.theme,
            console_output: String::new(),
            console_visible: false,
            console_scroll: 0,
            console_running: false,
            last_git_mtime: None,
            setup_step: 0,
            setup_cursor: 0,
            init_wizard_active: false,
            init_wizard_step: 0,
            init_cursor: 0,
            init_branch_name: String::new(),
            init_remote_url: String::new(),
            
            // Multiline Commit Modal
            show_commit_modal: false,
            commit_message_lines: vec![String::new()],
            commit_cursor_row: 0,
            commit_cursor_col: 0,
            commit_modal_scroll: 0,

            // Push/Pull/Remove Confirmations
            show_confirm_push: false,
            show_confirm_pull: false,
            show_confirm_remove: false,

            // Update notification
            show_update_modal: false,
            latest_version: String::new(),

            // Credentials Handler
            session_id: format!("{:.3}", std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs_f64()),
            tx: None,
            show_credentials_modal: false,
            credentials_prompt: String::new(),
            credentials_input: String::new(),
            credentials_cursor: 0,
            credentials_mask: false,
        };

        if config.language.is_empty() {
            state.setup_step = 1;
            state.status_message = "Welcome! Please configure Git Hero.".to_string();
        } else {
            state.refresh_git_status();
        }

        state
    }

    pub fn refresh_git_status(&mut self) {
        self.is_git_repo = git::is_inside_work_tree();
        if self.is_git_repo {
            // Single snapshot replaces 6+ separate git invocations.
            let ahead = match git::status_snapshot() {
                Ok(snap) => {
                    self.branch = if snap.branch == "(detached)" {
                        String::new()
                    } else {
                        snap.branch
                    };
                    // `upstream` is `remote/branch` — keep just the remote.
                    self.remote = snap
                        .upstream
                        .split('/')
                        .next()
                        .filter(|s| !s.is_empty())
                        .unwrap_or("origin")
                        .to_string();
                    self.behind = snap.behind;
                    self.ahead = snap.ahead;
                    self.files = snap.files.iter().map(file_snapshot_to_git_file).collect();
                    self.rebuild_file_tree();
                    self.ahead as usize
                }
                Err(e) => {
                    log::log_debug(&format!("status_snapshot failed: {e}"));
                    self.branch.clear();
                    self.remote.clear();
                    self.behind = 0;
                    self.ahead = 0;
                    self.files.clear();
                    self.flat_entries.clear();
                    self.status_message = format!("git status error: {e}");
                    self.commits.clear();
                    self.update_diff_content();
                    return;
                }
            };

            // One log call, then derive per-commit `pushed` from the ahead
            // count: the first `ahead` commits (newest) are unpushed.
            self.commits = match git::log_snapshot(15) {
                Ok(entries) => entries
                    .into_iter()
                    .enumerate()
                    .map(|(i, entry)| GitCommit {
                        hash: entry.hash,
                        date: entry.date,
                        subject: entry.subject,
                        pushed: i >= ahead,
                    })
                    .collect(),
                Err(e) => {
                    log::log_debug(&format!("log_snapshot failed: {e}"));
                    Vec::new()
                }
            };

            if !self.files.is_empty() && self.selected_file_idx >= self.files.len() {
                self.selected_file_idx = 0;
            }
            if !self.commits.is_empty() && self.selected_commit_idx >= self.commits.len() {
                self.selected_commit_idx = 0;
            }
            self.update_diff_content();
        } else {
            self.branch.clear();
            self.remote.clear();
            self.behind = 0;
            self.ahead = 0;
            self.files.clear();
            self.commits.clear();
            self.active_diff.clear();
            self.status_message = "Warning: Not a Git repository.".to_string();
        }
    }

    pub fn check_git_changes(&mut self) {
        if !self.is_git_repo {
            return;
        }
        let git_index = std::path::Path::new(".git/index");
        if let Ok(meta) = std::fs::metadata(git_index)
            && let Ok(mtime) = meta.modified()
        {
            match self.last_git_mtime {
                Some(prev) if prev == mtime => {
                    // No changes
                }
                _ => {
                    self.last_git_mtime = Some(mtime);
                    self.refresh_git_status();
                }
            }
        }
    }

    pub fn rebuild_file_tree(&mut self) {
        self.flat_entries.clear();

        if self.files.is_empty() {
            self.flat_idx = 0;
            return;
        }

        let mut indices: Vec<usize> = (0..self.files.len()).collect();
        indices.sort_by(|&a, &b| self.files[a].path.cmp(&self.files[b].path));

        for fi in indices {
            self.flat_entries.push(FlatEntry { file_idx: fi });
        }

        if self.flat_idx >= self.flat_entries.len() {
            self.flat_idx = self.flat_entries.len().saturating_sub(1);
        }

        if let Some(entry) = self.flat_entries.get(self.flat_idx)
            && entry.file_idx < self.files.len()
        {
            self.selected_file_idx = entry.file_idx;
        }
    }

    pub fn update_diff_content(&mut self) {
        // Phase 2.5: derive a cache key from the *inputs* to the diff
        // computation. Re-asking for the same key skips git entirely —
        // important for the auto-refresh loop (2s tick) when selection is
        // unchanged.
        let key = self.diff_key();

        if let Some((cached_key, _)) = &self.diff_cache
            && *cached_key == key
        {
            return; // Cache hit: active_diff already correct.
        }

        // Cache miss: run git (or read the untracked file) and store the result.
        self.active_diff = self.compute_diff(&key);
        self.diff_cache = Some((key, self.active_diff.clone()));

        // Invalidate the line cache — it was tied to the previous content.
        self.cached_diff_content.clear();
        self.cached_diff_lines.clear();
        self.cached_diff_width = 0;
        self.cached_diff_theme_name = None;
    }

    /// Compute the cache key for the current selection. See [`DiffKey`].
    fn diff_key(&self) -> DiffKey {
        if self.focus_pane == "commits"
            && !self.commits.is_empty()
            && self.selected_commit_idx < self.commits.len()
        {
            DiffKey::Commit(self.commits[self.selected_commit_idx].hash.clone())
        } else if !self.files.is_empty() && self.selected_file_idx < self.files.len() {
            let file = &self.files[self.selected_file_idx];
            DiffKey::File {
                path: file.path.clone(),
                staged: file.staged,
                status: file.status.clone(),
            }
        } else {
            DiffKey::Empty
        }
    }

    /// Actually run git (or read the file) for the given key. The only place
    /// `git diff` / `git show` should be invoked from.
    fn compute_diff(&self, key: &DiffKey) -> String {
        match key {
            DiffKey::Commit(hash) => git::run_git(&["show", "--name-status", "--format=", hash])
                .map(|out| {
                    let entries: Vec<String> = out
                        .lines()
                        .filter(|l| !l.is_empty())
                        .map(|l| l.replace('\t', "  "))
                        .collect();
                    if entries.is_empty() {
                        "(no files changed)".to_string()
                    } else {
                        entries.join("\n")
                    }
                })
                .unwrap_or_else(|e| format!("Error showing commit: {}", e)),
            DiffKey::File { path, staged, status } => {
                if status == "??" {
                    return fs::read_to_string(path)
                        .map(|content| {
                            let lines: Vec<&str> = content.split('\n').collect();
                            if lines.len() > 100 {
                                let mut truncated = lines[..100].join("\n");
                                truncated.push_str("\n... (truncated)");
                                truncated
                            } else {
                                content
                            }
                        })
                        .unwrap_or_else(|e| format!("Error reading untracked file: {}", e));
                }
                let mut diff_output = String::new();
                if *staged
                    && let Ok(out) = git::run_git(&["diff", "--cached", "--", path])
                    && !out.is_empty()
                {
                    diff_output.push_str(&format!("── Staged changes ──\n{}\n", out));
                }
                if let Ok(out) = git::run_git(&["diff", "--", path])
                    && !out.is_empty()
                {
                    if !diff_output.is_empty() {
                        diff_output.push_str("── Unstaged changes ──\n");
                    }
                    diff_output.push_str(&out);
                }
                if diff_output.is_empty() {
                    "No changes.".to_string()
                } else {
                    diff_output
                }
            }
            DiffKey::Empty => "Working directory clean.".to_string(),
        }
    }

    /// Invalidate the diff cache when `active_diff` is set directly by a
    /// command (e.g. `/branch` or `/config` writing the branch/config list
    /// into the diff panel).
    pub fn invalidate_diff_cache(&mut self) {
        self.diff_cache = None;
        self.cached_diff_content.clear();
        self.cached_diff_lines.clear();
        self.cached_diff_width = 0;
        self.cached_diff_theme_name = None;
    }

    pub fn get_cached_diff_lines(&mut self, width: u16) -> Vec<ratatui::text::Line<'static>> {
        // Phase 2.8: include theme.name in the key so theme changes force a
        // re-render even when `active_diff` content is identical.
        let needs_recompute = self.cached_diff_content != self.active_diff
            || self.cached_diff_width != width
            || self.cached_diff_theme_name != Some(self.theme.name);

        if needs_recompute {
            log::log_debug(&format!(
                "Diff cache miss: {} bytes, width={}",
                self.active_diff.len(),
                width
            ));
            self.cached_diff_content = self.active_diff.clone();
            self.cached_diff_width = width;
            self.cached_diff_theme_name = Some(self.theme.name);
            self.cached_diff_lines = super::rendering::render_diff_side_by_side(
                &self.active_diff,
                width,
                &self.theme,
            );
            log::log_debug(&format!("Diff cached: {} lines", self.cached_diff_lines.len()));
        }
        self.cached_diff_lines.clone()
    }

    pub fn toggle_stage_file(&mut self, idx: usize) {
        if idx >= self.files.len() {
            return;
        }
        let file = &self.files[idx];
        let args = if file.staged {
            vec!["restore", "--staged", "--", &file.path]
        } else {
            vec!["add", "--", &file.path]
        };
        let _ = git::run_git(&args);
        self.refresh_git_status();
    }

    pub fn get_icon_str(&self, key: &str) -> &'static str {
        icons::lookup(self.nerd_font, key)
    }

    /// Check whether a newer version of Git Hero is available on GitHub.
    /// If one exists (and the user hasn't skipped it), sets
    /// `show_update_modal` so the TUI can display the prompt on the next
    /// frame. Silently ignores network/git errors — the update check is
    /// a nice-to-have, not a requirement.
    pub fn check_for_updates(&mut self) {
        let current = crate::version::PKG_VERSION;
        // Skip if current version is not a release semver (e.g., debug builds).
        let current_ver = crate::version::Version::parse(current);
        if current_ver.is_none() {
            return;
        }

        match crate::git::check_latest_version() {
            Ok(version) => {
                if version == current {
                    return;
                }
                // Check if the user already chose to skip this version.
                if let Ok(cfg) = crate::config::load_config()
                    && cfg.skipped_version.as_deref() == Some(&version)
                {
                    return;
                }
                self.show_update_modal = true;
                self.latest_version = version;
            }
            Err(e) => {
                log::log_debug(&format!("Update check failed: {e}"));
            }
        }
    }

    /// Returns `true` if any modal is currently on top of the dashboard.
    /// Modals are mutually exclusive (only one shown at a time), but the
    /// renderer still asks "is *any* modal up?" to decide whether to
    /// paint the dimming overlay and hide the mini-console.
    pub fn has_active_modal(&self) -> bool {
        self.show_theme_modal
            || self.show_help_modal
            || self.show_docs_modal
            || self.show_commit_modal
            || self.show_confirm_push
            || self.show_confirm_pull
            || self.show_confirm_remove
            || self.show_credentials_modal
            || self.show_update_modal
    }
}

/// Convert a `git::FileSnapshot` (from the new v2-porcelain parser) into
/// the legacy `GitFile` shape the UI still consumes.
fn file_snapshot_to_git_file(fs: &git::FileSnapshot) -> GitFile {
    GitFile {
        path: fs.path.clone(),
        staged: fs.staged(),
        status: fs.status(),
    }
}
