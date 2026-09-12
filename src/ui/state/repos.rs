//! Multi-repo overview — the repository-manager core of Git Hero's niche.
//!
//! A scan probes every sibling (or, when outside a repo, child) directory
//! that contains a `.git`, pulling branch, ahead/behind and dirty count
//! from ONE `git status --branch --porcelain=v2` call per repo plus one
//! `git log -1` for the last-commit timestamp. Results are ordered newest
//! activity first.
//!
//! Scanning is on-demand: it runs when the overview is opened or
//! re-scanned with `g`, so the regular per-frame render path stays free.

use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::git::{self, StatusSnapshot};

/// Hard cap on probed repositories per scan — each probe costs two `git`
/// processes, so a huge "projects" folder must not freeze the UI.
const MAX_REPOS: usize = 40;

/// Hard cap on directory entries enumerated before probing (stat guard).
const MAX_CANDIDATES: usize = 200;

/// One row of the repository overview.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepoEntry {
    pub path: String,
    pub name: String,
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

/// Scan for repositories adjacent to the current working directory.
pub fn scan_sibling_repos() -> RepoScan {
    let cwd = match std::env::current_dir() {
        Ok(p) => p,
        Err(_) => return RepoScan::default(),
    };
    let root = scan_root(&cwd);
    let now = epoch_now();
    let mut entries: Vec<RepoEntry> = Vec::new();

    if let Ok(read) = std::fs::read_dir(&root) {
        let mut probed = 0usize;
        for cand in read.flatten().take(MAX_CANDIDATES) {
            let path = cand.path();
            if !path.is_dir() || !path.join(".git").exists() {
                continue;
            }
            if probed >= MAX_REPOS {
                break;
            }
            probed += 1;
            let path_str = path.to_string_lossy().into_owned();
            let Some(snap) = git::status_snapshot_in(&path_str).ok() else {
                continue; // broken/locked repo: skip rather than fail the scan
            };
            let epoch = git::last_commit_epoch_in(&path_str);
            entries.push(entry_from_snapshot(
                &cand.file_name(),
                path_str,
                &snap,
                epoch,
                now,
            ));
        }
    }

    sort_by_activity(&mut entries);
    RepoScan {
        root: root.to_string_lossy().into_owned(),
        entries,
    }
}

/// When the current directory IS a repo, list its siblings (the projects
/// folder); otherwise treat the current directory as a projects folder.
fn scan_root(cwd: &Path) -> PathBuf {
    scan_root_for(cwd.join(".git").exists(), cwd)
}

/// Pure decision behind [`scan_root`], testable without touching the disk.
fn scan_root_for(is_repo: bool, cwd: &Path) -> PathBuf {
    if is_repo {
        cwd.parent().map(Path::to_path_buf).unwrap_or_else(|| cwd.to_path_buf())
    } else {
        cwd.to_path_buf()
    }
}

/// Pure row builder: turns one status snapshot into a `RepoEntry`.
fn entry_from_snapshot(
    name: &OsStr,
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
        name: name.to_string_lossy().into_owned(),
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

/// Compact relative age for the overview column, git-`%ar`-flavoured.
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
        let e = entry_from_snapshot(OsStr::new("x"), "p".into(), &snap("(detached)", 0, 0, 0), None, 100);
        assert_eq!(e.branch, "\u{2205}");
        assert!(e.age.is_empty(), "repo without commits shows no age");
    }

    #[test]
    fn entry_counts_dirty_and_formats_age() {
        let e = entry_from_snapshot(
            OsStr::new("proj"),
            "/tmp/proj".into(),
            &snap("main", 2, 1, 4),
            Some(1000),
            1000 + 7_200,
        );
        assert_eq!(e.name, "proj");
        assert_eq!(e.dirty, 4);
        assert_eq!((e.ahead, e.behind), (2, 1));
        assert_eq!(e.age, "2h");
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
            path: String::new(),
            name: name.into(),
            branch: "main".into(),
            ahead: 0,
            behind: 0,
            dirty: 0,
            age: String::new(),
            last_commit: epoch,
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
        let scan = RepoScan {
            root: String::new(),
            entries: vec![
                RepoEntry { path: String::new(), name: "a".into(), branch: String::new(), ahead: 0, behind: 0, dirty: 2, age: String::new(), last_commit: None },
                RepoEntry { path: String::new(), name: "b".into(), branch: String::new(), ahead: 0, behind: 0, dirty: 0, age: String::new(), last_commit: None },
                RepoEntry { path: String::new(), name: "c".into(), branch: String::new(), ahead: 0, behind: 0, dirty: 1, age: String::new(), last_commit: None },
            ],
        };
        assert_eq!(scan.dirty_count(), 2);
    }
}
