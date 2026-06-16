// ── Application State ──────────────────────────────────────────────────
// Data models and business logic for the Git Hero TUI

use std::fs;
use std::time::SystemTime;

use crate::config::{load_config, Config};
use crate::git;
use crate::i18n::translate;
use crate::theme::{get_theme_by_name, get_themes, Theme};

#[derive(Clone)]
pub struct GitFile {
    pub path: String,
    pub staged: bool,
    pub status: String,
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
    pub behind: i32,
    pub ahead: i32,

    // ── Config State ───────────────────────────────────────────────
    pub theme: Theme,
    pub language: String,
    pub nerd_font: bool,
    pub status_message: String,
    pub fetching: bool,

    // ── Interactive Lists ──────────────────────────────────────────
    pub files: Vec<GitFile>,
    pub selected_file_idx: usize,
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
    cached_diff_content: String,
    cached_diff_lines: Vec<ratatui::text::Line<'static>>,
    cached_diff_width: u16,  // Width used for side-by-side rendering

    // ── Command Input ──────────────────────────────────────────────
    pub input_value: String,
    pub input_cursor_pos: usize,
    pub show_input: bool,
    pub suggestions: Vec<String>,
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


}

impl AppState {
    pub fn new() -> Self {
        let config = load_config().unwrap_or(Config {
            language: "en".to_string(),
            nerd_font: false,
            theme: "Tokyo Night".to_string(),
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
            self.branch = git::get_current_branch().unwrap_or_else(|_| "".to_string());
            self.remote = git::get_remote(&self.branch);
            self.behind = git::get_commits_behind(&self.remote, &self.branch);
            self.ahead = git::get_commits_ahead(&self.remote, &self.branch);

            self.files = self.get_changed_files();
            self.commits = self.get_recent_commits();

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

    /// Check if .git has changed since last check. If so, refresh the UI.
    /// Safe to call on every frame — only refreshes when actually needed.
    pub fn check_git_changes(&mut self) {
        if !self.is_git_repo {
            return;
        }
        let git_index = std::path::Path::new(".git/index");
        if let Ok(meta) = std::fs::metadata(git_index) {
            if let Ok(mtime) = meta.modified() {
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
    }

    // ── Git operations ──────────────────────────────────────────

    fn get_changed_files(&self) -> Vec<GitFile> {
        let mut files = Vec::new();
        if let Ok(out) = git::run_git(&["status", "--porcelain"]) {
            for line in out.split('\n') {
                if line.len() < 4 {
                    continue;
                }
                // Format: XY path (X=index/staged, Y=working tree)
                let x = line.chars().next().unwrap_or(' ');  // staged status
                let y = line.chars().nth(1).unwrap_or(' ');  // working tree status
                let path = line[3..].to_string();
                
                // A file is staged if X is not space and not '?'
                let staged = x != ' ' && x != '?';
                
                // Determine the display status
                let status = if x != ' ' && x != '?' {
                    // Has staged changes - show the staged status
                    x.to_string()
                } else if y != ' ' && y != '?' {
                    // Only working tree changes
                    y.to_string()
                } else if x == '?' || y == '?' {
                    "?".to_string()
                } else {
                    format!("{}{}", x, y)
                };
                
                files.push(GitFile {
                    path,
                    staged,
                    status,
                });
            }
        }
        files
    }

    fn get_recent_commits(&self) -> Vec<GitCommit> {
        let mut commits = Vec::new();

        // Get unpushed commit hashes (commits ahead of upstream)
        let remote = &self.remote;
        let branch = &self.branch;
        let mut unpushed: Vec<String> = Vec::new();
        if !remote.is_empty() && !branch.is_empty() {
            if let Ok(out) = git::run_git(&[
                "log", "--oneline", &format!("{}/{}..HEAD", remote, branch),
            ]) {
                for line in out.split('\n') {
                    if let Some(hash) = line.split(' ').next() {
                        unpushed.push(hash.to_string());
                    }
                }
            }
        }

        if let Ok(out) =
            git::run_git(&["log", "-n", "15", "--pretty=format:%h|%ar|%an|%s"])
        {
            for line in out.split('\n') {
                let parts: Vec<&str> = line.split('|').collect();
                if parts.len() >= 4 {
                    let hash = parts[0].to_string();
                    let pushed = !unpushed.contains(&hash);
                    commits.push(GitCommit {
                        hash,
                        date: parts[1].to_string(),
                        subject: parts[3].to_string(),
                        pushed,
                    });
                }
            }
        }
        commits
    }

    pub fn update_diff_content(&mut self) {
        if self.focus_pane == "commits"
            && !self.commits.is_empty()
            && self.selected_commit_idx < self.commits.len()
        {
            let hash = &self.commits[self.selected_commit_idx].hash;
            self.active_diff = match git::run_git(&["show", "--stat", "--patch", hash]) {
                Ok(out) => out,
                Err(e) => format!("Error showing commit: {}", e),
            };
        } else if !self.files.is_empty() && self.selected_file_idx < self.files.len() {
            let file = &self.files[self.selected_file_idx];
            if file.status == "??" {
                self.active_diff = match fs::read_to_string(&file.path) {
                    Ok(content) => {
                        let lines: Vec<&str> = content.split('\n').collect();
                        if lines.len() > 100 {
                            let mut truncated = lines[..100].join("\n");
                            truncated.push_str("\n... (truncated)");
                            truncated
                        } else {
                            content
                        }
                    }
                    Err(e) => format!("Error reading untracked file: {}", e),
                };
            } else {
                let args = if file.staged {
                    vec!["diff", "--cached", "--", &file.path]
                } else {
                    vec!["diff", "--", &file.path]
                };
                self.active_diff = match git::run_git(&args) {
                    Ok(out) => {
                        if out.is_empty() {
                            "No changes.".to_string()
                        } else {
                            out
                        }
                    }
                    Err(e) => format!("Error loading diff: {}", e),
                };
            }
        } else {
            self.active_diff = "Working directory clean.".to_string();
        }
        // Invalidate cache when diff content changes
        self.cached_diff_content.clear();
        self.cached_diff_lines.clear();
        self.cached_diff_width = 0;
    }

    /// Returns cached side-by-side diff lines, recomputing only if diff or width changed.
    pub fn get_cached_diff_lines(&mut self, width: u16) -> Vec<ratatui::text::Line<'static>> {
        let needs_recompute = self.cached_diff_content != self.active_diff 
                           || self.cached_diff_width != width;
        
        if needs_recompute {
            crate::log_debug(&format!("Diff cache miss: {} bytes, width={}", self.active_diff.len(), width));
            self.cached_diff_content = self.active_diff.clone();
            self.cached_diff_width = width;
            self.cached_diff_lines = super::rendering::render_diff_side_by_side(
                &self.active_diff, width, &self.theme
            );
            crate::log_debug(&format!("Diff cached: {} lines", self.cached_diff_lines.len()));
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

    // ── Icons ───────────────────────────────────────────────────

    pub fn get_icon_str(&self, key: &str) -> &'static str {
        if self.nerd_font {
            match key {
                "branch" => "\u{E725}",
                "dir" => "\u{F07C}",
                "fetch" => "\u{F021}",
                "commit" => "\u{F417}",
                "mod" => "\u{F448}",
                "add" => "\u{F067}",
                "del" => "\u{F057}",
                "untracked" => "\u{F128}",
                "ok" => "\u{2714}",
                "warn" => "\u{26A0}",
                _ => "",
            }
        } else {
            match key {
                "branch" => "\u{2A}",
                "dir" => "\u{2F}",
                "fetch" => "\u{2193}",
                "commit" => "\u{23}",
                "mod" => "\u{2192}",
                "add" => "\u{2B}",
                "del" => "\u{2212}",
                "untracked" => "\u{3F}",
                "ok" => "\u{2713}",
                "warn" => "\u{21}",
                _ => "",
            }
        }
    }

    // ── Commands ────────────────────────────────────────────────

    pub fn execute_command(&mut self, input: &str) {
        if input.starts_with("/cd ") {
            let path = &input[4..];
            let resolved = expand_path(path);
            if std::env::set_current_dir(resolved).is_ok() {
                if let Ok(new_cwd) = std::env::current_dir() {
                    self.cwd = new_cwd.to_string_lossy().into_owned();
                    self.selected_file_idx = 0;
                    self.selected_commit_idx = 0;
                    self.diff_scroll_offset = 0;
                    self.refresh_git_status();
                }
            } else {
                self.status_message = format!("Error changing directory to: {}", path);
            }
            return;
        }

        if input == "/fetch" {
            if !self.is_git_repo {
                self.status_message = translate(&self.language, "status_not_git");
                return;
            }
            self.console_running = true;
            self.console_visible = true;
            self.console_scroll = 0;
            self.console_output = format!("$ git fetch {} {}\n", self.remote, self.branch);
            self.status_message = translate(&self.language, "status_fetching");
            let remote = self.remote.clone();
            let branch = self.branch.clone();
            match git::run_git_verbose(&["fetch", &remote, &branch]) {
                Ok(out) => {
                    self.console_output.push_str(&out);
                    self.console_output.push_str("\n✓ Fetch complete.");
                }
                Err(e) => {
                    self.console_output.push_str(&e);
                }
            }
            self.console_running = false;
            self.fetching = false;
            self.refresh_git_status();
            return;
        }

        if input == "/pull" {
            if !self.is_git_repo {
                self.status_message = translate(&self.language, "status_not_git");
                return;
            }
            self.console_running = true;
            self.console_visible = true;
            self.console_scroll = 0;
            self.console_output = format!("$ git pull {} {}\n", self.remote, self.branch);
            self.status_message = translate(&self.language, "status_pulling");
            let remote = self.remote.clone();
            let branch = self.branch.clone();
            match git::run_git_verbose(&["pull", &remote, &branch]) {
                Ok(out) => {
                    self.console_output.push_str(&out);
                    self.console_output.push_str("\n✓ Pull complete.");
                }
                Err(e) => {
                    self.console_output.push_str(&e);
                }
            }
            self.console_running = false;
            self.refresh_git_status();
            return;
        }

        if input == "/push" {
            if !self.is_git_repo {
                self.status_message = translate(&self.language, "status_not_git");
                return;
            }
            self.console_running = true;
            self.console_visible = true;
            self.console_scroll = 0;
            self.console_output = format!("$ git push {} {}\n", self.remote, self.branch);
            self.status_message = translate(&self.language, "status_pushing");
            let remote = self.remote.clone();
            let branch = self.branch.clone();
            match git::run_git_verbose(&["push", &remote, &branch]) {
                Ok(out) => {
                    self.console_output.push_str(&out);
                    self.console_output.push_str("\n✓ Push complete.");
                }
                Err(e) => {
                    self.console_output.push_str(&e);
                }
            }
            self.console_running = false;
            self.refresh_git_status();
            return;
        }

        if input.starts_with("/commit ") {
            if !self.is_git_repo {
                self.status_message = translate(&self.language, "status_not_git");
                return;
            }
            let msg = &input[8..];
            if msg.is_empty() {
                self.status_message = "Error: commit message empty.".to_string();
                return;
            }
            let has_staged = self.files.iter().any(|f| f.staged);
            if !has_staged {
                let _ = git::git_add_all();
            }
            if let Err(e) = git::git_commit(msg) {
                self.status_message = format!("Error committing: {}", e);
            } else {
                self.selected_file_idx = 0;
                self.selected_commit_idx = 0;
                self.diff_scroll_offset = 0;
                self.refresh_git_status();
                self.status_message = translate(&self.language, "status_commit_success");
            }
            return;
        }

        if input == "/themes" {
            self.show_theme_modal = true;
            self.saved_theme = self.theme.name.to_string();
            self.theme_cursor = 0;
            let themes = get_themes();
            for (i, t) in themes.iter().enumerate() {
                if t.name == self.theme.name {
                    self.theme_cursor = i;
                    break;
                }
            }
            return;
        }

        if input == "/help" {
            self.show_help_modal = true;
            return;
        }

        if input == "/docs" {
            self.show_docs_modal = true;
            return;
        }

        // ── Stage all ──────────────────────────────────────────
        if input == "/stage-all" {
            if !self.is_git_repo { self.status_message = translate(&self.language, "status_not_git"); return; }
            match git::git_stage_all() {
                Ok(()) => { self.refresh_git_status(); self.status_message = translate(&self.language, "status_stage_all_ok"); }
                Err(e) => { self.status_message = format!("Error staging all: {}", e); }
            }
            return;
        }

        // ── Unstage all ────────────────────────────────────────
        if input == "/unstage-all" {
            if !self.is_git_repo { self.status_message = translate(&self.language, "status_not_git"); return; }
            match git::git_unstage_all() {
                Ok(()) => { self.refresh_git_status(); self.status_message = translate(&self.language, "status_unstage_all_ok"); }
                Err(e) => { self.status_message = format!("Error unstaging all: {}", e); }
            }
            return;
        }

        // ── Undo last commit ───────────────────────────────────
        if input == "/undo-commit" {
            if !self.is_git_repo { self.status_message = translate(&self.language, "status_not_git"); return; }
            self.status_message = translate(&self.language, "status_undo_commit");
            match git::git_reset_soft(1) {
                Ok(()) => { self.refresh_git_status(); self.status_message = translate(&self.language, "status_undo_commit_ok"); }
                Err(e) => { self.status_message = format!("Error undoing commit: {}", e); }
            }
            return;
        }

        // ── Remove .git repo ───────────────────────────────────
        if input == "/remove-repo" {
            if !self.is_git_repo { self.status_message = translate(&self.language, "status_not_git"); return; }
            match git::git_remove_repo() {
                Ok(()) => { self.refresh_git_status(); self.status_message = translate(&self.language, "status_remove_ok"); }
                Err(e) => { self.status_message = format!("Error removing repo: {}", e); }
            }
            return;
        }

        // ── Remote set-url ─────────────────────────────────────
        if input.starts_with("/remote ") {
            if !self.is_git_repo { self.status_message = translate(&self.language, "status_not_git"); return; }
            let url = &input[8..];
            if url.is_empty() {
                self.status_message = "Usage: /remote <url> - adds or updates origin remote".to_string();
                return;
            }
            match git::git_remote_set_url("origin", url) {
                Ok(()) => { self.refresh_git_status(); self.status_message = format!("Remote origin → {}", url); }
                Err(e) => { self.status_message = format!("Error setting remote: {}", e); }
            }
            return;
        }

        // ── Branch management ─────────────────────────────────
        if input == "/branch" || input == "/branches" {
            if !self.is_git_repo { self.status_message = translate(&self.language, "status_not_git"); return; }
            match git::git_branch_list() {
                Ok(list) => { self.active_diff = list; self.status_message = translate(&self.language, "status_branch_list"); }
                Err(e) => { self.status_message = format!("Error listing branches: {}", e); }
            }
            return;
        }

        if input.starts_with("/branch -d ") {
            if !self.is_git_repo { self.status_message = translate(&self.language, "status_not_git"); return; }
            let name = &input[11..];
            match git::git_branch_delete(name) {
                Ok(()) => { self.refresh_git_status(); self.status_message = format!("Branch '{}' deleted.", name); }
                Err(e) => { self.status_message = format!("Error deleting branch: {}", e); }
            }
            return;
        }

        if input.starts_with("/branch -s ") || input.starts_with("/switch ") {
            if !self.is_git_repo { self.status_message = translate(&self.language, "status_not_git"); return; }
            let name = if input.starts_with("/branch -s ") { &input[11..] } else { &input[8..] };
            match git::git_branch_switch(name) {
                Ok(()) => { self.refresh_git_status(); self.status_message = format!("Switched to branch: {}", name); }
                Err(_) => {
                    // Try creating the branch if switch fails
                    match git::git_branch_create_and_switch(name) {
                        Ok(()) => { self.refresh_git_status(); self.status_message = format!("Created and switched to branch: {}", name); }
                        Err(e2) => { self.status_message = format!("Error switching branch: {}", e2); }
                    }
                }
            }
            return;
        }

        if input.starts_with("/branch ") && !input.contains(" -") {
            if !self.is_git_repo { self.status_message = translate(&self.language, "status_not_git"); return; }
            let name = &input[8..];
            match git::git_branch_create_and_switch(name) {
                Ok(()) => { self.refresh_git_status(); self.status_message = format!("Created and switched to branch: {}", name); }
                Err(e) => { self.status_message = format!("Error creating branch: {}", e); }
            }
            return;
        }

        // ── Git config ─────────────────────────────────────────
        if input == "/config" {
            if !self.is_git_repo { self.status_message = translate(&self.language, "status_not_git"); return; }
            match git::git_config_list() {
                Ok(list) => { self.active_diff = list; self.status_message = translate(&self.language, "status_config_list"); }
                Err(e) => { self.status_message = format!("Error listing config: {}", e); }
            }
            return;
        }

        if input.starts_with("/config-global ") {
            let rest = &input[15..];
            let parts: Vec<&str> = rest.splitn(2, ' ').collect();
            if parts.len() == 2 {
                match git::git_config_set_global(parts[0], parts[1]) {
                    Ok(()) => { self.status_message = format!("Global config set: {} = {}", parts[0], parts[1]); }
                    Err(e) => { self.status_message = format!("Error setting global config: {}", e); }
                }
            } else if parts.len() == 1 {
                match git::git_config_get(parts[0]) {
                    Ok(val) => { self.status_message = format!("Global: {} = {}", parts[0], val); }
                    Err(e) => { self.status_message = format!("Error reading config: {}", e); }
                }
            } else {
                self.status_message = "Usage: /config-global <key> [value]".to_string();
            }
            return;
        }

        if input.starts_with("/config ") {
            let rest = &input[8..];
            let parts: Vec<&str> = rest.splitn(2, ' ').collect();
            if parts.len() == 2 {
                match git::git_config_set_local(parts[0], parts[1]) {
                    Ok(()) => { self.status_message = format!("Config set: {} = {}", parts[0], parts[1]); }
                    Err(e) => { self.status_message = format!("Error setting config: {}", e); }
                }
            } else if parts.len() == 1 {
                match git::git_config_get(parts[0]) {
                    Ok(val) => { self.status_message = format!("Local: {} = {}", parts[0], val); }
                    Err(_) => {
                        // Try global
                        match git::run_git(&["config", "--global", parts[0]]) {
                            Ok(val) if !val.is_empty() => { self.status_message = format!("Global: {} = {}", parts[0], val.trim()); }
                            _ => { self.status_message = format!("Config key '{}' not found.", parts[0]); }
                        }
                    }
                }
            } else {
                self.status_message = "Usage: /config <key> [value]".to_string();
            }
            return;
        }

        // ── Stash ───────────────────────────────────────────────
        if input == "/stash" {
            if !self.is_git_repo { self.status_message = translate(&self.language, "status_not_git"); return; }
            match git::git_stash() {
                Ok(()) => { self.refresh_git_status(); self.status_message = translate(&self.language, "status_stash_ok"); }
                Err(e) => { self.status_message = format!("Error stashing: {}", e); }
            }
            return;
        }

        if input == "/stash-pop" {
            if !self.is_git_repo { self.status_message = translate(&self.language, "status_not_git"); return; }
            match git::git_stash_pop() {
                Ok(()) => { self.refresh_git_status(); self.status_message = translate(&self.language, "status_stash_pop_ok"); }
                Err(e) => { self.status_message = format!("Error popping stash: {}", e); }
            }
            return;
        }

        self.status_message = format!("Unknown command: {}. Type /help.", input);
    }

    pub fn update_suggestions(&mut self) {
        let val = &self.input_value;
        if val.starts_with("/cd ") {
            self.suggestions = get_directory_suggestions(val);
        } else if val.starts_with('/') {
            self.suggestions = get_command_suggestions(val);
        } else {
            self.suggestions.clear();
        }
        if !self.suggestions.is_empty() && self.active_sug >= self.suggestions.len() {
            self.active_sug = 0;
        }
    }
}

// ── Path & Suggestions Helpers ─────────────────────────────────────────

pub fn expand_path(path: &str) -> String {
    if path.starts_with('~') {
        if let Some(home) = dirs::home_dir() {
            let home_str = home.to_string_lossy().into_owned();
            return path.replacen('~', &home_str, 1);
        }
    }
    path.to_string()
}

use std::path::Path;

pub fn get_directory_suggestions(input: &str) -> Vec<String> {
    if !input.starts_with("/cd ") {
        return Vec::new();
    }
    let path_arg = &input[4..];
    let resolved_path = expand_path(path_arg);

    let (search_dir, prefix) = if path_arg.is_empty() {
        (".", "")
    } else if path_arg == "~" {
        (&resolved_path as &str, "")
    } else if path_arg.ends_with('/') || path_arg.ends_with('\\') {
        (&resolved_path as &str, "")
    } else {
        let path = Path::new(&resolved_path);
        let parent = path.parent().and_then(|p| p.to_str()).unwrap_or(".");
        let file_name = path.file_name().and_then(|f| f.to_str()).unwrap_or("");
        (parent, file_name)
    };

    let mut suggestions = Vec::new();
    if let Ok(entries) = fs::read_dir(search_dir) {
        for entry in entries.flatten() {
            if let Ok(file_type) = entry.file_type() {
                if file_type.is_dir() {
                    let name = entry.file_name().to_string_lossy().into_owned();
                    if prefix.is_empty()
                        || name.to_lowercase().starts_with(&prefix.to_lowercase())
                    {
                        let mut base_path = path_arg.to_string();
                        if !prefix.is_empty() {
                            base_path =
                                path_arg[..path_arg.len() - prefix.len()].to_string();
                        }
                        if !base_path.is_empty()
                            && !base_path.ends_with('/')
                            && !base_path.ends_with('\\')
                        {
                            base_path.push('/');
                        }
                        suggestions.push(format!("/cd {}{}/", base_path, name));
                    }
                }
            }
        }
    }
    suggestions.truncate(5);
    suggestions
}

pub fn get_command_suggestions(input: &str) -> Vec<String> {
    let commands = vec![
        "/fetch".to_string(),
        "/pull".to_string(),
        "/push".to_string(),
        "/commit ".to_string(),
        "/stage-all".to_string(),
        "/unstage-all".to_string(),
        "/undo-commit".to_string(),
        "/remove-repo".to_string(),
        "/remote ".to_string(),
        "/branch ".to_string(),
        "/branches".to_string(),
        "/switch ".to_string(),
        "/config ".to_string(),
        "/config-global ".to_string(),
        "/stash".to_string(),
        "/stash-pop".to_string(),
        "/cd ".to_string(),
        "/themes".to_string(),
        "/help".to_string(),
        "/docs".to_string(),
        "/quit".to_string(),
    ];
    if input.is_empty() || input == "/" {
        return commands;
    }
    commands
        .into_iter()
        .filter(|c| c.starts_with(input))
        .collect()
}
