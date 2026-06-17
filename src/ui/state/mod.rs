use std::fs;
use std::time::SystemTime;

use crate::config::{load_config, Config};
use crate::git;
use crate::theme::{get_theme_by_name, Theme};

pub mod commands;
pub mod suggestions;

#[derive(Debug, Clone)]
pub enum TuiMessage {
    ConsoleOutput(String),
    CommandFinished(Result<(), String>),
}

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

    // ── Commit Message Editor ─────────────────────────────
    pub show_commit_modal: bool,
    pub commit_message_lines: Vec<String>,
    pub commit_cursor_row: usize,
    pub commit_cursor_col: usize,
    pub commit_modal_scroll: usize,

    // ── Push / Pull Confirmations ──────────────────────────────────
    pub show_confirm_push: bool,
    pub show_confirm_pull: bool,

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
            
            // Multiline Commit Modal
            show_commit_modal: false,
            commit_message_lines: vec![String::new()],
            commit_cursor_row: 0,
            commit_cursor_col: 0,
            commit_modal_scroll: 0,

            // Push/Pull Confirmations
            show_confirm_push: false,
            show_confirm_pull: false,

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

    fn get_changed_files(&self) -> Vec<GitFile> {
        let mut files = Vec::new();

        let staged_set: std::collections::HashSet<String> = git::run_git(
            &["diff", "--name-only", "--cached"]
        ).unwrap_or_default()
        .split('\n')
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect();

        if let Ok(out) = git::run_git(&["status", "--porcelain"]) {
            for line in out.split('\n') {
                if line.is_empty() {
                    continue;
                }
                let line = line.trim_end_matches(['\r', '\n']);
                if line.len() < 4 {
                    continue;
                }
                let bytes = line.as_bytes();
                let x = bytes[0] as char;
                let y = bytes[1] as char;
                let raw_path = if let Some(idx) = line.find(" -> ") {
                    &line[idx + 4..]
                } else {
                    &line[3..]
                };
                let path = raw_path.trim().to_string();

                let staged = staged_set.contains(&path);

                let status = if x == '?' && y == '?' {
                    "??".to_string()
                } else if staged && y != ' ' {
                    format!("{}{}", x, y)
                } else if staged {
                    x.to_string()
                } else if y != ' ' {
                    y.to_string()
                } else {
                    " ".to_string()
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
                Ok(out) => {
                    let lines: Vec<&str> = out.split('\n').collect();
                    const MAX_COMMIT_DIFF_LINES: usize = 500;
                    if lines.len() > MAX_COMMIT_DIFF_LINES {
                        let mut truncated = lines[..MAX_COMMIT_DIFF_LINES].join("\n");
                        truncated.push_str(&format!(
                            "\n... (truncated, {} more lines)",
                            lines.len() - MAX_COMMIT_DIFF_LINES
                        ));
                        truncated
                    } else {
                        out
                    }
                }
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
                let mut diff_output = String::new();
                
                if file.staged {
                    match git::run_git(&["diff", "--cached", "--", &file.path]) {
                        Ok(out) if !out.is_empty() => {
                            diff_output.push_str(&format!("── Staged changes ──\n{}\n", out));
                        }
                        _ => {}
                    }
                }
                
                match git::run_git(&["diff", "--", &file.path]) {
                    Ok(out) if !out.is_empty() => {
                        if !diff_output.is_empty() {
                            diff_output.push_str("── Unstaged changes ──\n");
                        }
                        diff_output.push_str(&out);
                    }
                    _ => {}
                }
                
                self.active_diff = if diff_output.is_empty() {
                    "No changes.".to_string()
                } else {
                    diff_output
                };
            }
        } else {
            self.active_diff = "Working directory clean.".to_string();
        }
        self.cached_diff_content.clear();
        self.cached_diff_lines.clear();
        self.cached_diff_width = 0;
    }

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
}
