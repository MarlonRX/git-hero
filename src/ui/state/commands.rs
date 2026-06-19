//! Slash-command dispatcher.
//!
//! `execute_command` is the single entry point used by the command bar
//! (`Enter` on `/foo`). It parses the input with [`Command::parse`] and
//! dispatches to a small `cmd_xxx` method on `AppState`.
//!
//! Two non-dispatch methods live here because they predate the refactor:
//! `run_pull` / `run_push` are called from the push/pull confirmation
//! modals (the user has *already* confirmed, so they bypass the dispatcher).

use crate::config::{load_config, save_config, Config};
use crate::git;
use crate::i18n::{translate, trf};
use crate::theme::get_themes;
use crate::ui::state::AppState;
use super::command::Command;
use super::suggestions::expand_path;

impl AppState {
    // ── Async helpers (unchanged) ─────────────────────────────────

    pub fn run_pull(&mut self) {
        if let Some(ref tx) = self.tx {
            self.console_running = true;
            self.console_visible = true;
            self.console_scroll = 0;
            self.console_output = format!("$ git pull {} {}\n", self.remote, self.branch);
            self.status_message = translate(&self.language, "status_pulling").into_owned();
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
            self.status_message = translate(&self.language, "status_pushing").into_owned();
            let remote = self.remote.clone();
            let branch = self.branch.clone();
            git::run_git_async(
                vec!["push".to_string(), remote, branch],
                self.session_id.clone(),
                tx.clone(),
            );
        }
    }

    // ── Dispatcher ───────────────────────────────────────────────

    /// Top-level entry point: parse + dispatch. Parse errors and unknown
    /// commands both produce a user-friendly `status_message`.
    pub fn execute_command(&mut self, input: &str) {
        match Command::parse(input) {
            Err(e) => {
                self.status_message = format!("{}: {} (input: {})", e.reason, e.input, e.reason);
                // Above is awkward — fall back to a simpler message:
                self.status_message = format!("{} ({})", e.reason, e.input);
            }
            Ok(Command::Unknown(input)) => {
                self.status_message = format!("Unknown command: {}. Type /help.", input);
            }
            Ok(cmd) => self.dispatch(cmd),
        }
    }

    fn dispatch(&mut self, cmd: Command) {
        match cmd {
            Command::Cd(path) => self.cmd_cd(&path),
            Command::Fetch => self.cmd_fetch(),
            Command::Pull => self.cmd_pull_prompt(),
            Command::Push => self.cmd_push_prompt(),
            Command::Commit => self.cmd_commit_modal(),
            Command::CommitMessage(msg) => self.cmd_commit_message(&msg),
            Command::Themes => self.cmd_themes(),
            Command::Help => self.show_help_modal = true,
            Command::Docs => self.show_docs_modal = true,
            Command::StageAll => self.cmd_stage_all(),
            Command::UnstageAll => self.cmd_unstage_all(),
            Command::UndoCommit => self.cmd_undo_commit(),
            Command::RemoveRepo => self.cmd_remove_repo(),
            Command::SetRemote(url) => self.cmd_set_remote(&url),
            Command::ListBranches => self.cmd_list_branches(),
            Command::DeleteBranch(name) => self.cmd_delete_branch(&name),
            Command::SwitchOrCreate(name) => self.cmd_switch_or_create(&name),
            Command::CreateBranch(name) => self.cmd_create_branch(&name),
            Command::ListConfig => self.cmd_list_config(),
            Command::ConfigLocal { key, value } => self.cmd_config_local(&key, value.as_deref()),
            Command::ConfigGlobal { key, value } => {
                self.cmd_config_global(&key, value.as_deref())
            }
            Command::Stash => self.cmd_stash(),
            Command::StashPop => self.cmd_stash_pop(),
            Command::SetLanguage(lang) => self.cmd_set_language(&lang),
            Command::Quit => {
                // No direct way to break the main loop from here; the
                // user can press `q`. `/quit` is a no-op for now.
            }
            Command::Unknown(_) => unreachable!("handled in execute_command"),
        }
    }

    // ── /cd ──────────────────────────────────────────────────────

    fn cmd_cd(&mut self, path: &str) {
        let resolved = expand_path(path);
        match std::env::set_current_dir(resolved.as_ref()) {
            Ok(()) => {
                if let Ok(new_cwd) = std::env::current_dir() {
                    self.cwd = new_cwd.to_string_lossy().into_owned();
                    self.selected_file_idx = 0;
                    self.selected_commit_idx = 0;
                    self.diff_scroll_offset = 0;
                    self.refresh_git_status();
                    self.status_message = trf(&self.language, "status_cd_ok", &[&self.cwd]);
                }
            }
            Err(_) => {
                self.status_message = trf(&self.language, "status_cd_err", &[path]);
            }
        }
    }

    // ── /fetch / /pull / /push ───────────────────────────────────

    fn cmd_fetch(&mut self) {
        if !self.require_repo() {
            return;
        }
        self.console_running = true;
        self.console_visible = true;
        self.console_scroll = 0;
        self.console_output = format!("$ git fetch {} {}\n", self.remote, self.branch);
        self.status_message = translate(&self.language, "status_fetching").into_owned();
        let remote = self.remote.clone();
        let branch = self.branch.clone();
        match git::run_git_verbose(&["fetch", &remote, &branch]) {
            Ok(out) => {
                self.console_output.push_str(&out);
                self.console_output.push('\n');
                self.console_output
                    .push_str(&translate(&self.language, "fetch_complete"));
            }
            Err(e) => self.console_output.push_str(&e),
        }
        self.console_running = false;
        self.fetching = false;
        self.refresh_git_status();
    }

    fn cmd_pull_prompt(&mut self) {
        if !self.require_repo() {
            return;
        }
        if self.console_running {
            self.status_message = translate(&self.language, "status_cmd_already_running")
                .into_owned();
            return;
        }
        self.show_confirm_pull = true;
    }

    fn cmd_push_prompt(&mut self) {
        if !self.require_repo() {
            return;
        }
        if self.console_running {
            self.status_message = translate(&self.language, "status_cmd_already_running")
                .into_owned();
            return;
        }
        self.show_confirm_push = true;
    }

    // ── /commit (modal + immediate) ─────────────────────────────

    fn cmd_commit_modal(&mut self) {
        if !self.require_repo() {
            return;
        }
        self.show_commit_modal = true;
        self.commit_message_lines = vec![String::new()];
        self.commit_cursor_row = 0;
        self.commit_cursor_col = 0;
        self.commit_modal_scroll = 0;
    }

    fn cmd_commit_message(&mut self, msg: &str) {
        if !self.require_repo() {
            return;
        }
        let msg = msg.trim();
        if msg.is_empty() {
            self.status_message = translate(&self.language, "status_err_commit_empty").into_owned();
            return;
        }
        if !self.files.iter().any(|f| f.staged) {
            let _ = git::git_add_all();
        }
        match git::git_commit(msg) {
            Ok(()) => {
                self.selected_file_idx = 0;
                self.selected_commit_idx = 0;
                self.diff_scroll_offset = 0;
                self.refresh_git_status();
                self.status_message = translate(&self.language, "status_commit_success").into_owned();
            }
            Err(e) => {
                self.status_message = trf(&self.language, "status_err_commit", &[&e]);
            }
        }
    }

    // ── /themes / /help / /docs ──────────────────────────────────

    fn cmd_themes(&mut self) {
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
    }

    // ── /stage-all / /unstage-all / /undo-commit / /remove-repo ─

    fn cmd_stage_all(&mut self) {
        if !self.require_repo() {
            return;
        }
        match git::git_stage_all() {
            Ok(()) => {
                self.refresh_git_status();
                self.status_message = translate(&self.language, "status_stage_all_ok").into_owned();
            }
            Err(e) => self.status_message = trf(&self.language, "status_err_stage", &[&e]),
        }
    }

    fn cmd_unstage_all(&mut self) {
        if !self.require_repo() {
            return;
        }
        match git::git_unstage_all() {
            Ok(()) => {
                self.refresh_git_status();
                self.status_message = translate(&self.language, "status_unstage_all_ok").into_owned();
            }
            Err(e) => self.status_message = trf(&self.language, "status_err_unstage", &[&e]),
        }
    }

    fn cmd_undo_commit(&mut self) {
        if !self.require_repo() {
            return;
        }
        self.status_message = translate(&self.language, "status_undo_commit").into_owned();
        match git::git_reset_soft(1) {
            Ok(()) => {
                self.refresh_git_status();
                self.status_message = translate(&self.language, "status_undo_commit_ok").into_owned();
            }
            Err(e) => self.status_message = trf(&self.language, "status_err_undo_commit", &[&e]),
        }
    }

    fn cmd_remove_repo(&mut self) {
        if !self.require_repo() {
            return;
        }
        // Phase 3.3: confirmation is mandatory (see handle_confirm_remove_key).
        self.show_confirm_remove = true;
    }

    // ── /remote <url> ────────────────────────────────────────────

    fn cmd_set_remote(&mut self, url: &str) {
        if !self.require_repo() {
            return;
        }
        if url.is_empty() {
            self.status_message = translate(&self.language, "status_usage_remote").into_owned();
            return;
        }
        match git::git_remote_set_url("origin", url) {
            Ok(()) => {
                self.refresh_git_status();
                self.status_message = trf(&self.language, "status_remote_set", &[url]);
            }
            Err(e) => self.status_message = trf(&self.language, "status_err_set_remote", &[&e]),
        }
    }

    // ── /branch ──────────────────────────────────────────────────

    fn cmd_list_branches(&mut self) {
        if !self.require_repo() {
            return;
        }
        match git::git_branch_list() {
            Ok(list) => {
                self.active_diff = list;
                self.invalidate_diff_cache();
                self.status_message = translate(&self.language, "status_branch_list").into_owned();
            }
            Err(e) => self.status_message = trf(&self.language, "status_err_list_branches", &[&e]),
        }
    }

    fn cmd_delete_branch(&mut self, name: &str) {
        if !self.require_repo() {
            return;
        }
        match git::git_branch_delete(name) {
            Ok(()) => {
                self.refresh_git_status();
                self.status_message = trf(&self.language, "status_branch_deleted", &[name]);
            }
            Err(e) => self.status_message = trf(&self.language, "status_err_branch_delete", &[&e]),
        }
    }

    fn cmd_switch_or_create(&mut self, name: &str) {
        if !self.require_repo() {
            return;
        }
        match git::git_branch_switch(name) {
            Ok(()) => {
                self.refresh_git_status();
                self.status_message = trf(&self.language, "status_branch_switched", &[name]);
            }
            Err(_) => match git::git_branch_create_and_switch(name) {
                Ok(()) => {
                    self.refresh_git_status();
                    self.status_message = trf(&self.language, "status_branch_created", &[name]);
                }
                Err(e2) => {
                    self.status_message = trf(&self.language, "status_err_branch_switch", &[&e2]);
                }
            },
        }
    }

    fn cmd_create_branch(&mut self, name: &str) {
        if !self.require_repo() {
            return;
        }
        match git::git_branch_create_and_switch(name) {
            Ok(()) => {
                self.refresh_git_status();
                self.status_message = trf(&self.language, "status_branch_created", &[name]);
            }
            Err(e) => self.status_message = trf(&self.language, "status_err_branch_create", &[&e]),
        }
    }

    // ── /config (local + global) ─────────────────────────────────

    fn cmd_list_config(&mut self) {
        if !self.require_repo() {
            return;
        }
        match git::git_config_list() {
            Ok(list) => {
                self.active_diff = list;
                self.invalidate_diff_cache();
                self.status_message = translate(&self.language, "status_config_list").into_owned();
            }
            Err(e) => self.status_message = trf(&self.language, "status_err_list_config", &[&e]),
        }
    }

    fn cmd_config_local(&mut self, key: &str, value: Option<&str>) {
        match value {
            Some(v) => match git::git_config_set_local(key, v) {
                Ok(()) => {
                    self.status_message = trf(
                        &self.language,
                        "status_config_set_local",
                        &[key, v],
                    );
                }
                Err(e) => {
                    self.status_message =
                        trf(&self.language, "status_err_config_set", &[&e]);
                }
            },
            None => match git::git_config_get(key) {
                Ok(val) => {
                    self.status_message = trf(
                        &self.language,
                        "status_config_get_local",
                        &[key, &val],
                    );
                }
                Err(_) => {
                    // Try global as a fallback.
                    match git::run_git(&["config", "--global", key]) {
                        Ok(val) if !val.is_empty() => {
                            self.status_message = trf(
                                &self.language,
                                "status_config_get_global",
                                &[key, val.trim()],
                            );
                        }
                        _ => {
                            self.status_message =
                                trf(&self.language, "status_config_not_found", &[key]);
                        }
                    }
                }
            },
        }
    }

    fn cmd_config_global(&mut self, key: &str, value: Option<&str>) {
        match value {
            Some(v) => match git::git_config_set_global(key, v) {
                Ok(()) => {
                    self.status_message = trf(
                        &self.language,
                        "status_config_set_global",
                        &[key, v],
                    );
                }
                Err(e) => {
                    self.status_message = trf(
                        &self.language,
                        "status_err_config_set_global",
                        &[&e],
                    );
                }
            },
            None => match git::git_config_get(key) {
                Ok(val) => {
                    self.status_message = trf(
                        &self.language,
                        "status_config_get_global",
                        &[key, &val],
                    );
                }
                Err(e) => {
                    self.status_message = trf(
                        &self.language,
                        "status_err_config_get",
                        &[&e],
                    );
                }
            },
        }
    }

    // ── /stash / /stash-pop ──────────────────────────────────────

    fn cmd_stash(&mut self) {
        if !self.require_repo() {
            return;
        }
        match git::git_stash() {
            Ok(()) => {
                self.refresh_git_status();
                self.status_message = translate(&self.language, "status_stash_ok").into_owned();
            }
            Err(e) => self.status_message = trf(&self.language, "status_err_stash", &[&e]),
        }
    }

    fn cmd_stash_pop(&mut self) {
        if !self.require_repo() {
            return;
        }
        match git::git_stash_pop() {
            Ok(()) => {
                self.refresh_git_status();
                self.status_message =
                    translate(&self.language, "status_stash_pop_ok").into_owned();
            }
            Err(e) => {
                self.status_message = trf(&self.language, "status_err_stash_pop", &[&e])
            }
        }
    }

    // ── /language <en|es> ────────────────────────────────────────

    fn cmd_set_language(&mut self, lang: &str) {
        let current = self.language.as_str();
        if current == lang {
            self.status_message = trf(&self.language, "status_language_same", &[lang]);
            return;
        }
        self.language = lang.to_string();
        // Preserve the skipped_version when saving.
        let existing_skipped = load_config().ok().and_then(|c| c.skipped_version);
        let _ = save_config(&Config {
            language: self.language.clone(),
            nerd_font: self.nerd_font,
            theme: self.theme.name.to_string(),
            skipped_version: existing_skipped,
        });
        self.status_message = trf(lang, "status_language_changed", &[lang]);
    }

    // ── Helpers ──────────────────────────────────────────────────

    /// Returns `true` if the current directory is a git repository. Sets
    /// `status_message` to the localised "not a git repo" error otherwise.
    fn require_repo(&mut self) -> bool {
        if self.is_git_repo {
            return true;
        }
        self.status_message = translate(&self.language, "status_not_git").into_owned();
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn execute_command_unknown_recognised_by_parser() {
        // We can't easily construct a full AppState in a unit test, but
        // we *can* exercise the `Command::parse` → `Unknown` arm by
        // asserting that an unknown input is recognised as such. The
        // dispatcher translates `Unknown(input)` into a friendly
        // "Unknown command: {input}. Type /help." status message.
        assert_eq!(
            Command::parse("/nope").unwrap(),
            Command::Unknown("/nope".into())
        );
    }
}
