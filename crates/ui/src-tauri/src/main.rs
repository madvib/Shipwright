// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use clap::Parser;
use cli::{handle_cli, Cli, Commands, McpCommands};
use std::env;

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();

    // Internal utility: generate tauri-specta bindings and exit.
    if args.len() > 1 && (args[1] == "--gen-bindings" || args[1] == "gen-bindings") {
        match gui::export_bindings() {
            Ok(path) => {
                println!("Generated bindings at {}", path.display());
                return;
            }
            Err(error) => {
                eprintln!("Error generating bindings: {}", error);
                std::process::exit(1);
            }
        }
    }

    // If no arguments beyond the program name, run the GUI
    if args.len() <= 1 {
        gui::run();
        return;
    }

    // Check for special commands that need async (like MCP)
    let cli = Cli::parse();
    if let Some(Commands::Mcp { ref action }) = cli.command {
        match action {
            None | Some(McpCommands::Serve) => {
                if let Err(e) = mcp::run_server().await {
                    eprintln!("MCP Error: {}", e);
                    std::process::exit(1);
                }
            }
            _ => {
                // Handle standard CLI commands synchronously (shared logic)
                if let Err(e) = handle_cli(cli) {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            }
        }
    } else {
        // Handle standard CLI commands synchronously (shared logic)
        if let Err(e) = handle_cli(cli) {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}
