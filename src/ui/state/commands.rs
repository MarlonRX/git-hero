use crate::git;
use crate::i18n::translate;
use crate::theme::get_themes;
use crate::ui::state::AppState;
use super::suggestions::expand_path;

impl AppState {
    pub fn run_pull(&mut self) {
        if let Some(ref tx) = self.tx {
            self.console_running = true;
            self.console_visible = true;
            self.console_scroll = 0;
            self.console_output = format!("$ git pull {} {}\n", self.remote, self.branch);
            self.status_message = translate(&self.language, "status_pulling");
            let remote = self.remote.clone();
            let branch = self.branch.clone();
            git::run_git_async(
                vec!["pull".to_string(), remote, branch],
                self.session_id.clone(),
                tx.clone(),
            );
        }
    }

    pub fn run_push(&mut self) {
        if let Some(ref tx) = self.tx {
            self.console_running = true;
            self.console_visible = true;
            self.console_scroll = 0;
            self.console_output = format!("$ git push {} {}\n", self.remote, self.branch);
            self.status_message = translate(&self.language, "status_pushing");
            let remote = self.remote.clone();
            let branch = self.branch.clone();
            git::run_git_async(
                vec!["push".to_string(), remote, branch],
                self.session_id.clone(),
                tx.clone(),
            );
        }
    }

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
            if self.console_running {
                self.status_message = "A command is already running.".to_string();
                return;
            }
            self.show_confirm_pull = true;
            return;
        }

        if input == "/push" {
            if !self.is_git_repo {
                self.status_message = translate(&self.language, "status_not_git");
                return;
            }
            if self.console_running {
                self.status_message = "A command is already running.".to_string();
                return;
            }
            self.show_confirm_push = true;
            return;
        }

        if input == "/commit" {
            if !self.is_git_repo {
                self.status_message = translate(&self.language, "status_not_git");
                return;
            }
            self.show_commit_modal = true;
            self.commit_message_lines = vec![String::new()];
            self.commit_cursor_row = 0;
            self.commit_cursor_col = 0;
            self.commit_modal_scroll = 0;
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
}
