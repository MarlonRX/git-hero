//! Error type for git operations.
//!
//! Used throughout `git-hero` for everything that talks to the `git` binary
//! or touches the on-disk repository. The variants are intentionally
//! coarse-grained — most failures are reported back to the user as-is and
//! don't need to be programmatically distinguished by callers.
//!
//! Callers that don't care about the details can convert via `From<String>`
//! (preserves the previous `Result<_, String>` API as a transitional shim).

use thiserror::Error;

/// A single error variant covering every failure mode of [`crate::git`].
#[derive(Debug, Error)]
#[allow(dead_code)] // Will be used in Phase 3 when `git::run_*` migrates to this type.
pub enum GitError {
    /// `git` could not be spawned (binary missing, PATH issue, permissions).
    #[error("failed to spawn git: {0}")]
    SpawnFailed(String),

    /// `git` ran but exited with a non-zero status. Contains the stderr.
    #[error("git exited with status {code:?}: {stderr}")]
    CommandFailed { stderr: String, code: Option<i32> },

    /// The current directory is not inside a Git working tree.
    #[error("not a git repository")]
    NotARepository,

    /// A lower-level I/O error (file missing, permission denied, etc.).
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Catch-all for situations that don't fit the other variants.
    #[error("{0}")]
    Other(String),
}

/// Transitional shim so existing call-sites that return `Result<_, String>`
/// can keep working while we migrate the rest of the API.
impl From<String> for GitError {
    fn from(s: String) -> Self {
        GitError::Other(s)
    }
}

impl From<&str> for GitError {
    fn from(s: &str) -> Self {
        GitError::Other(s.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_spawn_failed() {
        let e = GitError::SpawnFailed("not found".into());
        assert_eq!(e.to_string(), "failed to spawn git: not found");
    }

    #[test]
    fn display_command_failed_with_code() {
        let e = GitError::CommandFailed {
            stderr: "fatal: bad ref".into(),
            code: Some(128),
        };
        assert_eq!(e.to_string(), "git exited with status Some(128): fatal: bad ref");
    }

    #[test]
    fn display_command_failed_no_code() {
        let e = GitError::CommandFailed {
            stderr: "fatal: bad ref".into(),
            code: None,
        };
        assert_eq!(e.to_string(), "git exited with status None: fatal: bad ref");
    }

    #[test]
    fn display_not_a_repository() {
        assert_eq!(GitError::NotARepository.to_string(), "not a git repository");
    }

    #[test]
    fn from_io_error() {
        let io = std::io::Error::new(std::io::ErrorKind::NotFound, "missing");
        let e: GitError = io.into();
        assert!(matches!(e, GitError::Io(_)));
        assert!(e.to_string().contains("I/O error"));
    }

    #[test]
    fn from_str_and_string() {
        let _: GitError = "boom".into();
        let _: GitError = String::from("boom").into();
    }
}
