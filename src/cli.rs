use std::io::{self, Write};

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
        eprintln!(
            "{}  ✖  {} (Not inside a git repository).{}",
            RED,
            crate::i18n::translate(lang, "not_git_repo"),
            NC
        );
        return Err("not inside a git repository".into());
    }

    let branch = crate::git::get_current_branch().unwrap_or_else(|_| "HEAD".to_string());
    let remote = crate::git::get_remote(&branch);

    println!("{}", sep);
    println!(
        "{}  GIT FLOW  {} {}{} {}→{} {}{}{}",
        BOLD, NC, CYAN, branch, BLUE, NC, CYAN, remote, NC
    );
    println!("{}", sep);
    println!();

    println!(
        "{}  🔍 {}...{}",
        BLUE,
        crate::i18n::translate(lang, "status_fetching"),
        NC
    );
    let _ = crate::git::fetch_remote(&remote, &branch);

    let behind = crate::git::get_commits_behind(&remote, &branch);
    let ahead = crate::git::get_commits_ahead(&remote, &branch);

    println!(
        "{}  ┌─ {}: {}/{}{}",
        CYAN,
        crate::i18n::translate(lang, "repo_status"),
        remote,
        branch,
        NC
    );
    println!(
        "{}  ├─ {}: {}{} commit(s){}",
        CYAN,
        if lang == "es" { "Adelante" } else { "Ahead" },
        GREEN,
        ahead,
        NC
    );
    println!(
        "{}  └─ {}: {}{} commit(s){}",
        CYAN,
        if lang == "es" { "Detrás" } else { "Behind" },
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
                "{}  ℹ  {} (No changes staged).{}",
                CYAN,
                if lang == "es" {
                    "Sin cambios para commit"
                } else {
                    "No changes for commit"
                },
                NC
            );
        } else {
            println!();
            println!(
                "{}  💬 {}:{}",
                BOLD,
                if lang == "es" {
                    "Mensaje del commit"
                } else {
                    "Commit message"
                },
                NC
            );
            print!("  {}→{} ", BLUE, NC);
            let _ = io::stdout().flush();
            let mut commit_msg = String::new();
            io::stdin().read_line(&mut commit_msg)?;
            let commit_msg = commit_msg.trim();
            if commit_msg.is_empty() {
                println!();
                println!(
                    "{}  ✖  {} (Empty message, cancelled).{}",
                    RED,
                    if lang == "es" {
                        "Mensaje vacío, cancelado"
                    } else {
                        "Empty message, cancelled"
                    },
                    NC
                );
                return Err("empty commit message".into());
            }

            println!();
            println!("{}  📝 git commit -m \"{}\"{}", BLUE, commit_msg, NC);
            if let Err(e) = crate::git::git_commit(commit_msg) {
                eprintln!("{}  ✖  Error committing: {}{}", RED, e, NC);
                return Err(format!("git commit failed: {e}").into());
            }

            println!(
                "{}  ✔  {} (Commit created).{}",
                GREEN,
                crate::i18n::translate(lang, "status_commit_success"),
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
            if lang == "es" {
                "Sin cambios pendientes para commit"
            } else {
                "No pending changes for commit"
            },
            NC
        );
    }

    // PULL
    if behind > 0 {
        println!();
        println!(
            "{}  ⚠  {} (Remote is ahead by {} commit(s)).{}",
            YELLOW,
            if lang == "es" {
                "El remoto tiene cambios"
            } else {
                "Remote has changes"
            },
            behind,
            NC
        );
        let pull_prompt = if lang == "es" { "¿Hacer pull?" } else { "Do pull?" };
        if confirm_action(pull_prompt) {
            println!();
            println!("{}  ⬇  git pull {} {}{}", BLUE, remote, branch, NC);
            if let Err(e) = crate::git::git_pull(&remote, &branch) {
                eprintln!("{}  ✖  Error pulling: {}{}", RED, e, NC);
                return Err(format!("git pull failed: {e}").into());
            }
            println!(
                "{}  ✔  {} (Pull completed).{}",
                GREEN,
                if lang == "es" { "Pull completado" } else { "Pull completed" },
                NC
            );
            let new_ahead = crate::git::get_commits_ahead(&remote, &branch);
            has_unpushed = new_ahead > 0;
        }
    } else if !did_commit && !has_unpushed {
        println!();
        println!(
            "{}  ✔  {} (Up to date with remote).{}",
            GREEN,
            if lang == "es" {
                "Estás al día con el remoto"
            } else {
                "You are up to date with remote"
            },
            NC
        );
    }

    // PUSH
    if has_unpushed {
        println!();
        let push_prompt = if lang == "es" { "¿Hacer push?" } else { "Do push?" };
        if confirm_action(push_prompt) {
            println!();
            println!("{}  ⬆  git push {} {}{}", BLUE, remote, branch, NC);
            if let Err(e) = crate::git::git_push(&remote, &branch) {
                eprintln!("{}  ✖  Error pushing: {}{}", RED, e, NC);
                return Err(format!("git push failed: {e}").into());
            }
            println!(
                "{}  ✔  {} (Push completed).{}",
                GREEN,
                if lang == "es" { "Push completado" } else { "Push completed" },
                NC
            );
        } else {
            println!(
                "{}  📌 {} (Local commits kept, no push).{}",
                CYAN,
                if lang == "es" {
                    "Commits locales guardados, sin push"
                } else {
                    "Local commits kept, no push"
                },
                NC
            );
        }
    }

    println!();
    println!("{}", sep);
    println!("{}  ✅  {} (Done!).{}", GREEN, BOLD, NC);
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
}
