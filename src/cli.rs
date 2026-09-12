//! Non-interactive "git flow" CLI (`gith -cli`).
//!
//! All user-facing strings go through [`crate::i18n`] — the same dictionary
//! the TUI uses. The bilingual `if lang == "es"` pairs that used to live
//! here are gone (Phase 3 / i18n unification): the CLI and the TUI now show
//! identical phrasing for identical states, from one source of truth.
//!
//! The ANSI palette below is CLI-only by design: the TUI renders through
//! ratatui styles (`theme.rs`), while raw escape codes are the only option
//! here. They are single-owned by this module — not a duplication of the
//! theme, which carries no ANSI codes.

use std::io::{self, Write};

use crate::i18n::{translate, trf};

const GREEN: &str = "\x1b[0;32m";
const BLUE: &str = "\x1b[0;34m";
const YELLOW: &str = "\x1b[1;33m";
const RED: &str = "\x1b[0;31m";
const CYAN: &str = "\x1b[0;36m";
const BOLD: &str = "\x1b[1m";
const NC: &str = "\x1b[0m";

pub fn confirm_action(prompt: &str) -> bool {
    print!("  {} (y/N): ", prompt);
    let _ = io::stdout().flush();
    let mut ans = String::new();
    if io::stdin().read_line(&mut ans).is_err() {
        return false;
    }
    let ans = ans.trim().to_lowercase();
    ans == "y" || ans == "yes" || ans == "s" || ans == "si"
}

/// Run the non-interactive git flow. Returns an `Err` instead of calling
/// `std::process::exit(1)` directly (Phase 3.8) so that:
/// - unit tests can exercise the failure paths without terminating the test
///   harness,
/// - `main` can decide whether to print a stack-trace, dump context, or
///   just propagate the message.
///
/// `main` is the only place that converts the error to a non-zero exit code.
pub fn run_cli_flow() -> Result<(), Box<dyn std::error::Error>> {
    let config = crate::config::load_config().unwrap_or(crate::config::Config {
        language: "en".to_string(),
        nerd_font: false,
        theme: "Tokyo Night".to_string(),
        skipped_version: None,
    });
    let lang = &config.language;

    let sep = format!("{}════════════════════════════════════════{}", BLUE, NC);

    println!();

    if !crate::git::is_inside_work_tree() {
        eprintln!("{}  ✖  {}{}", RED, translate(lang, "not_git_repo"), NC);
        return Err("not inside a git repository".into());
    }

    let branch = crate::git::get_current_branch().unwrap_or_else(|_| "HEAD".to_string());
    let remote = crate::git::get_remote(&branch);

    println!("{}", sep);
    println!(
        "{}  {}  {} {}{} {}→{} {}{}{}",
        BOLD,
        translate(lang, "git_flow_title"),
        NC,
        CYAN,
        branch,
        BLUE,
        NC,
        CYAN,
        remote,
        NC
    );
    println!("{}", sep);
    println!();

    println!(
        "{}  🔍 {}{}",
        BLUE,
        translate(lang, "status_fetching"),
        NC
    );
    let _ = crate::git::fetch_remote(&remote, &branch);

    let behind = crate::git::get_commits_behind(&remote, &branch);
    let ahead = crate::git::get_commits_ahead(&remote, &branch);

    println!(
        "{}  ┌─ {}: {}/{}{}",
        CYAN,
        translate(lang, "repo_status"),
        remote,
        branch,
        NC
    );
    println!(
        "{}  ├─ {}: {}{} commit(s){}",
        CYAN,
        translate(lang, "ahead_label"),
        GREEN,
        ahead,
        NC
    );
    println!(
        "{}  └─ {}: {}{} commit(s){}",
        CYAN,
        translate(lang, "behind_label"),
        YELLOW,
        behind,
        NC
    );

    let mut did_commit = false;
    let mut has_unpushed = ahead > 0;

    // ADD + COMMIT (if there are changes)
    if crate::git::has_uncommitted_changes() {
        println!();
        println!("{}  📦 git add .{}", BLUE, NC);
        if let Err(e) = crate::git::git_add_all() {
            eprintln!("{}  ✖  Error: {}{}", RED, e, NC);
            return Err(format!("git add failed: {e}").into());
        }

        // Check if anything was staged
        let is_staged_empty = crate::git::run_git(&["diff", "--cached", "--quiet"]).is_ok();
        if is_staged_empty {
            println!(
                "{}  ℹ  {}{}",
                CYAN,
                translate(lang, "no_changes_to_commit"),
                NC
            );
        } else {
            println!();
            println!("{}  💬 {}{}", BOLD, translate(lang, "commit_message_label"), NC);
            print!("  {}→{} ", BLUE, NC);
            let _ = io::stdout().flush();
            let mut commit_msg = String::new();
            io::stdin().read_line(&mut commit_msg)?;
            let commit_msg = commit_msg.trim();
            if commit_msg.is_empty() {
                println!();
                println!(
                    "{}  ✖  {}{}",
                    RED,
                    translate(lang, "empty_message_cancelled"),
                    NC
                );
                return Err("empty commit message".into());
            }

            println!();
            println!("{}  📝 git commit -m \"{}\"{}", BLUE, commit_msg, NC);
            if let Err(e) = crate::git::git_commit(commit_msg) {
                eprintln!(
                    "{}  ✖  {}{}",
                    RED,
                    trf(lang, "status_err_commit", &[&e]),
                    NC
                );
                return Err(format!("git commit failed: {e}").into());
            }

            println!(
                "{}  ✔  {}{}",
                GREEN,
                translate(lang, "status_commit_success"),
                NC
            );
            did_commit = true;
            let new_ahead = crate::git::get_commits_ahead(&remote, &branch);
            has_unpushed = new_ahead > 0;
        }
    } else {
        println!();
        println!(
            "{}  ℹ  {}.{}",
            CYAN,
            translate(lang, "no_pending_changes"),
            NC
        );
    }

    // PULL
    if behind > 0 {
        println!();
        println!(
            "{}  ⚠  {} ↓{}{}",
            YELLOW,
            translate(lang, "remote_has_changes"),
            behind,
            NC
        );
        if confirm_action(&translate(lang, "do_pull")) {
            println!();
            println!("{}  ⬇  git pull {} {}{}", BLUE, remote, branch, NC);
            if let Err(e) = crate::git::git_pull(&remote, &branch) {
                eprintln!("{}  ✖  {}{}", RED, trf(lang, "cli_err_pull", &[&e]), NC);
                return Err(format!("git pull failed: {e}").into());
            }
            println!(
                "{}  ✔  {}{}",
                GREEN,
                translate(lang, "pull_completed"),
                NC
            );
            let new_ahead = crate::git::get_commits_ahead(&remote, &branch);
            has_unpushed = new_ahead > 0;
        }
    } else if !did_commit && !has_unpushed {
        println!();
        println!(
            "{}  ✔  {}{}",
            GREEN,
            translate(lang, "up_to_date_with_remote"),
            NC
        );
    }

    // PUSH
    if has_unpushed {
        println!();
        if confirm_action(&translate(lang, "do_push")) {
            println!();
            println!("{}  ⬆  git push {} {}{}", BLUE, remote, branch, NC);
            if let Err(e) = crate::git::git_push(&remote, &branch) {
                eprintln!("{}  ✖  {}{}", RED, trf(lang, "cli_err_push", &[&e]), NC);
                return Err(format!("git push failed: {e}").into());
            }
            println!(
                "{}  ✔  {}{}",
                GREEN,
                translate(lang, "push_completed"),
                NC
            );
        } else {
            println!(
                "{}  📌 {}{}",
                CYAN,
                translate(lang, "local_commits_kept"),
                NC
            );
        }
    }

    println!();
    println!("{}", sep);
    println!("{}  ✅  {}{}{}", GREEN, BOLD, translate(lang, "status_success"), NC);
    println!("{}", sep);
    println!();

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn confirm_action_rejects_default() {
        // We can't easily stub stdin in a unit test, so we just verify
        // the function compiles and returns a bool. Manual coverage for
        // the y/N branch logic via shell test.
        let _ = confirm_action;
    }

    #[test]
    fn every_key_used_by_the_cli_exists_in_both_dictionaries() {
        // The CLI must never reintroduce hardcoded bilingual pairs: any key
        // listed here has EN *and* ES values (i18n has its own parity test,
        // this one guards against a key being renamed out from under cli.rs).
        const CLI_KEYS: &[&str] = &[
            "not_git_repo",
            "git_flow_title",
            "status_fetching",
            "repo_status",
            "ahead_label",
            "behind_label",
            "no_changes_to_commit",
            "commit_message_label",
            "empty_message_cancelled",
            "status_commit_success",
            "status_err_commit",
            "no_pending_changes",
            "remote_has_changes",
            "do_pull",
            "cli_err_pull",
            "pull_completed",
            "up_to_date_with_remote",
            "do_push",
            "cli_err_push",
            "push_completed",
            "local_commits_kept",
            "status_success",
        ];
        for key in CLI_KEYS {
            // A missing key degrades to the key itself — detect that and fail.
            for lang in ["en", "es"] {
                let value = crate::i18n::translate(lang, key);
                assert_ne!(&*value, *key, "key {key} missing in {lang} dict");
            }
        }
    }
}
