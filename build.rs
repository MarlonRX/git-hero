// ── Build Script ────────────────────────────────────────────────────────
// Embeds the current git commit hash and build profile into the binary at
// compile time, so the in-app version badge can show real deployment info
// instead of a hardcoded string.
//
// Exposed via:
//   - GIT_HERO_GIT_HASH        short commit hash (e.g. "a1b2c3d"), or "unknown"
//   - GIT_HERO_GIT_DIRTY       "1" if the working tree is dirty, else "0"
//   - GIT_HERO_BUILD_PROFILE   "debug" or "release"

use std::process::Command;

fn run_git(args: &[&str]) -> Option<String> {
    let out = Command::new("git").args(args).output().ok()?;
    if !out.status.success() {
        return None;
    }
    let s = String::from_utf8(out.stdout).ok()?.trim().to_string();
    if s.is_empty() { None } else { Some(s) }
}

fn main() {
    // Re-run this build script whenever HEAD changes or a release build is triggered.
    println!("cargo:rerun-if-changed=.git/HEAD");
    println!("cargo:rerun-if-changed=.git/index");
    println!("cargo:rerun-if-env-changed=PROFILE");

    // Short commit hash
    let hash = run_git(&["rev-parse", "--short", "HEAD"]).unwrap_or_else(|| "unknown".into());
    println!("cargo:rustc-env=GIT_HERO_GIT_HASH={}", hash);

    // Dirty flag (uncommitted changes)
    let dirty = if run_git(&["status", "--porcelain"]).map(|s| !s.is_empty()).unwrap_or(false) {
        "1"
    } else {
        "0"
    };
    println!("cargo:rustc-env=GIT_HERO_GIT_DIRTY={}", dirty);

    // Build profile
    let profile = std::env::var("PROFILE").unwrap_or_else(|_| "debug".into());
    println!("cargo:rustc-env=GIT_HERO_BUILD_PROFILE={}", profile);
}
