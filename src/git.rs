use std::process::Command;
use std::str;

// Run a git command and return stdout+stderr combined (for verbose output).
// This is used by the mini-console to show real git progress.
pub fn run_git_verbose(args: &[&str]) -> Result<String, String> {
    let mut cmd = Command::new("git");
    cmd.args(args);
    match cmd.output() {
        Ok(output) => {
            let mut combined = String::new();
            let stdout = str::from_utf8(&output.stdout).unwrap_or("");
            let stderr = str::from_utf8(&output.stderr).unwrap_or("");
            if !stdout.is_empty() {
                combined.push_str(stdout.trim());
            }
            if !stderr.is_empty() {
                if !combined.is_empty() { combined.push('\n'); }
                combined.push_str(stderr.trim());
            }
            if output.status.success() {
                Ok(combined)
            } else {
                Err(if combined.is_empty() { "Unknown error".to_string() } else { combined })
            }
        }
        Err(e) => Err(e.to_string()),
    }
}

// Run a git command and return stdout as string or an error message
pub fn run_git(args: &[&str]) -> Result<String, String> {
    let mut cmd = Command::new("git");
    cmd.args(args);
    match cmd.output() {
        Ok(output) => {
            if output.status.success() {
                let s = str::from_utf8(&output.stdout)
                    .unwrap_or("")
                    .trim_end()
                    .to_string();
                Ok(s)
            } else {
                let err_msg = str::from_utf8(&output.stderr)
                    .unwrap_or("Unknown error")
                    .trim()
                    .to_string();
                Err(err_msg)
            }
        }
        Err(e) => Err(e.to_string()),
    }
}

pub fn is_inside_work_tree() -> bool {
    run_git(&["rev-parse", "--is-inside-work-tree"]).is_ok()
}

pub fn get_current_branch() -> Result<String, String> {
    run_git(&["symbolic-ref", "--short", "HEAD"])
}

pub fn get_remote(branch: &str) -> String {
    let key = format!("branch.{}.remote", branch);
    match run_git(&["config", &key]) {
        Ok(remote) if !remote.is_empty() => remote,
        _ => "origin".to_string(),
    }
}

pub fn fetch_remote(remote: &str, branch: &str) -> Result<(), String> {
    run_git(&["fetch", remote, branch])?;
    Ok(())
}

pub fn get_commits_behind(remote: &str, branch: &str) -> i32 {
    let r = format!("HEAD..{}/{}", remote, branch);
    if let Ok(out) = run_git(&["rev-list", "--count", &r]) {
        out.parse::<i32>().unwrap_or(0)
    } else {
        0
    }
}

pub fn get_commits_ahead(remote: &str, branch: &str) -> i32 {
    let r = format!("{}/{}..HEAD", remote, branch);
    if let Ok(out) = run_git(&["rev-list", "--count", &r]) {
        out.parse::<i32>().unwrap_or(0)
    } else {
        0
    }
}

pub fn has_uncommitted_changes() -> bool {
    if let Ok(out) = run_git(&["status", "--porcelain"]) {
        !out.is_empty()
    } else {
        false
    }
}

pub fn git_add_all() -> Result<(), String> {
    run_git(&["add", "."])?;
    Ok(())
}

pub fn git_commit(message: &str) -> Result<(), String> {
    run_git(&["commit", "-m", message])?;
    Ok(())
}

pub fn git_pull(remote: &str, branch: &str) -> Result<(), String> {
    run_git(&["pull", remote, branch])?;
    Ok(())
}

pub fn git_push(remote: &str, branch: &str) -> Result<(), String> {
    run_git(&["push", remote, branch])?;
    Ok(())
}

// ── Stage / Unstage ──────────────────────────────────────────────

pub fn git_stage_all() -> Result<(), String> {
    run_git(&["add", "."])?;
    Ok(())
}

pub fn git_unstage_all() -> Result<(), String> {
    run_git(&["reset", "HEAD"])?;
    Ok(())
}

// ── Undo commit ──────────────────────────────────────────────────

/// Returns the number of commits in the current branch (0 if none).
pub fn git_commit_count() -> i32 {
    match run_git(&["rev-list", "--count", "HEAD"]) {
        Ok(out) => out.trim().parse::<i32>().unwrap_or(0),
        Err(_) => 0,
    }
}

pub fn git_reset_soft(n: i32) -> Result<(), String> {
    let count = git_commit_count();
    if count < n {
        return Err(format!("Cannot undo {} commit(s): only {} commit(s) exist.", n, count));
    }
    run_git(&["reset", "--soft", &format!("HEAD~{}", n)])?;
    Ok(())
}

// ── Remove .git ──────────────────────────────────────────────────

pub fn git_remove_repo() -> Result<(), String> {
    let cwd = std::env::current_dir().map_err(|e| e.to_string())?;
    let git_dir = cwd.join(".git");
    if git_dir.exists() {
        std::fs::remove_dir_all(&git_dir).map_err(|e| e.to_string())?;
    }
    Ok(())
}

// ── Remote management ────────────────────────────────────────────

pub fn git_remote_set_url(remote: &str, url: &str) -> Result<(), String> {
    // Try set-url first (remote already exists)
    if run_git(&["remote", "set-url", remote, url]).is_ok() {
        return Ok(());
    }
    // Remote doesn't exist → add it
    run_git(&["remote", "add", remote, url])?;
    Ok(())
}

// ── Branch management ────────────────────────────────────────────

pub fn git_branch_list() -> Result<String, String> {
    run_git(&["branch", "--sort=-committerdate"])
}

pub fn git_branch_switch(name: &str) -> Result<(), String> {
    run_git(&["checkout", name])?;
    Ok(())
}

pub fn git_branch_create_and_switch(name: &str) -> Result<(), String> {
    run_git(&["checkout", "-b", name])?;
    Ok(())
}

pub fn git_branch_delete(name: &str) -> Result<(), String> {
    run_git(&["branch", "-d", name])?;
    Ok(())
}

// ── Config management ────────────────────────────────────────────

pub fn git_config_set_local(key: &str, value: &str) -> Result<(), String> {
    run_git(&["config", "--local", key, value])?;
    Ok(())
}

pub fn git_config_set_global(key: &str, value: &str) -> Result<(), String> {
    run_git(&["config", "--global", key, value])?;
    Ok(())
}

pub fn git_config_get(key: &str) -> Result<String, String> {
    run_git(&["config", key])
}

pub fn git_config_list() -> Result<String, String> {
    run_git(&["config", "--list", "--show-origin"])
}

// ── Stash ────────────────────────────────────────────────────────

pub fn git_stash() -> Result<(), String> {
    run_git(&["stash"])?;
    Ok(())
}

pub fn git_stash_pop() -> Result<(), String> {
    run_git(&["stash", "pop"])?;
    Ok(())
}

// ── Diff helpers ─────────────────────────────────────────────────

/// Get the full diff for a commit
pub fn git_diff_commit(commit_hash: &str) -> Result<String, String> {
    run_git(&["show", "--format=", commit_hash])
}

// ── Asynchronous execution with Askpass ───────────────────────────

pub fn run_git_async(
    args: Vec<String>,
    session_id: String,
    tx: std::sync::mpsc::Sender<crate::ui::state::TuiMessage>,
) {
    let tx_clone = tx.clone();
    std::thread::spawn(move || {
        let mut cmd = std::process::Command::new("git");
        cmd.args(&args);
        
        // Point Askpass to current executable
        if let Ok(exe) = std::env::current_exe() {
            cmd.env("GIT_ASKPASS", &exe);
            cmd.env("SSH_ASKPASS", &exe);
            cmd.env("SSH_ASKPASS_REQUIRE", "force");
            cmd.env("GIT_HERO_SESSION_ID", &session_id);
            // Disable default terminal prompts so it falls back to ASKPASS
            cmd.env("GIT_TERMINAL_PROMPT", "0");
        }

        cmd.stdout(std::process::Stdio::piped());
        cmd.stderr(std::process::Stdio::piped());
        cmd.stdin(std::process::Stdio::null());

        let mut child = match cmd.spawn() {
            Ok(c) => c,
            Err(e) => {
                let _ = tx_clone.send(crate::ui::state::TuiMessage::ConsoleOutput(format!("Failed to spawn git: {}\n", e)));
                let _ = tx_clone.send(crate::ui::state::TuiMessage::CommandFinished(Err(e.to_string())));
                return;
            }
        };

        let mut stdout = child.stdout.take().unwrap();
        let mut stderr = child.stderr.take().unwrap();

        let tx_stdout = tx_clone.clone();
        let stdout_thread = std::thread::spawn(move || {
            use std::io::Read;
            let mut buf = [0; 128];
            loop {
                match stdout.read(&mut buf) {
                    Ok(0) => break,
                    Ok(n) => {
                        if let Ok(s) = std::str::from_utf8(&buf[..n]) {
                            let _ = tx_stdout.send(crate::ui::state::TuiMessage::ConsoleOutput(s.to_string()));
                        }
                    }
                    Err(_) => break,
                }
            }
        });

        let tx_stderr = tx_clone.clone();
        let stderr_thread = std::thread::spawn(move || {
            use std::io::Read;
            let mut buf = [0; 128];
            loop {
                match stderr.read(&mut buf) {
                    Ok(0) => break,
                    Ok(n) => {
                        if let Ok(s) = std::str::from_utf8(&buf[..n]) {
                            let _ = tx_stderr.send(crate::ui::state::TuiMessage::ConsoleOutput(s.to_string()));
                        }
                    }
                    Err(_) => break,
                }
            }
        });

        let _ = stdout_thread.join();
        let _ = stderr_thread.join();

        match child.wait() {
            Ok(status) if status.success() => {
                let _ = tx_clone.send(crate::ui::state::TuiMessage::CommandFinished(Ok(())));
            }
            Ok(status) => {
                let _ = tx_clone.send(crate::ui::state::TuiMessage::CommandFinished(Err(format!("Command failed with exit status: {}", status))));
            }
            Err(e) => {
                let _ = tx_clone.send(crate::ui::state::TuiMessage::CommandFinished(Err(e.to_string())));
            }
        }
    });
}
