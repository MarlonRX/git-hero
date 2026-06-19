use std::borrow::Cow;
use std::io::Read;
use std::process::{Command, Stdio};
use std::str;

/// Default error message returned when git produces no (or non-UTF-8) stderr output.
const UNKNOWN_ERROR: &str = "Unknown error";

const PIPE_BUF_SIZE: usize = 4096;

// Run a git command and return stdout+stderr combined (for verbose output).
// This is used by the mini-console to show real git progress.
pub fn run_git_verbose(args: &[&str]) -> Result<String, String> {
    let output = Command::new("git")
        .args(args)
        .output()
        .map_err(|e| e.to_string())?;
    let combined = combine_outputs(&output.stdout, &output.stderr);
    if output.status.success() {
        return Ok(combined);
    }
    // On failure, give the user as much context as we have.
    if combined.is_empty() {
        let code = output
            .status
            .code()
            .map(|c| c.to_string())
            .unwrap_or_else(|| "no exit code".to_string());
        Err(format!("git failed with exit code {code} (no output)"))
    } else {
        Err(combined)
    }
}

// Run a git command and return stdout as string or an error message
pub fn run_git(args: &[&str]) -> Result<String, String> {
    let output = Command::new("git")
        .args(args)
        .output()
        .map_err(|e| e.to_string())?;
    if output.status.success() {
        Ok(trim_end_in_place(output.stdout))
    } else {
        Err(stderr_to_err(&output.stderr).into_owned())
    }
}

pub fn is_inside_work_tree() -> bool {
    // We only care about the exit status; discard stdout/stderr to skip buffer allocs.
    Command::new("git")
        .args(["rev-parse", "--is-inside-work-tree"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
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

// `git fetch` reports errors on stderr; stdout is not useful to us — discard it.
pub fn fetch_remote(remote: &str, branch: &str) -> Result<(), String> {
    let output = Command::new("git")
        .args(["fetch", remote, branch])
        .stdout(Stdio::null())
        .output()
        .map_err(|e| e.to_string())?;
    if output.status.success() {
        return Ok(());
    }
    Err(stderr_to_err(&output.stderr).into_owned())
}

pub fn get_commits_behind(remote: &str, branch: &str) -> u64 {
    count_commits(&format!("HEAD..{}/{}", remote, branch))
}

pub fn get_commits_ahead(remote: &str, branch: &str) -> u64 {
    count_commits(&format!("{}/{}..HEAD", remote, branch))
}

pub fn has_uncommitted_changes() -> bool {
    Command::new("git")
        .args(["status", "--porcelain"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| !o.stdout.is_empty())
        .unwrap_or(false)
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
pub fn git_commit_count() -> u64 {
    count_commits("HEAD")
}

pub fn git_reset_soft(n: u32) -> Result<(), String> {
    let count = git_commit_count();
    if count < n as u64 {
        return Err(format!(
            "Cannot undo {} commit(s): only {} commit(s) exist.",
            n, count
        ));
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
                let _ = tx.send(crate::ui::state::TuiMessage::ConsoleOutput(
                    format!("Failed to spawn git: {}\n", e),
                ));
                let _ = tx.send(crate::ui::state::TuiMessage::CommandFinished(
                    Err(e.to_string()),
                ));
                return;
            }
        };

        let stdout = match child.stdout.take() {
            Some(s) => s,
            None => {
                let _ = tx.send(crate::ui::state::TuiMessage::CommandFinished(
                    Err("Failed to capture git stdout".into()),
                ));
                return;
            }
        };
        let stderr = match child.stderr.take() {
            Some(s) => s,
            None => {
                let _ = tx.send(crate::ui::state::TuiMessage::CommandFinished(
                    Err("Failed to capture git stderr".into()),
                ));
                return;
            }
        };

        let stdout_thread = spawn_pipe_thread(stdout, tx.clone());
        let stderr_thread = spawn_pipe_thread(stderr, tx.clone());

        let _ = stdout_thread.join();
        let _ = stderr_thread.join();

        match child.wait() {
            Ok(status) if status.success() => {
                let _ = tx.send(crate::ui::state::TuiMessage::CommandFinished(Ok(())));
            }
            Ok(status) => {
                let _ = tx.send(crate::ui::state::TuiMessage::CommandFinished(Err(
                    format!("Command failed with exit status: {}", status),
                )));
            }
            Err(e) => {
                let _ = tx.send(crate::ui::state::TuiMessage::CommandFinished(
                    Err(e.to_string()),
                ));
            }
        }
    });
}

// ── Internal helpers ─────────────────────────────────────────────

/// Run `git rev-list --count <range>` and parse the result as u64 (0 on any failure).
fn count_commits(range: &str) -> u64 {
    let output = match Command::new("git")
        .args(["rev-list", "--count", range])
        .output()
    {
        Ok(o) => o,
        Err(_) => return 0,
    };
    if !output.status.success() {
        return 0;
    }
    str::from_utf8(&output.stdout)
        .ok()
        .and_then(|s| s.trim().parse::<u64>().ok())
        .unwrap_or(0)
}

/// Convert a raw stderr buffer into a human-readable error string.
///
/// Returns `Cow::Borrowed(UNKNOWN_ERROR)` (zero allocation) when the buffer
/// is empty or not valid UTF-8, and `Cow::Owned(trimmed)` when git produced
/// a real error message.
fn stderr_to_err(stderr: &[u8]) -> Cow<'static, str> {
    match str::from_utf8(stderr) {
        Ok(s) => match s.trim() {
            "" => Cow::Borrowed(UNKNOWN_ERROR),
            trimmed => Cow::Owned(trimmed.to_owned()),
        },
        Err(_) => Cow::Borrowed(UNKNOWN_ERROR),
    }
}

/// Trim trailing whitespace from a stdout buffer in place — single allocation.
fn trim_end_in_place(bytes: Vec<u8>) -> String {
    let mut s = String::from_utf8(bytes).unwrap_or_default();
    let trimmed_len = s.trim_end().len();
    s.truncate(trimmed_len);
    s
}

/// Combine trimmed stdout and stderr into a single string, reserving capacity up front
/// so `push_str` calls never reallocate.
fn combine_outputs(stdout: &[u8], stderr: &[u8]) -> String {
    let stdout_trim = str::from_utf8(stdout).map(str::trim).unwrap_or("");
    let stderr_trim = str::from_utf8(stderr).map(str::trim).unwrap_or("");
    let mut cap = stdout_trim.len() + stderr_trim.len();
    if !stdout_trim.is_empty() && !stderr_trim.is_empty() {
        cap += 1; // newline separator
    }
    let mut combined = String::with_capacity(cap);
    if !stdout_trim.is_empty() {
        combined.push_str(stdout_trim);
    }
    if !stderr_trim.is_empty() {
        if !combined.is_empty() {
            combined.push('\n');
        }
        combined.push_str(stderr_trim);
    }
    combined
}

/// Spawn a background thread that reads from `reader` and forwards each chunk
/// as a `TuiMessage::ConsoleOutput` to `tx`. Shared by the stdout/stderr pipes.
fn spawn_pipe_thread<R: Read + Send + 'static>(
    mut reader: R,
    tx: std::sync::mpsc::Sender<crate::ui::state::TuiMessage>,
) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        let mut buf = [0u8; PIPE_BUF_SIZE];
        loop {
            match reader.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => {
                    if let Ok(s) = str::from_utf8(&buf[..n]) {
                        let _ = tx.send(crate::ui::state::TuiMessage::ConsoleOutput(
                            s.to_owned(),
                        ));
                    }
                }
                Err(_) => break,
            }
        }
    })
}

// ── Snapshots (Phase 2) ────────────────────────────────────────────
//
// `status_snapshot` and `log_snapshot` replace the 6+ separate `git`
// invocations previously made by `AppState::refresh_git_status` with a
// single batched call each. Both return strongly-typed data that the
// caller can render without further parsing.

/// Compact representation of a single file in `StatusSnapshot::files`.
///
/// `xy[0]` is the index (staged) status, `xy[1]` the worktree (unstaged)
/// status. Each is one of: `M`, `A`, `D`, `R`, `C`, `U`, `?`, or ` `.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileSnapshot {
    pub path: String,
    pub xy: [char; 2],
}

impl FileSnapshot {
    /// True when the file has staged changes (X is not space or `?`).
    pub fn staged(&self) -> bool {
        self.xy[0] != ' ' && self.xy[0] != '?'
    }

    /// True when the file has unstaged changes (Y is not space or `?`).
    pub fn worktree(&self) -> bool {
        self.xy[1] != ' ' && self.xy[1] != '?'
    }

    /// Composite status string compatible with the old `GitFile::status` field:
    /// `"??"` for untracked, `"MM"` for staged+worktree, single char for
    /// staged-only or worktree-only, `" "` otherwise.
    pub fn status(&self) -> String {
        let x = self.xy[0];
        let y = self.xy[1];
        if x == '?' && y == '?' {
            "??".to_string()
        } else if self.staged() && self.worktree() {
            let mut s = String::with_capacity(2);
            s.push(x);
            s.push(y);
            s
        } else if self.staged() {
            x.to_string()
        } else if self.worktree() {
            y.to_string()
        } else {
            " ".to_string()
        }
    }
}

/// Branch + upstream + ahead/behind + file list, all from one `git status` call.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StatusSnapshot {
    /// Current branch name, or empty when HEAD is detached.
    pub branch: String,
    /// Upstream name in the form `remote/branch`, or empty if no upstream.
    pub upstream: String,
    /// Commits ahead of upstream (0 if no upstream).
    pub ahead: u64,
    /// Commits behind upstream (0 if no upstream).
    pub behind: u64,
    /// Changed, untracked and unmerged files.
    pub files: Vec<FileSnapshot>,
}

/// Run `git status --branch --porcelain=v2` and parse the result.
pub fn status_snapshot() -> Result<StatusSnapshot, crate::git_error::GitError> {
    let output = Command::new("git")
        .args(["status", "--branch", "--porcelain=v2"])
        .output()
        .map_err(|e| crate::git_error::GitError::SpawnFailed(e.to_string()))?;
    if !output.status.success() {
        return Err(crate::git_error::GitError::CommandFailed {
            stderr: stderr_to_err(&output.stderr).into_owned(),
            code: output.status.code(),
        });
    }
    let text = str::from_utf8(&output.stdout)
        .map_err(|_| crate::git_error::GitError::Other("non-UTF-8 in git status output".into()))?;
    Ok(parse_porcelain_v2(text))
}

/// Parse the textual output of `git status --porcelain=v2`.
///
/// Pure function — exposed for testing. The format is documented at
/// <https://git-scm.com/docs/git-status> in the "Porcelain Format Version 2"
/// section. Lines we handle:
///
/// - `# branch.head <name>` / `# branch.head (detached)`
/// - `# branch.upstream <remote>/<branch>`
/// - `# branch.ab +<ahead> -<behind>`
/// - `1 <XY> <sub> <mH> <mI> <mW> <hH> <hI> <path>` — ordinary changed
/// - `2 <XY> <sub> <mH> <mI> <mW> <hH> <hI> <X><score> <path>\t<orig>` — renamed/copied
/// - `u <XY> <sub> <m1> <m2> <m3> <mW> <h1> <h2> <h3> <path>` — unmerged
/// - `? <path>` — untracked
///
/// Ignored lines (`! <path>`) are silently dropped.
fn parse_porcelain_v2(text: &str) -> StatusSnapshot {
    let mut snap = StatusSnapshot::default();
    for line in text.lines() {
        if let Some(rest) = line.strip_prefix("# branch.head ") {
            snap.branch = rest.to_string();
        } else if let Some(rest) = line.strip_prefix("# branch.upstream ") {
            snap.upstream = rest.to_string();
        } else if let Some(rest) = line.strip_prefix("# branch.ab ") {
            // Format: `+<ahead> -<behind>` separated by whitespace.
            let mut parts = rest.split_whitespace();
            if let Some(a) = parts.next() {
                snap.ahead = a.trim_start_matches('+').parse().unwrap_or(0);
            }
            if let Some(b) = parts.next() {
                snap.behind = b.trim_start_matches('-').parse().unwrap_or(0);
            }
        } else if let Some(rest) = line.strip_prefix("1 ") {
            parse_v2_changed(&mut snap, rest);
        } else if let Some(rest) = line.strip_prefix("2 ") {
            parse_v2_renamed(&mut snap, rest);
        } else if let Some(rest) = line.strip_prefix("u ") {
            parse_v2_unmerged(&mut snap, rest);
        } else if let Some(rest) = line.strip_prefix("? ") {
            snap.files.push(FileSnapshot {
                path: rest.to_string(),
                xy: ['?', '?'],
            });
        }
        // "! <path>" (ignored) — skip
    }
    snap
}

fn parse_v2_changed(snap: &mut StatusSnapshot, rest: &str) {
    if rest.len() < 2 {
        return;
    }
    let bytes = rest.as_bytes();
    let x = bytes[0] as char;
    // In porcelain v2 the Y field uses `.` to mean "no worktree change";
    // normalise to a space so the rest of the codebase (and existing UI
    // code that matches on `f.status == "M"`) keeps working unchanged.
    let y = normalise_y(bytes[1] as char);
    if let Some(path) = rest.split_whitespace().nth(7) {
        snap.files.push(FileSnapshot {
            path: path.to_string(),
            xy: [x, y],
        });
    }
}

fn parse_v2_renamed(snap: &mut StatusSnapshot, rest: &str) {
    if rest.len() < 2 {
        return;
    }
    let bytes = rest.as_bytes();
    let x = bytes[0] as char;
    let y = normalise_y(bytes[1] as char);
    // The 9th whitespace-separated token is `<path>\t<origPath>`. Take only
    // the part before the tab — the new path is what we show in the UI.
    if let Some(field) = rest.split_whitespace().nth(8) {
        let path = field.split('\t').next().unwrap_or(field);
        snap.files.push(FileSnapshot {
            path: path.to_string(),
            xy: [x, y],
        });
    }
}

fn parse_v2_unmerged(snap: &mut StatusSnapshot, rest: &str) {
    if rest.len() < 2 {
        return;
    }
    let bytes = rest.as_bytes();
    let x = bytes[0] as char;
    let y = normalise_y(bytes[1] as char);
    if let Some(path) = rest.split_whitespace().nth(9) {
        snap.files.push(FileSnapshot {
            path: path.to_string(),
            xy: [x, y],
        });
    }
}

/// Translate the v2 `Y='.'` shorthand for "no worktree change" to a space,
/// matching v1 porcelain and the existing UI matchers.
#[inline]
fn normalise_y(y: char) -> char {
    if y == '.' { ' ' } else { y }
}

// ── Log snapshot ───────────────────────────────────────────────────

/// A single commit in the log, in display order (newest first).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogEntry {
    pub hash: String,
    pub date: String,
    pub subject: String,
}

/// Run `git log -n <limit>` and return at most `limit` entries.
///
/// Uses NUL (`%x00`) as the field separator so subjects with spaces
/// (and even tabs) parse correctly without escaping.
pub fn log_snapshot(limit: usize) -> Result<Vec<LogEntry>, crate::git_error::GitError> {
    let limit_str = format!("-{}", limit);
    // Format: hash\x00date\x00subject\x00 (trailing NUL guarantees clean chunking).
    let pretty = "--pretty=format:%h%x00%ar%x00%s%x00";
    let output = Command::new("git")
        .args(["log", &limit_str, pretty])
        .output()
        .map_err(|e| crate::git_error::GitError::SpawnFailed(e.to_string()))?;
    if !output.status.success() {
        return Err(crate::git_error::GitError::CommandFailed {
            stderr: stderr_to_err(&output.stderr).into_owned(),
            code: output.status.code(),
        });
    }
    let text = str::from_utf8(&output.stdout)
        .map_err(|_| crate::git_error::GitError::Other("non-UTF-8 in git log output".into()))?;
    Ok(parse_log_nul(text))
}

/// Parse the NUL-delimited output of `git log --pretty=format:%h\x00%ar\x00%s\x00`.
///
/// Pure function — exposed for testing. Splits by NUL, drops empty
/// fragments, and groups every three fragments into one `LogEntry`.
fn parse_log_nul(text: &str) -> Vec<LogEntry> {
    let parts: Vec<&str> = text.split('\0').filter(|s| !s.is_empty()).collect();
    parts
        .chunks_exact(3)
        .map(|c| LogEntry {
            hash: c[0].to_string(),
            date: c[1].to_string(),
            subject: c[2].to_string(),
        })
        .collect()
}

// ── Version check ─────────────────────────────────────────────────

/// Check the latest published version by inspecting remote tags.
/// Uses `git ls-remote --tags` to fetch the tag list, then parses the
/// most recent semver tag (e.g. `v0.2.0`).
pub fn check_latest_version() -> Result<String, crate::git_error::GitError> {
    let output = run_git(&[
        "ls-remote",
        "--tags",
        "https://github.com/MarlonRX/git-hero",
    ])?;

    let latest = output
        .lines()
        .filter_map(|line| {
            let tag = line.split("refs/tags/").nth(1)?.trim();
            let version = tag.strip_prefix('v').unwrap_or(tag);
            // Must be valid semver (x.y.z) with all digits.
            if version.split('.').count() == 3
                && version.chars().all(|c| c == '.' || c.is_ascii_digit())
            {
                Some(version.to_string())
            } else {
                None
            }
        })
        .max_by(|a, b| {
            let va = crate::version::Version::parse(a).unwrap_or(crate::version::Version { major: 0, minor: 0, patch: 0 });
            let vb = crate::version::Version::parse(b).unwrap_or(crate::version::Version { major: 0, minor: 0, patch: 0 });
            va.cmp(&vb)
        });

    latest.ok_or_else(|| {
        crate::git_error::GitError::Other("no version tags found".into())
    })
}

// ── Tests ──────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── parse_porcelain_v2 ──

    #[test]
    fn parse_v2_branch_only() {
        let s = parse_porcelain_v2("# branch.oid deadbeef\n# branch.head main\n");
        assert_eq!(s.branch, "main");
        assert_eq!(s.upstream, "");
        assert_eq!(s.ahead, 0);
        assert_eq!(s.behind, 0);
        assert!(s.files.is_empty());
    }

    #[test]
    fn parse_v2_with_upstream_and_ab() {
        let s = parse_porcelain_v2(
            "# branch.oid deadbeef\n\
             # branch.head feature\n\
             # branch.upstream origin/feature\n\
             # branch.ab +2 -3\n",
        );
        assert_eq!(s.branch, "feature");
        assert_eq!(s.upstream, "origin/feature");
        assert_eq!(s.ahead, 2);
        assert_eq!(s.behind, 3);
    }

    #[test]
    fn parse_v2_modified_file() {
        let s = parse_porcelain_v2(
            "# branch.head main\n\
             1 M. N... 100644 100644 100644 hash hash src/lib.rs\n",
        );
        assert_eq!(s.files.len(), 1);
        assert_eq!(s.files[0].path, "src/lib.rs");
        assert_eq!(s.files[0].xy, ['M', ' ']);
        assert!(s.files[0].staged());
        assert!(!s.files[0].worktree());
    }

    #[test]
    fn parse_v2_untracked() {
        let s = parse_porcelain_v2("# branch.head main\n? new.txt\n");
        assert_eq!(s.files.len(), 1);
        assert_eq!(s.files[0].path, "new.txt");
        assert_eq!(s.files[0].xy, ['?', '?']);
        assert_eq!(s.files[0].status(), "??");
    }

    #[test]
    fn parse_v2_renamed() {
        let s = parse_porcelain_v2(
            "# branch.head main\n\
             2 R. N... 100644 100644 100644 hash hash R100 new.rs\told.rs\n",
        );
        assert_eq!(s.files.len(), 1);
        assert_eq!(s.files[0].path, "new.rs");
        assert_eq!(s.files[0].xy, ['R', ' ']);
    }

    #[test]
    fn parse_v2_unmerged() {
        // Format: u XY sub m1 m2 m3 mW h1 h2 h3 path
        let s = parse_porcelain_v2(
            "# branch.head main\n\
             u UU N... 100644 100644 100644 100644 hash hash hash conflicted.rs\n",
        );
        assert_eq!(s.files.len(), 1);
        assert_eq!(s.files[0].path, "conflicted.rs");
        assert_eq!(s.files[0].xy, ['U', 'U']);
    }

    #[test]
    fn parse_v2_ignored_lines_are_dropped() {
        let s = parse_porcelain_v2("# branch.head main\n! ignored.txt\n");
        assert!(s.files.is_empty());
    }

    #[test]
    fn parse_v2_short_lines_are_skipped() {
        // First line has only 1 char after prefix; must not panic.
        let s = parse_porcelain_v2("# branch.head main\n1 M\n");
        assert!(s.files.is_empty());
    }

    // ── FileSnapshot::status ──

    #[test]
    fn status_string_table() {
        let cases = [
            (['M', ' '], "M"),
            (['M', 'M'], "MM"),
            (['A', ' '], "A"),
            (['D', 'M'], "DM"),
            (['?', '?'], "??"),
            ([' ', 'M'], "M"),
            ([' ', 'D'], "D"),
            ([' ', ' '], " "),
        ];
        for (xy, expected) in cases {
            let f = FileSnapshot { path: "x".into(), xy };
            assert_eq!(f.status(), expected, "xy={:?}", xy);
        }
    }

    // ── parse_log_nul ──

    #[test]
    fn parse_log_single_commit() {
        let s = parse_log_nul("abc123\x002 hours ago\x00first commit\x00");
        assert_eq!(s.len(), 1);
        assert_eq!(s[0].hash, "abc123");
        assert_eq!(s[0].date, "2 hours ago");
        assert_eq!(s[0].subject, "first commit");
    }

    #[test]
    fn parse_log_multiple_commits() {
        let s = parse_log_nul(
            "h1\x002h ago\x00msg 1\x00h2\x001h ago\x00msg 2\x00h3\x00now\x00msg 3\x00",
        );
        assert_eq!(s.len(), 3);
        assert_eq!(s[0].hash, "h1");
        assert_eq!(s[1].subject, "msg 2");
        assert_eq!(s[2].date, "now");
    }

    #[test]
    fn parse_log_subject_with_spaces() {
        // Subjects with multiple spaces must not be split.
        let s = parse_log_nul("h\x00now\x00a  b  c\x00");
        assert_eq!(s[0].subject, "a  b  c");
    }

    #[test]
    fn parse_log_empty_returns_empty_vec() {
        assert!(parse_log_nul("").is_empty());
        assert!(parse_log_nul("\x00\x00\x00").is_empty());
    }

    #[test]
    fn parse_log_partial_last_entry_is_dropped() {
        // If the trailing NUL is missing, the last entry is incomplete and
        // `chunks_exact(3)` discards it. Documented behaviour.
        let s = parse_log_nul("h1\x00now\x00msg 1\x00h2\x00now\x00");
        // We have 5 parts (h1, now, msg 1, h2, now). 5 / 3 = 1 complete + 2 leftover (dropped).
        assert_eq!(s.len(), 1);
        assert_eq!(s[0].hash, "h1");
    }
}