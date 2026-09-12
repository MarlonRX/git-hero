//! Repository browser — the repository-manager core of Git Hero's niche.
//!
//! A recursive walk of the projects folder (root = parent when the current
//! directory IS a repo, otherwise the current directory) finds every Git
//! repository up to `MAX_DEPTH`, pulling branch, ahead/behind and dirty
//! count from ONE `git status --branch --porcelain=v2` call per repo plus
//! one `git log -1` for the last-commit timestamp. Results are ordered
//! newest activity first.
//!
//! The walk is bounded (visited dirs, probed repos, depth) and skips
//! vendor/build folders plus anything hidden, so scanning a huge tree
//! cannot freeze the UI. Repositories are pruned at their own root — we
//! never descend into a `.git`-containing folder.
//!
//! Scanning is on-demand: it runs when the browser is opened or re-scanned
//! (Ctrl+R), so the regular per-frame render path stays free. Filtering is
//! a pure function over the already-scanned entries — typing never costs a
//! git call.

use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::git::{self, StatusSnapshot};
use crate::ui::state::AppState;

impl AppState {
    /// Recompute the browser's visible indices from `repo_filter` and
    /// clamp the cursor. Pure string work — no git invocations, safe to
    /// call on every keystroke.
    pub fn apply_repo_filter(&mut self) {
        self.repo_view = filter_matches(&self.repos, &self.repo_filter);
        self.repo_cursor = 0;
    }

    /// Open the repository under the cursor as the active repo (via the
    /// `/cd` pipeline) and close the browser.
    pub fn open_selected_repo(&mut self) {
        let Some(&idx) = self.repo_view.get(self.repo_cursor) else {
            self.show_repo_overview = false;
            return;
        };
        let Some(path) = self.repos.get(idx).map(|e| e.path.clone()) else {
            self.show_repo_overview = false;
            return;
        };
        self.show_repo_overview = false;
        self.repo_filter.clear();
        self.apply_repo_filter();
        self.execute_command(&format!("/cd {path}"));
    }
}

/// Hard cap on probed repositories per scan — each probe costs two `git`
/// processes, so a huge "projects" folder must not freeze the UI.
const MAX_REPOS: usize = 60;

/// Hard cap on directories visited (stat'd) during the recursive walk.
const MAX_WALK_DIRS: usize = 500;

/// Maximum recursion depth below the scan root.
const MAX_DEPTH: usize = 4;

/// Folders that are build/vendor noise — descending into them is slow and
/// almost never where a user's repositories live.
const SKIP_DIRS: &[&str] = &[
    "node_modules",
    "target",
    "vendor",
    "dist",
    "build",
    "__pycache__",
    "Library",
    "AppData",
    "VirtualBox VMs",
];

/// One row of the repository browser.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepoEntry {
    pub path: String,
    pub name: String,
    /// Path relative to the scan root, always `/`-separated (`group/app`).
    /// This is what the filter and the name column show.
    pub rel: String,
    pub branch: String,
    pub ahead: u64,
    pub behind: u64,
    /// Changed + untracked files: "is there junk to commit?" at a glance.
    pub dirty: usize,
    /// Compact relative age of the last commit ("now", "4h", "9d", "").
    pub age: String,
    /// Raw epoch seconds of the last commit, for sorting (`None` = no commits).
    pub last_commit: Option<u64>,
}

/// Result of one scan: the directory that was scanned and its repos,
/// already sorted by last activity (newest first, commit-less repos last).
#[derive(Debug, Clone, Default)]
pub struct RepoScan {
    pub root: String,
    pub entries: Vec<RepoEntry>,
}

impl RepoScan {
    /// How many repos have uncommitted changes (the "n dirty" headline).
    pub fn dirty_count(&self) -> usize {
        self.entries.iter().filter(|e| e.dirty > 0).count()
    }
}

/// Recursively find repositories near the current working directory.
pub fn scan_repos() -> RepoScan {
    let cwd = match std::env::current_dir() {
        Ok(p) => p,
        Err(_) => return RepoScan::default(),
    };
    let root = scan_root(&cwd);
    let now = epoch_now();
    let mut entries: Vec<RepoEntry> = Vec::new();
    let mut visited = 0usize;

    // Explicit stack walk (no recursion — deep trees must not blow it).
    let mut stack: Vec<(PathBuf, usize, String)> = vec![(root.clone(), 0, String::new())];
    while let Some((dir, depth, rel_prefix)) = stack.pop() {
        if entries.len() >= MAX_REPOS || visited >= MAX_WALK_DIRS {
            break;
        }
        visited += 1;
        let Ok(read) = std::fs::read_dir(&dir) else {
            continue;
        };
        for cand in read.flatten() {
            let Ok(file_type) = cand.file_type() else {
                continue;
            };
            if !file_type.is_dir() {
                continue;
            }
            let name_os = cand.file_name();
            let name = name_os.to_string_lossy();
            if name.starts_with('.') || SKIP_DIRS.iter().any(|s| name.eq_ignore_ascii_case(s)) {
                continue;
            }
            let child = cand.path();
            let rel = if rel_prefix.is_empty() {
                name.to_string()
            } else {
                format!("{rel_prefix}/{name}")
            };
            if child.join(".git").exists() {
                // A repo: probe it and prune (never descend inside repos).
                if entries.len() >= MAX_REPOS {
                    break;
                }
                let path_str = child.to_string_lossy().into_owned();
                let Some(snap) = git::status_snapshot_in(&path_str).ok() else {
                    continue; // broken/locked repo: skip rather than fail the scan
                };
                let epoch = git::last_commit_epoch_in(&path_str);
                entries.push(entry_from_snapshot(&name, rel, path_str, &snap, epoch, now));
            } else if depth + 1 < MAX_DEPTH {
                stack.push((child, depth + 1, rel));
            }
        }
    }

    sort_by_activity(&mut entries);
    RepoScan {
        root: root.to_string_lossy().into_owned(),
        entries,
    }
}

/// Case-insensitive substring filter over `rel` (covers the folder name
/// too). Returns indices into `entries` in scan order — pure, allocation
/// per keystroke is two `to_lowercase` copies at most, never a git call.
pub fn filter_matches(entries: &[RepoEntry], query: &str) -> Vec<usize> {
    if query.is_empty() {
        return (0..entries.len()).collect();
    }
    let q = query.to_lowercase();
    entries
        .iter()
        .enumerate()
        .filter(|(_, e)| e.rel.to_lowercase().contains(&q) || e.name.to_lowercase().contains(&q))
        .map(|(i, _)| i)
        .collect()
}

/// When the current directory IS a repo, scan its parent (the projects
/// folder); otherwise scan the current directory itself.
fn scan_root(cwd: &Path) -> PathBuf {
    scan_root_for(cwd.join(".git").exists(), cwd)
}

/// Pure decision behind [`scan_root`], testable without touching the disk.
fn scan_root_for(is_repo: bool, cwd: &Path) -> PathBuf {
    if is_repo {
        cwd.parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| cwd.to_path_buf())
    } else {
        cwd.to_path_buf()
    }
}

/// Pure row builder: turns one status snapshot into a `RepoEntry`.
fn entry_from_snapshot(
    name: &str,
    rel: String,
    path: String,
    snap: &StatusSnapshot,
    last_commit: Option<u64>,
    now: u64,
) -> RepoEntry {
    let age = match last_commit {
        Some(epoch) if epoch <= now => humanize_age(now - epoch),
        Some(_) => "now".to_string(),
        None => String::new(),
    };
    RepoEntry {
        path,
        name: name.to_string(),
        rel,
        branch: if snap.branch == "(detached)" {
            "\u{2205}".to_string()
        } else {
            snap.branch.clone()
        },
        ahead: snap.ahead,
        behind: snap.behind,
        dirty: snap.files.len(),
        age,
        last_commit,
    }
}

/// Newest first; repos without commits sink to the bottom; equal keys by name.
fn sort_by_activity(entries: &mut [RepoEntry]) {
    entries.sort_by(|a, b| match (a.last_commit, b.last_commit) {
        (Some(x), Some(y)) => y.cmp(&x).then_with(|| a.name.cmp(&b.name)),
        (Some(_), None) => std::cmp::Ordering::Less,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (None, None) => a.name.cmp(&b.name),
    });
}

fn epoch_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Compact relative age for the browser column, git-`%ar`-flavoured.
pub fn humanize_age(secs_ago: u64) -> String {
    const MIN: u64 = 60;
    const HOUR: u64 = 60 * MIN;
    const DAY: u64 = 24 * HOUR;
    const WEEK: u64 = 7 * DAY;
    const YEAR: u64 = 365 * DAY;
    if secs_ago < MIN {
        "now".to_string()
    } else if secs_ago < HOUR {
        format!("{}m", secs_ago / MIN)
    } else if secs_ago < DAY {
        format!("{}h", secs_ago / HOUR)
    } else if secs_ago < WEEK {
        format!("{}d", secs_ago / DAY)
    } else if secs_ago < YEAR {
        format!("{}w", secs_ago / WEEK)
    } else {
        format!("{}y", secs_ago / YEAR)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn snap(branch: &str, ahead: u64, behind: u64, files: usize) -> StatusSnapshot {
        StatusSnapshot {
            branch: branch.into(),
            upstream: String::new(),
            ahead,
            behind,
            files: (0..files)
                .map(|i| git::FileSnapshot {
                    path: format!("f{i}.rs"),
                    xy: ['M', ' '],
                })
                .collect(),
        }
    }

    fn entry(name: &str, rel: &str) -> RepoEntry {
        RepoEntry {
            path: format!("/root/{rel}"),
            name: name.into(),
            rel: rel.into(),
            branch: "main".into(),
            ahead: 0,
            behind: 0,
            dirty: 0,
            age: String::new(),
            last_commit: None,
        }
    }

    #[test]
    fn humanize_age_buckets() {
        assert_eq!(humanize_age(0), "now");
        assert_eq!(humanize_age(59), "now");
        assert_eq!(humanize_age(60), "1m");
        assert_eq!(humanize_age(3_600 * 4 + 10), "4h");
        assert_eq!(humanize_age(3_600 * 24 * 2), "2d");
        assert_eq!(humanize_age(3_600 * 24 * 9), "1w");
        assert_eq!(humanize_age(3_600 * 24 * 400), "1y");
    }

    #[test]
    fn entry_maps_detached_head_symbol() {
        let e = entry_from_snapshot(
            "x",
            "x".into(),
            "p".into(),
            &snap("(detached)", 0, 0, 0),
            None,
            100,
        );
        assert_eq!(e.branch, "\u{2205}");
        assert!(e.age.is_empty(), "repo without commits shows no age");
    }

    #[test]
    fn entry_counts_dirty_and_formats_age() {
        let e = entry_from_snapshot(
            "proj",
            "work/proj".into(),
            "/tmp/proj".into(),
            &snap("main", 2, 1, 4),
            Some(1000),
            1000 + 7_200,
        );
        assert_eq!(e.name, "proj");
        assert_eq!(e.rel, "work/proj");
        assert_eq!(e.dirty, 4);
        assert_eq!((e.ahead, e.behind), (2, 1));
        assert_eq!(e.age, "2h");
    }

    #[test]
    fn filter_empty_returns_all_in_order() {
        let v = vec![entry("a", "a"), entry("b", "grp/b")];
        assert_eq!(filter_matches(&v, ""), vec![0, 1]);
    }

    #[test]
    fn filter_matches_name_and_path_case_insensitive() {
        let v = vec![
            entry("api-server", "work/api-server"),
            entry("web", "side/web-client"),
            entry("docs", "docs"),
        ];
        assert_eq!(filter_matches(&v, "API"), vec![0]);
        assert_eq!(filter_matches(&v, "client"), vec![1]);
        assert_eq!(filter_matches(&v, "web"), vec![1]);
        assert_eq!(filter_matches(&v, "no-match"), Vec::<usize>::new());
    }

    #[test]
    fn scan_root_inside_repo_uses_parent() {
        let p = Path::new("/home/u/projects/app");
        assert_eq!(scan_root_for(true, p), PathBuf::from("/home/u/projects"));
    }

    #[test]
    fn scan_root_outside_repo_uses_cwd() {
        let p = Path::new("/home/u/projects");
        assert_eq!(scan_root_for(false, p), PathBuf::from("/home/u/projects"));
    }

    #[test]
    fn scan_root_at_filesystem_root_falls_back_to_cwd() {
        let p = Path::new("/");
        assert_eq!(scan_root_for(true, p), PathBuf::from("/"));
    }

    #[test]
    fn sort_puts_newest_first_and_commitless_last() {
        let mk = |name: &str, epoch: Option<u64>| RepoEntry {
            last_commit: epoch,
            ..entry(name, name)
        };
        let mut v = vec![mk("old", Some(100)), mk("none", None), mk("new", Some(200))];
        sort_by_activity(&mut v);
        assert_eq!(
            v.iter().map(|e| e.name.as_str()).collect::<Vec<_>>(),
            vec!["new", "old", "none"]
        );
    }

    #[test]
    fn dirty_count_aggregates() {
        let mut a = entry("a", "a");
        a.dirty = 2;
        let mut c = entry("c", "c");
        c.dirty = 1;
        let scan = RepoScan {
            root: String::new(),
            entries: vec![a, entry("b", "b"), c],
        };
        assert_eq!(scan.dirty_count(), 2);
    }

    /// Real-environment smoke test for the recursive scan: builds a temp
    /// "projects" folder with a dirty repo at depth 1, a clean repo nested
    /// one level deeper, and a decoy repo inside node_modules that must NOT
    /// be found. Ignored by default: spawns `git init` and changes the
    /// process working directory. Run with: `cargo test -- --ignored`.
    #[test]
    #[ignore]
    fn scan_recursively_finds_nested_repos_and_skips_vendor() {
        use std::process::Command;

        let base = std::env::temp_dir().join(format!("gith-scan-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        std::fs::create_dir_all(&base).unwrap();

        let make_repo = |rel: &str, msg: &str, dirty: bool| {
            let dir = base.join(rel);
            std::fs::create_dir_all(&dir).unwrap();
            let git = |args: &[&str]| {
                let out = Command::new("git")
                    .args(["-C", dir.to_str().unwrap()])
                    .args(args)
                    .output()
                    .unwrap();
                assert!(
                    out.status.success(),
                    "git {args:?} failed: {}",
                    String::from_utf8_lossy(&out.stderr)
                );
            };
            git(&["init", "-b", "main"]);
            git(&["config", "user.email", "t@t"]);
            git(&["config", "user.name", "t"]);
            std::fs::write(dir.join("f.txt"), msg).unwrap();
            git(&["add", "."]);
            git(&["commit", "-m", msg]);
            if dirty {
                std::fs::write(dir.join("junk.txt"), "untracked").unwrap();
            }
        };

        make_repo("app-new", "new commit", true);
        make_repo("group/app-old", "old commit", false);
        make_repo("x/node_modules/decoy", "vendor repo", false);
        std::fs::create_dir_all(base.join("not-a-repo")).unwrap();

        let prev = std::env::current_dir().unwrap();
        std::env::set_current_dir(&base).unwrap();
        let scan = scan_repos();
        std::env::set_current_dir(prev).unwrap();
        let _ = std::fs::remove_dir_all(&base);

        let rels: Vec<&str> = scan.entries.iter().map(|e| e.rel.as_str()).collect();
        assert_eq!(
            rels,
            vec!["app-new", "group/app-old"],
            "nested repos found, vendor + non-repos skipped, newest first"
        );
        assert_eq!(scan.dirty_count(), 1);
        assert_eq!(scan.entries[0].dirty, 1);
        assert_eq!(scan.entries[0].branch, "main");
        // Filtering works on the same data the browser uses.
        assert_eq!(filter_matches(&scan.entries, "old"), vec![1]);
    }
}
