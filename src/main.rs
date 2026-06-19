mod config;
mod theme;
mod i18n;
mod git;
mod git_error;
mod log;
mod cli;
mod ui;
mod version;

use std::env;

fn run_askpass_helper(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    use std::io::Write;
    let prompt = if args.len() > 2 {
        &args[2]
    } else {
        "Credentials required: "
    };

    let session_id = env::var("GIT_HERO_SESSION_ID").unwrap_or_default();
    let temp_dir = env::temp_dir();
    let prompt_path = temp_dir.join(format!("git-hero-askpass-prompt-{}.txt", session_id));
    let response_path = temp_dir.join(format!("git-hero-askpass-response-{}.txt", session_id));

    // Write prompt message to prompt file
    std::fs::write(&prompt_path, prompt)?;

    // Phase 3.6: poll every 500 ms instead of 50 ms. A human typing a
    // password is at least one order of magnitude slower than 20 Hz, and
    // the helper process is idle most of the time — 90% less wakeups.
    // Total timeout: 300 ticks × 500 ms = 2.5 minutes (was 5 minutes).
    let mut ticks = 0u32;
    loop {
        if response_path.exists() {
            let res = std::fs::read_to_string(&response_path)?;
            // Git expects the credential exactly on stdout (without trailing newline typically, but trim is safe)
            print!("{}", res);
            let _ = std::io::stdout().flush();
            let _ = std::fs::remove_file(&response_path);
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(500));
        ticks += 1;
        if ticks > 300 {
            let _ = std::fs::remove_file(&prompt_path);
            return Err("Askpass timed out waiting for user input".into());
        }
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    
    // Check if we are running as askpass helper
    if args.len() > 1 && args[1] == "--askpass-helper" {
        run_askpass_helper(&args)?;
        return Ok(());
    }

    let is_cli = args.iter().any(|arg| arg == "-cli" || arg == "-c" || arg == "--cli");
    let is_debug = args.iter().any(|arg| arg == "-debug" || arg == "-d" || arg == "--debug");

    // Clear debug log on startup
    if is_debug {
        log::clear();
        log::log_debug("=== Git Hero starting in DEBUG mode ===");
    }

    if is_cli {
        if is_debug { log::log_debug("Running in CLI mode"); }
        cli::run_cli_flow()?;
    } else {
        if is_debug { log::log_debug("Running in TUI mode"); }
        ui::run_tui(is_debug)?;
    }

    Ok(())
}
