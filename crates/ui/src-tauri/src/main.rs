// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use clap::Parser;
use cli::{handle_cli, Cli, Commands};
use std::env;

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();

    // If no arguments beyond the program name, run the GUI
    if args.len() <= 1 {
        gui::run();
        return;
    }

    // Check for special commands that need async (like MCP)
    let cli = Cli::parse();
    if let Some(Commands::Mcp) = cli.command {
        if let Err(e) = mcp::run_server().await {
            eprintln!("MCP Error: {}", e);
            std::process::exit(1);
        }
    } else {
        // Handle standard CLI commands synchronously (shared logic)
        if let Err(e) = handle_cli(cli) {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}
