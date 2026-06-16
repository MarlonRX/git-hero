mod config;
mod theme;
mod i18n;
mod git;
mod cli;
mod ui;
mod version;

use std::env;
use std::fs::OpenOptions;
use std::io::Write;

fn log_debug(msg: &str) {
    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open("/tmp/git-hero-debug.log")
    {
        let _ = writeln!(file, "[{}] {}", chrono_lite(), msg);
    }
}

fn chrono_lite() -> String {
    // Simple timestamp without external crate
    use std::time::{SystemTime, UNIX_EPOCH};
    let d = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default();
    format!("{:.3}", d.as_secs_f64())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let is_cli = args.iter().any(|arg| arg == "-cli" || arg == "-c" || arg == "--cli");
    let is_debug = args.iter().any(|arg| arg == "-debug" || arg == "-d" || arg == "--debug");

    // Clear debug log on startup
    if is_debug {
        let _ = std::fs::write("/tmp/git-hero-debug.log", "");
        log_debug("=== Git Hero starting in DEBUG mode ===");
    }

    if is_cli {
        if is_debug { log_debug("Running in CLI mode"); }
        cli::run_cli_flow()?;
    } else {
        if is_debug { log_debug("Running in TUI mode"); }
        ui::run_tui(is_debug)?;
    }

    Ok(())
}
