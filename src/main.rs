mod config;
mod theme;
mod i18n;
mod git;
mod cli;
mod ui;

use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let is_cli = args.iter().any(|arg| arg == "-cli" || arg == "-c" || arg == "--cli");

    if is_cli {
        cli::run_cli_flow()?;
    } else {
        ui::run_tui()?;
    }

    Ok(())
}
