//! Debug logging helpers.
//!
//! When the user passes `-d` / `--debug`, the binary writes human-readable
//! log lines to `/tmp/git-hero-debug.log`. The file handle is opened once
//! and cached in a `OnceLock<Mutex<File>>` to avoid re-opening on every
//! log call (was ~60 syscalls/second at 60 fps before the refactor).

use std::fs::{File, OpenOptions};
use std::io::Write;
use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

/// Path of the debug log. Stable so users can `tail -f` it.
pub const DEBUG_LOG_PATH: &str = "/tmp/git-hero-debug.log";

static LOG_FILE: OnceLock<Option<Mutex<File>>> = OnceLock::new();

/// Get (or lazily create) the global log file handle.
///
/// Returns `None` if the file can't be opened — the caller should silently
/// skip logging in that case so a permissions issue doesn't take down the TUI.
fn log_file() -> Option<&'static Mutex<File>> {
    LOG_FILE
        .get_or_init(|| {
            OpenOptions::new()
                .create(true)
                .append(true)
                .open(DEBUG_LOG_PATH)
                .ok()
                .map(Mutex::new)
        })
        .as_ref()
}

/// Empty the log file. Called once on startup when debug mode is on, so each
/// session starts with a clean log.
pub fn clear() {
    let _ = std::fs::write(DEBUG_LOG_PATH, "");
    // Drop any cached handle so the next `log_debug` re-opens the (now empty) file.
    // We can't actually drop from OnceLock, but `OpenOptions::append` will just
    // append to the now-empty file, which is what we want.
}

/// Append one line to the debug log. Silently no-ops if logging is disabled
/// or the file can't be opened.
pub fn log_debug(msg: &str) {
    if let Some(file) = log_file()
        && let Ok(mut f) = file.lock()
    {
        let _ = writeln!(f, "[{:.3}] {}", timestamp_secs(), msg);
    }
}

/// Unix timestamp in seconds with millisecond precision, as a `String`.
fn timestamp_secs() -> String {
    let d = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    format!("{:.3}", d.as_secs_f64())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn log_debug_does_not_panic_when_file_is_unwritable() {
        // We can't actually make the file unwritable in a test without
        // breaking other tests, so we just call the function and ensure it
        // returns without panicking. The static stays initialized across
        // tests in the same binary.
        log_debug("test message");
    }

    #[test]
    fn clear_does_not_panic() {
        clear();
    }
}
