use std::process::Command;
use std::str;

// Run a git command and return stdout as string or an error message
pub fn run_git(args: &[&str]) -> Result<String, String> {
    let mut cmd = Command::new("git");
    cmd.args(args);
    match cmd.output() {
        Ok(output) => {
            if output.status.success() {
                let s = str::from_utf8(&output.stdout)
                    .unwrap_or("")
                    .trim()
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
