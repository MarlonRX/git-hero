// ── Version Info ────────────────────────────────────────────────────────
// All version-related data is pulled from the build at compile time:
//   - `CARGO_PKG_VERSION` is automatically injected by Cargo from Cargo.toml
//     (the same source used by `scripts/deploy.sh`).
//   - `GIT_HERO_GIT_HASH`, `GIT_HERO_GIT_DIRTY`, and `GIT_HERO_BUILD_PROFILE`
//     are set by `build.rs` at compile time.

/// Semantic version from `Cargo.toml` (e.g. "0.1.0").
pub const PKG_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Short git commit hash (e.g. "a1b2c3d") or "unknown" when not in a git repo.
pub const GIT_HASH: &str = env!("GIT_HERO_GIT_HASH");

/// "1" if the working tree had uncommitted changes at build time, else "0".
pub const GIT_DIRTY: &str = env!("GIT_HERO_GIT_DIRTY");

/// "debug" or "release" — useful for marking dev builds.
pub const BUILD_PROFILE: &str = env!("GIT_HERO_BUILD_PROFILE");

/// Clean, short version string for the UI badge: `v0.1.0`.
pub fn short() -> String {
    format!("v{}", PKG_VERSION)
}

/// Full, professional version string for the UI badge.
///
/// Examples:
///   - Release build, clean tree:   `v0.1.0`
///   - Release build, dirty tree:   `v0.1.0+a1b2c3d*`
///   - Debug build,   clean tree:   `v0.1.0-dev (a1b2c3d)`
///   - Debug build,   dirty tree:   `v0.1.0-dev (a1b2c3d)*`
pub fn full() -> String {
    if is_release() {
        return short();
    }

    let profile_tag = if BUILD_PROFILE == "debug" { "-dev" } else { "" };
    let dirty_mark = if GIT_DIRTY == "1" { "*" } else { "" };
    format!("v{}{} ({}){}", PKG_VERSION, profile_tag, GIT_HASH, dirty_mark)
}

/// Returns `true` if the current binary is a release build with a clean tree.
pub fn is_release() -> bool {
    BUILD_PROFILE == "release" && GIT_DIRTY == "0"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn short_has_v_prefix() {
        assert!(short().starts_with('v'));
    }

    #[test]
    fn full_contains_hash() {
        assert!(full().contains(GIT_HASH));
    }
}
