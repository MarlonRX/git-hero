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

/// Semantic version tuple for ordering.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Version {
    pub major: u64,
    pub minor: u64,
    pub patch: u64,
}

impl Version {
    /// Parse `"v0.1.0"` or `"0.1.0"`. Returns `None` for invalid input.
    pub fn parse(s: &str) -> Option<Self> {
        let s = s.strip_prefix('v').unwrap_or(s);
        let mut parts = s.splitn(3, '.');
        Some(Self {
            major: parts.next()?.parse().ok()?,
            minor: parts.next()?.parse().ok()?,
            patch: parts.next()?.parse().ok()?,
        })
    }
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

    #[test]
    fn version_parse_v_prefix() {
        let v = Version::parse("v0.1.0").unwrap();
        assert_eq!(v, Version { major: 0, minor: 1, patch: 0 });
    }

    #[test]
    fn version_parse_no_prefix() {
        let v = Version::parse("1.2.3").unwrap();
        assert_eq!(v, Version { major: 1, minor: 2, patch: 3 });
    }

    #[test]
    fn version_parse_invalid() {
        assert!(Version::parse("foo").is_none());
        assert!(Version::parse("1.2").is_none());
        assert!(Version::parse("").is_none());
    }

    #[test]
    fn version_ordering() {
        let v1 = Version { major: 0, minor: 9, patch: 0 };
        let v2 = Version { major: 0, minor: 10, patch: 0 };
        assert!(v1 < v2);
        assert!(v2 > v1);
    }
}
